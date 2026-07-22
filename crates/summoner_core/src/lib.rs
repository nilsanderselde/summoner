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
pub use smoothing::SmoothParam;
pub use graph::{Edge, NodeGraph};
pub use voice::{PolyphonicVoice, VoicePool};
pub use param_bus::{AtomicParam, ParamBus, ParamId};
