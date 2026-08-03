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
#![warn(missing_docs)]

pub mod allocator;
pub mod audio;
pub mod mpe;
pub mod node;
pub mod pipeline;
pub mod sample;
pub mod sequence;
pub mod track;
pub mod transport;
pub mod wav;
pub mod panner;
pub mod midi;
pub mod smoothing;
pub mod graph;
pub mod voice;
pub mod param_bus;
pub mod midi_clock;
pub mod audio_drivers;
pub mod embedded_hardware;
pub mod midi_filter;
pub mod adaptive_buffer;
pub use adaptive_buffer::AdaptiveBufferScaler;
pub use audio::{ChannelLayout, Frame, FixedAudioBuffer, MultichannelAudioBuffer, Sample};
pub use midi::MidiEvent;
pub use mpe::{MpeEvent, MpeVoiceId, MpeVoiceState, MpeRouter, MpeExpressionCurveEditor, ExpressionCurveType};
pub use midi_filter::{MidiFilterEngine, MidiInputFilter, VelocityCurve};
pub use smoothing::SmoothParam;
pub use graph::{Edge, NodeGraph};
pub use voice::{PolyphonicVoice, VoicePool};
pub use param_bus::{AtomicParam, ParamBus, ParamId};
pub use midi_clock::{MidiClockGenerator, MidiClockReceiver, MIDI_CLOCK_BYTE, MIDI_CLOCK_PPQN};
pub use audio_drivers::{
    AAudioDriver, AlsaDriver, AsapiDriver, AudioUnitDriver, NativeAudioDriver,
    NativeAudioDriverTuner, WasapiDriver,
};
pub use embedded_hardware::{
    BatteryMonitor, BleMidiPeripheral, BootToSynthEngine, BypassRelayTrigger,
    EepromPresetStore, EmbeddedHardwareConfig, EurorackCvGateInterface, GpioDriver,
    GpioEvent, HardwareEmulationHarness, HardwareWatchdogService, MemoryEstimator,
    MidiUsbGadgetMode, MidiUartSerialDriver, OledDisplayDriver, RotaryEncoderDebouncer,
    ThermalThrottlingListener, WebConfigDashboard, PI_FIRMWARE_RELEASE_TAG,
};
pub use node::KNOWN_NODE_TYPES;


