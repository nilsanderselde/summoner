// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Declarative DSP node descriptors and UI builders for `SamplerSlicer`.

use std::collections::HashMap;

use super::super::types::*;
use super::super::{DspNodeUi, GenericDspNodeUi};

pub(in crate::dsp_node_ui) fn register_descriptors(
    descriptors: &mut HashMap<String, DspNodeDescriptor>,
) {
    descriptors.insert("DrumMachineDevice".to_string(), DspNodeDescriptor::new("DrumMachineDevice", "24-Pad Zero-Allocation Sampling Drum Machine", DspNodeCategory::SamplerSlicer, "24-Pad Zero-Allocation Sampling Drum Machine")
        .with_param(DspParamSchema::knob("master_volume", "Master Kit Volume", 0.0, 1.0, 0.85, "%", MacroRole::Tone, "Master Kit Volume parameter"))
        .with_param(DspParamSchema::knob("pitch_offset", "Kit Pitch Offset", -12.0, 12.0, 0.0, "st", MacroRole::Tone, "Kit Pitch Offset parameter"))
        .with_param(DspParamSchema::knob("pad_decay", "Global Pad Decay Scale", 0.1, 2.0, 1.0, "x", MacroRole::Tone, "Global Pad Decay Scale parameter"))
        .with_param(DspParamSchema::knob("dynamic_punch", "Master Dynamic Punch", 0.0, 1.0, 0.6, "%", MacroRole::Tone, "Master Dynamic Punch parameter"))
    );
    descriptors.insert("SamplerDevice".to_string(), DspNodeDescriptor::new("SamplerDevice", "Multi-Sample Instrument with ADSR Envelope & Filter", DspNodeCategory::SamplerSlicer, "Multi-Sample Instrument with ADSR Envelope & Filter")
        .with_param(DspParamSchema::log_knob("filter_cutoff", "Ladder Filter Cutoff", 20.0, 20000.0, 18000.0, "Hz", MacroRole::Tone, "Ladder Filter Cutoff logarithmic parameter"))
        .with_param(DspParamSchema::knob("filter_res", "Ladder Resonance", 0.0, 3.9, 0.7, "Q", MacroRole::Tone, "Ladder Resonance parameter"))
        .with_param(DspParamSchema::knob("amp_attack", "Attack Time", 0.001, 2.0, 0.005, "s", MacroRole::Tone, "Attack Time parameter"))
        .with_param(DspParamSchema::knob("amp_release", "Release Time", 0.01, 5.0, 0.4, "s", MacroRole::Tone, "Release Time parameter"))
        .with_param(DspParamSchema::knob("gain", "Master Output Gain", 0.0, 1.0, 0.8, "%", MacroRole::Tone, "Master Output Gain parameter"))
    );
    descriptors.insert("MultiSamplerNode".to_string(), DspNodeDescriptor::new("MultiSamplerNode", "Velocity-Layered Multi-Zone Audio Sampler", DspNodeCategory::SamplerSlicer, "Velocity-Layered Multi-Zone Audio Sampler")
        .with_param(DspParamSchema::knob("root_key", "Center Root Key", 0.0, 127.0, 60.0, "midi", MacroRole::Character, "Center Root Key integer control"))
        .with_param(DspParamSchema::knob("velocity_track", "Velocity Sensitivity", 0.0, 1.0, 0.8, "%", MacroRole::Tone, "Velocity Sensitivity parameter"))
        .with_param(DspParamSchema::choice("interpolation", "Resampling Kernel", &["Linear", "Hermite Cubic", "Sinc Bandlimited"], 1, "Resampling Kernel mode selector"))
        .with_param(DspParamSchema::knob("gain", "Sample Playback Level", 0.0, 1.0, 0.9, "%", MacroRole::Tone, "Sample Playback Level parameter"))
    );
    descriptors.insert("SampleSlicerNode".to_string(), DspNodeDescriptor::new("SampleSlicerNode", "Transient Beat Slicer & Loop Fragmenter", DspNodeCategory::SamplerSlicer, "Transient Beat Slicer & Loop Fragmenter")
        .with_param(DspParamSchema::knob("threshold_db", "Transient Slicer Sensitivity", -60.0, 0.0, -24.0, "dB", MacroRole::Tone, "Transient Slicer Sensitivity parameter"))
        .with_param(DspParamSchema::knob("min_slice_ms", "Minimum Slice Duration", 10.0, 500.0, 50.0, "ms", MacroRole::Tone, "Minimum Slice Duration parameter"))
        .with_param(DspParamSchema::toggle("bpm_sync", "Snap Slices to Beat Grid", true, "Snap Slices to Beat Grid toggle switch"))
        .with_param(DspParamSchema::knob("transient_sensitivity", "Onset Detection Threshold", 0.0, 1.0, 0.7, "%", MacroRole::Tone, "Onset Detection Threshold parameter"))
    );
    descriptors.insert("LoopSlicerNode".to_string(), DspNodeDescriptor::new("LoopSlicerNode", "Beat-Synchronized Grid Loop Slicer", DspNodeCategory::SamplerSlicer, "Beat-Synchronized Grid Loop Slicer")
        .with_param(DspParamSchema::choice("grid_division", "Beat Division", &["1/4 Beat", "1/8 Beat", "1/16 Beat", "1/32 Beat", "Triplet 1/8"], 2, "Beat Division mode selector"))
        .with_param(DspParamSchema::knob("crossfade_ms", "Slice Edge Crossfade", 0.0, 50.0, 5.0, "ms", MacroRole::Tone, "Slice Edge Crossfade parameter"))
        .with_param(DspParamSchema::toggle("pitch_match", "Preserve Original Pitch", true, "Preserve Original Pitch toggle switch"))
        .with_param(DspParamSchema::toggle("reverse_slices", "Reverse Alternate Slices", false, "Reverse Alternate Slices toggle switch"))
    );
    descriptors.insert("AutomatedDrumReplacer".to_string(), DspNodeDescriptor::new("AutomatedDrumReplacer", "AI Automated Drum Replacement Trigger", DspNodeCategory::SamplerSlicer, "Intelligent Transient-Triggered Multi-Layer Drum Sample Replacer")
        .with_param(DspParamSchema::knob("trigger_thresh", "Trigger Threshold", -60.0, 0.0, -24.0, "dB", MacroRole::Punch, "Transient onset detector threshold parameter"))
        .with_param(DspParamSchema::knob("sensitivity", "Sensitivity", 0.0, 1.0, 0.75, "%", MacroRole::Character, "Spectral flux detector sensitivity parameter"))
        .with_param(DspParamSchema::knob("velocity_track", "Velocity Tracking", 0.0, 1.0, 0.85, "%", MacroRole::Punch, "Input dynamic energy to sample velocity scaling parameter"))
        .with_param(DspParamSchema::knob("retrigger_ms", "Retrigger Holdoff", 5.0, 200.0, 40.0, "ms", MacroRole::Tone, "Minimum lockout between consecutive strikes parameter"))
        .with_param(DspParamSchema::choice("sample_slot", "Sample Preset", &["Acoustic Snare Direct", "808 Sub Kick", "Tight Studio Rim", "Layered Transient"], 0, "Loaded replacement sample selector"))
        .with_param(DspParamSchema::knob("dry_wet", "Dry / Wet Mix", 0.0, 1.0, 1.0, "%", MacroRole::Space, "Blend between original audio and triggered sample parameter"))
    );
    descriptors.insert("AutoSlicer".to_string(), DspNodeDescriptor::new("AutoSlicer", "Intelligent Transient Detector & Sample Slicer", DspNodeCategory::SamplerSlicer, "Offline and real-time transient detection and beat slicing using energy derivative, spectral flux, and ONNX neural models")
        .with_param(DspParamSchema::knob("threshold", "Transient Peak Threshold", 0.01, 1.0, 0.25, "%", MacroRole::Punch, "Transient onset detection threshold parameter"))
        .with_param(DspParamSchema::choice("algorithm", "Detection Algorithm", &["Energy Derivative", "Spectral Flux", "ONNX Neural Model"], 1, "Slicing onset detector engine"))
        .with_param(DspParamSchema::knob("min_slice_length_ms", "Minimum Slice Window", 5.0, 500.0, 50.0, "ms", MacroRole::Space, "Minimum allowable slice duration parameter"))
        .with_param(DspParamSchema::knob("sensitivity", "Spectral Flux Sensitivity", 0.1, 5.0, 1.5, "x", MacroRole::Tone, "Spectral flux onset multiplier parameter"))
    );
    descriptors.insert("AudioReverse".to_string(), DspNodeDescriptor::new("AudioReverse", "Real-Time Buffer Reverse & Granular Rewind", DspNodeCategory::SamplerSlicer, "Dual-ping-pong buffer rewinder providing real-time backwards audio playback with smooth crossfades")
        .with_param(DspParamSchema::knob("buffer_length_ms", "Reverse Buffer Size", 20.0, 2000.0, 250.0, "ms", MacroRole::Space, "Duration of audio reversed in each playback block parameter"))
        .with_param(DspParamSchema::knob("crossfade_ms", "Boundary Crossfade Slew", 2.0, 50.0, 10.0, "ms", MacroRole::Tone, "Window smoothing time eliminating boundary click transients parameter"))
        .with_param(DspParamSchema::knob("playback_speed", "Reverse Speed Multiplier", 0.25, 2.0, 1.0, "x", MacroRole::Punch, "Tape rewind transport speed ratio parameter"))
        .with_param(DspParamSchema::toggle("reverse_trigger", "Engage Reverse Playback", true, "Toggle between forward through-signal and backward reverse playback parameter"))
    );
    descriptors.insert("DrumReplacementTrigger".to_string(), DspNodeDescriptor::new("DrumReplacementTrigger", "Transient Acoustic Drum Replacement & Multi-Velocity Trigger", DspNodeCategory::SamplerSlicer, "Real-time acoustic drum transient detector with bandpass filtering and velocity-sensitive sample triggering")
        .with_param(DspParamSchema::knob("sensitivity_thresh_db", "Trigger Threshold", -60.0, 0.0, -24.0, "dB", MacroRole::Punch, "Detection threshold for transient strike trigger parameter"))
        .with_param(DspParamSchema::knob("retrigger_mask_ms", "Retrigger Mask", 5.0, 200.0, 35.0, "ms", MacroRole::Tone, "Hold-off time window preventing false double-triggers parameter"))
        .with_param(DspParamSchema::knob("sample_blend", "Sample Mix Ratio", 0.0, 1.0, 0.85, "%", MacroRole::Character, "Blend between original acoustic audio and replaced sample parameter"))
        .with_param(DspParamSchema::knob("velocity_tracking", "Dynamic Velocity Scale", 0.0, 1.0, 0.90, "%", MacroRole::Space, "Maps detected hit energy to MIDI velocity and layer crossfade parameter"))
    );
    descriptors.insert("SampleEditor".to_string(), DspNodeDescriptor::new("SampleEditor", "Interactive Waveform Sample Slicer & Looper", DspNodeCategory::SamplerSlicer, "Visual sample editor with interactive zero-crossing snap, loop crossfade, and transient slice markers")
        .with_param(DspParamSchema::knob("loop_start_sec", "Loop Start Position", 0.0, 60.0, 0.0, "s", MacroRole::Tone, "Sample loop start time offset parameter"))
        .with_param(DspParamSchema::knob("loop_end_sec", "Loop End Position", 0.01, 60.0, 4.0, "s", MacroRole::Space, "Sample loop end boundary timestamp parameter"))
        .with_param(DspParamSchema::log_knob("crossfade_len_ms", "Loop Crossfade Length", 1.0, 1000.0, 25.0, "ms", MacroRole::Character, "Equal-power loop point crossfade duration parameter"))
        .with_param(DspParamSchema::toggle("snap_to_zero", "Snap to Zero Crossing", true, "Snap slice boundaries to closest zero-crossing point parameter"))
        .with_param(DspParamSchema::toggle("normalize_on_slice", "Auto-Normalize Slices", true, "Peak normalize sliced regions automatically parameter"))
    );
    descriptors.insert("MultitrackSessionLooper".to_string(), DspNodeDescriptor::new("MultitrackSessionLooper", "Live Multitrack Looping & Overdubbing DSP Engine", DspNodeCategory::SamplerSlicer, "Lock-free circular audio buffer looper with seamless bar-quantized record, overdub, multiply, divide, undo/redo layers, and sample-accurate transport sync")
        .with_param(DspParamSchema::knob("feedback", "Overdub Feedback Decay", 0.0, 1.0, 0.95, "%", MacroRole::Tone, "Overdub feedback decay level parameter"))
        .with_param(DspParamSchema::knob("track_count", "Looper Track Count", 1.0, 16.0, 4.0, "tracks", MacroRole::Character, "Active looper track count parameter"))
        .with_param(DspParamSchema::knob("crossfade_ms", "Boundary Crossfade Length", 1.0, 50.0, 10.0, "ms", MacroRole::Space, "Boundary turnaround crossfade length parameter"))
        .with_param(DspParamSchema::choice("quantize_bars", "Quantization Boundary", &["Off (Instant)", "1 Beat", "1 Bar", "2 Bars", "4 Bars"], 2, "Quantization boundary sync parameter"))
        .with_param(DspParamSchema::knob("dry_wet", "Dry / Wet Mix", 0.0, 1.0, 1.0, "%", MacroRole::Punch, "Dry input / looped playback blend parameter"))
    );
    descriptors.insert("SamplerNode".to_string(), DspNodeDescriptor::new("SamplerNode", "Multi-Layer Velocity-Switched Audio Sample Playback Engine", DspNodeCategory::SamplerSlicer, "Multi-layer velocity-switched audio sample playback engine with interpolation, forward/reverse looping, and pitch envelope")
        .with_param(DspParamSchema::choice("interpolation_mode", "Resampling Interpolation", &["Linear (Fast)", "Hermite Cubic", "Sinc 32-Point", "Sinc 64-Point (Pristine)"], 1, "Interpolation algorithm used for pitch transposition"))
        .with_param(DspParamSchema::knob("playback_speed", "Playback Speed Factor", 0.1, 4.0, 1.0, "x", MacroRole::Tone, "Audio playback rate multiplier"))
        .with_param(DspParamSchema::knob("root_pitch_semitones", "Root Key Offset", -48.0, 48.0, 0.0, "st", MacroRole::Tone, "Base pitch offset in semitones"))
        .with_param(DspParamSchema::toggle("loop_enabled", "Loop Segment Active", true, "Enables continuous sustain loop between markers"))
        .with_param(DspParamSchema::knob("loop_crossfade_ms", "Loop Crossfade Duration", 0.0, 250.0, 25.0, "ms", MacroRole::Space, "Equal-power crossfade window smoothing loop boundaries"))
        .with_param(DspParamSchema::knob("velocity_tracking", "Velocity Dynamic Gain Scaling", 0.0, 1.0, 0.75, "%", MacroRole::Punch, "Scales playback output gain with incoming MIDI velocity"))
    );
    descriptors.insert("ClipMatrix".to_string(), DspNodeDescriptor::new("ClipMatrix", "Non-Linear Session View Clip Launcher Matrix", DspNodeCategory::SamplerSlicer, "Non-linear grid matrix launching multi-track MIDI and audio clips with sample-accurate quantization, scene triggering, and follow actions")
        .with_param(DspParamSchema::choice("launch_quantize", "Launch Quantization Grid", &["1 Bar", "1/2 Bar", "1/4 Note", "1/8 Note", "1/16 Note", "None"], 0, "Grid timing boundary for launching clips"))
        .with_param(DspParamSchema::toggle("scene_tempo_sync", "Sync Scene Tempo Bpm", true, "Synchronizes session tempo to metadata stored in triggered scene"))
        .with_param(DspParamSchema::knob("follow_action_prob", "Follow Action Probability", 0.0, 1.0, 0.80, "%", MacroRole::Character, "Likelihood of executing clip follow action on loop completion"))
        .with_param(DspParamSchema::knob("crossfade_curve", "Clip Switch Crossfade", 0.0, 100.0, 10.0, "ms", MacroRole::Tone, "Crossfade buffer duration when swapping audio clips in slot"))
        .with_param(DspParamSchema::toggle("loop_continuous", "Seamless Audio Loop", true, "Maintains endless continuous audio looping on triggered clips"))
    );
    descriptors.insert("CompTrack".to_string(), DspNodeDescriptor::new("CompTrack", "Multi-Take Audio & Vocal Comping Crossfade Engine", DspNodeCategory::SamplerSlicer, "Multi-lane take management engine with intuitive swipe comping, automatic boundary crossfading, and phase-aligned slice merging")
        .with_param(DspParamSchema::knob("crossfade_time_ms", "Take Crossfade Duration", 1.0, 100.0, 15.0, "ms", MacroRole::Tone, "Crossfade length between spliced comp take regions"))
        .with_param(DspParamSchema::toggle("equal_power_crossfade", "Equal Power 3dB Crossfade", true, "Uses sinusoidal equal-power crossfade to prevent volume dip"))
        .with_param(DspParamSchema::knob("take_selection", "Active Take Lane Index", 1.0, 16.0, 1.0, "lane", MacroRole::Character, "Currently highlighted take lane for region auditioning"))
        .with_param(DspParamSchema::toggle("auto_align_phase", "Transient Cross-Correlation Auto-Align", true, "Automatically shifts take offset to align initial transient phase"))
        .with_param(DspParamSchema::knob("pre_roll_lead_in_ms", "Comp Pre-Roll Headroom", 0.0, 200.0, 25.0, "ms", MacroRole::Space, "Breath and pick attack headroom before crossfade cut point"))
    );
    descriptors.insert("DrumMacroStrip".to_string(), DspNodeDescriptor::new("DrumMacroStrip", "16-Pad Tactile Drum Machine & Groove Macro Console HUD", DspNodeCategory::SamplerSlicer, "Dedicated live drum console exposing velocity curves, transient punch, decay envelope, pitch tuning, and master groove swing")
        .with_param(DspParamSchema::knob("global_groove_swing", "Drum Machine Global Shuffle & Swing", 50.0, 75.0, 58.0, "%", MacroRole::Tone, "Rhythmic groove displacement of even 16th notes creating swing feel"))
        .with_param(DspParamSchema::knob("transient_punch_boost_db", "Master Drum Transient Punch Boost", -6.0, 12.0, 3.5, "dB", MacroRole::Punch, "Transient designer attack boost adding snap and presence to snare and kick hits"))
        .with_param(DspParamSchema::knob("global_drum_decay_ratio", "Global Drum Voice Decay Time Scaling", 0.2, 3.0, 1.0, "x", MacroRole::Space, "Multiplies decay envelopes of all 16 drum pads for tight or ambient kits"))
        .with_param(DspParamSchema::knob("drum_master_saturation", "Drum Bus Discrete Tape Saturation Drive", 0.0, 100.0, 18.0, "%", MacroRole::Character, "Warm discrete transformer and magnetic tape drive glued across drum bus"))
        .with_param(DspParamSchema::knob("pad_velocity_compression", "Pad Dynamic Velocity Compression Curve", 0.0, 100.0, 40.0, "%", MacroRole::Character, "Dynamic compression leveling differences between soft and hard pad strikes"))
    );
    descriptors.insert("SfzPresetPatch".to_string(), DspNodeDescriptor::new("SfzPresetPatch", "SFZ Multi-Zone Soundfont & Instrument Patch HUD", DspNodeCategory::SamplerSlicer, "Standard SFZ opcode instrument format loader with multi-velocity split zones, sample root transposition, loop crossfades, and filter envelope tracking")
        .with_param(DspParamSchema::choice("keyzone_low_note", "Lowest Keyzone Active Note Boundary", &["C-1 (MIDI 0)", "C1 (MIDI 24)", "C2 (MIDI 36)", "C3 (MIDI 48)"], 0, "Lowest active MIDI keyzone boundary for current instrument layer"))
        .with_param(DspParamSchema::choice("keyzone_high_note", "Highest Keyzone Active Note Boundary", &["C5 (MIDI 72)", "C6 (MIDI 84)", "C7 (MIDI 96)", "G9 (MIDI 127)"], 3, "Highest active MIDI keyzone boundary for current instrument layer"))
        .with_param(DspParamSchema::knob("velocity_split_threshold", "Dynamic Velocity Crossfade Split Boundary", 1.0, 127.0, 64.0, "vel", MacroRole::Punch, "Dynamic MIDI velocity boundary between soft and hard multi-sample sound layers"))
        .with_param(DspParamSchema::knob("loop_crossfade_ms", "Loop Transition Equal-Power Crossfade", 0.0, 500.0, 45.0, "ms", MacroRole::Space, "Equal-power crossfade length at sustained sample loop transition points"))
        .with_param(DspParamSchema::knob("patch_attenuation_db", "Master Instrument Patch Trim Gain", -48.0, 12.0, -3.0, "dB", MacroRole::Tone, "Master patch gain offset compensating for multi-layer loudness differences"))
    );
    descriptors.insert("SfzMultisampleConverter".to_string(), DspNodeDescriptor::new("SfzMultisampleConverter", "SFZ Instrument to Native Multisample Keymap Converter HUD", DspNodeCategory::SamplerSlicer, "Parses SFZ instrument definition files, maps multi-velocity sample layers, and compiles native zero-allocation binary keymap instruments")
        .with_param(DspParamSchema::choice("velocity_layer_detection_mode", "Velocity Zone Mapping Strategy", &["SFZ Opcode Parsing (lovel/hivel)", "Automatic Dynamic Clustering", "Uniform Velocity Slicing"], 0, "Method used to extract note velocity boundary splits from instrument definitions"))
        .with_param(DspParamSchema::knob("round_robin_group_depth", "Maximum Round-Robin Performance Cycles", 1.0, 16.0, 4.0, "layers", MacroRole::Character, "Max cyclic sample alternations per key to prevent machine-gun repetition"))
        .with_param(DspParamSchema::knob("root_key_extrapolation_semitones", "Maximum Root Pitch Extrapolation Distance", 1.0, 24.0, 6.0, "st", MacroRole::Tone, "Limits high-quality pitch interpolation distance when keyzones have missing samples"))
        .with_param(DspParamSchema::toggle("trim_leading_sample_silence", "Auto-Trim Pre-Attack Sample Silence", true, "Removes sub-millisecond silent offsets at sample starts to ensure sharp transient response"))
        .with_param(DspParamSchema::choice("export_native_format", "Target Native Instrument Keymap Format", &["Summoner Keymap (.skm)", "Multi-Channel SoundFont (.sf3)", "Embedded Single-File Preset"], 0, "Output container format compiled for real-time sampler playback"))
    );
    descriptors.insert("MultiSamplePresetLoader".to_string(), DspNodeDescriptor::new("MultiSamplePresetLoader", "Multi-Sample SFZ Keygroup & Velocity Zone Loader HUD", DspNodeCategory::SamplerSlicer, "Loads multi-sample SFZ and Summoner instrument patches, mapping sample zones across MIDI key ranges, velocity layers, loop points, and crossfaded keygroups")
        .with_param(DspParamSchema::choice("active_sample_base_directory", "Sample Base Search Directory Scope", &["Default Local Project Assets", "System Global Samples Library", "External NVMe Sample SSD Mount", "Embedded Memory Cache"], 0, "Root path directory used to resolve relative sample audio paths"))
        .with_param(DspParamSchema::choice("velocity_layer_interpolation", "Velocity Layer Crossfade Interpolation", &["Equal-Power Sine Crossfade", "Linear Amplitude Ramp", "Hard Velocity Split (No Crossfade)"], 0, "Interpolation algorithm blending adjacent velocity sample layers"))
        .with_param(DspParamSchema::toggle("preload_zone_memory_buffers", "Pre-Load All Sample Buffers into RAM", true, "Pre-reads uncompressed audio PCM data into memory eliminating disk stream latency"))
        .with_param(DspParamSchema::choice("round_robin_cycling_mode", "Multi-Sample Round-Robin Cycling Strategy", &["Sequential Cycle 1-2-3-4", "Pseudo-Random No-Repeat", "True Random", "Velocity-Stochastic"], 1, "Algorithm cycling through alternate sample takes on repeated note triggers"))
        .with_param(DspParamSchema::knob("polyphonic_voice_limit", "Maximum Polyphonic Active Sample Voices", 8.0, 128.0, 32.0, "voices", MacroRole::Punch, "Maximum number of concurrent stereo sample voice channels rendered before voice stealing"))
    );
    descriptors.insert("MultiTakeCompManager".to_string(), DspNodeDescriptor::new("MultiTakeCompManager", "Sample-Accurate Multi-Take Audio Comping & Lane Splice Engine HUD", DspNodeCategory::SamplerSlicer, "Non-destructive comping controller managing multiple recorded audio takes, interactive lane swipe splicing, and equal-power boundary crossfades")
        .with_param(DspParamSchema::knob("splice_crossfade_duration_ms", "Comp Splice Boundary Crossfade Time", 1.0, 50.0, 5.0, "ms", MacroRole::Space, "Crossfade duration between adjacent take segments to eliminate boundary clicks and phase pops"))
        .with_param(DspParamSchema::choice("crossfade_curve_power", "Splice Boundary Crossfade Geometry", &["Equal Power (3dB Dip Free)", "Linear Slope", "S-Curve Logarithmic", "Fast Cut (0.5ms)"], 0, "Geometric gain transfer profile applied across spliced take region boundaries"))
        .with_param(DspParamSchema::choice("comp_lane_audition_solo", "Comp Lane Audition Routing", &["Master Comp Composite", "Solo Lane 1", "Solo Lane 2", "Solo Lane 3", "Solo Lane 4"], 0, "Routes composite comp result or isolated individual audio take directly to channel monitor fader"))
        .with_param(DspParamSchema::choice("splice_boundary_snap_grid", "Splice Boundary Grid Snapping Mode", &["Free Hand (Sample-Accurate)", "1/64 Note Beat Snap", "Transient Hit Boundary", "Zero-Crossing Detection"], 3, "Snaps user comp swipe markers to musical divisions, detected acoustic transients, or waveform zero-crossings"))
        .with_param(DspParamSchema::toggle("non_destructive_slip_editing", "Enable Non-Destructive Take Slip Editing", true, "Permits shifting audio take contents horizontally within fixed comp boundaries without altering splice points"))
    );
    descriptors.insert("LooperTrackManager".to_string(), DspNodeDescriptor::new("LooperTrackManager", "Non-Blocking Multitrack Circular Looper & Overdub Layer Manager HUD", DspNodeCategory::SamplerSlicer, "Multi-track audio looper module with bar-quantized record, overdub, multiply, divide, non-destructive undo/redo stacks, and loop decay")
        .with_param(DspParamSchema::knob("overdub_feedback_decay_pct", "Overdub Layer Feedback Decay Retention", 50.0, 100.0, 100.0, "%", MacroRole::Punch, "Decay coefficient attenuating earlier audio passes during continuous sound-on-sound overdubbing"))
        .with_param(DspParamSchema::choice("loop_quantize_division", "Loop Record & Punch Quantize Grid", &["1 Bar (Strict Measure)", "2 Bars", "4 Bars", "1/4 Beat Freeform", "Free Run (Unquantized)"], 0, "Clock division synchronizing punch-in, punch-out, and playback boundary turnaround"))
        .with_param(DspParamSchema::knob("boundary_crossfade_samples", "Turnaround Splice Crossfade Window", 16.0, 256.0, 64.0, "samples", MacroRole::Space, "Sample-accurate crossfade length applied across circular buffer loop endpoints to prevent boundary clicks"))
        .with_param(DspParamSchema::choice("playback_speed_multiplier", "Loop Playback Rate & Direction", &["1x Normal Speed", "2x Double Speed", "0.5x Half Speed Reverse", "-1x Reverse Playback"], 0, "Resampling speed and direction multiplier applied to loop playback head"))
        .with_param(DspParamSchema::toggle("enable_infinite_overdub_layers", "Non-Destructive Multi-Layer Undo Stack", true, "Preserves independent overdub audio layers in memory enabling multi-step undo and layer mute"))
    );
    descriptors.insert("LooperTrack".to_string(), DspNodeDescriptor::new("LooperTrack", "Real-Time Multitrack Overdub Audio Looper & Layer HUD", DspNodeCategory::SamplerSlicer, "Non-blocking circular audio looper track supporting synchronized overdub layers, half-speed/reverse playback, per-layer decay feedback, and transient crossfade punch-in")
        .with_param(DspParamSchema::choice("loop_quantize_grid", "Audio Looper Punch-In & Punch-Out Quantization Grid", &["1 Bar (Downbeat Synced)", "2 Bars", "4 Bars", "1/4 Note (Beat Synced)", "Free (Unquantized Tap)"], 0, "Tempo grid lock aligning loop record boundaries to musical meter"))
        .with_param(DspParamSchema::knob("overdub_feedback_decay", "Overdub Layer Decay Feedback Retention", 0.0, 1.0, 0.92, "%", MacroRole::Space, "Fraction of previous sound retained with each continuous overdub cycle"))
        .with_param(DspParamSchema::knob("punch_crossfade_duration_ms", "Loop Boundary Punch-In Anti-Click Crossfade Duration", 1.0, 50.0, 8.0, "ms", MacroRole::Punch, "Equal-power crossfade length applied at loop start/end points"))
        .with_param(DspParamSchema::choice("playback_speed_multiplier", "Looper Playback Speed & Octave Transposition", &["1x Normal Speed", "0.5x Half Speed (Octave Down)", "2x Double Speed (Octave Up)", "-1x Reverse Direction"], 0, "Playback rate and direction of looped audio memory buffer"))
        .with_param(DspParamSchema::knob("auto_trim_silence_threshold_db", "Auto-Record Input Signal Detection Threshold", -60.0, -20.0, -42.0, "dB", MacroRole::Tone, "Audio input level above which recording triggers automatically"))
    );
    descriptors.insert("SfzRegionConfig".to_string(), DspNodeDescriptor::new("SfzRegionConfig", "SFZ Multisample Zone & Velocity Split HUD", DspNodeCategory::SamplerSlicer, "SFZ soundfont multisample region definition configuring keyboard key zones, velocity layers, sample root note pitch, sample loop boundaries, and crossfade attenuation")
        .with_param(DspParamSchema::knob("key_low", "Keyboard Zone Lower MIDI Note Limit", 0.0, 127.0, 36.0, "MIDI", MacroRole::Tone, "Lowest MIDI key triggering this sample region"))
        .with_param(DspParamSchema::knob("key_high", "Keyboard Zone Upper MIDI Note Limit", 0.0, 127.0, 48.0, "MIDI", MacroRole::Tone, "Highest MIDI key triggering this sample region"))
        .with_param(DspParamSchema::knob("velocity_low", "Velocity Layer Lower Sensitivity Limit", 1.0, 127.0, 1.0, "vel", MacroRole::Punch, "Minimum strike velocity triggering this layer"))
        .with_param(DspParamSchema::knob("velocity_high", "Velocity Layer Upper Sensitivity Limit", 1.0, 127.0, 127.0, "vel", MacroRole::Punch, "Maximum strike velocity triggering this layer"))
        .with_param(DspParamSchema::knob("sample_tune_cents", "Fine Tuning Pitch Offset Calibration", -100.0, 100.0, 0.0, "cents", MacroRole::Tone, "Pitch fine-tuning deviation in cents for sample playback"))
    );
    descriptors.insert("SampleBuffer".to_string(), DspNodeDescriptor::new("SampleBuffer", "Shared Linear Audio Sample Buffer Cache HUD", DspNodeCategory::SamplerSlicer, "Shared multi-channel audio sample buffer management node exposing memory footprint, channel topology, duration in seconds, sample rate conversion, and peak normalization")
        .with_param(DspParamSchema::knob("buffer_capacity_seconds", "Sample Audio Duration", 0.1, 600.0, 4.5, "s", MacroRole::Space, "Total length of audio sample stored in pre-allocated linear PCM buffer"))
        .with_param(DspParamSchema::knob("sample_rate_khz", "Internal Sampling Rate", 22.05, 192.0, 48.0, "kHz", MacroRole::Tone, "Audio asset recording sample rate determining playback pitch speed"))
        .with_param(DspParamSchema::knob("channel_count", "Audio Channel Configuration", 1.0, 8.0, 2.0, "ch", MacroRole::Punch, "Number of interleaved audio channels (1=Mono, 2=Stereo, 6=Surround)"))
        .with_param(DspParamSchema::knob("memory_consumption_mb", "RAM Allocation Footprint", 0.1, 512.0, 1.7, "MB", MacroRole::Character, "Physical RAM allocation in megabytes dedicated to sample storage"))
        .with_param(DspParamSchema::knob("normalize_peak_dbfs", "Non-Destructive Peak Normalization", -24.0, 0.0, -0.5, "dBFS", MacroRole::Punch, "Gain offset scaling sample buffer peak to target digital headroom ceiling"))
    );
    descriptors.insert("SampleRegion".to_string(), DspNodeDescriptor::new("SampleRegion", "Multi-Sample Instrument Velocity Keyzone Region HUD", DspNodeCategory::SamplerSlicer, "Mapped keyboard and velocity region inside a multi-sampled instrument defining key limits, pitch keycenter root, velocity boundaries, loop modes, and crossfade smoothing")
        .with_param(DspParamSchema::knob("low_key_limit", "Velocity Zone Low MIDI Key", 0.0, 127.0, 36.0, "MIDI", MacroRole::Tone, "Bottom MIDI note number triggering this sampled acoustic layer"))
        .with_param(DspParamSchema::knob("high_key_limit", "Velocity Zone High MIDI Key", 0.0, 127.0, 72.0, "MIDI", MacroRole::Tone, "Top MIDI note number triggering this sampled acoustic layer"))
        .with_param(DspParamSchema::knob("root_key_center", "Sample Root Fundamental Pitch", 0.0, 127.0, 60.0, "MIDI", MacroRole::Tone, "Original pitch at which audio sample was recorded (no pitch shifting required)"))
        .with_param(DspParamSchema::knob("velocity_threshold_low", "Low Velocity Trigger Threshold", 1.0, 127.0, 1.0, "vel", MacroRole::Punch, "Minimum strike velocity required to switch to this dynamic sample layer"))
        .with_param(DspParamSchema::knob("loop_crossfade_duration_ms", "Loop Region Boundary Equal-Power Crossfade", 0.0, 500.0, 25.0, "ms", MacroRole::Space, "Micro-fade crossfade duration eliminating click transients at loop boundaries"))
    );
    descriptors.insert("CloudGrain".to_string(), DspNodeDescriptor::new("CloudGrain", "Asynchronous Granular Cloud Grain Engine HUD", DspNodeCategory::SamplerSlicer, "Individual micro-grain state inside an asynchronous granular cloud engine managing sample read playhead, duration, playback pitch ratio, stereo pan spread, and Tukey envelope taper")
        .with_param(DspParamSchema::knob("grain_duration_ms", "Grain Window Duration", 5.0, 500.0, 50.0, "ms", MacroRole::Space, "Temporal length of single synthesized audio micro-grain in milliseconds"))
        .with_param(DspParamSchema::knob("pitch_ratio", "Micro-Grain Playback Pitch Speed Ratio", 0.25, 4.0, 1.0, "x", MacroRole::Tone, "Pitch transposing playback rate factor (1.0 = original pitch, 2.0 = octave up)"))
        .with_param(DspParamSchema::knob("stereo_pan_spread", "Grain Stereo Spatial Panning Position", -1.0, 1.0, 0.0, "pan", MacroRole::Space, "Stereo position coordinate (-1.0 = hard left, +1.0 = hard right)"))
        .with_param(DspParamSchema::knob("grain_amplitude_peak", "Individual Grain Peak Amplitude", 0.0, 2.0, 0.8, "amp", MacroRole::Punch, "Peak amplitude gain envelope scaling factor for this grain instance"))
        .with_param(DspParamSchema::knob("tukey_alpha", "Tukey Window Taper Envelope Slope", 0.0, 1.0, 0.5, "alpha", MacroRole::Character, "Cosine taper fraction of Tukey window (0.0 = rectangular, 1.0 = Hann window)"))
    );
    descriptors.insert("SampleMarker".to_string(), DspNodeDescriptor::new("SampleMarker", "Non-Destructive Sample Boundary Transient Marker HUD", DspNodeCategory::SamplerSlicer, "Sample editor boundary marker managing sample position offset, zero-crossing snapping, micro-fade crossfade duration, transient peak detection threshold, and regional gain trim")
        .with_param(DspParamSchema::knob("marker_position_samples", "Marker Boundary Offset in Samples", 0.0, 10000000.0, 24000.0, "samples", MacroRole::Space, "Absolute sample frame index marking cue point, slice split, or loop boundary"))
        .with_param(DspParamSchema::toggle("snap_to_zero_crossing", "Snap Marker to Nearest Zero-Crossing", true, "Prevents DC pops and clicks by aligning marker to 0.0 amplitude sample zero crossings"))
        .with_param(DspParamSchema::knob("marker_crossfade_ms", "Marker Region Boundary Crossfade", 0.0, 100.0, 5.0, "ms", MacroRole::Space, "Micro-fade crossfade window duration applied when playback crosses this marker"))
        .with_param(DspParamSchema::knob("transient_threshold_db", "Transient Peak Detection Threshold", -48.0, 0.0, -12.0, "dB", MacroRole::Punch, "Transient detector sensitivity threshold auto-snapping marker to percussive hits"))
        .with_param(DspParamSchema::knob("marker_gain_trim_db", "Regional Gain Trim Offset", -24.0, 12.0, 0.0, "dB", MacroRole::Tone, "Gain trim offset applied to sample region starting at this marker"))
    );
    descriptors.insert("Grain".to_string(), DspNodeDescriptor::new("Grain", "Microscopic Granular Audio Resynthesis Particle", DspNodeCategory::SamplerSlicer, "Micro-acoustic granular synthesis particle with Hann envelope windowing, stereo pan displacement, playback speed transposition, and reverse direction")
        .with_param(DspParamSchema::knob("grain_duration_ms", "Particle Grain Duration", 2.0, 500.0, 45.0, "ms", MacroRole::Tone, "Temporal lifespan of an individual granular synthesis burst"))
        .with_param(DspParamSchema::knob("playback_rate", "Grain Playback Rate Pitch Transpose", 0.125, 4.0, 1.0, "x", MacroRole::Tone, "Resampling playback speed factor controlling micro-pitch"))
        .with_param(DspParamSchema::choice("window_shape", "Windowing Envelope Function", &["Hann Raised Cosine", "Gaussian", "Blackman-Harris", "Tukey 10%", "Rectangular"], 0, "Mathematical window envelope shaping grain attack and decay"))
        .with_param(DspParamSchema::knob("stereo_pan", "Spatial Stereo Pan Position", -1.0, 1.0, 0.0, "pan", MacroRole::Space, "Stereo panoramic placement of individual synthesized grain"))
        .with_param(DspParamSchema::toggle("reverse_playback", "Reverse Grain Time Direction", false, "Reverses buffer playback direction for stochastic backwards textures"))
    );
    descriptors.insert("DrumPad".to_string(), DspNodeDescriptor::new("DrumPad", "Multi-Velocity Layer Acoustic & Electronic Drum Pad", DspNodeCategory::SamplerSlicer, "Dynamic drum sampler voice pad featuring multi-velocity velocity split layering, round-robin sample switching, transient choke grouping, and pitch tune")
        .with_param(DspParamSchema::knob("tune_coarse_semitones", "Drum Pad Pitch Tuning", -24.0, 24.0, 0.0, "st", MacroRole::Tone, "Coarse chromatic transposition of drum voice sample playback"))
        .with_param(DspParamSchema::knob("velocity_sensitivity", "Dynamic Velocity Impact Sensitivity", 0.0, 1.0, 0.85, "norm", MacroRole::Punch, "Maps MIDI note velocity to dynamic sample layer and filter cutoff"))
        .with_param(DspParamSchema::knob("decay_time_ms", "Drum Envelope Amplitude Decay", 10.0, 5000.0, 450.0, "ms", MacroRole::Tone, "Natural ringing decay time of drum membrane resonance"))
        .with_param(DspParamSchema::knob("choke_group", "Mute Choke Exclusive Group ID", 0.0, 16.0, 1.0, "grp", MacroRole::Character, "Exclusive choke group muting previous voice on trigger (e.g. Hi-Hat)"))
        .with_param(DspParamSchema::knob("round_robin_count", "Alternate Hit Round-Robin Variation Count", 1.0, 8.0, 4.0, "hits", MacroRole::Tone, "Cycles sequential sample variations to prevent machine-gun effect"))
    );
    descriptors.insert("SampleSlice".to_string(), DspNodeDescriptor::new("SampleSlice", "Zero-Crossing Audio Sample Transient Slice", DspNodeCategory::SamplerSlicer, "Non-destructive audio slice segment with precise sample start and end boundaries, transient attack gain, playback loop mode, and pitch repitch ratio")
        .with_param(DspParamSchema::knob("start_frame", "Slice In-Point Audio Frame", 0.0, 10000000.0, 0.0, "frames", MacroRole::Tone, "Sample offset marking boundary where slice playback begins"))
        .with_param(DspParamSchema::knob("end_frame", "Slice Out-Point Audio Frame", 1.0, 10000000.0, 48000.0, "frames", MacroRole::Tone, "Sample offset marking boundary where slice playback terminates"))
        .with_param(DspParamSchema::knob("playback_pitch_ratio", "Slice Chromatic Resampling Pitch Ratio", 0.25, 4.0, 1.0, "x", MacroRole::Tone, "Pitch transposition multiplier for this isolated slice"))
        .with_param(DspParamSchema::knob("transient_gain_db", "Transient Attack Punch Gain", -12.0, 12.0, 0.0, "dB", MacroRole::Punch, "Dedicated volume trim boosting the transient strike envelope"))
        .with_param(DspParamSchema::choice("loop_mode", "Segment Loop Playback Direction", &["One-Shot Forward", "Forward Loop", "Ping-Pong Bi-Directional", "Reverse One-Shot"], 0, "Looping behavior executed when slice gate remains held"))
    );
    descriptors.insert("DrumMacroView".to_string(), DspNodeDescriptor::new("DrumMacroView", "Percussion Kit Master Gain & Pad Mute Controller", DspNodeCategory::SamplerSlicer, "Performance controller and macro surface for zero-allocation 24-pad drum machine kit managing master output gain, individual pad mutes, and dynamic choke group policies")
        .with_param(DspParamSchema::knob("master_gain", "Global Kit Master Gain", 0.0, 1.5, 0.85, "%", MacroRole::Punch, "Global drum kit audio output level multiplier"))
        .with_param(DspParamSchema::toggle("kick_mute", "Pad 0 Kick Drum Mute", false, "Pad 0 Kick Drum channel mute state"))
        .with_param(DspParamSchema::toggle("snare_mute", "Pad 1 Snare Drum Mute", false, "Pad 1 Snare Drum channel mute state"))
        .with_param(DspParamSchema::toggle("hihat_mute", "Pad 2 Hi-Hat Mute", false, "Pad 2 Hi-Hat channel mute state"))
        .with_param(DspParamSchema::choice("choke_mode", "Dynamic Voice Choke Policy", &["Exclusive Choke Group", "Polyphonic Ringing", "Round-Robin Layered", "Velocity Choked"], 0, "Choking strategy preventing overlapping cymbal and percussion resonances"))
    );
    descriptors.insert("SamplerMacroView".to_string(), DspNodeDescriptor::new("SamplerMacroView", "Multisample Engine Macro Performance Strip", DspNodeCategory::SamplerSlicer, "Top-tier macro performance surface for polyphonic multisample playback engine controlling filter cutoff, resonance, ADSR attack and release envelopes, and gain scaling")
        .with_param(DspParamSchema::log_knob("filter_cutoff", "SVF Low-Pass Cutoff Frequency", 20.0, 20000.0, 4500.0, "Hz", MacroRole::Tone, "State-variable lowpass filter cutoff frequency for sample playback"))
        .with_param(DspParamSchema::knob("filter_res", "Filter Resonance Emphasis Q", 0.0, 4.0, 0.8, "Q", MacroRole::Character, "Filter resonance emphasis at cutoff frequency"))
        .with_param(DspParamSchema::knob("amp_attack", "Amplitude Envelope Attack Time", 0.001, 5.0, 0.015, "s", MacroRole::Punch, "Amplitude envelope attack ramp duration"))
        .with_param(DspParamSchema::knob("amp_release", "Amplitude Envelope Release Time", 0.01, 10.0, 0.45, "s", MacroRole::Space, "Amplitude envelope release decay duration after key release"))
        .with_param(DspParamSchema::knob("gain", "Multisample Volume Scaling", 0.0, 1.5, 1.0, "%", MacroRole::Tone, "Multisample playback engine output level scaler"))
    );
    descriptors.insert("SliceMarker".to_string(), DspNodeDescriptor::new("SliceMarker", "Audio Waveform Zero-Crossing Transient Cut Marker", DspNodeCategory::SamplerSlicer, "Non-destructive transient slice boundary marker specifying start and end sample frames with automated zero-crossing anti-click splicing and independent chromatic pitch shifting")
        .with_param(DspParamSchema::knob("start_sample", "Slice In-Point Sample Frame", 0.0, 10000000.0, 0.0, "samples", MacroRole::Tone, "Sample frame index where slice playback begins"))
        .with_param(DspParamSchema::knob("end_sample", "Slice Out-Point Sample Frame", 1.0, 10000000.0, 24000.0, "samples", MacroRole::Tone, "Sample frame index where slice playback terminates"))
        .with_param(DspParamSchema::knob("crossfade_samples", "Anti-Click Splice Crossfade Window", 0.0, 512.0, 32.0, "samples", MacroRole::Space, "Zero-crossing anti-click micro-crossfade smoothing window"))
        .with_param(DspParamSchema::knob("pitch_semitones", "Slice Pitch Transposition", -24.0, 24.0, 0.0, "st", MacroRole::Character, "Independent chromatic semitone transposition for isolated slice"))
        .with_param(DspParamSchema::toggle("snap_to_zero_crossing", "Auto Snap Zero-Crossing Boundary", true, "Snap marker boundaries automatically to closest audio waveform zero crossing"))
    );
    descriptors.insert("WarpMarker".to_string(), DspNodeDescriptor::new("WarpMarker", "Elastic Time-Stretch Transient Warp Anchor", DspNodeCategory::SamplerSlicer, "Elastic audio warping anchor linking original source buffer sample frame coordinates with timeline beat targets, managing local stretch ratios and formant preservation")
        .with_param(DspParamSchema::knob("source_frame", "Source Audio Buffer Anchor Frame", 0.0, 10000000.0, 0.0, "frames", MacroRole::Tone, "Unaltered source audio sample frame anchor position"))
        .with_param(DspParamSchema::knob("target_frame", "Warped Beat Grid Destination Frame", 0.0, 10000000.0, 0.0, "frames", MacroRole::Tone, "Warped timeline destination frame locked to musical beat grid"))
        .with_param(DspParamSchema::knob("time_stretch_ratio", "Local Time-Stretch Speed Ratio", 0.25, 4.0, 1.0, "x", MacroRole::Punch, "Local time-stretch compression or expansion speed factor"))
        .with_param(DspParamSchema::toggle("pitch_lock", "Preserve Spectral Formant Envelope", true, "Lock spectral pitch formant envelope during aggressive time stretching"))
        .with_param(DspParamSchema::knob("transient_sensitivity", "Peak Detection Smear Sensitivity", 0.1, 1.0, 0.85, "%", MacroRole::Character, "Phase vocoder transient smearing prevention detection sensitivity"))
    );
    descriptors.insert("LoopMode".to_string(), DspNodeDescriptor::new("LoopMode", "Polyphonic Audio Sampler Waveform Looping & Direction Mode", DspNodeCategory::SamplerSlicer, "Polyphonic sample playback loop manager configuring forward looping, alternating ping-pong bounce, continuous background cycling, and release tail looping with zero-crossing auto-snap and equal-power crossfading")
        .with_param(DspParamSchema::choice("sampler_loop_direction_mode", "Audio Buffer Looping Trajectory", &["No Loop (One-Shot Playback)", "Loop Forward (Standard Sustained Loop)", "Loop Continuous (Infinite Background Cycle)", "Ping-Pong (Forward-Reverse Alternation)", "Loop on Release (Trigger Tail Loop)"], 1, "Loop playback direction and trigger mode"))
        .with_param(DspParamSchema::knob("loop_start_marker_samples", "Loop Cycle In-Point Sample Boundary", 0.0, 960000.0, 44100.0, "smp", MacroRole::Punch, "Sample offset marking the beginning of loop cycle"))
        .with_param(DspParamSchema::knob("loop_end_marker_samples", "Loop Cycle Out-Point Sample Boundary", 0.0, 960000.0, 88200.0, "smp", MacroRole::Tone, "Sample offset marking the termination of loop cycle"))
        .with_param(DspParamSchema::knob("crossfade_smoothing_duration_ms", "Loop Boundary Equal-Power Crossfade Time", 0.0, 500.0, 25.0, "ms", MacroRole::Character, "Duration of crossfade smoothing seam between loop end and start"))
        .with_param(DspParamSchema::knob("crossfade_curve_power", "Crossfade Interpolation Curve Curvature", 0.1, 3.0, 1.0, "pow", MacroRole::Space, "Power curve ensuring constant perceived volume across seam"))
        .with_param(DspParamSchema::toggle("enable_zero_crossing_auto_snap", "Auto-Snap Loop Markers to Nearest Zero-Crossing", true, "Prevent clicks by locking boundaries to phase zero-crossing samples"))
    );
    descriptors.insert("SliceAlgorithm".to_string(), DspNodeDescriptor::new("SliceAlgorithm", "Intelligent Audio Slicing & Transient Detection Engine", DspNodeCategory::SamplerSlicer, "Sample slicing transient detector utilizing energy derivative calculation, high-resolution spectral flux FFT bin tracking, or deep neural ONNX machine learning model weights")
        .with_param(DspParamSchema::choice("transient_detection_algorithm", "Transient Detection Mathematical Model", &["Energy Derivative (Fast Time-Domain Envelope Slope)", "Spectral Flux (FFT Spectral Magnitude Difference)", "ONNX Neural Network (Deep Learned Drum & Transient Classifier)"], 1, "Selected mathematical engine detecting rhythm markers"))
        .with_param(DspParamSchema::knob("detection_sensitivity_threshold", "Transient Detection Sensitivity Threshold", 0.01, 1.0, 0.35, "%", MacroRole::Punch, "Minimum transient spike amplitude required to place slice marker"))
        .with_param(DspParamSchema::knob("minimum_slice_distance_ms", "Minimum Inter-Slice Time Gap Interval", 10.0, 500.0, 50.0, "ms", MacroRole::Tone, "Enforce minimum temporal spacing preventing duplicate triggers"))
        .with_param(DspParamSchema::knob("transient_lookahead_padding_ms", "Pre-Transient Marker Safety Padding", 1.0, 25.0, 5.0, "ms", MacroRole::Character, "Offset slice start point backwards to capture attack transient intact"))
        .with_param(DspParamSchema::knob("slice_fade_out_time_ms", "Slice Playback Micro-Fade Out Envelope", 1.0, 50.0, 8.0, "ms", MacroRole::Space, "Micro-fade applied to tail of slice preventing audio pops"))
        .with_param(DspParamSchema::toggle("enable_zero_crossing_snap", "Auto-Snap Slice Markers to Nearest Zero Crossing", true, "Shift markers to closest zero phase sample for clean playback"))
    );
    descriptors.insert("LooperState".to_string(), DspNodeDescriptor::new("LooperState", "Multi-Track Live Performance Looper State Machine", DspNodeCategory::SamplerSlicer, "Live looping engine tracking track operational states across Empty, Pending Record, Recording, Pending Play, Playing, Pending Overdub, Overdubbing, Paused, and Muted with infinite undo/redo")
        .with_param(DspParamSchema::choice("track_operational_state", "Looper Track Real-Time Lifecycle State", &["Empty (Track Clear Ready for Initial Record)", "Pending Record (Quantized Arming on Next Downbeat)", "Recording (Capturing Master Loop Boundary Length)", "Pending Play (Finishing Recording at Next Measure)", "Playing (Looping Captured Audio Continuously)", "Pending Overdub (Arming Overdub on Next Downbeat)", "Overdubbing (Layering Sound-on-Sound Audio)", "Paused (Playhead Frozen at Current Position)", "Muted (Silent Sync Tracking with Master Tempo)"], 4, "Current operational state of live looping engine"))
        .with_param(DspParamSchema::knob("feedback_overdub_decay", "Overdub Feedback / Sound-on-Sound Decay", 0.0, 1.0, 0.95, "%", MacroRole::Space, "Decay of previous loop layers when adding new overdub audio"))
        .with_param(DspParamSchema::knob("loop_master_tempo_bpm", "Captured Loop Master Tempo BPM", 40.0, 280.0, 120.0, "BPM", MacroRole::Tone, "Calculated tempo derived from initial master loop length"))
        .with_param(DspParamSchema::knob("crossfade_boundary_ms", "Loop Seam Equal-Power Crossfade Time", 1.0, 50.0, 10.0, "ms", MacroRole::Punch, "Equal-power crossfade smoothing loop wrap-around boundary"))
        .with_param(DspParamSchema::knob("pitch_playback_speed_multiplier", "Loop Playback Speed & Pitch Transposition", 0.25, 4.0, 1.0, "x", MacroRole::Character, "Varispeed tape multiplier altering loop playback speed and pitch"))
        .with_param(DspParamSchema::toggle("enable_infinite_undo_history", "Retain Multi-Take Non-Destructive Undo Layers", true, "Preserve discrete overdub passes in memory for non-destructive undo"))
    );
    descriptors.insert("LooperCommand".to_string(), DspNodeDescriptor::new("LooperCommand", "Live Performance Looper State Machine Quantized Action Trigger", DspNodeCategory::SamplerSlicer, "Quantized action dispatcher triggering Record, Play, Overdub, Multiply, Divide, Undo, Redo, Mute, Reverse, and Half/Double-speed commands in the live looper")
        .with_param(DspParamSchema::choice("dispatched_command_action", "Quantized Looper Command Action Trigger", &["TriggerRecord (Arm/Execute Initial Recording Pass)", "TriggerPlay (Transition from Recording into Looping)", "TriggerOverdub (Arm/Execute Sound-on-Sound Layer)", "TriggerMultiply (Double Effective Master Loop Duration)", "TriggerDivide (Halve Loop Duration to Shorter Cycle)", "TriggerUndo (Roll Back Most Recent Audio Overdub Layer)", "TriggerRedo (Re-apply Most Recently Undone Overdub)", "TriggerClear (Erase Loop Buffer Completely)", "TriggerToggleMute (Silence Output While Tracking Phase)", "TriggerReverse (Invert Trajectory of Buffer Playhead)", "TriggerHalfSpeed (Slow Playback Down One Octave)", "TriggerDoubleSpeed (Double Playback Speed Up One Octave)"], 0, "Discrete quantized state transition trigger dispatched to looper"))
        .with_param(DspParamSchema::knob("quantized_execution_alignment", "Command Transport Execution Timing", 0.0, 4.0, 1.0, "quant", MacroRole::Punch, "Musical beat grid alignment locking command execution to downbeat"))
        .with_param(DspParamSchema::knob("track_multiply_divide_ratio", "Loop Multiply / Divide Integer Scaling", 1.0, 16.0, 2.0, "x", MacroRole::Character, "Integer scaling factor multiplying or dividing looper buffer length"))
        .with_param(DspParamSchema::knob("reverse_turnaround_fade_ms", "Reverse Trajectory Boundary Equal-Power Fade", 1.0, 100.0, 15.0, "ms", MacroRole::Space, "Crossfade smoothing boundary click when reversing playback"))
        .with_param(DspParamSchema::knob("playback_speed_octave_transposition", "Looper Playback Speed Octave Transposition", -2.0, 2.0, 0.0, "oct", MacroRole::Tone, "Tape-style speed multiplier shifting playback pitch and speed"))
        .with_param(DspParamSchema::toggle("enable_instant_seamless_overdub", "Seamless Instant Overdub Punch-In Transition", true, "Punch in directly from playback to overdub without gap"))
    );
    descriptors.insert("LooperQuantize".to_string(), DspNodeDescriptor::new("LooperQuantize", "Beat & Bar Quantization Interval Alignment for Live Loopers", DspNodeCategory::SamplerSlicer, "Transport quantization grid alignment ensuring live loop capture and boundary turnaround snaps cleanly to beat and bar boundaries")
        .with_param(DspParamSchema::choice("quantization_interval_grid", "Quantization Alignment Time Metric Grid", &["None (Instant Unquantized Free-Running Execution)", "EighthBeat (0.125 Beat Micro-Quantize Alignment)", "QuarterBeat (0.25 Beat 16th-Note Subdivision Alignment)", "WholeBeat (1.0 Beat Quarter-Note Transport Grid)", "WholeBar (4.0 Beats Standard Measure Downbeat Snap)"], 2, "Time metric grid locking looper punch-in/out to master transport"))
        .with_param(DspParamSchema::knob("beat_subdivision_fraction", "Quantized Beat Subdivision Duration", 0.25, 8.0, 1.0, "beats", MacroRole::Punch, "Fractional beat duration locking trigger execution window"))
        .with_param(DspParamSchema::knob("bar_boundary_interval", "Whole Bar Turnaround Repetition Length", 1.0, 16.0, 1.0, "bars", MacroRole::Character, "Whole musical measure count defining turnaround boundary"))
        .with_param(DspParamSchema::knob("preroll_latency_lead_ms", "Input Latency Trigger Pre-Roll Compensation", 0.0, 100.0, 10.0, "ms", MacroRole::Space, "Audio interface buffer latency lead compensation for tight loop seams"))
        .with_param(DspParamSchema::knob("turnaround_crossfade_samples", "Loop Turnaround Seam Crossfade Sample Buffer", 4.0, 512.0, 32.0, "smp", MacroRole::Tone, "Buffer sample length crossfaded across loop seam turnaround"))
        .with_param(DspParamSchema::toggle("enable_master_transport_phase_lock", "Lock Playhead Phase to DAW Master Transport", true, "Maintain continuous phase alignment with global song bar counter"))
    );
    descriptors.insert("LooperSyncMode".to_string(), DspNodeDescriptor::new("LooperSyncMode", "Multitrack Looper Independent vs Master-Locked Synchronization Mode", DspNodeCategory::SamplerSlicer, "Synchronization controller coordinating multitrack looper lanes across free-running, master tempo reference, and slave phase alignment")
        .with_param(DspParamSchema::choice("multitrack_synchronization_mode", "Looper Track Clock Alignment Hierarchy", &["Free (Independent Free-Running Loop Buffers)", "Master (Master Clock Defining Song Tempo & Bar Length)", "Slave (Phase-Locked Follower Synchronized to Master)"], 1, "Clock relationship between individual multitrack looper lanes"))
        .with_param(DspParamSchema::knob("tempo_drift_tolerance_bpm", "Auto-Resync Phase Drift Tolerance Window", 0.01, 5.0, 0.50, "BPM", MacroRole::Punch, "Acceptable tempo drift deviation before auto-resync triggers"))
        .with_param(DspParamSchema::knob("master_lane_assignment_index", "Master Reference Looper Channel Index", 0.0, 15.0, 0.0, "track", MacroRole::Character, "Track channel index functioning as master clock source"))
        .with_param(DspParamSchema::knob("slave_drift_correction_gain", "Slave Phase Correction Resync Pull Gain", 0.01, 1.0, 0.15, "gain", MacroRole::Space, "Rate at which slave playheads adjust phase to match master"))
        .with_param(DspParamSchema::knob("varispeed_interpolation_slew_ms", "Varispeed Pitch Correction Slew Smooth Time", 1.0, 200.0, 20.0, "ms", MacroRole::Tone, "Slew rate smoothing pitch adjustments during phase resync"))
        .with_param(DspParamSchema::toggle("enable_time_stretch_loop_preservation", "Maintain Pitch During Master Tempo Changes", true, "Apply WSOLA time stretching when master tempo fluctuates"))
    );
    descriptors.insert("SfzLoopMode".to_string(), DspNodeDescriptor::new("SfzLoopMode", "Multi-Sample SFZ Region Sustain Looping & Crossfade Envelope", DspNodeCategory::SamplerSlicer, "High-fidelity audio sample looping engine implementing NoLoop, Continuous Forward, Sustain Hold, and Ping-Pong bidirectional loop modes with equal-power boundary crossfading")
        .with_param(DspParamSchema::choice("region_loop_playback_mode", "SFZ Sample Region Sustain Loop Mode", &["NoLoop (One-Shot Decay to Sample End)", "LoopContinuous (Infinite Forward Loop to Key Release)", "LoopSustain (Loop While Key Held, Play Tail on Release)", "LoopBidirectional (Ping-Pong Forward/Reverse Oscillating Loop)"], 1, "Sustain looping behavior for mapped multisample instrument zones"))
        .with_param(DspParamSchema::knob("loop_crossfade_duration_ms", "Loop Boundary Equal-Power Crossfade Time", 0.0, 500.0, 25.0, "ms", MacroRole::Tone, "Smoothing transition duration between loop end and loop start"))
        .with_param(DspParamSchema::knob("loop_start_offset_samples", "Loop Cycle Start Sample Index Offset", 0.0, 1000000.0, 4410.0, "spl", MacroRole::Punch, "Beginning sample point of the repeating sustain region"))
        .with_param(DspParamSchema::knob("loop_end_offset_samples", "Loop Cycle End Sample Index Boundary", 1000.0, 10000000.0, 88200.0, "spl", MacroRole::Character, "Terminal sample boundary concluding each repeating loop iteration"))
        .with_param(DspParamSchema::knob("loop_tune_fine_cents", "Loop Segment Micro-Pitch Tuning Correction", -50.0, 50.0, 0.0, "cents", MacroRole::Space, "Cent-accurate pitch trim eliminating frequency beating during looping"))
        .with_param(DspParamSchema::toggle("enable_equal_power_loop_crossfade", "Enable Sinusoidal Equal-Power Crossfading", true, "Maintain constant perceived acoustic energy across loop transition seam"))
    );
    descriptors.insert("GrainWindowType".to_string(), DspNodeDescriptor::new("GrainWindowType", "Granular Cloud Synthesis Windowing Envelope Function", DspNodeCategory::SamplerSlicer, "Window envelope weighting function shaper modulating individual grain attack and decay skirts across Hann, Tukey, Blackman, Hamming, Trapezoidal, Sine, Gaussian, and Welch geometries")
        .with_param(DspParamSchema::choice("window_function", "Grain Temporal Windowing Envelope Function", &["Hann", "Tukey", "Blackman", "Hamming", "Trapezoidal", "Sine", "Gaussian", "Welch"], 0, "Selects envelope geometry shaping grain onset transients and spectral side-lobe leakage"))
        .with_param(DspParamSchema::knob("tukey_alpha", "Tukey Window Flat-Top Cosine Taper Alpha", 0.0, 1.0, 0.5, "alpha", MacroRole::Tone, "Fraction of window duration dedicated to cosine taper edges versus flat sustaining center"))
        .with_param(DspParamSchema::knob("gaussian_sigma", "Gaussian Standard Deviation Bell Width", 0.1, 1.0, 0.35, "sigma", MacroRole::Space, "Standard deviation parameter governing bell-curve sharpness and smooth transient decay"))
        .with_param(DspParamSchema::knob("grain_overlap_ratio", "Inter-Grain Window Overlap Density", 1.0, 16.0, 4.0, "x", MacroRole::Punch, "Number of concurrent overlapping grains required for constant-power amplitude sum"))
        .with_param(DspParamSchema::knob("spectral_leakage_suppression", "Side-Lobe Spectral Leakage Rejection", 12.0, 96.0, 48.0, "dB", MacroRole::Tone, "Attenuation of side-lobe frequency splatter preventing metallic granular buzzing"))
        .with_param(DspParamSchema::knob("cosine_taper_symmetry", "Envelope Attack / Release Asymmetry Ratio", 0.0, 1.0, 0.5, "sym", MacroRole::Space, "Skews window center to create sharp percussive attack with long exponential decay tail"))
    );
    descriptors.insert("CliAutoSlicer".to_string(), DspNodeDescriptor::new("CliAutoSlicer", "Spectral Flux Transient Auto-Slice Marker Generator", DspNodeCategory::SamplerSlicer, "Intelligent sample slicer analyzing spectral flux, high-frequency energy content, and phase deviation to place zero-crossing slice markers for drum hits and loops")
        .with_param(DspParamSchema::choice("slicing_algorithm", "Transient Detection Mathematical Algorithm", &["Spectral Flux", "High-Frequency Content (HFC)", "Complex Phase Deviation", "Energy Onset Envelope"], 0, "DSP onset detection function used to identify percussive transients"))
        .with_param(DspParamSchema::knob("detection_threshold", "Transient Peak Sensitivity Threshold", 0.01, 1.0, 0.15, "thresh", MacroRole::Punch, "Sensitivity of onset peak detector determining number of generated slices"))
        .with_param(DspParamSchema::knob("min_slice_distance_ms", "Minimum Temporal Distance Between Slice Points", 10.0, 500.0, 50.0, "ms", MacroRole::Tone, "Refractory period preventing multiple triggers on a single acoustic impact"))
        .with_param(DspParamSchema::toggle("snap_to_zero_crossing", "Snap Detected Slices to Nearest Signal Zero Crossing", true, "Adjusts slice boundaries to exact zero amplitude crossing to prevent click artifacts"))
        .with_param(DspParamSchema::knob("pre_transient_padding_ms", "Pre-Transient Lookahead Attack Margin", 0.0, 20.0, 2.5, "ms", MacroRole::Space, "Safety margin before transient peak preserving initial attack click"))
        .with_param(DspParamSchema::toggle("export_slice_metadata", "Write Sliced Drum Kit TOML Map for SamplerDevice", true, "Outputs ready-to-play multi-pad drum instrument configuration"))
    );
    descriptors.insert("GranularCloudTextureMorpher".to_string(), DspNodeDescriptor::new("GranularCloudTextureMorpher", "Asynchronous Multi-Head Granular Cloud & Texture Generator", DspNodeCategory::SamplerSlicer, "High-density granular synthesizer generating thousands of overlapping micro-grains with stochastic grain duration, pitch jitter, position scrubbing, and 3D spatial dispersion")
        .with_param(DspParamSchema::knob("grain_density_cloud_rate_hz", "Granular Cloud Emission Density", 1.0, 200.0, 45.0, "Hz", MacroRole::Punch, "Rate of grain emission creating continuous sonic clouds or rhythmic clicks"))
        .with_param(DspParamSchema::knob("grain_duration_envelope_ms", "Micro-Grain Playback Duration", 2.0, 500.0, 65.0, "ms", MacroRole::Tone, "Individual grain playback window duration"))
        .with_param(DspParamSchema::knob("playback_buffer_position_pct", "Sample Buffer Playback Scrub Offset", 0.0, 100.0, 35.0, "%", MacroRole::Character, "Source sample buffer playback position offset"))
        .with_param(DspParamSchema::knob("grain_pitch_micro_jitter_cents", "Stochastic Grain Pitch Detune Jitter", 0.0, 1200.0, 25.0, "ct", MacroRole::Tone, "Random pitch randomization per grain for luscious chorus smear"))
        .with_param(DspParamSchema::knob("spatial_grain_azimuth_scatter", "3D Spatial Azimuth Dispersion Width", 0.0, 1.0, 0.6, "%", MacroRole::Space, "Stochastic stereo and surround pan dispersion per grain"))
        .with_param(DspParamSchema::toggle("enable_hanning_window_anti_glitch", "Raised Cosine Grain Window Anti-Glitch", true, "Applies smooth Hann cosine windowing to prevent boundary click artifacts"))
    );
    descriptors.insert("GranularFreezeBufferDrone".to_string(), DspNodeDescriptor::new("GranularFreezeBufferDrone", "Infinite Granular Freeze Buffer & Ambient Drone Engine", DspNodeCategory::SamplerSlicer, "Captures live audio into an infinite circular RAM buffer, generating dense ambient soundscapes via asynchronous multi-grain playback with pitch jitter and spatial dispersal")
        .with_param(DspParamSchema::knob("freeze_buffer_duration_s", "Circular Freeze Buffer Memory Duration", 0.5, 10.0, 3.0, "s", MacroRole::Space, "Duration of the cyclic frozen audio RAM buffer"))
        .with_param(DspParamSchema::knob("grain_cloud_density_hz", "Granular Cloud Emission Frequency", 5.0, 120.0, 35.0, "Hz", MacroRole::Punch, "Frequency of grain generation spawning overlapping textures"))
        .with_param(DspParamSchema::knob("grain_playback_window_ms", "Grain Window Envelope Duration", 10.0, 800.0, 120.0, "ms", MacroRole::Tone, "Individual grain duration with raised cosine Hann envelope"))
        .with_param(DspParamSchema::knob("random_pitch_dispersion_cents", "Stochastic Micro-Grain Pitch Jitter", 0.0, 1200.0, 45.0, "ct", MacroRole::Tone, "Stochastic pitch detune applied per micro-grain"))
        .with_param(DspParamSchema::knob("spatial_azimuth_spread_pct", "Stereo Azimuth Pan Cloud Scatter", 0.0, 100.0, 80.0, "%", MacroRole::Space, "Random stereo panorama dispersion of individual grains"))
        .with_param(DspParamSchema::toggle("enable_infinite_freeze_hold", "Continuous Infinite Buffer Freeze Lock", true, "Locks freeze buffer preventing new incoming audio from overwriting drone"))
    );
    descriptors.insert("PhaseAlignedGranularTimeStretcher".to_string(), DspNodeDescriptor::new("PhaseAlignedGranularTimeStretcher", "Phase-Aligned Granular Time & Pitch Stretcher", DspNodeCategory::SamplerSlicer, "High-resolution time-stretching and pitch-shifting engine utilizing phase-synchronized overlapping grains with formant correction and transient preservation")
        .with_param(DspParamSchema::knob("time_stretch_ratio_factor", "Playback Speed Time Stretch Factor", 0.1, 10.0, 1.0, "x", MacroRole::Space, "Playback speed scaling without altering fundamental musical pitch"))
        .with_param(DspParamSchema::knob("pitch_shift_cents_detune", "Chromatic Pitch Transposition Detune", -2400.0, 2400.0, 0.0, "ct", MacroRole::Tone, "Pitch transposition in cents independent of temporal playback speed"))
        .with_param(DspParamSchema::knob("grain_slice_window_duration_ms", "Synchronized Grain Window Size", 10.0, 250.0, 45.0, "ms", MacroRole::Tone, "Temporal duration of individual raised-cosine Hann grain envelopes"))
        .with_param(DspParamSchema::knob("formant_spectral_lock_ratio", "Formant Envelope Preservation Lock", 0.0, 1.0, 0.9, "%", MacroRole::Character, "Spectral envelope tracking preventing unnatural pitch formant shifts"))
        .with_param(DspParamSchema::knob("transient_energy_sensitivity", "Transient Slice Boundary Snapping", 0.0, 1.0, 0.8, "%", MacroRole::Punch, "Aligns grain window boundaries to detected transients avoiding smearing"))
        .with_param(DspParamSchema::toggle("enable_intergrain_phase_locking", "Inter-Grain Phase Synchronization", true, "Synchronizes FFT phase bins across overlapping grain boundaries"))
    );
    descriptors.insert("PhaseAlignedStereoGranularDelay".to_string(), DspNodeDescriptor::new("PhaseAlignedStereoGranularDelay", "Phase-Aligned Stereo Granular Delay", DspNodeCategory::SamplerSlicer, "Granular delay line with zero-crossing grain windowing, pitch transposition, and stereo spatial scattering")
        .with_param(DspParamSchema::knob("grain_duration_ms", "Granular Window Duration Length", 5.0, 500.0, 65.0, "ms", MacroRole::Tone, "Temporal duration of individual audio grain envelopes"))
        .with_param(DspParamSchema::knob("grain_spawn_density_hz", "Grain Generation Emission Rate", 1.0, 100.0, 28.0, "Hz", MacroRole::Punch, "Rate of grain triggering per second"))
        .with_param(DspParamSchema::knob("delay_buffer_offset_ms", "Delay Line Tap Offset Time", 10.0, 5000.0, 375.0, "ms", MacroRole::Space, "Main delay buffer read head offset time"))
        .with_param(DspParamSchema::knob("grain_pitch_semitones", "Grain Playback Pitch Shift", -24.0, 24.0, 0.0, "st", MacroRole::Character, "Speed transposition applied to each grain playback buffer"))
        .with_param(DspParamSchema::knob("stereo_pan_diffusion_spread", "Stereo Spatial Pan Jitter Spread", 0.0, 1.0, 0.65, "%", MacroRole::Space, "Randomized stereo panning dispersion across grain cloud"))
        .with_param(DspParamSchema::toggle("enable_hanning_window_tapering", "Raised Cosine Grain Envelope Taper", true, "Applies smooth Hann tapering preventing edge clicks and clicks"))
    );
    descriptors.insert("AsynchronousGranularCloudSaturator".to_string(), DspNodeDescriptor::new("AsynchronousGranularCloudSaturator", "Asynchronous Granular Cloud Texture Saturator", DspNodeCategory::SamplerSlicer, "High-density stochastic grain emitter combining Poisson grain scheduling, non-linear waveshaping per grain, and asynchronous stereo spatial scattering")
        .with_param(DspParamSchema::knob("grain_emission_rate_hz", "Grain Emission Density Rate", 1.0, 500.0, 80.0, "Hz", MacroRole::Tone, "Average number of micro-sound grains generated per second"))
        .with_param(DspParamSchema::knob("grain_duration_ms", "Individual Grain Window Size", 5.0, 400.0, 45.0, "ms", MacroRole::Space, "Length of each granular time slice"))
        .with_param(DspParamSchema::knob("per_grain_saturation_drive_db", "Per-Grain Non-Linear Saturation", 0.0, 36.0, 12.0, "dB", MacroRole::Punch, "Individual waveshaping drive applied independently inside each grain envelope"))
        .with_param(DspParamSchema::knob("pitch_dispersion_random_cents", "Pitch Scatter Distribution Spread", 0.0, 1200.0, 50.0, "cents", MacroRole::Character, "Random micro-pitch transposition range applied across active grain cloud"))
        .with_param(DspParamSchema::knob("stereo_spatial_dispersion_pct", "Stereo Binaural Spatial Scatter", 0.0, 1.0, 0.75, "%", MacroRole::Space, "Stochastic left/right panning scatter distributing grains across stereo panorama"))
        .with_param(DspParamSchema::toggle("enable_quasi_gaussian_window_taper", "Gaussian Envelope Micro-Windowing", true, "Eliminates boundary edge clicks during high-density grain overlap"))
    );
    descriptors.insert("GranularSpectralGlitchDeconstruct".to_string(), DspNodeDescriptor::new("GranularSpectralGlitchDeconstruct", "Stochastic Spectral Fragment & Time-Splay Glitcher", DspNodeCategory::SamplerSlicer, "Granular stutter and micro-sound deconstruction engine with Poisson grain scheduling, reverse playback probability, buffer time-splay, and pitch hops")
        .with_param(DspParamSchema::knob("shard_emission_rate_hz", "Glitch Grain Emission Rate", 1.0, 250.0, 45.0, "Hz", MacroRole::Tone, "Frequency of micro-sound grain window triggering"))
        .with_param(DspParamSchema::knob("time_splay_jitter_ms", "Buffer Read Pointer Splay Jitter", 0.0, 500.0, 120.0, "ms", MacroRole::Space, "Random temporal displacement from current audio playback playhead"))
        .with_param(DspParamSchema::knob("pitch_random_octave_jump", "Discrete Harmonic Interval Hop", 0.0, 24.0, 12.0, "st", MacroRole::Character, "Permissible pitch transposition intervals (+/- 1 octave, 5ths, minor 7ths)"))
        .with_param(DspParamSchema::knob("reverse_playback_probability_pct", "Reverse Grain Playback Probability", 0.0, 1.0, 0.3, "%", MacroRole::Punch, "Odds of an emitted granular micro-fragment playing backwards"))
        .with_param(DspParamSchema::knob("spectral_smear_window_size", "Spectral Window Blurring Size", 64.0, 4096.0, 1024.0, "samples", MacroRole::Tone, "FFT phase randomizer window length creating metallic spectral wash"))
        .with_param(DspParamSchema::toggle("enable_buffer_stutter_freeze", "Infinite Buffer Stutter Loop Freeze", false, "Freezes circular memory buffer into repeating rhythmic glitch stutter pattern"))
    );
    descriptors.insert("AsynchronousSpectralStutterBuffer".to_string(), DspNodeDescriptor::new("AsynchronousSpectralStutterBuffer", "Asynchronous Spectral Stutter & Freeze Buffer", DspNodeCategory::SamplerSlicer, "Real-time spectral circular buffer with Phase Vocoder time-stretching, frozen spectral magnitude bins, rhythmic tempo-synced repeat slicing, and randomized glitch repeats")
        .with_param(DspParamSchema::knob("spectral_freeze_hold_mix", "Spectral Magnitude Freeze Blend", 0.0, 1.0, 0.0, "freeze", MacroRole::Space, "Locks instantaneous FFT bin amplitudes for infinite crystalline drone sustain"))
        .with_param(DspParamSchema::knob("stutter_slice_division_sub", "Rhythmic Stutter Beat Division", 1.0, 32.0, 8.0, "div", MacroRole::Punch, "Metric subdivider for repeat loops (1/4, 1/8, 1/16, 1/32 notes)"))
        .with_param(DspParamSchema::knob("time_stretch_ratio_factor", "Phase Vocoder Time Stretch Ratio", 0.1, 10.0, 1.0, "x", MacroRole::Tone, "Continuous time expansion without altering pitch fundamentals"))
        .with_param(DspParamSchema::knob("pitch_transposition_cents", "Spectral Pitch Transposition", -2400.0, 2400.0, 0.0, "cents", MacroRole::Character, "Pitch shifting via spectral peak reassignment (+/- 2 octaves)"))
        .with_param(DspParamSchema::knob("stutter_probability_gate", "Stutter Glitch Trigger Probability", 0.0, 1.0, 0.45, "%", MacroRole::Tone, "Likelihood of engaging rhythmic stutter slice on beat downbeats"))
        .with_param(DspParamSchema::toggle("enable_phase_vocoder_transient_bypass", "Transient Attack Phase Lock Preservation", true, "Passes sharp transients around vocoder bins to preserve drum snap"))
    );
    descriptors.insert("GranularAsynchronousReverseSlicer".to_string(), DspNodeDescriptor::new("GranularAsynchronousReverseSlicer", "Granular Asynchronous Time-Reverse Slicer", DspNodeCategory::SamplerSlicer, "Granular time-manipulation processor slicing incoming audio into asynchronous backwards grains with Hann window smoothing, randomized spray jitter, and pitch shifting")
        .with_param(DspParamSchema::knob("grain_duration_length_ms", "Individual Grain Duration Window", 10.0, 500.0, 85.0, "ms", MacroRole::Tone, "Length in milliseconds of each spawned backwards grain"))
        .with_param(DspParamSchema::knob("reverse_playback_probability", "Reverse Grain Playback Probability", 0.0, 1.0, 0.75, "prob", MacroRole::Character, "Probability that an extracted audio grain will play in reverse"))
        .with_param(DspParamSchema::knob("grain_position_spray_jitter_ms", "Grain Inception Spray Jitter", 0.0, 200.0, 35.0, "ms", MacroRole::Space, "Random temporal displacement from current playback playhead"))
        .with_param(DspParamSchema::knob("granular_pitch_shift_semitones", "Grain Microtone Pitch Shift", -24.0, 24.0, 0.0, "st", MacroRole::Punch, "Pitch transposition applied to individual audio grains"))
        .with_param(DspParamSchema::knob("grain_density_overlap_factor", "Simultaneous Grain Cloud Density", 1.0, 16.0, 4.0, "grains", MacroRole::Space, "Number of concurrent overlapping audio grains active at once"))
        .with_param(DspParamSchema::toggle("enable_transient_sync_slicing", "Transient Marker Grain Alignment", true, "Aligns grain inception markers to incoming percussive transients"))
    );
    descriptors.insert("AsynchronousGranularTimeStretchWarper".to_string(), DspNodeDescriptor::new("AsynchronousGranularTimeStretchWarper", "Asynchronous Granular Time-Stretch & Pitch Warper", DspNodeCategory::SamplerSlicer, "Real-time asynchronous granular pitch-shifting and time-stretching engine with grain size jitter, multi-head scatter density, Gaussian window shaping, and reverse grain probability")
        .with_param(DspParamSchema::knob("time_stretch_playback_speed", "Time-Stretch Playback Rate Speed", 0.05, 4.0, 1.0, "rate", MacroRole::Tone, "Playback speed multiplier stretching audio without altering pitch"))
        .with_param(DspParamSchema::knob("pitch_shift_playback_semitones", "Granular Pitch Shift Interval", -24.0, 24.0, 0.0, "semi", MacroRole::Tone, "Pitch shifting interval in chromatic semitones without altering duration"))
        .with_param(DspParamSchema::knob("grain_duration_length_ms", "Individual Grain Duration Window", 5.0, 350.0, 65.0, "ms", MacroRole::Space, "Length of each sampled micro-grain buffer determining texture smoothness vs stutter"))
        .with_param(DspParamSchema::knob("grain_stream_density_overlap", "Simultaneous Grain Cloud Stream Density", 1.0, 32.0, 8.0, "grains", MacroRole::Character, "Number of concurrent overlapping granular voices playing simultaneously"))
        .with_param(DspParamSchema::knob("reverse_grain_playback_probability", "Reverse Grain Playback Probability", 0.0, 1.0, 0.15, "prob", MacroRole::Character, "Statistical chance that an individual grain plays backwards for psychedelic textures"))
        .with_param(DspParamSchema::toggle("enable_stereo_scatter_panning", "Stereo Grain Cloud Scatter Panning", true, "Randomizes stereo spatial position of each newly spawned grain across stereo panorama"))
    );
}

pub(in crate::dsp_node_ui) fn create_node_ui(
    target_name: &str,
) -> Option<Box<dyn DspNodeUi>> {
    let node: Box<dyn DspNodeUi> = match target_name {
        "DrumMachineDevice" => Box::new(
            GenericDspNodeUi::new("DrumMachineDevice", "24-Pad Zero-Allocation Sampling Drum Machine", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_linear("master_volume", "Master Kit Volume", 0.0, 1.0, 0.85, "%", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("pitch_offset", "Kit Pitch Offset", -12.0, 12.0, 0.0, "st", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("pad_decay", "Global Pad Decay Scale", 0.1, 2.0, 1.0, "x", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("dynamic_punch", "Master Dynamic Punch", 0.0, 1.0, 0.6, "%", (0, 230, 118)))
        ),
        "SamplerDevice" => Box::new(
            GenericDspNodeUi::new("SamplerDevice", "Multi-Sample Instrument with ADSR Envelope & Filter", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_log("filter_cutoff", "Ladder Filter Cutoff", 20.0, 20000.0, 18000.0, "Hz", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("filter_res", "Ladder Resonance", 0.0, 3.9, 0.7, "Q", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("amp_attack", "Attack Time", 0.001, 2.0, 0.005, "s", (0, 230, 118)))
                .with_param(DspParamDescriptor::new_linear("amp_release", "Release Time", 0.01, 5.0, 0.4, "s", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("gain", "Master Output Gain", 0.0, 1.0, 0.8, "%", (255, 61, 113)))
        ),
        "MultiSamplerNode" => Box::new(
            GenericDspNodeUi::new("MultiSamplerNode", "Velocity-Layered Multi-Zone Audio Sampler", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_int("root_key", "Center Root Key", 0, 127, 60, "midi", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("velocity_track", "Velocity Sensitivity", 0.0, 1.0, 0.8, "%", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_enum("interpolation", "Resampling Kernel", vec!["Linear".into(), "Hermite Cubic".into(), "Sinc Bandlimited".into()], 1, (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("gain", "Sample Playback Level", 0.0, 1.0, 0.9, "%", (0, 230, 118)))
        ),
        "SampleSlicerNode" => Box::new(
            GenericDspNodeUi::new("SampleSlicerNode", "Transient Beat Slicer & Loop Fragmenter", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_linear("threshold_db", "Transient Slicer Sensitivity", -60.0, 0.0, -24.0, "dB", (180, 195, 215)))
                .with_param(DspParamDescriptor::new_linear("min_slice_ms", "Minimum Slice Duration", 10.0, 500.0, 50.0, "ms", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_bool("bpm_sync", "Snap Slices to Beat Grid", true, (0, 230, 118)))
                .with_param(DspParamDescriptor::new_linear("transient_sensitivity", "Onset Detection Threshold", 0.0, 1.0, 0.7, "%", (255, 171, 0)))
        ),
        "LoopSlicerNode" => Box::new(
            GenericDspNodeUi::new("LoopSlicerNode", "Beat-Synchronized Grid Loop Slicer", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_enum("grid_division", "Beat Division", vec!["1/4 Beat".into(), "1/8 Beat".into(), "1/16 Beat".into(), "1/32 Beat".into(), "Triplet 1/8".into()], 2, (180, 195, 215)))
                .with_param(DspParamDescriptor::new_linear("crossfade_ms", "Slice Edge Crossfade", 0.0, 50.0, 5.0, "ms", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_bool("pitch_match", "Preserve Original Pitch", true, (0, 230, 118)))
                .with_param(DspParamDescriptor::new_bool("reverse_slices", "Reverse Alternate Slices", false, (255, 61, 113)))
        ),
        "AutomatedDrumReplacer" => Box::new(
            GenericDspNodeUi::new("AutomatedDrumReplacer", "AI Automated Drum Replacement Trigger", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_linear("trigger_thresh", "Trigger Threshold", -60.0, 0.0, -24.0, "dB", (16, 185, 129)))
                .with_param(DspParamDescriptor::new_linear("sensitivity", "Sensitivity", 0.0, 1.0, 0.75, "%", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("velocity_track", "Velocity Tracking", 0.0, 1.0, 0.85, "%", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_enum("sample_slot", "Sample Preset", vec!["Acoustic Snare Direct".into(), "808 Sub Kick".into(), "Tight Studio Rim".into(), "Layered Transient".into()], 0, (16, 185, 129)))
        ),
        "AutoSlicer" => Box::new(
            GenericDspNodeUi::new("AutoSlicer", "Intelligent Transient Detector & Sample Slicer", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_linear("threshold", "Transient Peak Threshold", 0.01, 1.0, 0.25, "%", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_enum("algorithm", "Detection Algorithm", vec!["Energy Derivative".into(), "Spectral Flux".into(), "ONNX Neural Model".into()], 1, (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("min_slice_length_ms", "Minimum Slice Window", 5.0, 500.0, 50.0, "ms", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("sensitivity", "Spectral Flux Sensitivity", 0.1, 5.0, 1.5, "x", (245, 158, 11)))
        ),
        "AudioReverse" => Box::new(
            GenericDspNodeUi::new("AudioReverse", "Real-Time Buffer Reverse & Granular Rewind", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_linear("buffer_length_ms", "Reverse Buffer Size", 20.0, 2000.0, 250.0, "ms", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("crossfade_ms", "Boundary Crossfade Slew", 2.0, 50.0, 10.0, "ms", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("playback_speed", "Reverse Speed Multiplier", 0.25, 2.0, 1.0, "x", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_enum("reverse_trigger", "Transport Reverse State", vec!["Forward Through".into(), "Active Reverse".into()], 1, (99, 102, 241)))
        ),
        "DrumReplacementTrigger" => Box::new(
            GenericDspNodeUi::new("DrumReplacementTrigger", "Transient Acoustic Drum Replacement & Multi-Velocity Trigger", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_linear("sensitivity_thresh_db", "Trigger Threshold", -60.0, 0.0, -24.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("retrigger_mask_ms", "Retrigger Mask", 5.0, 200.0, 35.0, "ms", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("sample_blend", "Sample Mix Ratio", 0.0, 1.0, 0.85, "%", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("velocity_tracking", "Dynamic Velocity Scale", 0.0, 1.0, 0.90, "%", (16, 185, 129)))
        ),
        "SampleEditor" => Box::new(
            GenericDspNodeUi::new("SampleEditor", "Interactive Waveform Sample Slicer & Looper", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_linear("loop_start_sec", "Loop Start Position", 0.0, 60.0, 0.0, "s", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("loop_end_sec", "Loop End Position", 0.01, 60.0, 4.0, "s", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_log("crossfade_len_ms", "Loop Crossfade Length", 1.0, 1000.0, 25.0, "ms", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_bool("snap_to_zero", "Snap to Zero Crossing", true, (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("normalize_on_slice", "Auto-Normalize Slices", true, (16, 185, 129)))
        ),
        "MultitrackSessionLooper" => Box::new(
            GenericDspNodeUi::new("MultitrackSessionLooper", "Live Multitrack Looping & Overdubbing DSP Engine", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_linear("feedback", "Overdub Feedback Decay", 0.0, 1.0, 0.95, "%", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("track_count", "Looper Track Count", 1.0, 16.0, 4.0, "tracks", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_log("crossfade_ms", "Boundary Crossfade Length", 1.0, 50.0, 10.0, "ms", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("quantize_bars", "Quantization Boundary", 0.0, 4.0, 2.0, "bars", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("dry_wet", "Dry / Wet Mix", 0.0, 1.0, 1.0, "%", (16, 185, 129)))
        ),
        "SamplerNode" => Box::new(
            GenericDspNodeUi::new("SamplerNode", "Multi-Layer Velocity-Switched Audio Sample Playback Engine", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_enum("interpolation_mode", "Resampling Interpolation", vec!["Linear (Fast)".into(), "Hermite Cubic".into(), "Sinc 32-Point".into(), "Sinc 64-Point (Pristine)".into()], 1, (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("playback_speed", "Playback Speed Factor", 0.1, 4.0, 1.0, "x", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("root_pitch_semitones", "Root Key Offset", -48.0, 48.0, 0.0, "st", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_bool("loop_enabled", "Loop Segment Active", true, (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("loop_crossfade_ms", "Loop Crossfade Duration", 0.0, 250.0, 25.0, "ms", (16, 185, 129)))
                .with_param(DspParamDescriptor::new_linear("velocity_tracking", "Velocity Dynamic Gain Scaling", 0.0, 1.0, 0.75, "%", (236, 72, 153)))
        ),
        "ClipMatrix" => Box::new(
            GenericDspNodeUi::new("ClipMatrix", "Non-Linear Session View Clip Launcher Matrix", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_enum("launch_quantize", "Launch Quantization Grid", vec!["1 Bar".into(), "1/2 Bar".into(), "1/4 Note".into(), "1/8 Note".into(), "1/16 Note".into(), "None".into()], 0, (16, 185, 129)))
                .with_param(DspParamDescriptor::new_bool("scene_tempo_sync", "Sync Scene Tempo Bpm", true, (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("follow_action_prob", "Follow Action Probability", 0.0, 1.0, 0.80, "%", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("crossfade_curve", "Clip Switch Crossfade", 0.0, 100.0, 10.0, "ms", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_bool("loop_continuous", "Seamless Audio Loop", true, (168, 85, 247)))
        ),
        "CompTrack" => Box::new(
            GenericDspNodeUi::new("CompTrack", "Multi-Take Audio & Vocal Comping Crossfade Engine", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_linear("crossfade_time_ms", "Take Crossfade Duration", 1.0, 100.0, 15.0, "ms", (16, 185, 129)))
                .with_param(DspParamDescriptor::new_bool("equal_power_crossfade", "Equal Power 3dB Crossfade", true, (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("take_selection", "Active Take Lane Index", 1.0, 16.0, 1.0, "lane", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_bool("auto_align_phase", "Transient Cross-Correlation Auto-Align", true, (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("pre_roll_lead_in_ms", "Comp Pre-Roll Headroom", 0.0, 200.0, 25.0, "ms", (168, 85, 247)))
        ),
        "DrumMacroStrip" => Box::new(
            GenericDspNodeUi::new("DrumMacroStrip", "16-Pad Tactile Drum Machine & Groove Macro Console HUD", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_linear("global_groove_swing", "Drum Machine Global Shuffle & Swing", 50.0, 75.0, 58.0, "%", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("transient_punch_boost_db", "Master Drum Transient Punch Boost", -6.0, 12.0, 3.5, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("global_drum_decay_ratio", "Global Drum Voice Decay Time Scaling", 0.2, 3.0, 1.0, "x", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("drum_master_saturation", "Drum Bus Discrete Tape Saturation Drive", 0.0, 100.0, 18.0, "%", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("pad_velocity_compression", "Pad Dynamic Velocity Compression Curve", 0.0, 100.0, 40.0, "%", (34, 197, 94)))
        ),
        "SfzPresetPatch" => Box::new(
            GenericDspNodeUi::new("SfzPresetPatch", "SFZ Multi-Zone Soundfont & Instrument Patch HUD", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_enum("keyzone_low_note", "Lowest Keyzone Active Note Boundary", vec!["C-1 (MIDI 0)".into(), "C1 (MIDI 24)".into(), "C2 (MIDI 36)".into(), "C3 (MIDI 48)".into()], 0, (56, 189, 248)))
                .with_param(DspParamDescriptor::new_enum("keyzone_high_note", "Highest Keyzone Active Note Boundary", vec!["C5 (MIDI 72)".into(), "C6 (MIDI 84)".into(), "C7 (MIDI 96)".into(), "G9 (MIDI 127)".into()], 3, (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("velocity_split_threshold", "Dynamic Velocity Crossfade Split Boundary", 1.0, 127.0, 64.0, "vel", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("loop_crossfade_ms", "Loop Transition Equal-Power Crossfade", 0.0, 500.0, 45.0, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("patch_attenuation_db", "Master Instrument Patch Trim Gain", -48.0, 12.0, -3.0, "dB", (34, 197, 94)))
        ),
        "SfzMultisampleConverter" => Box::new(
            GenericDspNodeUi::new("SfzMultisampleConverter", "SFZ Instrument to Native Multisample Keymap Converter HUD", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_enum("velocity_layer_detection_mode", "Velocity Zone Mapping Strategy", vec!["SFZ Opcode Parsing (lovel/hivel)".into(), "Automatic Dynamic Clustering".into(), "Uniform Velocity Slicing".into()], 0, (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("round_robin_group_depth", "Maximum Round-Robin Performance Cycles", 1.0, 16.0, 4.0, "layers", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("root_key_extrapolation_semitones", "Maximum Root Pitch Extrapolation Distance", 1.0, 24.0, 6.0, "st", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_bool("trim_leading_sample_silence", "Auto-Trim Pre-Attack Sample Silence", true, (168, 85, 247)))
                .with_param(DspParamDescriptor::new_enum("export_native_format", "Target Native Instrument Keymap Format", vec!["Summoner Keymap (.skm)".into(), "Multi-Channel SoundFont (.sf3)".into(), "Embedded Single-File Preset".into()], 0, (34, 197, 94)))
        ),
        "MultiSamplePresetLoader" => Box::new(
            GenericDspNodeUi::new("MultiSamplePresetLoader", "Multi-Sample SFZ Keygroup & Velocity Zone Loader HUD", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_enum("active_sample_base_directory", "Sample Base Search Directory Scope", vec!["Default Local Project Assets".into(), "System Global Samples Library".into(), "External NVMe Sample SSD Mount".into(), "Embedded Memory Cache".into()], 0, (56, 189, 248)))
                .with_param(DspParamDescriptor::new_enum("velocity_layer_interpolation", "Velocity Layer Crossfade Interpolation", vec!["Equal-Power Sine Crossfade".into(), "Linear Amplitude Ramp".into(), "Hard Velocity Split (No Crossfade)".into()], 0, (245, 158, 11)))
                .with_param(DspParamDescriptor::new_bool("preload_zone_memory_buffers", "Pre-Load All Sample Buffers into RAM", true, (239, 68, 68)))
                .with_param(DspParamDescriptor::new_enum("round_robin_cycling_mode", "Multi-Sample Round-Robin Cycling Strategy", vec!["Sequential Cycle 1-2-3-4".into(), "Pseudo-Random No-Repeat".into(), "True Random".into(), "Velocity-Stochastic".into()], 1, (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("polyphonic_voice_limit", "Maximum Polyphonic Active Sample Voices", 8.0, 128.0, 32.0, "voices", (34, 197, 94)))
        ),
        "MultiTakeCompManager" => Box::new(
            GenericDspNodeUi::new("MultiTakeCompManager", "Sample-Accurate Multi-Take Audio Comping & Lane Splice Engine HUD", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_linear("splice_crossfade_duration_ms", "Comp Splice Boundary Crossfade Time", 1.0, 50.0, 5.0, "ms", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_enum("crossfade_curve_power", "Splice Boundary Crossfade Geometry", vec!["Equal Power (3dB Dip Free)".into(), "Linear Slope".into(), "S-Curve Logarithmic".into(), "Fast Cut (0.5ms)".into()], 0, (245, 158, 11)))
                .with_param(DspParamDescriptor::new_enum("comp_lane_audition_solo", "Comp Lane Audition Routing", vec!["Master Comp Composite".into(), "Solo Lane 1".into(), "Solo Lane 2".into(), "Solo Lane 3".into(), "Solo Lane 4".into()], 0, (239, 68, 68)))
                .with_param(DspParamDescriptor::new_enum("splice_boundary_snap_grid", "Splice Boundary Grid Snapping Mode", vec!["Free Hand (Sample-Accurate)".into(), "1/64 Note Beat Snap".into(), "Transient Hit Boundary".into(), "Zero-Crossing Detection".into()], 3, (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("non_destructive_slip_editing", "Enable Non-Destructive Take Slip Editing", true, (34, 197, 94)))
        ),
        "LooperTrackManager" => Box::new(
            GenericDspNodeUi::new("LooperTrackManager", "Non-Blocking Multitrack Circular Looper & Overdub Layer Manager HUD", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_linear("overdub_feedback_decay_pct", "Overdub Layer Feedback Decay Retention", 50.0, 100.0, 100.0, "%", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_enum("loop_quantize_division", "Loop Record & Punch Quantize Grid", vec!["1 Bar (Strict Measure)".into(), "2 Bars".into(), "4 Bars".into(), "1/4 Beat Freeform".into(), "Free Run (Unquantized)".into()], 0, (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("boundary_crossfade_samples", "Turnaround Splice Crossfade Window", 16.0, 256.0, 64.0, "samples", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_enum("playback_speed_multiplier", "Loop Playback Rate & Direction", vec!["1x Normal Speed".into(), "2x Double Speed".into(), "0.5x Half Speed Reverse".into(), "-1x Reverse Playback".into()], 0, (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("enable_infinite_overdub_layers", "Non-Destructive Multi-Layer Undo Stack", true, (34, 197, 94)))
        ),
        "LooperTrack" => Box::new(
            GenericDspNodeUi::new("LooperTrack", "Real-Time Multitrack Overdub Audio Looper & Layer HUD", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_enum("loop_quantize_grid", "Audio Looper Punch-In & Punch-Out Quantization Grid", vec!["1 Bar (Downbeat Synced)".into(), "2 Bars".into(), "4 Bars".into(), "1/4 Note (Beat Synced)".into(), "Free (Unquantized Tap)".into()], 0, (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("overdub_feedback_decay", "Overdub Layer Decay Feedback Retention", 0.0, 1.0, 0.92, "%", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("punch_crossfade_duration_ms", "Loop Boundary Punch-In Anti-Click Crossfade Duration", 1.0, 50.0, 8.0, "ms", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_enum("playback_speed_multiplier", "Looper Playback Speed & Octave Transposition", vec!["1x Normal Speed".into(), "0.5x Half Speed (Octave Down)".into(), "2x Double Speed (Octave Up)".into(), "-1x Reverse Direction".into()], 0, (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("auto_trim_silence_threshold_db", "Auto-Record Input Signal Detection Threshold", -60.0, -20.0, -42.0, "dB", (34, 197, 94)))
        ),
        "SfzRegionConfig" => Box::new(
            GenericDspNodeUi::new("SfzRegionConfig", "SFZ Multisample Zone & Velocity Split HUD", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_linear("key_low", "Keyboard Zone Lower MIDI Note Limit", 0.0, 127.0, 36.0, "MIDI", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("key_high", "Keyboard Zone Upper MIDI Note Limit", 0.0, 127.0, 48.0, "MIDI", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("velocity_low", "Velocity Layer Lower Sensitivity Limit", 1.0, 127.0, 1.0, "vel", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("velocity_high", "Velocity Layer Upper Sensitivity Limit", 1.0, 127.0, 127.0, "vel", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("sample_tune_cents", "Fine Tuning Pitch Offset Calibration", -100.0, 100.0, 0.0, "cents", (34, 197, 94)))
        ),
        "SampleBuffer" => Box::new(
            GenericDspNodeUi::new("SampleBuffer", "Shared Linear Audio Sample Buffer Cache HUD", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_linear("buffer_capacity_seconds", "Sample Audio Duration", 0.1, 600.0, 4.5, "s", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("sample_rate_khz", "Internal Sampling Rate", 22.05, 192.0, 48.0, "kHz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("channel_count", "Audio Channel Configuration", 1.0, 8.0, 2.0, "ch", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("memory_consumption_mb", "RAM Allocation Footprint", 0.1, 512.0, 1.7, "MB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("normalize_peak_dbfs", "Non-Destructive Peak Normalization", -24.0, 0.0, -0.5, "dBFS", (34, 197, 94)))
        ),
        "SampleRegion" => Box::new(
            GenericDspNodeUi::new("SampleRegion", "Multi-Sample Instrument Velocity Keyzone Region HUD", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_linear("low_key_limit", "Velocity Zone Low MIDI Key", 0.0, 127.0, 36.0, "MIDI", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("high_key_limit", "Velocity Zone High MIDI Key", 0.0, 127.0, 72.0, "MIDI", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("root_key_center", "Sample Root Fundamental Pitch", 0.0, 127.0, 60.0, "MIDI", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("velocity_threshold_low", "Low Velocity Trigger Threshold", 1.0, 127.0, 1.0, "vel", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("loop_crossfade_duration_ms", "Loop Region Boundary Equal-Power Crossfade", 0.0, 500.0, 25.0, "ms", (34, 197, 94)))
        ),
        "CloudGrain" => Box::new(
            GenericDspNodeUi::new("CloudGrain", "Asynchronous Granular Cloud Grain Engine HUD", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_linear("grain_duration_ms", "Grain Window Duration", 5.0, 500.0, 50.0, "ms", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("pitch_ratio", "Micro-Grain Playback Pitch Speed Ratio", 0.25, 4.0, 1.0, "x", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("stereo_pan_spread", "Grain Stereo Spatial Panning Position", -1.0, 1.0, 0.0, "pan", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("grain_amplitude_peak", "Individual Grain Peak Amplitude", 0.0, 2.0, 0.8, "amp", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("tukey_alpha", "Tukey Window Taper Envelope Slope", 0.0, 1.0, 0.5, "alpha", (34, 197, 94)))
        ),
        "SampleMarker" => Box::new(
            GenericDspNodeUi::new("SampleMarker", "Non-Destructive Sample Boundary Transient Marker HUD", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_linear("marker_position_samples", "Marker Boundary Offset in Samples", 0.0, 10000000.0, 24000.0, "samples", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_bool("snap_to_zero_crossing", "Snap Marker to Nearest Zero-Crossing", true, (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("marker_crossfade_ms", "Marker Region Boundary Crossfade", 0.0, 100.0, 5.0, "ms", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("transient_threshold_db", "Transient Peak Detection Threshold", -48.0, 0.0, -12.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("marker_gain_trim_db", "Regional Gain Trim Offset", -24.0, 12.0, 0.0, "dB", (34, 197, 94)))
        ),
        "Grain" => Box::new(
            GenericDspNodeUi::new("Grain", "Microscopic Granular Audio Resynthesis Particle", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_linear("grain_duration_ms", "Particle Grain Duration", 2.0, 500.0, 45.0, "ms", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("playback_rate", "Grain Playback Rate Pitch Transpose", 0.125, 4.0, 1.0, "x", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_enum("window_shape", "Windowing Envelope Function", vec!["Hann Raised Cosine".into(), "Gaussian".into(), "Blackman-Harris".into(), "Tukey 10%".into(), "Rectangular".into()], 0, (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("stereo_pan", "Spatial Stereo Pan Position", -1.0, 1.0, 0.0, "pan", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("reverse_playback", "Reverse Grain Time Direction", false, (34, 197, 94)))
        ),
        "DrumPad" => Box::new(
            GenericDspNodeUi::new("DrumPad", "Multi-Velocity Layer Acoustic & Electronic Drum Pad", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_linear("tune_coarse_semitones", "Drum Pad Pitch Tuning", -24.0, 24.0, 0.0, "st", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("velocity_sensitivity", "Dynamic Velocity Impact Sensitivity", 0.0, 1.0, 0.85, "norm", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("decay_time_ms", "Drum Envelope Amplitude Decay", 10.0, 5000.0, 450.0, "ms", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("choke_group", "Mute Choke Exclusive Group ID", 0.0, 16.0, 1.0, "grp", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("round_robin_count", "Alternate Hit Round-Robin Variation Count", 1.0, 8.0, 4.0, "hits", (34, 197, 94)))
        ),
        "SampleSlice" => Box::new(
            GenericDspNodeUi::new("SampleSlice", "Zero-Crossing Audio Sample Transient Slice", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_linear("start_frame", "Slice In-Point Audio Frame", 0.0, 10000000.0, 0.0, "frames", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("end_frame", "Slice Out-Point Audio Frame", 1.0, 10000000.0, 48000.0, "frames", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("playback_pitch_ratio", "Slice Chromatic Resampling Pitch Ratio", 0.25, 4.0, 1.0, "x", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("transient_gain_db", "Transient Attack Punch Gain", -12.0, 12.0, 0.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_enum("loop_mode", "Segment Loop Playback Direction", vec!["One-Shot Forward".into(), "Forward Loop".into(), "Ping-Pong Bi-Directional".into(), "Reverse One-Shot".into()], 0, (34, 197, 94)))
        ),
        "DrumMacroView" => Box::new(
            GenericDspNodeUi::new("DrumMacroView", "Percussion Kit Master Gain & Pad Mute Controller", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_linear("master_gain", "Global Kit Master Gain", 0.0, 1.5, 0.85, "%", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_bool("kick_mute", "Pad 0 Kick Drum Mute", false, (245, 158, 11)))
                .with_param(DspParamDescriptor::new_bool("snare_mute", "Pad 1 Snare Drum Mute", false, (239, 68, 68)))
                .with_param(DspParamDescriptor::new_bool("hihat_mute", "Pad 2 Hi-Hat Mute", false, (168, 85, 247)))
                .with_param(DspParamDescriptor::new_enum("choke_mode", "Dynamic Voice Choke Policy", vec!["Exclusive Choke Group".into(), "Polyphonic Ringing".into(), "Round-Robin Layered".into(), "Velocity Choked".into()], 0, (34, 197, 94)))
        ),
        "SamplerMacroView" => Box::new(
            GenericDspNodeUi::new("SamplerMacroView", "Multisample Engine Macro Performance Strip", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_log("filter_cutoff", "SVF Low-Pass Cutoff Frequency", 20.0, 20000.0, 4500.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("filter_res", "Filter Resonance Emphasis Q", 0.0, 4.0, 0.8, "Q", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("amp_attack", "Amplitude Envelope Attack Time", 0.001, 5.0, 0.015, "s", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("amp_release", "Amplitude Envelope Release Time", 0.01, 10.0, 0.45, "s", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("gain", "Multisample Volume Scaling", 0.0, 1.5, 1.0, "%", (34, 197, 94)))
        ),
        "SliceMarker" => Box::new(
            GenericDspNodeUi::new("SliceMarker", "Audio Waveform Zero-Crossing Transient Cut Marker", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_linear("start_sample", "Slice In-Point Sample Frame", 0.0, 10000000.0, 0.0, "samples", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("end_sample", "Slice Out-Point Sample Frame", 1.0, 10000000.0, 24000.0, "samples", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("crossfade_samples", "Anti-Click Splice Crossfade Window", 0.0, 512.0, 32.0, "samples", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("pitch_semitones", "Slice Pitch Transposition", -24.0, 24.0, 0.0, "st", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("snap_to_zero_crossing", "Auto Snap Zero-Crossing Boundary", true, (34, 197, 94)))
        ),
        "WarpMarker" => Box::new(
            GenericDspNodeUi::new("WarpMarker", "Elastic Time-Stretch Transient Warp Anchor", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_linear("source_frame", "Source Audio Buffer Anchor Frame", 0.0, 10000000.0, 0.0, "frames", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("target_frame", "Warped Beat Grid Destination Frame", 0.0, 10000000.0, 0.0, "frames", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("time_stretch_ratio", "Local Time-Stretch Speed Ratio", 0.25, 4.0, 1.0, "x", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_bool("pitch_lock", "Preserve Spectral Formant Envelope", true, (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("transient_sensitivity", "Peak Detection Smear Sensitivity", 0.1, 1.0, 0.85, "%", (34, 197, 94)))
        ),
        "LoopMode" => Box::new(
            GenericDspNodeUi::new("LoopMode", "Polyphonic Audio Sampler Waveform Looping & Direction Mode", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_linear("sampler_loop_direction_mode", "Audio Buffer Looping Trajectory", 0.0, 4.0, 1.0, "dir", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("loop_start_marker_samples", "Loop Cycle In-Point Sample Boundary", 0.0, 960000.0, 44100.0, "smp", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("loop_end_marker_samples", "Loop Cycle Out-Point Sample Boundary", 0.0, 960000.0, 88200.0, "smp", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("crossfade_smoothing_duration_ms", "Loop Boundary Equal-Power Crossfade Time", 0.0, 500.0, 25.0, "ms", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("crossfade_curve_power", "Crossfade Interpolation Curve Curvature", 0.1, 3.0, 1.0, "pow", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_zero_crossing_auto_snap", "Auto-Snap Loop Markers to Nearest Zero-Crossing", true, (14, 165, 233)))
        ),
        "SliceAlgorithm" => Box::new(
            GenericDspNodeUi::new("SliceAlgorithm", "Intelligent Audio Slicing & Transient Detection Engine", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_linear("transient_detection_algorithm", "Transient Detection Mathematical Model", 0.0, 2.0, 1.0, "algo", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("detection_sensitivity_threshold", "Transient Detection Sensitivity Threshold", 0.01, 1.0, 0.35, "%", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("minimum_slice_distance_ms", "Minimum Inter-Slice Time Gap Interval", 10.0, 500.0, 50.0, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("transient_lookahead_padding_ms", "Pre-Transient Marker Safety Padding", 1.0, 25.0, 5.0, "ms", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("slice_fade_out_time_ms", "Slice Playback Micro-Fade Out Envelope", 1.0, 50.0, 8.0, "ms", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_zero_crossing_snap", "Auto-Snap Slice Markers to Nearest Zero Crossing", true, (14, 165, 233)))
        ),
        "LooperState" => Box::new(
            GenericDspNodeUi::new("LooperState", "Multi-Track Live Performance Looper State Machine", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_linear("track_operational_state", "Looper Track Real-Time Lifecycle State", 0.0, 8.0, 4.0, "state", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("feedback_overdub_decay", "Overdub Feedback / Sound-on-Sound Decay", 0.0, 1.0, 0.95, "%", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("loop_master_tempo_bpm", "Captured Loop Master Tempo BPM", 40.0, 280.0, 120.0, "BPM", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("crossfade_boundary_ms", "Loop Seam Equal-Power Crossfade Time", 1.0, 50.0, 10.0, "ms", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("pitch_playback_speed_multiplier", "Loop Playback Speed & Pitch Transposition", 0.25, 4.0, 1.0, "x", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_infinite_undo_history", "Retain Multi-Take Non-Destructive Undo Layers", true, (14, 165, 233)))
        ),
        "LooperCommand" => Box::new(
            GenericDspNodeUi::new("LooperCommand", "Live Performance Looper State Machine Quantized Action Trigger", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_linear("dispatched_command_action", "Quantized Looper Command Action Trigger", 0.0, 11.0, 0.0, "cmd", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("quantized_execution_alignment", "Command Transport Execution Timing", 0.0, 4.0, 1.0, "quant", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("track_multiply_divide_ratio", "Loop Multiply / Divide Integer Scaling", 1.0, 16.0, 2.0, "x", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("reverse_turnaround_fade_ms", "Reverse Trajectory Boundary Equal-Power Fade", 1.0, 100.0, 15.0, "ms", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("playback_speed_octave_transposition", "Looper Playback Speed Octave Transposition", -2.0, 2.0, 0.0, "oct", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_instant_seamless_overdub", "Seamless Instant Overdub Punch-In Transition", true, (14, 165, 233)))
        ),
        "LooperQuantize" => Box::new(
            GenericDspNodeUi::new("LooperQuantize", "Beat & Bar Quantization Interval Alignment for Live Loopers", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_linear("quantization_interval_grid", "Quantization Alignment Time Metric Grid", 0.0, 3.0, 2.0, "grid", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("beat_subdivision_fraction", "Quantized Beat Subdivision Duration", 0.25, 8.0, 1.0, "beats", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("bar_boundary_interval", "Whole Bar Turnaround Repetition Length", 1.0, 16.0, 1.0, "bars", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("preroll_latency_lead_ms", "Input Latency Trigger Pre-Roll Compensation", 0.0, 100.0, 10.0, "ms", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("turnaround_crossfade_samples", "Loop Turnaround Seam Crossfade Sample Buffer", 4.0, 512.0, 32.0, "smp", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_master_transport_phase_lock", "Lock Playhead Phase to DAW Master Transport", true, (14, 165, 233)))
        ),
        "LooperSyncMode" => Box::new(
            GenericDspNodeUi::new("LooperSyncMode", "Multitrack Looper Independent vs Master-Locked Synchronization Mode", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_linear("multitrack_synchronization_mode", "Looper Track Clock Alignment Hierarchy", 0.0, 2.0, 1.0, "mode", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("tempo_drift_tolerance_bpm", "Auto-Resync Phase Drift Tolerance Window", 0.01, 5.0, 0.50, "BPM", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("master_lane_assignment_index", "Master Reference Looper Channel Index", 0.0, 15.0, 0.0, "track", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("slave_drift_correction_gain", "Slave Phase Correction Resync Pull Gain", 0.01, 1.0, 0.15, "gain", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("varispeed_interpolation_slew_ms", "Varispeed Pitch Correction Slew Smooth Time", 1.0, 200.0, 20.0, "ms", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_time_stretch_loop_preservation", "Maintain Pitch During Master Tempo Changes", true, (14, 165, 233)))
        ),
        "SfzLoopMode" => Box::new(
            GenericDspNodeUi::new("SfzLoopMode", "Multi-Sample SFZ Region Sustain Looping & Crossfade Envelope", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_linear("region_loop_playback_mode", "SFZ Sample Region Sustain Loop Mode", 0.0, 3.0, 1.0, "mode", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("loop_crossfade_duration_ms", "Loop Boundary Equal-Power Crossfade Time", 0.0, 500.0, 25.0, "ms", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("loop_start_offset_samples", "Loop Cycle Start Sample Index Offset", 0.0, 1000000.0, 4410.0, "spl", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("loop_end_offset_samples", "Loop Cycle End Sample Index Boundary", 1000.0, 10000000.0, 88200.0, "spl", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("loop_tune_fine_cents", "Loop Segment Micro-Pitch Tuning Correction", -50.0, 50.0, 0.0, "cents", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_equal_power_loop_crossfade", "Enable Sinusoidal Equal-Power Crossfading", true, (14, 165, 233)))
        ),
        "GrainWindowType" => Box::new(
            GenericDspNodeUi::new("GrainWindowType", "Granular Cloud Synthesis Windowing Envelope Function", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_linear("window_function", "Grain Temporal Windowing Envelope Function", 0.0, 7.0, 0.0, "win", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("tukey_alpha", "Tukey Window Flat-Top Cosine Taper Alpha", 0.0, 1.0, 0.5, "alpha", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("gaussian_sigma", "Gaussian Standard Deviation Bell Width", 0.1, 1.0, 0.35, "sigma", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("grain_overlap_ratio", "Inter-Grain Window Overlap Density", 1.0, 16.0, 4.0, "x", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("spectral_leakage_suppression", "Side-Lobe Spectral Leakage Rejection", 12.0, 96.0, 48.0, "dB", (14, 165, 233)))
                .with_param(DspParamDescriptor::new_linear("cosine_taper_symmetry", "Envelope Attack / Release Asymmetry Ratio", 0.0, 1.0, 0.5, "sym", (52, 211, 153)))
        ),
        "CliAutoSlicer" => Box::new(
            GenericDspNodeUi::new("CliAutoSlicer", "Spectral Flux Transient Auto-Slice Marker Generator", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_linear("slicing_algorithm", "Transient Detection Mathematical Algorithm", 0.0, 3.0, 0.0, "algo", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("detection_threshold", "Transient Peak Sensitivity Threshold", 0.01, 1.0, 0.15, "thresh", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("min_slice_distance_ms", "Minimum Temporal Distance Between Slice Points", 10.0, 500.0, 50.0, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("snap_to_zero_crossing", "Snap Detected Slices to Nearest Signal Zero Crossing", true, (52, 211, 153)))
                .with_param(DspParamDescriptor::new_linear("pre_transient_padding_ms", "Pre-Transient Lookahead Attack Margin", 0.0, 20.0, 2.5, "ms", (14, 165, 233)))
                .with_param(DspParamDescriptor::new_bool("export_slice_metadata", "Write Sliced Drum Kit TOML Map for SamplerDevice", true, (239, 68, 68)))
        ),
        "GranularCloudTextureMorpher" => Box::new(
            GenericDspNodeUi::new("GranularCloudTextureMorpher", "Asynchronous Multi-Head Granular Cloud & Texture Generator", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_linear("grain_density_cloud_rate_hz", "Granular Cloud Emission Density", 1.0, 200.0, 45.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("grain_duration_envelope_ms", "Micro-Grain Playback Duration", 2.0, 500.0, 65.0, "ms", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("playback_buffer_position_pct", "Sample Buffer Playback Scrub Offset", 0.0, 100.0, 35.0, "%", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("grain_pitch_micro_jitter_cents", "Stochastic Grain Pitch Detune Jitter", 0.0, 1200.0, 25.0, "ct", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("spatial_grain_azimuth_scatter", "3D Spatial Azimuth Dispersion Width", 0.0, 1.0, 0.6, "%", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_hanning_window_anti_glitch", "Raised Cosine Grain Window Anti-Glitch", true, (14, 165, 233)))
        ),
        "GranularFreezeBufferDrone" => Box::new(
            GenericDspNodeUi::new("GranularFreezeBufferDrone", "Infinite Granular Freeze Buffer & Ambient Drone Engine", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_linear("freeze_buffer_duration_s", "Circular Freeze Buffer Memory Duration", 0.5, 10.0, 3.0, "s", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("grain_cloud_density_hz", "Granular Cloud Emission Frequency", 5.0, 120.0, 35.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("grain_playback_window_ms", "Grain Window Envelope Duration", 10.0, 800.0, 120.0, "ms", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("random_pitch_dispersion_cents", "Stochastic Micro-Grain Pitch Jitter", 0.0, 1200.0, 45.0, "ct", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("spatial_azimuth_spread_pct", "Stereo Azimuth Pan Cloud Scatter", 0.0, 100.0, 80.0, "%", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_infinite_freeze_hold", "Continuous Infinite Buffer Freeze Lock", true, (14, 165, 233)))
        ),
        "PhaseAlignedGranularTimeStretcher" => Box::new(
            GenericDspNodeUi::new("PhaseAlignedGranularTimeStretcher", "Phase-Aligned Granular Time & Pitch Stretcher", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_linear("time_stretch_ratio_factor", "Playback Speed Time Stretch Factor", 0.1, 10.0, 1.0, "x", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("pitch_shift_cents_detune", "Chromatic Pitch Transposition Detune", -2400.0, 2400.0, 0.0, "ct", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("grain_slice_window_duration_ms", "Synchronized Grain Window Size", 10.0, 250.0, 45.0, "ms", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("formant_spectral_lock_ratio", "Formant Envelope Preservation Lock", 0.0, 1.0, 0.9, "%", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("transient_energy_sensitivity", "Transient Slice Boundary Snapping", 0.0, 1.0, 0.8, "%", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_intergrain_phase_locking", "Inter-Grain Phase Synchronization", true, (14, 165, 233)))
        ),
        "PhaseAlignedStereoGranularDelay" => Box::new(
            GenericDspNodeUi::new("PhaseAlignedStereoGranularDelay", "Phase-Aligned Stereo Granular Delay", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_linear("grain_duration_ms", "Granular Window Duration Length", 5.0, 500.0, 65.0, "ms", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("grain_spawn_density_hz", "Grain Generation Emission Rate", 1.0, 100.0, 28.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("delay_buffer_offset_ms", "Delay Line Tap Offset Time", 10.0, 5000.0, 375.0, "ms", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("grain_pitch_semitones", "Grain Playback Pitch Shift", -24.0, 24.0, 0.0, "st", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("stereo_pan_diffusion_spread", "Stereo Spatial Pan Jitter Spread", 0.0, 1.0, 0.65, "%", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_hanning_window_tapering", "Raised Cosine Grain Envelope Taper", true, (14, 165, 233)))
        ),
        "AsynchronousGranularCloudSaturator" => Box::new(
            GenericDspNodeUi::new("AsynchronousGranularCloudSaturator", "Asynchronous Granular Cloud Texture Saturator", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_linear("grain_emission_rate_hz", "Grain Emission Density Rate", 1.0, 500.0, 80.0, "Hz", (16, 185, 129)))
                .with_param(DspParamDescriptor::new_linear("grain_duration_ms", "Individual Grain Window Size", 5.0, 400.0, 45.0, "ms", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("per_grain_saturation_drive_db", "Per-Grain Non-Linear Saturation", 0.0, 36.0, 12.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("pitch_dispersion_random_cents", "Pitch Scatter Distribution Spread", 0.0, 1200.0, 50.0, "cents", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("stereo_spatial_dispersion_pct", "Stereo Binaural Spatial Scatter", 0.0, 1.0, 0.75, "%", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_bool("enable_quasi_gaussian_window_taper", "Gaussian Envelope Micro-Windowing", true, (34, 197, 94)))
        ),
        "GranularSpectralGlitchDeconstruct" => Box::new(
            GenericDspNodeUi::new("GranularSpectralGlitchDeconstruct", "Stochastic Spectral Fragment & Time-Splay Glitcher", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_linear("shard_emission_rate_hz", "Glitch Grain Emission Rate", 1.0, 250.0, 45.0, "Hz", (16, 185, 129)))
                .with_param(DspParamDescriptor::new_linear("time_splay_jitter_ms", "Buffer Read Pointer Splay Jitter", 0.0, 500.0, 120.0, "ms", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("pitch_random_octave_jump", "Discrete Harmonic Interval Hop", 0.0, 24.0, 12.0, "st", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("reverse_playback_probability_pct", "Reverse Grain Playback Probability", 0.0, 1.0, 0.3, "%", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("spectral_smear_window_size", "Spectral Window Blurring Size", 64.0, 4096.0, 1024.0, "samples", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_bool("enable_buffer_stutter_freeze", "Infinite Buffer Stutter Loop Freeze", false, (34, 197, 94)))
        ),
        "AsynchronousSpectralStutterBuffer" => Box::new(
            GenericDspNodeUi::new("AsynchronousSpectralStutterBuffer", "Asynchronous Spectral Stutter & Freeze Buffer", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_linear("spectral_freeze_hold_mix", "Spectral Magnitude Freeze Blend", 0.0, 1.0, 0.0, "freeze", (16, 185, 129)))
                .with_param(DspParamDescriptor::new_linear("stutter_slice_division_sub", "Rhythmic Stutter Beat Division", 1.0, 32.0, 8.0, "div", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("time_stretch_ratio_factor", "Phase Vocoder Time Stretch Ratio", 0.1, 10.0, 1.0, "x", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("pitch_transposition_cents", "Spectral Pitch Transposition", -2400.0, 2400.0, 0.0, "cents", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("stutter_probability_gate", "Stutter Glitch Trigger Probability", 0.0, 1.0, 0.45, "%", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_bool("enable_phase_vocoder_transient_bypass", "Transient Attack Phase Lock Preservation", true, (34, 197, 94)))
        ),
        "GranularAsynchronousReverseSlicer" => Box::new(
            GenericDspNodeUi::new("GranularAsynchronousReverseSlicer", "Granular Asynchronous Time-Reverse Slicer", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_linear("grain_duration_length_ms", "Individual Grain Duration Window", 10.0, 500.0, 85.0, "ms", (16, 185, 129)))
                .with_param(DspParamDescriptor::new_linear("reverse_playback_probability", "Reverse Grain Playback Probability", 0.0, 1.0, 0.75, "prob", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("grain_position_spray_jitter_ms", "Grain Inception Spray Jitter", 0.0, 200.0, 35.0, "ms", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("granular_pitch_shift_semitones", "Grain Microtone Pitch Shift", -24.0, 24.0, 0.0, "st", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("grain_density_overlap_factor", "Simultaneous Grain Cloud Density", 1.0, 16.0, 4.0, "grains", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_bool("enable_transient_sync_slicing", "Transient Marker Grain Alignment", true, (34, 197, 94)))
        ),
        "AsynchronousGranularTimeStretchWarper" => Box::new(
            GenericDspNodeUi::new("AsynchronousGranularTimeStretchWarper", "Asynchronous Granular Time-Stretch & Pitch Warper", DspNodeCategory::SamplerSlicer)
                .with_param(DspParamDescriptor::new_linear("time_stretch_playback_speed", "Time-Stretch Playback Rate Speed", 0.05, 4.0, 1.0, "rate", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("pitch_shift_playback_semitones", "Granular Pitch Shift Interval", -24.0, 24.0, 0.0, "semi", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("grain_duration_length_ms", "Individual Grain Duration Window", 5.0, 350.0, 65.0, "ms", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("grain_stream_density_overlap", "Simultaneous Grain Cloud Stream Density", 1.0, 32.0, 8.0, "grains", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("reverse_grain_playback_probability", "Reverse Grain Playback Probability", 0.0, 1.0, 0.15, "prob", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_bool("enable_stereo_scatter_panning", "Stereo Grain Cloud Scatter Panning", true, (34, 197, 94)))
        ),
        _ => return None,
    };
    Some(node)
}
