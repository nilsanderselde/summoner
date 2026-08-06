// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Modular panel docking system supporting tiled splits, drag handles, and floating panels.

use crate::layout_math::{Point2D, Rect, Size2D, SpatialLayoutCalculator};
use serde::{Deserialize, Serialize};

/// Layout preset variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum DockPreset {
    #[default]
    DefaultTiled,
    DualMonitor,
    SingleFocus,
    Custom,
}

/// Direction of split between panel containers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

/// Docked or floating panel node in the layout tree.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PanelNode {
    Leaf {
        id: String,
        title: String,
        min_size: Size2D,
    },
    Split {
        direction: SplitDirection,
        ratio: f32, // 0.0 ..= 1.0
        first: Box<PanelNode>,
        second: Box<PanelNode>,
    },
}

impl PanelNode {
    pub fn leaf(id: impl Into<String>, title: impl Into<String>, min_size: Size2D) -> Self {
        PanelNode::Leaf {
            id: id.into(),
            title: title.into(),
            min_size,
        }
    }

    pub fn split(
        direction: SplitDirection,
        ratio: f32,
        first: PanelNode,
        second: PanelNode,
    ) -> Self {
        PanelNode::Split {
            direction,
            ratio: ratio.clamp(0.05, 0.95),
            first: Box::new(first),
            second: Box::new(second),
        }
    }

    /// Calculate minimum size requirements recursively.
    pub fn min_size(&self) -> Size2D {
        match self {
            PanelNode::Leaf { min_size, .. } => *min_size,
            PanelNode::Split {
                direction,
                first,
                second,
                ..
            } => {
                let s1 = first.min_size();
                let s2 = second.min_size();
                match direction {
                    SplitDirection::Horizontal => {
                        Size2D::new(s1.width + s2.width, s1.height.max(s2.height))
                    }
                    SplitDirection::Vertical => {
                        Size2D::new(s1.width.max(s2.width), s1.height + s2.height)
                    }
                }
            }
        }
    }
}

/// Floating panel description.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FloatingPanel {
    pub id: String,
    pub title: String,
    pub bounds: Rect,
    pub min_size: Size2D,
    pub is_visible: bool,
}

/// Drag handle generated between split containers for interactive resizing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DragHandle {
    pub id: usize,
    pub bounds: Rect,
    pub hit_bounds: Rect,
    pub direction: SplitDirection,
    pub current_ratio: f32,
}

/// Computed panel position result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComputedPanel {
    pub id: String,
    pub title: String,
    pub bounds: Rect,
    pub is_floating: bool,
}

/// Result of complete layout evaluation.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ComputedLayout {
    pub panels: Vec<ComputedPanel>,
    pub handles: Vec<DragHandle>,
}

/// Modular panel docking layout manager.
#[derive(Debug, Clone)]
pub struct DockingLayoutManager {
    preset: DockPreset,
    root: Option<PanelNode>,
    floating_panels: Vec<FloatingPanel>,
    viewport: Rect,
    spatial_calc: SpatialLayoutCalculator,
}

impl DockingLayoutManager {
    pub fn new(preset: DockPreset, viewport: Rect) -> Self {
        let spatial_calc = SpatialLayoutCalculator::default();
        let mut manager = Self {
            preset,
            root: None,
            floating_panels: Vec::new(),
            viewport,
            spatial_calc,
        };
        manager.load_preset(preset);
        manager
    }

    pub fn viewport(&self) -> Rect {
        self.viewport
    }

    pub fn set_viewport(&mut self, viewport: Rect) {
        self.viewport = viewport;
    }

    pub fn preset(&self) -> DockPreset {
        self.preset
    }

    pub fn root_node(&self) -> Option<&PanelNode> {
        self.root.as_ref()
    }

    pub fn floating_panels(&self) -> &[FloatingPanel] {
        &self.floating_panels
    }

    pub fn set_root(&mut self, root: PanelNode) {
        self.root = Some(root);
        self.preset = DockPreset::Custom;
    }

    pub fn load_preset(&mut self, preset: DockPreset) {
        self.preset = preset;
        self.floating_panels.clear();

        match preset {
            DockPreset::DefaultTiled => {
                // Browser (left), Center (Arranger top / Mixer bottom), Inspector (right)
                let browser = PanelNode::leaf("browser", "File Browser", Size2D::new(180.0, 200.0));
                let arranger = PanelNode::leaf("arranger", "Arranger", Size2D::new(400.0, 300.0));
                let mixer = PanelNode::leaf("mixer", "Console Mixer", Size2D::new(400.0, 200.0));
                let inspector =
                    PanelNode::leaf("inspector", "Inspector", Size2D::new(200.0, 200.0));

                let center_split =
                    PanelNode::split(SplitDirection::Vertical, 0.65, arranger, mixer);

                let main_split =
                    PanelNode::split(SplitDirection::Horizontal, 0.20, browser, center_split);

                let root =
                    PanelNode::split(SplitDirection::Horizontal, 0.80, main_split, inspector);

                self.root = Some(root);
            }

            DockPreset::DualMonitor => {
                // Extended workspace for dual displays or large Ultrawide setup
                let browser = PanelNode::leaf("browser", "File Browser", Size2D::new(220.0, 300.0));
                let arranger =
                    PanelNode::leaf("arranger", "Arranger View", Size2D::new(600.0, 400.0));
                let sequencer =
                    PanelNode::leaf("sequencer", "Pattern Sequencer", Size2D::new(400.0, 300.0));
                let mixer = PanelNode::leaf("mixer", "Master Console", Size2D::new(500.0, 250.0));

                let left_work =
                    PanelNode::split(SplitDirection::Horizontal, 0.25, browser, arranger);

                let right_work = PanelNode::split(SplitDirection::Vertical, 0.55, sequencer, mixer);

                let root =
                    PanelNode::split(SplitDirection::Horizontal, 0.60, left_work, right_work);

                self.root = Some(root);

                // Add floating plugin windows preset
                self.floating_panels.push(FloatingPanel {
                    id: "plugin_rack".into(),
                    title: "FX Plugin Rack".into(),
                    bounds: Rect::new(100.0, 100.0, 400.0, 300.0),
                    min_size: Size2D::new(250.0, 200.0),
                    is_visible: true,
                });
            }

            DockPreset::SingleFocus => {
                // Maximum focus area for Arranger with minimal side panel
                let arranger =
                    PanelNode::leaf("arranger", "Arranger Focus", Size2D::new(600.0, 400.0));
                let inspector =
                    PanelNode::leaf("inspector", "Quick Controls", Size2D::new(160.0, 200.0));

                let root = PanelNode::split(SplitDirection::Horizontal, 0.88, arranger, inspector);

                self.root = Some(root);
            }

            DockPreset::Custom => {}
        }
    }

    /// Evaluates tree layout and computes panel bounds and drag handles.
    pub fn compute_layout(&self) -> ComputedLayout {
        let mut layout = ComputedLayout::default();

        if let Some(ref root) = self.root {
            let mut handle_counter = 0;
            self.eval_node(root, self.viewport, &mut layout, &mut handle_counter);
        }

        for fp in &self.floating_panels {
            if fp.is_visible {
                layout.panels.push(ComputedPanel {
                    id: fp.id.clone(),
                    title: fp.title.clone(),
                    bounds: fp.bounds,
                    is_floating: true,
                });
            }
        }

        layout
    }

    fn eval_node(
        &self,
        node: &PanelNode,
        bounds: Rect,
        layout: &mut ComputedLayout,
        handle_counter: &mut usize,
    ) {
        match node {
            PanelNode::Leaf { id, title, .. } => {
                layout.panels.push(ComputedPanel {
                    id: id.clone(),
                    title: title.clone(),
                    bounds,
                    is_floating: false,
                });
            }
            PanelNode::Split {
                direction,
                ratio,
                first,
                second,
            } => {
                let handle_id = *handle_counter;
                *handle_counter += 1;

                let handle_thickness = 4.0;

                let (first_bounds, second_bounds, handle_bounds) = match direction {
                    SplitDirection::Horizontal => {
                        let total_w = (bounds.width - handle_thickness).max(0.0);
                        let w1 = (total_w * ratio).max(first.min_size().width);
                        let w1 = w1.min((total_w - second.min_size().width).max(0.0));
                        let w2 = (total_w - w1).max(0.0);

                        let r1 = Rect::new(bounds.x, bounds.y, w1, bounds.height);
                        let h_rect =
                            Rect::new(bounds.x + w1, bounds.y, handle_thickness, bounds.height);
                        let r2 = Rect::new(
                            bounds.x + w1 + handle_thickness,
                            bounds.y,
                            w2,
                            bounds.height,
                        );

                        (r1, r2, h_rect)
                    }
                    SplitDirection::Vertical => {
                        let total_h = (bounds.height - handle_thickness).max(0.0);
                        let h1 = (total_h * ratio).max(first.min_size().height);
                        let h1 = h1.min((total_h - second.min_size().height).max(0.0));
                        let h2 = (total_h - h1).max(0.0);

                        let r1 = Rect::new(bounds.x, bounds.y, bounds.width, h1);
                        let h_rect =
                            Rect::new(bounds.x, bounds.y + h1, bounds.width, handle_thickness);
                        let r2 =
                            Rect::new(bounds.x, bounds.y + h1 + handle_thickness, bounds.width, h2);

                        (r1, r2, h_rect)
                    }
                };

                let hit_bounds = self.spatial_calc.ensure_min_hit_target(handle_bounds);

                layout.handles.push(DragHandle {
                    id: handle_id,
                    bounds: handle_bounds,
                    hit_bounds,
                    direction: *direction,
                    current_ratio: *ratio,
                });

                self.eval_node(first, first_bounds, layout, handle_counter);
                self.eval_node(second, second_bounds, layout, handle_counter);
            }
        }
    }

    /// Modifies a split ratio for a specific handle, subject to child min width/height bounds.
    pub fn drag_handle(&mut self, target_handle_id: usize, delta: Point2D) -> bool {
        let mut handle_counter = 0;
        if let Some(ref mut root) = self.root {
            return Self::update_split_ratio(
                root,
                target_handle_id,
                delta,
                self.viewport,
                &mut handle_counter,
            );
        }
        false
    }

    fn update_split_ratio(
        node: &mut PanelNode,
        target_id: usize,
        delta: Point2D,
        bounds: Rect,
        handle_counter: &mut usize,
    ) -> bool {
        match node {
            PanelNode::Leaf { .. } => false,
            PanelNode::Split {
                direction,
                ratio,
                first,
                second,
            } => {
                let handle_id = *handle_counter;
                *handle_counter += 1;

                if handle_id == target_id {
                    let handle_thickness = 4.0;
                    match direction {
                        SplitDirection::Horizontal => {
                            let total_w = (bounds.width - handle_thickness).max(1.0);
                            let min1 = first.min_size().width;
                            let min2 = second.min_size().width;

                            let current_w1 = total_w * (*ratio);
                            let new_w1 =
                                (current_w1 + delta.x).clamp(min1, (total_w - min2).max(min1));
                            *ratio = (new_w1 / total_w).clamp(0.05, 0.95);
                        }
                        SplitDirection::Vertical => {
                            let total_h = (bounds.height - handle_thickness).max(1.0);
                            let min1 = first.min_size().height;
                            let min2 = second.min_size().height;

                            let current_h1 = total_h * (*ratio);
                            let new_h1 =
                                (current_h1 + delta.y).clamp(min1, (total_h - min2).max(min1));
                            *ratio = (new_h1 / total_h).clamp(0.05, 0.95);
                        }
                    }
                    return true;
                }

                // Recurse into children
                let handle_thickness = 4.0;
                let (first_bounds, second_bounds) = match direction {
                    SplitDirection::Horizontal => {
                        let total_w = (bounds.width - handle_thickness).max(0.0);
                        let w1 = total_w * (*ratio);
                        let r1 = Rect::new(bounds.x, bounds.y, w1, bounds.height);
                        let r2 = Rect::new(
                            bounds.x + w1 + handle_thickness,
                            bounds.y,
                            (total_w - w1).max(0.0),
                            bounds.height,
                        );
                        (r1, r2)
                    }
                    SplitDirection::Vertical => {
                        let total_h = (bounds.height - handle_thickness).max(0.0);
                        let h1 = total_h * (*ratio);
                        let r1 = Rect::new(bounds.x, bounds.y, bounds.width, h1);
                        let r2 = Rect::new(
                            bounds.x,
                            bounds.y + h1 + handle_thickness,
                            bounds.width,
                            (total_h - h1).max(0.0),
                        );
                        (r1, r2)
                    }
                };

                if Self::update_split_ratio(first, target_id, delta, first_bounds, handle_counter) {
                    return true;
                }
                Self::update_split_ratio(second, target_id, delta, second_bounds, handle_counter)
            }
        }
    }

    /// Adds a new floating panel.
    pub fn add_floating_panel(
        &mut self,
        id: impl Into<String>,
        title: impl Into<String>,
        bounds: Rect,
        min_size: Size2D,
    ) {
        self.floating_panels.push(FloatingPanel {
            id: id.into(),
            title: title.into(),
            bounds,
            min_size,
            is_visible: true,
        });
    }

    /// Move floating panel by delta.
    pub fn move_floating_panel(&mut self, id: &str, delta: Point2D) -> bool {
        if let Some(panel) = self.floating_panels.iter_mut().find(|p| p.id == id) {
            panel.bounds.x += delta.x;
            panel.bounds.y += delta.y;
            true
        } else {
            false
        }
    }

    /// Search computed bounds of panel by ID.
    pub fn get_panel_bounds(&self, id: &str) -> Option<Rect> {
        let layout = self.compute_layout();
        layout
            .panels
            .into_iter()
            .find(|p| p.id == id)
            .map(|p| p.bounds)
    }
}

impl Default for DockingLayoutManager {
    fn default() -> Self {
        Self::new(
            DockPreset::DefaultTiled,
            Rect::new(0.0, 0.0, 1920.0, 1080.0),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presets_initialization() {
        let viewport = Rect::new(0.0, 0.0, 1920.0, 1080.0);

        let default_mgr = DockingLayoutManager::new(DockPreset::DefaultTiled, viewport);
        let layout = default_mgr.compute_layout();
        assert_eq!(layout.panels.len(), 4); // browser, arranger, mixer, inspector
        assert_eq!(layout.handles.len(), 3);

        let dual_mgr = DockingLayoutManager::new(DockPreset::DualMonitor, viewport);
        let dual_layout = dual_mgr.compute_layout();
        assert_eq!(dual_layout.panels.len(), 5); // 4 tiled + 1 floating
        assert!(dual_layout.panels.iter().any(|p| p.is_floating));

        let focus_mgr = DockingLayoutManager::new(DockPreset::SingleFocus, viewport);
        let focus_layout = focus_mgr.compute_layout();
        assert_eq!(focus_layout.panels.len(), 2);
    }

    #[test]
    fn test_min_hit_target_on_drag_handles() {
        let viewport = Rect::new(0.0, 0.0, 1920.0, 1080.0);
        let mgr = DockingLayoutManager::new(DockPreset::DefaultTiled, viewport);
        let layout = mgr.compute_layout();

        for handle in &layout.handles {
            assert!(handle.hit_bounds.width >= 44.0);
            assert!(handle.hit_bounds.height >= 44.0);
        }
    }

    #[test]
    fn test_drag_handle_resizing_and_min_bounds() {
        let viewport = Rect::new(0.0, 0.0, 1000.0, 1000.0);
        let mut mgr = DockingLayoutManager::new(DockPreset::SingleFocus, viewport);
        // SingleFocus has arranger (min 600) and inspector (min 160).
        // Let's set root split ratio to 0.60 for initial test baseline
        let arranger = PanelNode::leaf("arranger", "Arranger Focus", Size2D::new(600.0, 400.0));
        let inspector = PanelNode::leaf("inspector", "Quick Controls", Size2D::new(160.0, 200.0));
        mgr.set_root(PanelNode::split(
            SplitDirection::Horizontal,
            0.65,
            arranger,
            inspector,
        ));

        let initial_layout = mgr.compute_layout();
        let handle = &initial_layout.handles[0];
        let initial_ratio = handle.current_ratio;

        // Try dragging right by 50px
        let success = mgr.drag_handle(0, Point2D::new(50.0, 0.0));
        assert!(success);

        let new_layout = mgr.compute_layout();
        assert!(new_layout.handles[0].current_ratio > initial_ratio);

        // Try dragging excessively left (past min width bounds of arranger min width 600)
        mgr.drag_handle(0, Point2D::new(-1000.0, 0.0));
        let bounded_layout = mgr.compute_layout();
        let arranger_panel = bounded_layout
            .panels
            .iter()
            .find(|p| p.id == "arranger")
            .unwrap();
        assert!(arranger_panel.bounds.width >= 600.0);
    }

    #[test]
    fn test_floating_panel_movement() {
        let viewport = Rect::new(0.0, 0.0, 1920.0, 1080.0);
        let mut mgr = DockingLayoutManager::new(DockPreset::DefaultTiled, viewport);
        mgr.add_floating_panel(
            "plugin_vst",
            "Synth Plugin",
            Rect::new(50.0, 50.0, 300.0, 200.0),
            Size2D::new(100.0, 100.0),
        );

        let moved = mgr.move_floating_panel("plugin_vst", Point2D::new(25.0, 15.0));
        assert!(moved);

        let bounds = mgr.get_panel_bounds("plugin_vst").unwrap();
        assert_eq!(bounds.x, 75.0);
        assert_eq!(bounds.y, 65.0);
    }
}
