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
            evaluation_order: Vec::new(),
            buffers: Vec::new(),
            input_buffers: Vec::new(),
            max_block_size,
            max_channels,
        }
    }

    pub fn add_node(&mut self, node: Box<dyn AudioNode>) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(node);
        self.buffers.push(vec![vec![0.0; self.max_block_size]; self.max_channels]);
        self.input_buffers.push(vec![vec![0.0; self.max_block_size]; self.max_channels]);
        idx
    }

    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
        self.compile();
    }

    /// Run Kahn's algorithm to generate topological evaluation order.
    pub fn compile(&mut self) {
        let num_nodes = self.nodes.len();
        if num_nodes == 0 {
            self.evaluation_order.clear();
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
            order = (0..num_nodes).collect();
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

        // Process nodes in topological evaluation order
        for &node_idx in &self.evaluation_order {
            // Copy edge data from upstream nodes into node_idx's input_buffers
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
}
