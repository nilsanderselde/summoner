// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Shared audio rendering engine, node factory, graph runner, and live CPAL stream host.

pub mod audio_engine;
pub mod graph;
pub mod metronome;
pub mod midi_engine;

pub use audio_engine::{
    default_render_frames, render_project_to_wav, render_project_track_stems, run_live,
    spawn_live_engine, spawn_live_engine_with_settings, AudioDeviceInfo, AudioDeviceManager,
    AudioEngineSettings, EngineCommand, LiveEngineHandle, ParamUpdate,
};
pub use graph::{graph_from_project, GraphRunner, NodeFactory};
pub use metronome::MetronomeGenerator;
pub use midi_engine::{MidiDeviceManager, MidiInputReceiver, MidiPortInfo, TimestampedMidiEvent};

