// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.

//! Core audio abstractions, node graph evaluation, and zero-allocation primitives for Summoner DAW.
#![allow(missing_docs)]

pub mod adaptive_buffer;
pub mod allocator;
pub mod articulation_bus;
pub mod audio;
pub mod audio_drivers;
pub mod bellows_bus;
pub mod breath_bus;
pub mod cadence_bus;
pub mod clavinet_bus;
pub mod embedded_hardware;
pub mod ep_bus;
pub mod formant_bus;
pub mod glass_bus;
pub mod grand_piano_bus;
pub mod graph;
pub mod hurdy_gurdy_bus;
pub mod koto_bus;
pub mod mallet_bus;
pub mod midi;
pub mod midi_clock;
pub mod midi_filter;
pub mod mpe;
pub mod node;
pub mod panner;
pub mod param_bus;
pub mod pipeline;
pub mod pipe_organ_bus;
pub mod plectrum_bus;
pub mod rotary_bus;
pub mod sample;
pub mod sequence;
pub mod shakuhachi_bus;
pub mod sidechain_matrix;
pub mod sitar_bus;
pub mod smoothing;
pub mod spatial_bus;
pub mod spring_bus;
pub mod track;
pub mod transport;
pub mod tuning_matrix;
pub mod voice;
pub mod wav;
pub use adaptive_buffer::AdaptiveBufferScaler;
pub use articulation_bus::*;
pub use audio::{ChannelLayout, FixedAudioBuffer, Frame, MultichannelAudioBuffer, Sample};
pub use audio_drivers::{
    AAudioDriver, AlsaDriver, AsapiDriver, AudioUnitDriver, NativeAudioDriver,
    NativeAudioDriverTuner, WasapiDriver,
};
pub use bellows_bus::*;
pub use breath_bus::*;
pub use cadence_bus::*;
pub use clavinet_bus::*;
pub use embedded_hardware::{
    BatteryMonitor, BleMidiPeripheral, BootToSynthEngine, BypassRelayTrigger, EepromPresetStore,
    EmbeddedHardwareConfig, EurorackCvGateInterface, GpioDriver, GpioEvent,
    HardwareEmulationHarness, HardwareWatchdogService, MemoryEstimator, MidiUartSerialDriver,
    MidiUsbGadgetMode, OledDisplayDriver, RotaryEncoderDebouncer, ThermalThrottlingListener,
    WebConfigDashboard, PI_FIRMWARE_RELEASE_TAG,
};
pub use ep_bus::*;
pub use formant_bus::*;
pub use glass_bus::*;
pub use grand_piano_bus::*;
pub use graph::{Edge, GraphSchedule, NodeGraph};
pub use hurdy_gurdy_bus::*;
pub use koto_bus::*;
pub use mallet_bus::*;
pub use midi::MidiEvent;
pub use midi_clock::{MidiClockGenerator, MidiClockReceiver, MIDI_CLOCK_BYTE, MIDI_CLOCK_PPQN};
pub use midi_filter::{MidiFilterEngine, MidiInputFilter, VelocityCurve};
pub use mpe::{
    ExpressionCurveType, MpeEvent, MpeExpressionCurveEditor, MpeRouter, MpeVoiceId, MpeVoiceState,
};
pub use node::KNOWN_NODE_TYPES;
pub use param_bus::{AtomicParam, ParamBus, ParamId};
pub use pipe_organ_bus::*;
pub use plectrum_bus::*;
pub use rotary_bus::*;
pub use shakuhachi_bus::*;
pub use sidechain_matrix::*;
pub use sitar_bus::*;
pub use smoothing::SmoothParam;
pub use spatial_bus::*;
pub use spring_bus::*;
pub use tuning_matrix::*;
pub use voice::{PolyphonicVoice, VoicePool};
