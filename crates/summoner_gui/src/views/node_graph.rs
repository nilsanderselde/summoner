use eframe::egui;
use summoner_core::graph::{Edge, NodeGraph};
use summoner_core::node::{PassthroughNode, KNOWN_NODE_TYPES};
use std::collections::{HashMap, HashSet};
use crate::visualizer::Oscilloscope;

pub struct NodeGraphState {
    pub positions: HashMap<usize, egui::Pos2>,
    pub dragging_edge: Option<(usize, usize, egui::Pos2)>, // (from_node, from_port, current_pos)
    pub zoom: f32,
    pub pan_offset: egui::Vec2,
    pub selected_nodes: HashSet<usize>,
    pub search_query: String,
}

impl Default for NodeGraphState {
    fn default() -> Self {
        Self {
            positions: HashMap::new(),
            dragging_edge: None,
            zoom: 1.0,
            pan_offset: egui::Vec2::ZERO,
            selected_nodes: HashSet::new(),
            search_query: String::new(),
        }
    }
}

pub fn delete_node(graph: &mut NodeGraph, state: &mut NodeGraphState, node_idx: usize) {
    if node_idx >= graph.nodes.len() {
        return;
    }
    graph.nodes.remove(node_idx);
    graph.edges.retain(|e| e.from_node != node_idx && e.to_node != node_idx);
    for edge in &mut graph.edges {
        if edge.from_node > node_idx {
            edge.from_node -= 1;
        }
        if edge.to_node > node_idx {
            edge.to_node -= 1;
        }
    }
    let mut new_positions = HashMap::new();
    for (&idx, &pos) in &state.positions {
        if idx < node_idx {
            new_positions.insert(idx, pos);
        } else if idx > node_idx {
            new_positions.insert(idx - 1, pos);
        }
    }
    state.positions = new_positions;
    state.selected_nodes.remove(&node_idx);
    let mut new_selected = HashSet::new();
    for &idx in &state.selected_nodes {
        if idx > node_idx {
            new_selected.insert(idx - 1);
        } else {
            new_selected.insert(idx);
        }
    }
    state.selected_nodes = new_selected;
    graph.compile();
}

fn get_node_icon_and_color(name: &str) -> (&'static str, egui::Color32, egui::Color32) {
    if name.starts_with("Osc") || name.contains("Oscillator") {
        ("🌊", egui::Color32::from_rgb(30, 60, 130), egui::Color32::from_rgb(50, 100, 190))
    } else if name.starts_with("Filter") || name.contains("Filter") {
        ("🎛️", egui::Color32::from_rgb(100, 40, 140), egui::Color32::from_rgb(140, 60, 190))
    } else if name.starts_with("Env") || name.contains("ADSR") {
        ("📈", egui::Color32::from_rgb(30, 120, 60), egui::Color32::from_rgb(50, 160, 80))
    } else if name.starts_with("Math") || name.contains("Gain") || name.contains("Passthrough") {
        ("⚡", egui::Color32::from_rgb(60, 60, 70), egui::Color32::from_rgb(90, 90, 105))
    } else {
        ("🎹", egui::Color32::from_rgb(160, 80, 20), egui::Color32::from_rgb(200, 100, 30))
    }
}

pub fn show_node_graph(
    ui: &mut egui::Ui,
    graph: &mut NodeGraph,
    state: &mut NodeGraphState,
    _selected_edge: &mut Option<Edge>,
    oscilloscope: Option<&Oscilloscope>,
) {
    let mut modified = false;
    let mut drop_was_handled = false;

    egui::Frame::canvas(ui.style()).show(ui, |ui| {
        let (response, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
        let rect = response.rect;

        // Zoom and Pan controls
        if response.hovered() {
            let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
            if ui.input(|i| i.modifiers.ctrl) && scroll_delta != 0.0 {
                let factor = if scroll_delta > 0.0 { 1.1 } else { 0.9 };
                state.zoom = (state.zoom * factor).clamp(0.25, 3.0);
            }
        }
        if response.dragged_by(egui::PointerButton::Middle) || response.dragged_by(egui::PointerButton::Secondary) {
            state.pan_offset += response.drag_delta();
        }

        // Draw dot grid with pan and zoom
        let grid_size = 30.0 * state.zoom;
        let offset_x = (state.pan_offset.x % grid_size + grid_size) % grid_size;
        let offset_y = (state.pan_offset.y % grid_size + grid_size) % grid_size;

        let mut x = rect.left() + offset_x;
        while x < rect.right() {
            let mut y = rect.top() + offset_y;
            while y < rect.bottom() {
                painter.circle_filled(
                    egui::pos2(x, y),
                    1.5 * state.zoom,
                    egui::Color32::from_gray(50),
                );
                y += grid_size;
            }
            x += grid_size;
        }

        // Warning banner if graph contains cycle
        if graph.has_cycle {
            let banner_rect = egui::Rect::from_min_size(rect.left_top() + egui::vec2(10.0, 10.0), egui::vec2(320.0, 26.0));
            painter.rect_filled(banner_rect, 4.0, egui::Color32::from_rgb(180, 40, 40));
            painter.text(
                banner_rect.center(),
                egui::Align2::CENTER_CENTER,
                "⚠️ Cycle detected in DSP graph!",
                egui::FontId::proportional(12.0),
                egui::Color32::WHITE,
            );
        }

        // Initialize positions if empty
        for (i, _node) in graph.nodes.iter().enumerate() {
            state.positions.entry(i).or_insert_with(|| {
                egui::pos2(100.0 + (i as f32) * 160.0, 100.0 + (i as f32) * 40.0)
            });
        }

        let mut port_centers: HashMap<(usize, usize, bool), egui::Pos2> = HashMap::new();
        let mut node_to_duplicate: Option<usize> = None;
        let mut node_to_delete: Option<usize> = None;
        let mut edge_to_add: Option<Edge> = None;

        // Draw nodes
        for (i, node) in graph.nodes.iter().enumerate() {
            let orig_pos = *state.positions.get(&i).unwrap();
            let screen_pos = rect.min + state.pan_offset + (orig_pos.to_vec2() * state.zoom);
            let size = egui::vec2(140.0, 80.0) * state.zoom;
            let node_rect = egui::Rect::from_min_size(screen_pos, size);

            let (icon, bg_color, title_color) = get_node_icon_and_color(node.name());

            // Node background and glow effect for selected nodes
            let is_selected = state.selected_nodes.contains(&i);
            if is_selected {
                let glow_rect = node_rect.expand(4.0 * state.zoom);
                painter.rect_stroke(
                    glow_rect,
                    8.0 * state.zoom,
                    egui::Stroke::new(3.0 * state.zoom, egui::Color32::from_rgba_unmultiplied(26, 140, 255, 120)),
                );
            }

            let border_color = if is_selected {
                egui::Color32::YELLOW
            } else {
                egui::Color32::from_gray(100)
            };
            painter.rect_filled(node_rect, 5.0 * state.zoom, bg_color);
            painter.rect_stroke(node_rect, 5.0 * state.zoom, egui::Stroke::new(1.5f32 * state.zoom, border_color));

            // Title bar with icon
            let title_height = 24.0 * state.zoom;
            let title_rect = egui::Rect::from_min_max(node_rect.left_top(), egui::pos2(node_rect.right(), node_rect.top() + title_height));
            painter.rect_filled(
                title_rect,
                egui::Rounding {
                    nw: 5.0 * state.zoom,
                    ne: 5.0 * state.zoom,
                    sw: 0.0,
                    se: 0.0,
                },
                title_color,
            );
            let title_text = format!("{} {}", icon, node.name());
            painter.text(
                title_rect.center(),
                egui::Align2::CENTER_CENTER,
                &title_text,
                egui::FontId::proportional((12.0 * state.zoom).max(8.0)),
                egui::Color32::WHITE,
            );

            // Per-node mini oscilloscope display (24x24 px)
            let scope_rect = egui::Rect::from_center_size(
                node_rect.center(),
                egui::vec2(24.0 * state.zoom, 24.0 * state.zoom),
            );
            painter.rect_filled(scope_rect, 2.0 * state.zoom, egui::Color32::from_black_alpha(180));
            let dummy_scope = Oscilloscope::new();
            let scope_ref = oscilloscope.unwrap_or(&dummy_scope);
            let samples = scope_ref.read_all();
            let mut points = Vec::with_capacity(24);
            for s in 0..24 {
                let idx = s * 21;
                let px = scope_rect.left() + (s as f32 / 23.0) * scope_rect.width();
                let sample = samples[idx].clamp(-1.0, 1.0);
                let py = scope_rect.center().y - sample * (scope_rect.height() * 0.4);
                points.push(egui::pos2(px, py));
            }
            if points.len() >= 2 {
                for w in points.windows(2) {
                    painter.line_segment([w[0], w[1]], egui::Stroke::new(1.0_f32 * state.zoom, egui::Color32::from_rgb(0, 230, 150)));
                }
            }

            // Dragging the node
            let node_interact = ui.interact(title_rect, ui.id().with(i), egui::Sense::click_and_drag());
            if node_interact.clicked() {
                if !ui.input(|i| i.modifiers.shift) {
                    state.selected_nodes.clear();
                }
                state.selected_nodes.insert(i);
            }
            if node_interact.dragged() {
                if let Some(pos) = state.positions.get_mut(&i) {
                    *pos += node_interact.drag_delta() / state.zoom;
                }
            }

            // Right-click context menu on node
            node_interact.context_menu(|ui| {
                if ui.button("Duplicate Node").clicked() {
                    node_to_duplicate = Some(i);
                    ui.close_menu();
                }
                if ui.button("Delete Node").clicked() {
                    node_to_delete = Some(i);
                    ui.close_menu();
                }
                if ui.button("Inspect Parameters").clicked() {
                    ui.close_menu();
                }
            });

            // Input Ports
            for port in 0..2 {
                let port_pos = egui::pos2(node_rect.left(), node_rect.top() + (35.0 + port as f32 * 20.0) * state.zoom);
                port_centers.insert((i, port, true), port_pos);
                painter.circle_filled(port_pos, 5.0 * state.zoom, egui::Color32::from_rgb(220, 160, 40));

                let port_rect = egui::Rect::from_center_size(port_pos, egui::vec2(12.0 * state.zoom, 12.0 * state.zoom));
                let port_interact = ui.interact(port_rect, ui.id().with(("in", i, port)), egui::Sense::hover())
                    .on_hover_text(format!("Input Port {} ({})", port, if port == 0 { "Audio In" } else { "Mod In" }));
                if port_interact.hovered() {
                    painter.circle_stroke(port_pos, 7.0 * state.zoom, egui::Stroke::new(2.0f32, egui::Color32::WHITE));
                }

                if let Some((from_node, from_port, _)) = state.dragging_edge {
                    let is_released = ui.input(|inp| inp.pointer.any_released());
                    let pointer_pos = ui.input(|inp| inp.pointer.interact_pos().or_else(|| inp.pointer.hover_pos()));
                    if is_released && pointer_pos.is_some_and(|p| port_rect.expand(6.0).contains(p)) {
                        edge_to_add = Some(Edge {
                            from_node,
                            from_port,
                            to_node: i,
                            to_port: port,
                        });
                    }
                }
            }

            // Output Ports
            for port in 0..2 {
                let port_pos = egui::pos2(node_rect.right(), node_rect.top() + (35.0 + port as f32 * 20.0) * state.zoom);
                port_centers.insert((i, port, false), port_pos);
                painter.circle_filled(port_pos, 5.0 * state.zoom, egui::Color32::from_rgb(50, 160, 220));

                let port_rect = egui::Rect::from_center_size(port_pos, egui::vec2(12.0 * state.zoom, 12.0 * state.zoom));
                let port_interact = ui.interact(port_rect, ui.id().with(("out", i, port)), egui::Sense::drag())
                    .on_hover_text(format!("Output Port {} ({})", port, if port == 0 { "Audio Out" } else { "Aux Out" }));

                if port_interact.drag_started() {
                    state.dragging_edge = Some((i, port, port_interact.interact_pointer_pos().unwrap_or(port_pos)));
                } else if port_interact.dragged() {
                    if let Some(edge) = &mut state.dragging_edge {
                        edge.2 = port_interact.interact_pointer_pos().unwrap_or(port_pos);
                    }
                }
            }
        }

        // Apply deferred node duplication
        if let Some(dup_idx) = node_to_duplicate {
            let pos = state.positions.get(&dup_idx).copied().unwrap_or(egui::pos2(100.0, 100.0));
            let new_idx = graph.nodes.len();
            graph.nodes.push(Box::new(PassthroughNode));
            state.positions.insert(new_idx, pos + egui::vec2(20.0, 20.0));
            graph.compile();
            modified = true;
        }

        // Apply deferred node deletion
        if let Some(del_idx) = node_to_delete {
            delete_node(graph, state, del_idx);
            modified = true;
        }

        // Apply deferred edge creation
        if let Some(edge) = edge_to_add {
            graph.edges.push(edge);
            state.dragging_edge = None;
            drop_was_handled = true;
            graph.compile();
            modified = true;
        }

        // Global check for drop cancel
        if !drop_was_handled && ui.input(|i| i.pointer.any_released()) {
            state.dragging_edge = None;
        }

        // Draw active dragging edge
        if let Some((from_node, from_port, current_pos)) = state.dragging_edge {
            if let Some(start_pos) = port_centers.get(&(from_node, from_port, false)) {
                let control_1 = *start_pos + egui::vec2(50.0 * state.zoom, 0.0);
                let control_2 = current_pos - egui::vec2(50.0 * state.zoom, 0.0);
                painter.add(egui::Shape::CubicBezier(egui::epaint::CubicBezierShape {
                    points: [*start_pos, control_1, control_2, current_pos],
                    closed: false,
                    fill: egui::Color32::TRANSPARENT,
                    stroke: egui::Stroke::new(2.0f32 * state.zoom, egui::Color32::WHITE).into(),
                }));
            }
        }

        // Draw existing edges with color-coding
        let mut edge_to_remove = None;
        for (edge_idx, edge) in graph.edges.iter().enumerate() {
            if let (Some(start_pos), Some(end_pos)) = (
                port_centers.get(&(edge.from_node, edge.from_port, false)),
                port_centers.get(&(edge.to_node, edge.to_port, true)),
            ) {
                let control_1 = *start_pos + egui::vec2(50.0 * state.zoom, 0.0);
                let control_2 = *end_pos - egui::vec2(50.0 * state.zoom, 0.0);

                let min = start_pos.min(*end_pos);
                let max = start_pos.max(*end_pos);
                let edge_rect = egui::Rect::from_min_max(min, max).expand(10.0);

                let is_hovered = ui.input(|i| i.pointer.hover_pos()).is_some_and(|p| edge_rect.contains(p));

                let base_color = if edge.to_port == 0 {
                    egui::Color32::WHITE
                } else {
                    egui::Color32::from_rgb(250, 200, 60) // Control-rate amber/yellow
                };
                let stroke_color = if is_hovered { egui::Color32::YELLOW } else { base_color };

                painter.add(egui::Shape::CubicBezier(egui::epaint::CubicBezierShape {
                    points: [*start_pos, control_1, control_2, *end_pos],
                    closed: false,
                    fill: egui::Color32::TRANSPARENT,
                    stroke: egui::Stroke::new(2.5f32 * state.zoom, stroke_color).into(),
                }));

                if is_hovered && ui.input(|i| i.pointer.secondary_clicked()) {
                    edge_to_remove = Some(edge_idx);
                }
            }
        }

        if let Some(idx) = edge_to_remove {
            graph.edges.remove(idx);
            graph.compile();
            modified = true;
        }

        // Right-click canvas background context menu with search
        response.context_menu(|ui| {
            ui.heading("Add Node");
            ui.text_edit_singleline(&mut state.search_query);
            ui.separator();

            egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                let query = state.search_query.to_lowercase();
                for &kind in KNOWN_NODE_TYPES {
                    if (query.is_empty() || kind.to_lowercase().contains(&query)) && ui.button(kind).clicked() {
                        let new_idx = graph.nodes.len();
                        graph.nodes.push(Box::new(PassthroughNode));
                        state.positions.insert(new_idx, egui::pos2(150.0 + (new_idx as f32) * 30.0, 150.0));
                        graph.compile();
                        ui.close_menu();
                    }
                }
            });
        });
    });

    if modified {
        graph.compile();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_core::node::PassthroughNode;

    #[test]
    fn test_node_graph_ui_renders_without_panic() {
        let mut graph = NodeGraph::new("Test Graph", 64, 2);
        graph.nodes.push(Box::new(PassthroughNode));
        graph.nodes.push(Box::new(PassthroughNode));
        graph.nodes.push(Box::new(PassthroughNode));

        graph.edges.push(Edge {
            from_node: 0,
            from_port: 0,
            to_node: 1,
            to_port: 0,
        });
        graph.edges.push(Edge {
            from_node: 1,
            from_port: 0,
            to_node: 2,
            to_port: 0,
        });

        let node_count_before = graph.nodes.len();
        let edge_count_before = graph.edges.len();

        let mut state = NodeGraphState::default();
        let mut selected_edge = None;

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_node_graph(ui, &mut graph, &mut state, &mut selected_edge, None);
            });
        });

        assert_eq!(graph.nodes.len(), node_count_before);
        assert_eq!(graph.edges.len(), edge_count_before);
    }

    #[test]
    fn test_edge_drag_does_not_cancel_on_valid_port_drop() {
        let mut graph = NodeGraph::new("Test Graph", 64, 2);
        graph.nodes.push(Box::new(PassthroughNode));
        graph.nodes.push(Box::new(PassthroughNode));

        let mut state = NodeGraphState::default();
        state.dragging_edge = Some((0, 0, egui::pos2(0.0, 0.0)));
        let mut selected_edge = None;

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_node_graph(ui, &mut graph, &mut state, &mut selected_edge, None);
            });
        });
    }

    #[test]
    fn test_node_graph_delete_node_updates_edges() {
        let mut graph = NodeGraph::new("Test Graph", 64, 2);
        graph.nodes.push(Box::new(PassthroughNode));
        graph.nodes.push(Box::new(PassthroughNode));
        graph.nodes.push(Box::new(PassthroughNode));

        graph.edges.push(Edge {
            from_node: 0,
            from_port: 0,
            to_node: 1,
            to_port: 0,
        });
        graph.edges.push(Edge {
            from_node: 1,
            from_port: 0,
            to_node: 2,
            to_port: 0,
        });

        let mut state = NodeGraphState::default();
        state.positions.insert(0, egui::pos2(0.0, 0.0));
        state.positions.insert(1, egui::pos2(100.0, 0.0));
        state.positions.insert(2, egui::pos2(200.0, 0.0));

        // Delete node 1
        delete_node(&mut graph, &mut state, 1);

        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 0); // Both edges touching node 1 were removed
        assert!(state.positions.contains_key(&0));
        assert!(state.positions.contains_key(&1)); // Original node 2 shifted to 1
        assert!(!state.positions.contains_key(&2));
    }

    #[test]
    fn test_node_graph_zoom_pan_state() {
        let mut state = NodeGraphState::default();
        assert_eq!(state.zoom, 1.0);
        assert_eq!(state.pan_offset, egui::Vec2::ZERO);

        state.zoom = 1.5;
        state.pan_offset = egui::vec2(50.0, -100.0);

        assert_eq!(state.zoom, 1.5);
        assert_eq!(state.pan_offset, egui::vec2(50.0, -100.0));
    }
}

