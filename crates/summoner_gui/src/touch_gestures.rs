// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

//! Touch screen hit targets and gesture recognition (Steps 745-747).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwipeDirection {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewModeDirection {
    NextView,
    PrevView,
}

/// Step 745: Touch gesture manager for touch screens and Stage View swipe controls.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TouchGestureManager {
    pub hit_target_scale: f32,
    pub touch_active: bool,
}

impl Default for TouchGestureManager {
    fn default() -> Self {
        Self {
            hit_target_scale: 1.5,
            touch_active: false,
        }
    }
}

impl TouchGestureManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Step 745: Scale button or widget hit dimensions for touch compatibility.
    pub fn scaled_target_size(&self, base_size: (f32, f32)) -> (f32, f32) {
        (
            base_size.0 * self.hit_target_scale,
            base_size.1 * self.hit_target_scale,
        )
    }

    /// Step 745: Detect swipe direction given start and end touch coordinates.
    pub fn detect_swipe(&self, start: (f32, f32), end: (f32, f32)) -> Option<SwipeDirection> {
        let dx = end.0 - start.0;
        let dy = end.1 - start.1;
        let threshold = 30.0;

        if dx.abs() > dy.abs() {
            if dx > threshold {
                Some(SwipeDirection::Right)
            } else if dx < -threshold {
                Some(SwipeDirection::Left)
            } else {
                None
            }
        } else {
            if dy > threshold {
                Some(SwipeDirection::Down)
            } else if dy < -threshold {
                Some(SwipeDirection::Up)
            } else {
                None
            }
        }
    }

    /// Step 746: Calculate two-finger pinch-to-zoom ratio based on touch point distance change.
    pub fn two_finger_zoom(dist_start: f32, dist_current: f32) -> f32 {
        if dist_start <= 0.001 {
            1.0
        } else {
            (dist_current / dist_start).clamp(0.2, 5.0)
        }
    }

    /// Step 747: Detect three-finger horizontal swipe gesture to switch GUI view mode.
    pub fn three_finger_swipe(
        starts: &[(f32, f32); 3],
        ends: &[(f32, f32); 3],
    ) -> Option<ViewModeDirection> {
        let avg_dx: f32 = starts
            .iter()
            .zip(ends.iter())
            .map(|(s, e)| e.0 - s.0)
            .sum::<f32>()
            / 3.0;
        let threshold = 40.0;

        if avg_dx > threshold {
            Some(ViewModeDirection::PrevView)
        } else if avg_dx < -threshold {
            Some(ViewModeDirection::NextView)
        } else {
            None
        }
    }
}
