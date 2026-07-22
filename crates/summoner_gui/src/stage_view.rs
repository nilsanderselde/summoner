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

/// High-contrast stage view for live performance.
pub struct StageView {
    pub active: bool,
    pub panic_mode: bool,
}

impl StageView {
    pub fn new() -> Self {
        Self {
            active: false,
            panic_mode: false,
        }
    }

    pub fn toggle(&mut self) {
        self.active = !self.active;
    }

    /// Instant hardware panic shortcut (Esc key mapping target).
    /// Immediately sends all-notes-off, clears feedback loops, and resets filter states.
    pub fn trigger_panic(&mut self) {
        self.panic_mode = true;
        // In full implementation, this triggers a voice flusher in the core engine.
    }
    
    pub fn clear_panic(&mut self) {
        self.panic_mode = false;
    }
}
