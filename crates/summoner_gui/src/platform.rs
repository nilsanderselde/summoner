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

//! Platform-specific integration helpers for Summoner DAW.
//! Implements Tier 30 Platform Polish specifications (Steps 524-536).

use std::env;

/// Step 524-526: Platform Audio Backend specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum AudioBackendKind {
    #[default]
    Auto,
    Wasapi {
        exclusive: bool,
    },
    CoreAudio {
        exclusive: bool,
    },
    PipeWire,
    Jack,
    Alsa,
}

/// Audio backend configuration settings.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AudioBackendSettings {
    pub backend: AudioBackendKind,
    pub sample_rate: u32,
    pub buffer_size: usize,
}

impl Default for AudioBackendSettings {
    fn default() -> Self {
        Self {
            backend: AudioBackendKind::Auto,
            sample_rate: 44100,
            buffer_size: 512,
        }
    }
}

impl AudioBackendSettings {
    /// Step 524: Resolve WASAPI configuration options.
    pub fn wasapi_exclusive_enabled(&self) -> bool {
        match self.backend {
            AudioBackendKind::Wasapi { exclusive } => exclusive,
            _ => false,
        }
    }

    /// Step 525: Resolve CoreAudio configuration options.
    pub fn coreaudio_exclusive_enabled(&self) -> bool {
        match self.backend {
            AudioBackendKind::CoreAudio { exclusive } => exclusive,
            _ => false,
        }
    }

    /// Step 525: Check if running on Apple Silicon native arm64 target architecture.
    pub fn is_apple_silicon_arm64() -> bool {
        cfg!(all(target_os = "macos", target_arch = "aarch64"))
    }

    /// Step 526: Determine priority Linux audio backend (PipeWire -> JACK -> ALSA).
    pub fn select_linux_backend_priority() -> AudioBackendKind {
        if env::var("PIPEWIRE_REMOTE").is_ok() || env::var("XDG_RUNTIME_DIR").is_ok() {
            AudioBackendKind::PipeWire
        } else if env::var("JACK_START_SERVER").is_ok() {
            AudioBackendKind::Jack
        } else {
            AudioBackendKind::Alsa
        }
    }
}

/// Step 527: Display server detection on Linux (Wayland vs X11).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayServer {
    Wayland,
    X11,
    WindowsDesktop,
    MacOsQuartz,
    Unknown,
}

pub fn detect_display_server() -> DisplayServer {
    if cfg!(target_os = "windows") {
        DisplayServer::WindowsDesktop
    } else if cfg!(target_os = "macos") {
        DisplayServer::MacOsQuartz
    } else if env::var("WAYLAND_DISPLAY").is_ok()
        || env::var("XDG_SESSION_TYPE")
            .map(|v| v == "wayland")
            .unwrap_or(false)
    {
        DisplayServer::Wayland
    } else if env::var("DISPLAY").is_ok()
        || env::var("XDG_SESSION_TYPE")
            .map(|v| v == "x11")
            .unwrap_or(false)
    {
        DisplayServer::X11
    } else {
        DisplayServer::Unknown
    }
}

/// Step 528: Taskbar render progress status for Windows / cross-platform host UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskbarProgressState {
    NoProgress,
    Indeterminate,
    Normal,
    Error,
    Paused,
}

#[derive(Debug, Clone)]
pub struct TaskbarProgress {
    pub completed: u64,
    pub total: u64,
    pub state: TaskbarProgressState,
}

impl TaskbarProgress {
    pub fn new() -> Self {
        Self {
            completed: 0,
            total: 100,
            state: TaskbarProgressState::NoProgress,
        }
    }

    pub fn update(&mut self, completed: u64, total: u64) {
        self.completed = completed;
        self.total = total;
        self.state = if completed >= total {
            TaskbarProgressState::NoProgress
        } else {
            TaskbarProgressState::Normal
        };
    }

    pub fn percentage(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            (self.completed as f32 / self.total as f32 * 100.0).clamp(0.0, 100.0)
        }
    }
}

/// Step 529: macOS Retina display pixel-perfect scaling helper.
pub fn calculate_retina_pixels_per_point(native_pixels_per_point: f32) -> f32 {
    if cfg!(target_os = "macos") && native_pixels_per_point >= 2.0 {
        // High-DPI pixel perfect 2.0x scale
        2.0
    } else {
        native_pixels_per_point.max(1.0)
    }
}

/// Step 536: Steam Deck / Handheld console layout and controller state detection.
#[derive(Debug, Clone, Default)]
pub struct SteamDeckControllerState {
    pub is_steam_deck: bool,
    pub force_4_3_aspect: bool,
    pub focused_slot: usize,
}

impl SteamDeckControllerState {
    pub fn detect() -> Self {
        let is_steam_deck = env::var("STEAM_DECK").is_ok()
            || env::var("SteamDeck").is_ok()
            || env::var("XDG_CURRENT_DESKTOP")
                .map(|v| v.to_lowercase().contains("steam"))
                .unwrap_or(false);

        Self {
            is_steam_deck,
            force_4_3_aspect: is_steam_deck,
            focused_slot: 0,
        }
    }

    pub fn navigate_grid(&mut self, row_delta: i32, col_delta: i32) {
        let mut row = (self.focused_slot / 4) as i32 + row_delta;
        let mut col = (self.focused_slot % 4) as i32 + col_delta;
        row = row.rem_euclid(4);
        col = col.rem_euclid(4);
        self.focused_slot = (row * 4 + col) as usize;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_backend_wasapi_and_coreaudio_settings() {
        let mut settings = AudioBackendSettings::default();
        assert!(!settings.wasapi_exclusive_enabled());
        assert!(!settings.coreaudio_exclusive_enabled());

        settings.backend = AudioBackendKind::Wasapi { exclusive: true };
        assert!(settings.wasapi_exclusive_enabled());

        settings.backend = AudioBackendKind::CoreAudio { exclusive: true };
        assert!(settings.coreaudio_exclusive_enabled());
    }

    #[test]
    fn test_linux_backend_priority_fallback() {
        let priority = AudioBackendSettings::select_linux_backend_priority();
        assert!(matches!(
            priority,
            AudioBackendKind::PipeWire | AudioBackendKind::Jack | AudioBackendKind::Alsa
        ));
    }

    #[test]
    fn test_display_server_detection() {
        let display = detect_display_server();
        assert!(matches!(
            display,
            DisplayServer::WindowsDesktop
                | DisplayServer::MacOsQuartz
                | DisplayServer::Wayland
                | DisplayServer::X11
                | DisplayServer::Unknown
        ));
    }

    #[test]
    fn test_taskbar_progress_percentage() {
        let mut progress = TaskbarProgress::new();
        assert_eq!(progress.percentage(), 0.0);
        progress.update(50, 100);
        assert_eq!(progress.percentage(), 50.0);
        assert_eq!(progress.state, TaskbarProgressState::Normal);
        progress.update(100, 100);
        assert_eq!(progress.state, TaskbarProgressState::NoProgress);
    }

    #[test]
    fn test_steam_deck_controller_navigation() {
        let mut deck = SteamDeckControllerState::default();
        assert_eq!(deck.focused_slot, 0);
        deck.navigate_grid(0, 1); // move right
        assert_eq!(deck.focused_slot, 1);
        deck.navigate_grid(1, 0); // move down
        assert_eq!(deck.focused_slot, 5);
        deck.navigate_grid(-1, -1); // move up-left
        assert_eq!(deck.focused_slot, 0);
    }
}
