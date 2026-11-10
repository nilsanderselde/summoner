// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Live Modular Patching Canvas & Reason-Style Rack Rear Panel Routing Engine (Tasks H1 & H6).
//!
//! Provides:
//! - Drag-to-connect audio, CV, gate, and MIDI patch cables while the audio engine runs.
//! - Subpatches and modular abstractions with exposed inlets, outlets, and controls.
//! - Presentation Mode (Max/MSP-style clean macro layout hiding spaghetti cables).
//! - Rear Panel Mode (Reason-style Tab key 3D flip revealing jacks and hanging physics cables).

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Cable signal type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CableType {
    Audio,
    Cv,
    Gate,
    Midi,
}

impl CableType {
    pub fn default_color(&self) -> [u8; 4] {
        match self {
            Self::Audio => [230, 40, 40, 255],  // Red
            Self::Cv => [240, 200, 20, 255],   // Yellow
            Self::Gate => [50, 220, 80, 255],   // Green
            Self::Midi => [180, 80, 240, 255],  // Purple
        }
    }
}

/// Port connector on a canvas node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasPort {
    pub id: usize,
    pub name: String,
    pub cable_type: CableType,
    pub is_input: bool,
    pub pos_offset: (f32, f32),
}

/// An exposed macro control parameter on a canvas node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasParam {
    pub id: String,
    pub name: String,
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub exposed_in_presentation: bool,
    pub presentation_pos: (f32, f32),
}

/// A node on the modular patching canvas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasNode {
    pub id: usize,
    pub name: String,
    pub category: String,
    pub pos: (f32, f32),
    pub size: (f32, f32),
    pub inputs: Vec<CanvasPort>,
    pub outputs: Vec<CanvasPort>,
    pub params: Vec<CanvasParam>,
    pub is_subpatch: bool,
    pub subpatch_id: Option<usize>,
}

impl CanvasNode {
    pub fn new(id: usize, name: &str, category: &str, pos: (f32, f32)) -> Self {
        Self {
            id,
            name: name.to_string(),
            category: category.to_string(),
            pos,
            size: (180.0, 120.0),
            inputs: Vec::new(),
            outputs: Vec::new(),
            params: Vec::new(),
            is_subpatch: false,
            subpatch_id: None,
        }
    }

    pub fn add_input(&mut self, port_id: usize, name: &str, cable_type: CableType) {
        let count = self.inputs.len() as f32;
        self.inputs.push(CanvasPort {
            id: port_id,
            name: name.to_string(),
            cable_type,
            is_input: true,
            pos_offset: (0.0, 25.0 + count * 22.0),
        });
    }

    pub fn add_output(&mut self, port_id: usize, name: &str, cable_type: CableType) {
        let count = self.outputs.len() as f32;
        self.outputs.push(CanvasPort {
            id: port_id,
            name: name.to_string(),
            cable_type,
            is_input: false,
            pos_offset: (self.size.0, 25.0 + count * 22.0),
        });
    }

    pub fn add_param(&mut self, id: &str, name: &str, val: f32, min: f32, max: f32) {
        self.params.push(CanvasParam {
            id: id.to_string(),
            name: name.to_string(),
            value: val,
            min,
            max,
            exposed_in_presentation: false,
            presentation_pos: (0.0, 0.0),
        });
    }

    pub fn expose_param(&mut self, param_id: &str, pres_pos: (f32, f32)) -> bool {
        if let Some(param) = self.params.iter_mut().find(|p| p.id == param_id) {
            param.exposed_in_presentation = true;
            param.presentation_pos = pres_pos;
            true
        } else {
            false
        }
    }
}

/// A connection cable between two canvas node ports.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasCable {
    pub id: usize,
    pub from_node: usize,
    pub from_port: usize,
    pub to_node: usize,
    pub to_port: usize,
    pub cable_type: CableType,
    pub color_rgba: [u8; 4],
    pub sag_factor: f32,
    pub muted: bool,
}

/// A packaged subpatch abstraction holding encapsulated nodes and internal wiring.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Subpatch {
    pub id: usize,
    pub name: String,
    pub inner_nodes: Vec<CanvasNode>,
    pub inner_cables: Vec<CanvasCable>,
    pub inlets: Vec<CanvasPort>,
    pub outlets: Vec<CanvasPort>,
}

/// Active drag interaction state when pulling a new cable from a jack.
#[derive(Debug, Clone, PartialEq)]
pub struct ActiveCableDrag {
    pub from_node: usize,
    pub from_port: usize,
    pub cable_type: CableType,
    pub current_pointer_pos: (f32, f32),
}

/// The interactive live modular patching canvas state and engine.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PatchCanvas {
    pub nodes: Vec<CanvasNode>,
    pub cables: Vec<CanvasCable>,
    pub subpatches: Vec<Subpatch>,
    pub presentation_mode: bool,
    pub rear_panel_flipped: bool, // Task H6: Reason-style rear panel flip
    next_id: usize,
    #[serde(skip)]
    pub active_drag: Option<ActiveCableDrag>,
}

impl PatchCanvas {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            cables: Vec::new(),
            subpatches: Vec::new(),
            presentation_mode: false,
            rear_panel_flipped: false,
            next_id: 1,
            active_drag: None,
        }
    }

    /// Add a node to the canvas.
    pub fn add_node(&mut self, mut node: CanvasNode) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        node.id = id;
        self.nodes.push(node);
        id
    }

    /// Remove a node and all connected cables.
    pub fn remove_node(&mut self, node_id: usize) -> bool {
        let before_nodes = self.nodes.len();
        self.nodes.retain(|n| n.id != node_id);
        self.cables
            .retain(|c| c.from_node != node_id && c.to_node != node_id);
        self.nodes.len() < before_nodes
    }

    /// Start dragging a cable from a node's output port.
    pub fn start_drag(
        &mut self,
        from_node: usize,
        from_port: usize,
        initial_pos: (f32, f32),
    ) -> Result<(), String> {
        let node = self
            .nodes
            .iter()
            .find(|n| n.id == from_node)
            .ok_or_else(|| format!("Source node {} not found", from_node))?;

        let port = node
            .outputs
            .iter()
            .find(|p| p.id == from_port)
            .ok_or_else(|| format!("Output port {} not found on node {}", from_port, from_node))?;

        self.active_drag = Some(ActiveCableDrag {
            from_node,
            from_port,
            cable_type: port.cable_type,
            current_pointer_pos: initial_pos,
        });

        Ok(())
    }

    /// Update pointer position during cable dragging.
    pub fn update_drag(&mut self, pointer_pos: (f32, f32)) {
        if let Some(drag) = &mut self.active_drag {
            drag.current_pointer_pos = pointer_pos;
        }
    }

    /// Connect dragged cable to a target input port.
    pub fn finish_drag(&mut self, to_node: usize, to_port: usize) -> Result<usize, String> {
        let drag = self
            .active_drag
            .take()
            .ok_or_else(|| "No active cable drag in progress".to_string())?;

        self.connect_cable(drag.from_node, drag.from_port, to_node, to_port)
    }

    /// Connect two ports with a cable, verifying direction, type compatibility, and acyclicity.
    pub fn connect_cable(
        &mut self,
        from_node: usize,
        from_port: usize,
        to_node: usize,
        to_port: usize,
    ) -> Result<usize, String> {
        if from_node == to_node {
            return Err("Self-feedback loops are not permitted on direct ports".to_string());
        }

        let src_node = self
            .nodes
            .iter()
            .find(|n| n.id == from_node)
            .ok_or_else(|| format!("Source node {} not found", from_node))?;
        let src_port = src_node
            .outputs
            .iter()
            .find(|p| p.id == from_port)
            .ok_or_else(|| format!("Output port {} not found on node {}", from_port, from_node))?;

        let dst_node = self
            .nodes
            .iter()
            .find(|n| n.id == to_node)
            .ok_or_else(|| format!("Destination node {} not found", to_node))?;
        let dst_port = dst_node
            .inputs
            .iter()
            .find(|p| p.id == to_port)
            .ok_or_else(|| format!("Input port {} not found on node {}", to_port, to_node))?;

        // Type compatibility check
        if src_port.cable_type != dst_port.cable_type {
            return Err(format!(
                "Type mismatch: cannot connect {:?} output to {:?} input",
                src_port.cable_type, dst_port.cable_type
            ));
        }

        // Check if connection already exists
        if self.cables.iter().any(|c| {
            c.from_node == from_node
                && c.from_port == from_port
                && c.to_node == to_node
                && c.to_port == to_port
        }) {
            return Err("Cable connection already exists".to_string());
        }

        // Cycle check using DFS
        if self.creates_cycle(from_node, to_node) {
            return Err("Connection would create an invalid cyclic dependency".to_string());
        }

        let cable_id = self.next_id;
        self.next_id += 1;

        let cable = CanvasCable {
            id: cable_id,
            from_node,
            from_port,
            to_node,
            to_port,
            cable_type: src_port.cable_type,
            color_rgba: src_port.cable_type.default_color(),
            sag_factor: 0.35,
            muted: false,
        };

        self.cables.push(cable);
        Ok(cable_id)
    }

    /// Disconnect a cable by ID.
    pub fn disconnect_cable(&mut self, cable_id: usize) -> bool {
        let before = self.cables.len();
        self.cables.retain(|c| c.id != cable_id);
        self.cables.len() < before
    }

    /// Check if adding an edge from `from` to `to` would create a cycle.
    fn creates_cycle(&self, from: usize, to: usize) -> bool {
        let mut visited = HashSet::new();
        let mut stack = vec![to];

        while let Some(curr) = stack.pop() {
            if curr == from {
                return true;
            }
            if visited.insert(curr) {
                for c in &self.cables {
                    if c.from_node == curr {
                        stack.push(c.to_node);
                    }
                }
            }
        }
        false
    }

    /// Package a selection of nodes into an encapsulated Subpatch abstraction.
    pub fn create_subpatch(&mut self, name: &str, node_ids: &[usize]) -> Result<usize, String> {
        if node_ids.is_empty() {
            return Err("Cannot create an empty subpatch".to_string());
        }

        let set: HashSet<usize> = node_ids.iter().copied().collect();
        let mut inner_nodes = Vec::new();
        let mut inner_cables = Vec::new();

        for n in &self.nodes {
            if set.contains(&n.id) {
                inner_nodes.push(n.clone());
            }
        }

        // Partition cables into internal vs boundary
        let mut remaining_cables = Vec::new();
        for c in &self.cables {
            let from_inside = set.contains(&c.from_node);
            let to_inside = set.contains(&c.to_node);

            if from_inside && to_inside {
                inner_cables.push(c.clone());
            } else if !from_inside && !to_inside {
                remaining_cables.push(c.clone());
            }
        }

        let subpatch_id = self.next_id;
        self.next_id += 1;

        // Inlets and outlets from boundary connections
        let mut inlets = Vec::new();
        let mut outlets = Vec::new();
        for (i, node) in inner_nodes.iter().enumerate() {
            for inp in &node.inputs {
                inlets.push(CanvasPort {
                    id: i * 100 + inp.id,
                    name: format!("{}_{}", node.name, inp.name),
                    cable_type: inp.cable_type,
                    is_input: true,
                    pos_offset: (0.0, 20.0 + inlets.len() as f32 * 20.0),
                });
            }
            for out in &node.outputs {
                outlets.push(CanvasPort {
                    id: i * 100 + out.id,
                    name: format!("{}_{}", node.name, out.name),
                    cable_type: out.cable_type,
                    is_input: false,
                    pos_offset: (180.0, 20.0 + outlets.len() as f32 * 20.0),
                });
            }
        }

        let subpatch = Subpatch {
            id: subpatch_id,
            name: name.to_string(),
            inner_nodes,
            inner_cables,
            inlets: inlets.clone(),
            outlets: outlets.clone(),
        };
        self.subpatches.push(subpatch);

        // Remove encapsulated nodes from top-level canvas
        self.nodes.retain(|n| !set.contains(&n.id));
        self.cables = remaining_cables;

        // Insert subpatch representative node onto canvas
        let mut subpatch_node = CanvasNode::new(subpatch_id, name, "Subpatch", (100.0, 100.0));
        subpatch_node.is_subpatch = true;
        subpatch_node.subpatch_id = Some(subpatch_id);
        subpatch_node.inputs = inlets;
        subpatch_node.outputs = outlets;
        self.nodes.push(subpatch_node);

        Ok(subpatch_id)
    }

    /// Toggle Presentation Mode (hides cables and non-macro elements for clean live performance).
    pub fn toggle_presentation_mode(&mut self) -> bool {
        self.presentation_mode = !self.presentation_mode;
        self.presentation_mode
    }

    /// Toggle Rear Panel view (Reason-style Tab flip revealing back jacks and physics patch cables).
    pub fn toggle_rear_panel(&mut self) -> bool {
        self.rear_panel_flipped = !self.rear_panel_flipped;
        self.rear_panel_flipped
    }

    /// Calculate physics hanging catenary curve waypoints for realistic patch cable droop.
    pub fn compute_cable_catenary(
        p0: (f32, f32),
        p1: (f32, f32),
        sag: f32,
        num_points: usize,
    ) -> Vec<(f32, f32)> {
        let mut points = Vec::with_capacity(num_points);
        let dx = p1.0 - p0.0;
        let dy = p1.1 - p0.1;
        let dist = (dx * dx + dy * dy).sqrt();
        let max_sag = dist * sag;

        for i in 0..=num_points {
            let t = i as f32 / num_points as f32;
            let straight_x = p0.0 + t * dx;
            let straight_y = p0.1 + t * dy;
            // Parabolic catenary approximation: sag * 4 * t * (1 - t)
            let droop = max_sag * 4.0 * t * (1.0 - t);
            points.push((straight_x, straight_y + droop));
        }

        points
    }

    /// Render ASCII wireframe representation of modular patching canvas.
    pub fn render_ascii(&self) -> String {
        let mut out = String::new();
        out.push_str("+-----------------------------------------------------------+\n");
        let mode_str = if self.presentation_mode {
            "[PRESENTATION MODE]"
        } else if self.rear_panel_flipped {
            "[RACK REAR PANEL - CV/GATE JACS]"
        } else {
            "[PATCHING CANVAS - FRONT PANEL]"
        };
        out.push_str(&format!("| MODULAR PATCH CANVAS {:<34} |\n", mode_str));
        out.push_str("+-----------------------------------------------------------+\n");

        for node in &self.nodes {
            out.push_str(&format!(
                "| Node #{:<2} [{:<16}] ({:<10}) @ ({:.0}, {:.0})\n",
                node.id, node.name, node.category, node.pos.0, node.pos.1
            ));
            if !self.presentation_mode {
                out.push_str(&format!(
                    "|   Inputs: {} | Outputs: {} | Cables: {}\n",
                    node.inputs.len(),
                    node.outputs.len(),
                    self.cables
                        .iter()
                        .filter(|c| c.from_node == node.id || c.to_node == node.id)
                        .count()
                ));
            } else {
                let exp_params: Vec<_> = node
                    .params
                    .iter()
                    .filter(|p| p.exposed_in_presentation)
                    .map(|p| format!("{}: {:.2}", p.name, p.value))
                    .collect();
                out.push_str(&format!("|   Controls: {}\n", exp_params.join(", ")));
            }
        }

        out.push_str("+-----------------------------------------------------------+\n");
        out.push_str(&format!(
            "| Total Nodes: {:<3} | Total Cables: {:<3} | Subpatches: {:<2} |\n",
            self.nodes.len(),
            self.cables.len(),
            self.subpatches.len()
        ));
        out.push_str("+-----------------------------------------------------------+\n");
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_patch_canvas_node_and_cable_management() {
        let mut canvas = PatchCanvas::new();

        let mut osc = CanvasNode::new(1, "Oscillator", "Generator", (100.0, 100.0));
        osc.add_output(1, "Audio Out", CableType::Audio);
        osc.add_output(2, "Pitch CV Out", CableType::Cv);
        osc.add_param("freq", "Frequency", 440.0, 20.0, 20000.0);
        let osc_id = canvas.add_node(osc);

        let mut filter = CanvasNode::new(2, "Ladder Filter", "Filter", (350.0, 100.0));
        filter.add_input(1, "Audio In", CableType::Audio);
        filter.add_input(2, "Cutoff CV", CableType::Cv);
        filter.add_output(3, "Audio Out", CableType::Audio);
        let filter_id = canvas.add_node(filter);

        assert_eq!(canvas.nodes.len(), 2);

        // Connect audio output to audio input
        let _cable_audio = canvas.connect_cable(osc_id, 1, filter_id, 1).unwrap();
        assert_eq!(canvas.cables.len(), 1);
        assert_eq!(canvas.cables[0].cable_type, CableType::Audio);

        // Connect CV output to CV input
        let cable_cv = canvas.connect_cable(osc_id, 2, filter_id, 2).unwrap();
        assert_eq!(canvas.cables.len(), 2);
        assert_eq!(canvas.cables[1].cable_type, CableType::Cv);

        // Type mismatch check: connecting Audio to CV should fail
        assert!(canvas.connect_cable(osc_id, 1, filter_id, 2).is_err());

        // Disconnect cable
        assert!(canvas.disconnect_cable(cable_cv));
        assert_eq!(canvas.cables.len(), 1);

        // Remove node cleans up connected cables
        assert!(canvas.remove_node(osc_id));
        assert_eq!(canvas.nodes.len(), 1);
        assert_eq!(canvas.cables.len(), 0);
    }

    #[test]
    fn test_cable_drag_interaction_and_cycle_prevention() {
        let mut canvas = PatchCanvas::new();

        let mut n1 = CanvasNode::new(1, "Node A", "Processor", (100.0, 100.0));
        n1.add_output(1, "Out", CableType::Audio);
        n1.add_input(1, "In", CableType::Audio);
        let id1 = canvas.add_node(n1);

        let mut n2 = CanvasNode::new(2, "Node B", "Processor", (300.0, 100.0));
        n2.add_input(1, "In", CableType::Audio);
        n2.add_output(1, "Out", CableType::Audio);
        let id2 = canvas.add_node(n2);

        // Drag from Node A to Node B
        canvas.start_drag(id1, 1, (100.0, 100.0)).unwrap();
        assert!(canvas.active_drag.is_some());
        canvas.update_drag((250.0, 110.0));
        assert_eq!(canvas.active_drag.as_ref().unwrap().current_pointer_pos, (250.0, 110.0));

        let cable_id = canvas.finish_drag(id2, 1).unwrap();
        assert_eq!(cable_id, canvas.cables[0].id);
        assert!(canvas.active_drag.is_none());

        // Attempting to connect Node B back to Node A would create a cycle -> must fail
        let cycle_res = canvas.connect_cable(id2, 1, id1, 1);
        assert!(cycle_res.is_err());
        assert!(cycle_res.unwrap_err().contains("cyclic"));
    }

    #[test]
    fn test_subpatches_presentation_mode_and_rear_panel() {
        let mut canvas = PatchCanvas::new();

        let mut osc = CanvasNode::new(1, "VCO", "Osc", (50.0, 50.0));
        osc.add_output(1, "Out", CableType::Audio);
        osc.add_param("tune", "Tuning", 0.0, -12.0, 12.0);
        osc.expose_param("tune", (50.0, 50.0));
        let o_id = canvas.add_node(osc);

        let mut vcf = CanvasNode::new(2, "VCF", "Filter", (200.0, 50.0));
        vcf.add_input(1, "In", CableType::Audio);
        vcf.add_output(2, "Out", CableType::Audio);
        let f_id = canvas.add_node(vcf);

        canvas.connect_cable(o_id, 1, f_id, 1).unwrap();

        // Package into subpatch
        let _sub_id = canvas.create_subpatch("SynthVoice", &[o_id, f_id]).unwrap();
        assert_eq!(canvas.subpatches.len(), 1);
        assert_eq!(canvas.nodes.len(), 1);
        assert_eq!(canvas.nodes[0].name, "SynthVoice");
        assert!(canvas.nodes[0].is_subpatch);

        // Presentation Mode toggle
        assert!(canvas.toggle_presentation_mode());
        assert!(canvas.presentation_mode);
        assert!(!canvas.toggle_presentation_mode());

        // Rear Panel flip (Reason-style Tab key toggle)
        assert!(canvas.toggle_rear_panel());
        assert!(canvas.rear_panel_flipped);

        // Catenary cable physics
        let curve = PatchCanvas::compute_cable_catenary((0.0, 0.0), (100.0, 0.0), 0.3, 10);
        assert_eq!(curve.len(), 11);
        // Midpoint should sag downwards
        assert!(curve[5].1 > 20.0);
    }
}
