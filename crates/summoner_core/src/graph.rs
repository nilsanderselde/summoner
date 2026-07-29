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

//! DAG signal graph evaluation engine with Kahn's topological sorting and pre-allocated buffers.

use crate::audio::Sample;
use crate::node::{AudioNode, ProcessContext};
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Edge {
    pub from_node: usize,
    pub from_port: usize,
    pub to_node: usize,
    pub to_port: usize,
}

pub struct NodeGraph {
    pub name: String,
    pub nodes: Vec<Box<dyn AudioNode>>,
    pub edges: Vec<Edge>,
    pub has_cycle: bool,
    pub levels: Vec<Vec<usize>>,
    pub parallel_execution: bool,
    pub node_timings: Vec<std::time::Duration>,
    evaluation_order: Vec<usize>,
    // Pre-allocated buffers for each node's output: buffers[node_index][channel][sample_index]
    buffers: Vec<Vec<Vec<Sample>>>,
    // Pre-allocated buffers for each node's input: input_buffers[node_index][channel][sample_index]
    input_buffers: Vec<Vec<Vec<Sample>>>,
    max_block_size: usize,
    max_channels: usize,
}

impl std::fmt::Debug for NodeGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeGraph")
            .field("name", &self.name)
            .field("node_count", &self.nodes.len())
            .field("edges", &self.edges)
            .field("has_cycle", &self.has_cycle)
            .field("levels", &self.levels)
            .field("parallel_execution", &self.parallel_execution)
            .field("evaluation_order", &self.evaluation_order)
            .finish()
    }
}

impl NodeGraph {
    pub fn new(name: impl Into<String>, max_block_size: usize, max_channels: usize) -> Self {
        Self {
            name: name.into(),
            nodes: Vec::new(),
            edges: Vec::new(),
            has_cycle: false,
            levels: Vec::new(),
            parallel_execution: true,
            node_timings: Vec::new(),
            evaluation_order: Vec::new(),
            buffers: Vec::new(),
            input_buffers: Vec::new(),
            max_block_size,
            max_channels,
        }
    }

    /// Adds a new audio node to the graph and allocates internal buffers for it.
    ///
    /// # Examples
    ///
    /// ```
    /// use summoner_core::graph::NodeGraph;
    /// use summoner_core::node::GainNode;
    ///
    /// let mut graph = NodeGraph::new("TestGraph", 512, 2);
    /// let node_idx = graph.add_node(Box::new(GainNode::new(0.8)));
    /// assert_eq!(node_idx, 0);
    /// ```
    pub fn add_node(&mut self, node: Box<dyn AudioNode>) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(node);
        self.buffers.push(vec![vec![0.0; self.max_block_size]; self.max_channels]);
        self.input_buffers.push(vec![vec![0.0; self.max_block_size]; self.max_channels]);
        self.node_timings.push(std::time::Duration::ZERO);
        idx
    }

    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
        self.compile();
    }

    /// Run Kahn's algorithm to generate topological evaluation order and compute independent levels.
    pub fn compile(&mut self) {
        let num_nodes = self.nodes.len();
        if num_nodes == 0 {
            self.evaluation_order.clear();
            self.levels.clear();
            return;
        }

        let mut in_degree = vec![0usize; num_nodes];
        let mut adj = vec![vec![]; num_nodes];

        for edge in &self.edges {
            if edge.from_node < num_nodes && edge.to_node < num_nodes {
                adj[edge.from_node].push(edge.to_node);
                in_degree[edge.to_node] += 1;
            }
        }

        let mut queue = VecDeque::new();
        for (idx, &degree) in in_degree.iter().enumerate() {
            if degree == 0 {
                queue.push_back(idx);
            }
        }

        let mut order = Vec::with_capacity(num_nodes);
        while let Some(u) = queue.pop_front() {
            order.push(u);
            for &v in &adj[u] {
                in_degree[v] -= 1;
                if in_degree[v] == 0 {
                    queue.push_back(v);
                }
            }
        }

        // If cycle detected or unvisited nodes exist, fallback to linear index order
        if order.len() < num_nodes {
            self.has_cycle = true;
            order = (0..num_nodes).collect();
            self.levels = vec![order.clone()];
        } else {
            self.has_cycle = false;
            let mut level_map = vec![0usize; num_nodes];
            for &u in &order {
                let current_lvl = level_map[u];
                for edge in &self.edges {
                    if edge.from_node == u && edge.to_node < num_nodes {
                        level_map[edge.to_node] = level_map[edge.to_node].max(current_lvl + 1);
                    }
                }
            }
            let max_lvl = level_map.iter().copied().max().unwrap_or(0);
            let mut levels = vec![Vec::new(); max_lvl + 1];
            for (idx, &lvl) in level_map.iter().enumerate() {
                levels[lvl].push(idx);
            }
            self.levels = levels;
        }

        self.evaluation_order = order;
    }
}

impl AudioNode for NodeGraph {
    fn name(&self) -> &str {
        &self.name
    }

    fn process(
        &mut self,
        _input: &[&[Sample]],
        output: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        if self.nodes.is_empty() || output.is_empty() {
            return;
        }

        let block_len = output[0].len().min(self.max_block_size);

        if self.parallel_execution && !self.levels.is_empty() {
            for level_nodes in &self.levels {
                // Copy edge data for nodes in this level
                for &node_idx in level_nodes {
                    for edge in &self.edges {
                        if edge.to_node == node_idx && edge.from_node < self.nodes.len() {
                            let from_ch = edge.from_port.min(self.max_channels - 1);
                            let to_ch = edge.to_port.min(self.max_channels - 1);
                            let src = &self.buffers[edge.from_node][from_ch][..block_len];
                            let dst = &mut self.input_buffers[edge.to_node][to_ch][..block_len];
                            dst.copy_from_slice(src);
                        }
                    }
                }

                if level_nodes.len() > 1 {
                    use rayon::prelude::*;
                    let nodes_ptr = self.nodes.as_mut_ptr() as usize;
                    let input_bufs_ptr = self.input_buffers.as_ptr() as usize;
                    let bufs_ptr = self.buffers.as_mut_ptr() as usize;
                    let timings_ptr = self.node_timings.as_mut_ptr() as usize;

                    level_nodes.par_iter().for_each(|&node_idx| {
                        let start_time = std::time::Instant::now();
                        unsafe {
                            let node: &mut Box<dyn AudioNode> = &mut *(nodes_ptr as *mut Box<dyn AudioNode>).add(node_idx);
                            let input_buf: &Vec<Vec<Sample>> = &*(input_bufs_ptr as *const Vec<Vec<Sample>>).add(node_idx);
                            let node_buf: &mut Vec<Vec<Sample>> = &mut *(bufs_ptr as *mut Vec<Vec<Sample>>).add(node_idx);
                            let timings: &mut std::time::Duration = &mut *(timings_ptr as *mut std::time::Duration).add(node_idx);

                            let in_slices: Vec<&[Sample]> = input_buf
                                .iter()
                                .map(|ch_buf| &ch_buf[..block_len])
                                .collect();

                            let mut out_slices: Vec<&mut [Sample]> = node_buf
                                .iter_mut()
                                .map(|ch_buf| &mut ch_buf[..block_len])
                                .collect();

                            node.process(&in_slices[..], &mut out_slices[..], ctx);
                            *timings = start_time.elapsed();
                        }
                    });
                } else {
                    let node_idx = level_nodes[0];
                    let start_time = std::time::Instant::now();
                    let in_slices: Vec<&[Sample]> = self.input_buffers[node_idx]
                        .iter()
                        .map(|ch_buf| &ch_buf[..block_len])
                        .collect();

                    let node_buf = &mut self.buffers[node_idx];
                    let mut out_slices: Vec<&mut [Sample]> = node_buf
                        .iter_mut()
                        .map(|ch_buf| &mut ch_buf[..block_len])
                        .collect();

                    self.nodes[node_idx].process(&in_slices[..], &mut out_slices[..], ctx);
                    self.node_timings[node_idx] = start_time.elapsed();
                }
            }
        } else {
            // Process nodes in topological evaluation order serially
            for &node_idx in &self.evaluation_order {
                let start_time = std::time::Instant::now();
                for edge in &self.edges {
                    if edge.to_node == node_idx && edge.from_node < self.nodes.len() {
                        let from_ch = edge.from_port.min(self.max_channels - 1);
                        let to_ch = edge.to_port.min(self.max_channels - 1);
                        let src = &self.buffers[edge.from_node][from_ch][..block_len];
                        let dst = &mut self.input_buffers[edge.to_node][to_ch][..block_len];
                        dst.copy_from_slice(src);
                    }
                }

                let in_slices: Vec<&[Sample]> = self.input_buffers[node_idx]
                    .iter()
                    .map(|ch_buf| &ch_buf[..block_len])
                    .collect();

                let node_buf = &mut self.buffers[node_idx];
                let mut out_slices: Vec<&mut [Sample]> = node_buf
                    .iter_mut()
                    .map(|ch_buf| &mut ch_buf[..block_len])
                    .collect();

                self.nodes[node_idx].process(&in_slices[..], &mut out_slices[..], ctx);
                self.node_timings[node_idx] = start_time.elapsed();
            }
        }

        // Copy final node's output buffer to graph output
        if let Some(&last_node) = self.evaluation_order.last() {
            let last_buf = &self.buffers[last_node];
            let channels = output.len().min(last_buf.len());
            for ch in 0..channels {
                let len = output[ch].len().min(block_len);
                output[ch][..len].copy_from_slice(&last_buf[ch][..len]);
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::{GainNode, SineOscillatorNode};
    use crate::transport::Transport;

    #[test]
    fn test_node_graph_execution() {
        let mut graph = NodeGraph::new("TestGraph", 512, 2);
        let osc_idx = graph.add_node(Box::new(SineOscillatorNode::new(440.0)));
        let gain_idx = graph.add_node(Box::new(GainNode::new(0.5)));

        graph.add_edge(Edge {
            from_node: osc_idx,
            from_port: 0,
            to_node: gain_idx,
            to_port: 0,
        });

        let transport = Transport::new(44100, 120.0);
        let ctx = ProcessContext::from_transport(&transport);

        let mut out_l = vec![0.0f32; 512];
        let mut out_r = vec![0.0f32; 512];

        graph.process(&[], &mut [&mut out_l[..], &mut out_r[..]], &ctx);

        assert!(out_l.iter().any(|&s| s != 0.0), "Graph output should contain non-zero audio");
    }

    #[test]
    fn test_parallel_serial_bit_identical() {
        let transport = Transport::new(44100, 120.0);
        let ctx = ProcessContext::from_transport(&transport);

        let mut create_test_graph = || {
            let mut graph = NodeGraph::new("BitIdenticalGraph", 512, 2);
            let osc1 = graph.add_node(Box::new(SineOscillatorNode::new(440.0)));
            let osc2 = graph.add_node(Box::new(SineOscillatorNode::new(880.0)));
            let gain1 = graph.add_node(Box::new(GainNode::new(0.5)));
            let gain2 = graph.add_node(Box::new(GainNode::new(0.3)));
            let mixer = graph.add_node(Box::new(GainNode::new(1.0)));

            graph.add_edge(Edge { from_node: osc1, from_port: 0, to_node: gain1, to_port: 0 });
            graph.add_edge(Edge { from_node: osc2, from_port: 0, to_node: gain2, to_port: 0 });
            graph.add_edge(Edge { from_node: gain1, from_port: 0, to_node: mixer, to_port: 0 });
            graph.add_edge(Edge { from_node: gain2, from_port: 0, to_node: mixer, to_port: 1 });

            graph
        };

        let mut graph_serial = create_test_graph();
        graph_serial.parallel_execution = false;
        let mut out_l_serial = vec![0.0f32; 512];
        let mut out_r_serial = vec![0.0f32; 512];
        graph_serial.process(&[], &mut [&mut out_l_serial[..], &mut out_r_serial[..]], &ctx);

        let mut graph_parallel = create_test_graph();
        graph_parallel.parallel_execution = true;
        let mut out_l_parallel = vec![0.0f32; 512];
        let mut out_r_parallel = vec![0.0f32; 512];
        graph_parallel.process(&[], &mut [&mut out_l_parallel[..], &mut out_r_parallel[..]], &ctx);

        assert_eq!(out_l_serial, out_l_parallel, "Parallel output must be bit-identical to serial output");
        assert_eq!(out_r_serial, out_r_parallel, "Parallel output must be bit-identical to serial output");
    }
}

