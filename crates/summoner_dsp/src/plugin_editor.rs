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

//! Native plugin GUI window host bridge supporting HWND (Windows), NSView (macOS),
//! and X11 (Linux) child window embedding (Task F6).

use serde::{Deserialize, Serialize};
use crate::vst3_host::ViewRect;

/// Target operating system windowing platform type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativePlatformWindowType {
    /// Windows Win32 HWND child window (`kPlatformTypeHWND`).
    Hwnd,
    /// macOS Cocoa NSView pointer (`kPlatformTypeNSView`).
    NsView,
    /// Linux X11 XEmbed window ID (`kPlatformTypeX11EmbedWindowID`).
    X11,
    /// Wayland (Wayland does not support sub-surface foreign reparenting; floating window fallback).
    WaylandFloating,
}

impl NativePlatformWindowType {
    pub fn current() -> Self {
        #[cfg(target_os = "windows")]
        {
            Self::Hwnd
        }
        #[cfg(target_os = "macos")]
        {
            Self::NsView
        }
        #[cfg(target_os = "linux")]
        {
            if std::env::var("WAYLAND_DISPLAY").is_ok() {
                Self::WaylandFloating
            } else {
                Self::X11
            }
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            Self::Hwnd
        }
    }

    pub fn to_vst3_type_str(&self) -> &'static str {
        match self {
            Self::Hwnd => "HWND",
            Self::NsView => "NSView",
            Self::X11 => "X11EmbedWindowID",
            Self::WaylandFloating => "WaylandFloating",
        }
    }
}

/// Life-cycle status of an embedded native plugin window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PluginWindowState {
    #[default]
    Detached,
    Attached,
    Hidden,
    Closed,
}

/// Native plugin editor window host manager handling child window embedding,
/// size constraints, high-DPI scaling, and focus management.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativePluginWindowHost {
    pub plugin_id: String,
    pub title: String,
    pub platform: NativePlatformWindowType,
    pub parent_handle: usize,
    pub child_handle: usize,
    pub current_rect: ViewRect,
    pub min_size: (i32, i32),
    pub max_size: (i32, i32),
    pub scale_factor: f32,
    pub can_resize: bool,
    pub state: PluginWindowState,
}

impl NativePluginWindowHost {
    pub fn new(plugin_id: &str, title: &str) -> Self {
        Self {
            plugin_id: plugin_id.to_string(),
            title: title.to_string(),
            platform: NativePlatformWindowType::current(),
            parent_handle: 0,
            child_handle: 0,
            current_rect: ViewRect {
                left: 0,
                top: 0,
                right: 800,
                bottom: 600,
            },
            min_size: (300, 200),
            max_size: (3840, 2160),
            scale_factor: 1.0,
            can_resize: true,
            state: PluginWindowState::Detached,
        }
    }

    /// Attach plugin view to a native OS parent window handle.
    pub fn attach_to_parent(&mut self, parent_handle: usize) -> Result<(), String> {
        if parent_handle == 0 {
            return Err("Parent native window handle cannot be null".into());
        }
        self.parent_handle = parent_handle;
        self.state = PluginWindowState::Attached;
        Ok(())
    }

    /// Request window resizing according to plugin constraints.
    pub fn resize(&mut self, width: i32, height: i32) -> Result<ViewRect, String> {
        if !self.can_resize {
            return Ok(self.current_rect);
        }

        let clamped_w = width.clamp(self.min_size.0, self.max_size.0);
        let clamped_h = height.clamp(self.min_size.1, self.max_size.1);

        self.current_rect.right = self.current_rect.left + clamped_w;
        self.current_rect.bottom = self.current_rect.top + clamped_h;

        Ok(self.current_rect)
    }

    /// Set High-DPI scale factor (e.g. 1.25, 1.5, 2.0).
    pub fn set_dpi_scale(&mut self, scale: f32) {
        if scale > 0.1 && scale < 10.0 {
            self.scale_factor = scale;
        }
    }

    /// Detach and close the native window view.
    pub fn detach(&mut self) {
        self.parent_handle = 0;
        self.child_handle = 0;
        self.state = PluginWindowState::Detached;
    }

    pub fn width(&self) -> i32 {
        self.current_rect.right - self.current_rect.left
    }

    pub fn height(&self) -> i32 {
        self.current_rect.bottom - self.current_rect.top
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_native_plugin_window_lifecycle_and_resizing() {
        let mut host = NativePluginWindowHost::new("vst3.synthesizer", "Synthesizer Pro");
        assert_eq!(host.state, PluginWindowState::Detached);
        assert_eq!(host.width(), 800);
        assert_eq!(host.height(), 600);

        // Fail on null handle
        assert!(host.attach_to_parent(0).is_err());

        // Attach with mock native handle
        assert!(host.attach_to_parent(0x12345678).is_ok());
        assert_eq!(host.state, PluginWindowState::Attached);

        // Resize within bounds
        let rect = host.resize(1024, 768).unwrap();
        assert_eq!(rect.right - rect.left, 1024);
        assert_eq!(rect.bottom - rect.top, 768);

        // Clamp to min/max
        let rect_clamped = host.resize(100, 5000).unwrap();
        assert_eq!(rect_clamped.right - rect_clamped.left, 300); // clamped to min 300
        assert_eq!(rect_clamped.bottom - rect_clamped.top, 2160); // clamped to max 2160

        // Detach
        host.detach();
        assert_eq!(host.state, PluginWindowState::Detached);
        assert_eq!(host.parent_handle, 0);
    }
}

