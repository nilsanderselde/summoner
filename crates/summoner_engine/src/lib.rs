// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Shared audio rendering engine, node factory, graph runner, and live CPAL stream host.

pub mod audio_engine;
pub mod disk_stream;
pub mod graph;
pub mod latency_probe;
pub mod metronome;
pub mod midi_engine;
pub mod midi_recorder;
pub mod recorder;
pub mod virtual_midi;

pub use audio_engine::{
    backend_to_host_id, default_render_frames, host_id_to_backend, render_project_to_wav,
    render_project_track_stems, run_live, spawn_live_engine, spawn_live_engine_with_settings,
    AudioDeviceInfo, AudioDeviceManager, AudioEngineSettings, EngineCommand, LiveEngineHandle,
    ParamUpdate,
};
pub use disk_stream::{DiskStreamConfig, DiskStreamHandle, DiskStreamManager};
pub use graph::{graph_from_project, GraphRunner, NodeFactory};
pub use latency_probe::{
    LatencyDetector, LatencyProbeGenerator, RecordingLatencyCompensation, RoundTripLatency,
};
pub use metronome::MetronomeGenerator;
pub use midi_engine::{
    MidiClockOutRunner, MidiDeviceManager, MidiInputReceiver, MidiOutputSender, MidiPortInfo,
    MidiSyncReceiver, MidiSyncState, TimestampedMidiEvent,
};
pub use midi_recorder::{
    MidiRecordMode, MidiRecorder, QuantizeGrid, RecordedMidiNote, RetrospectiveMidiBuffer,
    StepInputController,
};
pub use recorder::{
    AudioInputSource, AudioRecorder, LoopRecordSettings, PunchSettings, RecordState,
    RecordedTakeResult,
};
pub use virtual_midi::{
    VirtualMidiDirection, VirtualMidiManager, VirtualMidiPort, VirtualMidiPortInfo,
};
