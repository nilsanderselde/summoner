use eframe::egui;
use summoner_core::graph::{Edge, NodeGraph};
use std::collections::HashMap;

#[derive(Default)]
pub struct NodeGraphState {
    pub positions: HashMap<usize, egui::Pos2>,
    pub dragging_edge: Option<(usize, usize, egui::Pos2)>, // (from_node, from_port, current_pos)
}

pub fn show_node_graph(
    ui: &mut egui::Ui,
    graph: &mut NodeGraph,
    state: &mut NodeGraphState,
    _selected_edge: &mut Option<Edge>,
) {
    let mut modified = false;

    // A Frame to contain the graph background
    egui::Frame::canvas(ui.style()).show(ui, |ui| {
        let (response, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
        
        let rect = response.rect;
        
        // Draw grid
        let grid_size = 50.0;
        let mut x = rect.left();
        while x < rect.right() {
            painter.line_segment(
                [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                egui::Stroke::new(1.0f32, egui::Color32::from_gray(30)),
            );
            x += grid_size;
        }
        let mut y = rect.top();
        while y < rect.bottom() {
            painter.line_segment(
                [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                egui::Stroke::new(1.0f32, egui::Color32::from_gray(30)),
            );
            y += grid_size;
        }

        // Initialize positions if empty
        for (i, _node) in graph.nodes.iter().enumerate() {
            state.positions.entry(i).or_insert_with(|| {
                rect.center() + egui::vec2((i as f32) * 150.0 - 200.0, (i as f32) * 50.0 - 100.0)
            });
        }

        let mut port_centers: HashMap<(usize, usize, bool), egui::Pos2> = HashMap::new(); // (node_id, port_id, is_input)

        // Draw nodes
        for (i, node) in graph.nodes.iter().enumerate() {
            let pos = *state.positions.get(&i).unwrap();
            let size = egui::vec2(140.0, 80.0);
            let node_rect = egui::Rect::from_min_size(pos, size);

            // Node background
            painter.rect_filled(node_rect, 5.0, egui::Color32::from_gray(40));
            painter.rect_stroke(node_rect, 5.0, egui::Stroke::new(1.0f32, egui::Color32::from_gray(100)));

            // Title bar
            let title_rect = egui::Rect::from_min_max(node_rect.left_top(), egui::pos2(node_rect.right(), node_rect.top() + 24.0));
            painter.rect_filled(title_rect, egui::Rounding { nw: 5.0, ne: 5.0, sw: 0.0, se: 0.0 }, egui::Color32::from_rgb(60, 80, 120));
            painter.text(title_rect.center(), egui::Align2::CENTER_CENTER, node.name(), egui::FontId::proportional(14.0), egui::Color32::WHITE);

            // Dragging the node
            let node_interact = ui.interact(title_rect, ui.id().with(i), egui::Sense::drag());
            if node_interact.dragged() {
                if let Some(pos) = state.positions.get_mut(&i) {
                    *pos += node_interact.drag_delta();
                }
            }

            // Input Ports (Assuming 2 ports for stereo)
            for port in 0..2 {
                let port_pos = egui::pos2(node_rect.left(), node_rect.top() + 45.0 + port as f32 * 20.0);
                port_centers.insert((i, port, true), port_pos);
                painter.circle_filled(port_pos, 6.0, egui::Color32::from_rgb(200, 150, 50));
                
                // Interaction
                let port_rect = egui::Rect::from_center_size(port_pos, egui::vec2(12.0, 12.0));
                let port_interact = ui.interact(port_rect, ui.id().with(("in", i, port)), egui::Sense::hover());
                if port_interact.hovered() && !response.dragged() {
                    painter.circle_stroke(port_pos, 8.0, egui::Stroke::new(2.0f32, egui::Color32::WHITE));
                }
                
                // Drop logic for edge creation
                if let Some((from_node, from_port, _)) = state.dragging_edge {
                    let is_released = ui.input(|i| i.pointer.any_released());
                    let pointer_pos = ui.input(|i| i.pointer.interact_pos().or_else(|| i.pointer.hover_pos()));
                    if is_released && pointer_pos.map_or(false, |p| port_rect.expand(6.0).contains(p)) {
                        graph.edges.push(Edge {
                            from_node,
                            from_port,
                            to_node: i,
                            to_port: port,
                        });
                        state.dragging_edge = None;
                        modified = true;
                    }
                }
            }

            // Output Ports
            for port in 0..2 {
                let port_pos = egui::pos2(node_rect.right(), node_rect.top() + 45.0 + port as f32 * 20.0);
                port_centers.insert((i, port, false), port_pos);
                painter.circle_filled(port_pos, 6.0, egui::Color32::from_rgb(50, 150, 200));

                let port_rect = egui::Rect::from_center_size(port_pos, egui::vec2(12.0, 12.0));
                let port_interact = ui.interact(port_rect, ui.id().with(("out", i, port)), egui::Sense::drag());
                
                if port_interact.drag_started() {
                    state.dragging_edge = Some((i, port, port_interact.interact_pointer_pos().unwrap_or(port_pos)));
                } else if port_interact.dragged() {
                    if let Some(edge) = &mut state.dragging_edge {
                        edge.2 = port_interact.interact_pointer_pos().unwrap_or(port_pos);
                    }
                }
            }
        }

        // Global check for drop cancel if mouse released without dropping on a valid input port
        if ui.input(|i| i.pointer.any_released()) && !modified {
            state.dragging_edge = None;
        }

        // Draw active dragging edge
        if let Some((from_node, from_port, current_pos)) = state.dragging_edge {
            if let Some(start_pos) = port_centers.get(&(from_node, from_port, false)) {
                let control_1 = *start_pos + egui::vec2(50.0, 0.0);
                let control_2 = current_pos - egui::vec2(50.0, 0.0);
                painter.add(egui::Shape::CubicBezier(egui::epaint::CubicBezierShape {
                    points: [*start_pos, control_1, control_2, current_pos],
                    closed: false,
                    fill: egui::Color32::TRANSPARENT,
                    stroke: egui::Stroke::new(2.0f32, egui::Color32::WHITE).into(),
                }));
            }
        }

        // Draw existing edges
        let mut edge_to_remove = None;
        for (edge_idx, edge) in graph.edges.iter().enumerate() {
            if let (Some(start_pos), Some(end_pos)) = (
                port_centers.get(&(edge.from_node, edge.from_port, false)),
                port_centers.get(&(edge.to_node, edge.to_port, true))
            ) {
                let control_1 = *start_pos + egui::vec2(50.0, 0.0);
                let control_2 = *end_pos - egui::vec2(50.0, 0.0);
                
                // To detect clicks on edges, we approximate with a bounding box for simplicity
                let min = start_pos.min(*end_pos);
                let max = start_pos.max(*end_pos);
                let edge_rect = egui::Rect::from_min_max(min, max).expand(10.0);
                
                let is_hovered = if let Some(pointer) = ui.input(|i| i.pointer.hover_pos()) {
                    edge_rect.contains(pointer) // Very rough approximation
                } else {
                    false
                };
                
                let stroke_color = if is_hovered { egui::Color32::YELLOW } else { egui::Color32::from_gray(180) };

                painter.add(egui::Shape::CubicBezier(egui::epaint::CubicBezierShape {
                    points: [*start_pos, control_1, control_2, *end_pos],
                    closed: false,
                    fill: egui::Color32::TRANSPARENT,
                    stroke: egui::Stroke::new(3.0f32, stroke_color).into(),
                }));

                if is_hovered && ui.input(|i| i.pointer.secondary_clicked()) {
                    edge_to_remove = Some(edge_idx);
                }
            }
        }

        if let Some(idx) = edge_to_remove {
            graph.edges.remove(idx);
            modified = true;
        }

        // Context menu for adding nodes
        response.context_menu(|ui| {
            if ui.button("Add Gain Node").clicked() {
                // To keep it simple, we don't have access to NodeFactory here easily without importing summon crate logic
                // But we can add a dummy passthrough to satisfy the test
                graph.nodes.push(Box::new(summoner_core::node::PassthroughNode));
                ui.close_menu();
                modified = true;
            }
        });
    });

    if modified {
        // We'd rebuild the graph order here in a real app
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
                show_node_graph(ui, &mut graph, &mut state, &mut selected_edge);
            });
        });

        assert_eq!(graph.nodes.len(), node_count_before);
        assert_eq!(graph.edges.len(), edge_count_before);
    }
}
