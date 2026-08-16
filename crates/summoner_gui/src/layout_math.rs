// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Spatial math, bounding box collision validation, and DPI scaling engine.

use serde::{Deserialize, Serialize};

/// Supported operating systems for platform-specific layout calculations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OperatingSystem {
    Windows,
    MacOS,
    Linux,
}

impl OperatingSystem {
    /// Detects current operating system based on compilation target.
    pub fn current() -> Self {
        if cfg!(target_os = "windows") {
            OperatingSystem::Windows
        } else if cfg!(target_os = "macos") {
            OperatingSystem::MacOS
        } else {
            OperatingSystem::Linux
        }
    }
}

/// Platform-specific layout style configuration including DPI scale factor and scrollbar padding.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PlatformStyleConfig {
    pub os: OperatingSystem,
    pub dpi_scale: f32,
    pub scrollbar_padding_px: f32,
    pub min_hit_target_pt: f32,
}

impl PlatformStyleConfig {
    /// Generates platform defaults:
    /// - Windows: 1.25x DPI scale default, 17px scrollbar pad, 44pt min hit target
    /// - macOS: 1.0x DPI scale, 0px overlay scrollbar pad, 44pt min hit target
    /// - Linux: 1.0x DPI scale, 14px scrollbar pad, 44pt min hit target
    pub fn for_os(os: OperatingSystem) -> Self {
        match os {
            OperatingSystem::Windows => Self {
                os,
                dpi_scale: 1.25,
                scrollbar_padding_px: 17.0,
                min_hit_target_pt: 44.0,
            },
            OperatingSystem::MacOS => Self {
                os,
                dpi_scale: 1.0,
                scrollbar_padding_px: 0.0,
                min_hit_target_pt: 44.0,
            },
            OperatingSystem::Linux => Self {
                os,
                dpi_scale: 1.0,
                scrollbar_padding_px: 14.0,
                min_hit_target_pt: 44.0,
            },
        }
    }

    /// Default config for the host OS.
    pub fn current_host() -> Self {
        Self::for_os(OperatingSystem::current())
    }
}

impl Default for PlatformStyleConfig {
    fn default() -> Self {
        Self::current_host()
    }
}

/// 2D Point.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Point2D {
    pub x: f32,
    pub y: f32,
}

impl Point2D {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// 2D Size.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Size2D {
    pub width: f32,
    pub height: f32,
}

impl Size2D {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

/// Explicit Padding around elements.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Padding {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Padding {
    pub fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    pub fn uniform(val: f32) -> Self {
        Self::new(val, val, val, val)
    }

    pub fn symmetric(vertical: f32, horizontal: f32) -> Self {
        Self::new(vertical, horizontal, vertical, horizontal)
    }

    pub fn total_horizontal(&self) -> f32 {
        self.left + self.right
    }

    pub fn total_vertical(&self) -> f32 {
        self.top + self.bottom
    }
}

/// Explicit Margin around elements.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Margin {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Margin {
    pub fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    pub fn uniform(val: f32) -> Self {
        Self::new(val, val, val, val)
    }

    pub fn symmetric(vertical: f32, horizontal: f32) -> Self {
        Self::new(vertical, horizontal, vertical, horizontal)
    }

    pub fn total_horizontal(&self) -> f32 {
        self.left + self.right
    }

    pub fn total_vertical(&self) -> f32 {
        self.top + self.bottom
    }
}

/// Axis-Aligned Bounding Box (AABB) Rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width: width.max(0.0),
            height: height.max(0.0),
        }
    }

    pub fn from_min_max(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Self {
        Self::new(
            min_x,
            min_y,
            (max_x - min_x).max(0.0),
            (max_y - min_y).max(0.0),
        )
    }

    pub fn min_x(&self) -> f32 {
        self.x
    }

    pub fn max_x(&self) -> f32 {
        self.x + self.width
    }

    pub fn min_y(&self) -> f32 {
        self.y
    }

    pub fn max_y(&self) -> f32 {
        self.y + self.height
    }

    pub fn center(&self) -> Point2D {
        Point2D::new(self.x + self.width * 0.5, self.y + self.height * 0.5)
    }

    pub fn contains_point(&self, point: Point2D) -> bool {
        point.x >= self.min_x()
            && point.x <= self.max_x()
            && point.y >= self.min_y()
            && point.y <= self.max_y()
    }

    pub fn contains(&self, x: f32, y: f32) -> bool {
        self.contains_point(Point2D::new(x, y))
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        self.min_x() < other.max_x()
            && self.max_x() > other.min_x()
            && self.min_y() < other.max_y()
            && self.max_y() > other.min_y()
    }

    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        if !self.intersects(other) {
            return None;
        }
        let min_x = self.min_x().max(other.min_x());
        let max_x = self.max_x().min(other.max_x());
        let min_y = self.min_y().max(other.min_y());
        let max_y = self.max_y().min(other.max_y());
        Some(Rect::from_min_max(min_x, min_y, max_x, max_y))
    }

    pub fn shrink(&self, padding: Padding) -> Rect {
        let x = self.x + padding.left;
        let y = self.y + padding.top;
        let width = (self.width - padding.total_horizontal()).max(0.0);
        let height = (self.height - padding.total_vertical()).max(0.0);
        Rect::new(x, y, width, height)
    }

    pub fn expand(&self, margin: Margin) -> Rect {
        let x = self.x - margin.left;
        let y = self.y - margin.top;
        let width = self.width + margin.total_horizontal();
        let height = self.height + margin.total_vertical();
        Rect::new(x, y, width, height)
    }

    /// Enforces minimum hit target dimensions while keeping center fixed.
    pub fn enforce_min_hit_target(&self, min_target: f32) -> Rect {
        let mut new_width = self.width;
        let mut new_height = self.height;
        let mut new_x = self.x;
        let mut new_y = self.y;

        if new_width < min_target {
            let diff = min_target - new_width;
            new_x -= diff * 0.5;
            new_width = min_target;
        }

        if new_height < min_target {
            let diff = min_target - new_height;
            new_y -= diff * 0.5;
            new_height = min_target;
        }

        Rect::new(new_x, new_y, new_width, new_height)
    }
}

/// Isolated spatial math and bounding box collision validation engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpatialLayoutCalculator {
    config: PlatformStyleConfig,
}

impl SpatialLayoutCalculator {
    pub fn new(config: PlatformStyleConfig) -> Self {
        Self { config }
    }

    pub fn for_os(os: OperatingSystem) -> Self {
        Self::new(PlatformStyleConfig::for_os(os))
    }

    pub fn config(&self) -> &PlatformStyleConfig {
        &self.config
    }

    pub fn set_config(&mut self, config: PlatformStyleConfig) {
        self.config = config;
    }

    pub fn dpi_scale_factor(&self) -> f32 {
        self.config.dpi_scale
    }

    /// Scrollbar padding in pixels adjusted for platform and DPI.
    pub fn scrollbar_padding_px(&self) -> f32 {
        self.config.scrollbar_padding_px * self.config.dpi_scale
    }

    /// Minimum hit target size in physical points/pixels.
    pub fn min_hit_target_pt(&self) -> f32 {
        self.config.min_hit_target_pt * self.config.dpi_scale
    }

    /// Scale padding by DPI scale factor.
    pub fn calculate_padding(&self, base_padding: Padding) -> Padding {
        let scale = self.config.dpi_scale;
        Padding::new(
            base_padding.top * scale,
            base_padding.right * scale,
            base_padding.bottom * scale,
            base_padding.left * scale,
        )
    }

    /// Scale margin by DPI scale factor.
    pub fn calculate_margin(&self, base_margin: Margin) -> Margin {
        let scale = self.config.dpi_scale;
        Margin::new(
            base_margin.top * scale,
            base_margin.right * scale,
            base_margin.bottom * scale,
            base_margin.left * scale,
        )
    }

    /// Ensures a bounding box meets the platform's minimum hit target.
    pub fn ensure_min_hit_target(&self, bounds: Rect) -> Rect {
        bounds.enforce_min_hit_target(self.min_hit_target_pt())
    }

    /// Calculate flex proportions for a list of relative weights.
    pub fn calculate_flex_ratios(
        &self,
        total_span: f32,
        flex_weights: &[f32],
        gap: f32,
    ) -> Vec<f32> {
        if flex_weights.is_empty() {
            return Vec::new();
        }
        let total_weight: f32 = flex_weights.iter().sum();
        if total_weight <= 0.0 {
            let equal = total_span / flex_weights.len() as f32;
            return vec![equal; flex_weights.len()];
        }

        let num_gaps = (flex_weights.len() - 1) as f32;
        let available_span = (total_span - (num_gaps * gap)).max(0.0);

        flex_weights
            .iter()
            .map(|&w| (w / total_weight) * available_span)
            .collect()
    }

    /// Layout children in horizontal or vertical flex orientation.
    pub fn calculate_flex_layout(
        &self,
        container: Rect,
        flex_weights: &[f32],
        is_horizontal: bool,
        gap: f32,
    ) -> Vec<Rect> {
        if flex_weights.is_empty() {
            return Vec::new();
        }

        let total_span = if is_horizontal {
            container.width
        } else {
            container.height
        };

        let sizes = self.calculate_flex_ratios(total_span, flex_weights, gap);
        let mut rects = Vec::with_capacity(flex_weights.len());

        let mut offset = if is_horizontal {
            container.x
        } else {
            container.y
        };

        for size in sizes {
            if is_horizontal {
                rects.push(Rect::new(offset, container.y, size, container.height));
            } else {
                rects.push(Rect::new(container.x, offset, container.width, size));
            }
            offset += size + gap;
        }

        rects
    }

    /// Validates if two rectangles collide.
    pub fn check_collision(&self, rect_a: Rect, rect_b: Rect) -> bool {
        rect_a.intersects(&rect_b)
    }

    /// Finds all indices of elements that collide with target rect.
    pub fn find_collisions(&self, target: Rect, elements: &[Rect]) -> Vec<usize> {
        elements
            .iter()
            .enumerate()
            .filter_map(|(idx, elem)| {
                if target.intersects(elem) {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Resolves collision by sliding target out of obstacle along minimum translation vector.
    pub fn resolve_collision_slide(&self, target: Rect, obstacle: Rect) -> Rect {
        let intersection = match target.intersection(&obstacle) {
            Some(rect) => rect,
            None => return target,
        };

        let target_center = target.center();
        let obstacle_center = obstacle.center();

        let mut resolved = target;

        if intersection.width < intersection.height {
            if target_center.x < obstacle_center.x {
                resolved.x -= intersection.width;
            } else {
                resolved.x += intersection.width;
            }
        } else {
            if target_center.y < obstacle_center.y {
                resolved.y -= intersection.height;
            } else {
                resolved.y += intersection.height;
            }
        }

        resolved
    }
}

impl Default for SpatialLayoutCalculator {
    fn default() -> Self {
        Self::new(PlatformStyleConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_defaults() {
        let win_cfg = PlatformStyleConfig::for_os(OperatingSystem::Windows);
        assert_eq!(win_cfg.dpi_scale, 1.25);
        assert_eq!(win_cfg.scrollbar_padding_px, 17.0);
        assert_eq!(win_cfg.min_hit_target_pt, 44.0);

        let mac_cfg = PlatformStyleConfig::for_os(OperatingSystem::MacOS);
        assert_eq!(mac_cfg.dpi_scale, 1.0);
        assert_eq!(mac_cfg.scrollbar_padding_px, 0.0);
        assert_eq!(mac_cfg.min_hit_target_pt, 44.0);

        let lin_cfg = PlatformStyleConfig::for_os(OperatingSystem::Linux);
        assert_eq!(lin_cfg.dpi_scale, 1.0);
        assert_eq!(lin_cfg.scrollbar_padding_px, 14.0);
        assert_eq!(lin_cfg.min_hit_target_pt, 44.0);
    }

    #[test]
    fn test_dpi_scaled_calculations() {
        let win_calc =
            SpatialLayoutCalculator::new(PlatformStyleConfig::for_os(OperatingSystem::Windows));
        assert_eq!(win_calc.scrollbar_padding_px(), 17.0 * 1.25);
        assert_eq!(win_calc.min_hit_target_pt(), 44.0 * 1.25);

        let base_padding = Padding::new(10.0, 20.0, 10.0, 20.0);
        let scaled_padding = win_calc.calculate_padding(base_padding);
        assert_eq!(scaled_padding, Padding::new(12.5, 25.0, 12.5, 25.0));
    }

    #[test]
    fn test_min_hit_target_enforcement() {
        let mac_calc =
            SpatialLayoutCalculator::new(PlatformStyleConfig::for_os(OperatingSystem::MacOS));
        let small_rect = Rect::new(100.0, 100.0, 20.0, 20.0);
        let enforced = mac_calc.ensure_min_hit_target(small_rect);

        assert_eq!(enforced.width, 44.0);
        assert_eq!(enforced.height, 44.0);
        assert_eq!(enforced.center(), small_rect.center());
    }

    #[test]
    fn test_rect_intersections_and_collisions() {
        let r1 = Rect::new(0.0, 0.0, 100.0, 100.0);
        let r2 = Rect::new(50.0, 50.0, 100.0, 100.0);
        let r3 = Rect::new(200.0, 200.0, 50.0, 50.0);

        let calc = SpatialLayoutCalculator::default();
        assert!(calc.check_collision(r1, r2));
        assert!(!calc.check_collision(r1, r3));

        let collisions = calc.find_collisions(r1, &[r2, r3]);
        assert_eq!(collisions, vec![0]);
    }

    #[test]
    fn test_collision_slide_resolution() {
        let calc = SpatialLayoutCalculator::default();
        let target = Rect::new(40.0, 0.0, 50.0, 50.0);
        let obstacle = Rect::new(0.0, 0.0, 50.0, 50.0);

        // Slide target out to the right
        let resolved = calc.resolve_collision_slide(target, obstacle);
        assert_eq!(resolved.x, 50.0);
        assert!(!calc.check_collision(resolved, obstacle));
    }

    #[test]
    fn test_flex_layout_horizontal_and_vertical() {
        let calc = SpatialLayoutCalculator::default();
        let container = Rect::new(0.0, 0.0, 300.0, 100.0);
        let weights = vec![1.0, 2.0];

        let rects = calc.calculate_flex_layout(container, &weights, true, 0.0);
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0], Rect::new(0.0, 0.0, 100.0, 100.0));
        assert_eq!(rects[1], Rect::new(100.0, 0.0, 200.0, 100.0));
    }
}
