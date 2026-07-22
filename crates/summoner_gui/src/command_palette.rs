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

/// Command Palette for rapid keyboard-first navigation (Cmd+K).
pub struct CommandPalette {
    pub is_open: bool,
    pub search_query: String,
}

impl CommandPalette {
    pub fn new() -> Self {
        Self {
            is_open: false,
            search_query: String::new(),
        }
    }

    pub fn open(&mut self) {
        self.is_open = true;
        self.search_query.clear();
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }

    pub fn execute_search(&self) -> Option<String> {
        if self.search_query.is_empty() {
            None
        } else {
            // Scaffold: Would fuzzy search across node types, tracks, and parameters.
            Some(format!("Routed: {}", self.search_query))
        }
    }
}
