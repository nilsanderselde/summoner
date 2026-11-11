//! Real-time safety (REL-1): every node kind the engine factory can build must render
//! blocks without touching the heap once warmed up.

use std::collections::HashMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use summoner_core::allocator::AllocGuard;
use summoner_core::node::ProcessContext;
use summoner_core::transport::Transport;
use summoner_engine::graph::GraphRunner;
use summoner_project::{create_default_project, NodeConfig, TrackConfig};

const BLOCK: usize = 256;

/// Node kinds that currently allocate while processing. Entries must be removed as the
/// underlying nodes are fixed; the test fails if an entry starts passing (stale) or if a
/// new kind starts failing.
const KNOWN_ALLOCATING: &[&str] = &[];

fn factory_kinds() -> Vec<String> {
    let src = include_str!("../src/graph.rs");
    let start = src
        .find("pub fn create_node_with_rate")
        .expect("factory fn");
    let mut kinds = Vec::new();
    for line in src[start..].lines() {
        let t = line.trim();
        if t.starts_with("pub fn ") && !t.contains("create_node_with_rate") {
            break;
        }
        if t.starts_with('"') && t.ends_with("=> {") || t.starts_with('"') && t.contains(" =>") {
            let head = t.split("=>").next().unwrap_or("");
            for part in head.split('|') {
                let name = part.trim().trim_matches('"');
                if !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                    kinds.push(name.to_string());
                }
            }
        }
    }
    kinds.sort();
    kinds.dedup();
    kinds
}

fn run_guarded(kind: &str) -> Result<(), String> {
    let mut project = create_default_project("rt");
    let track = TrackConfig {
        id: 99,
        name: "t".into(),
        channels: 2,
        gain: 1.0,
        nodes: vec![NodeConfig {
            kind: kind.to_string(),
            params: HashMap::new(),
            plugin_state: None,
        }],
        ..Default::default()
    };
    project.tracks = vec![track];

    let mut transport = Transport::new(44100, 120.0);
    transport.play();
    let ctx = ProcessContext::from_transport(&transport);
    let mut runner = GraphRunner::new(&project);
    let mut l = vec![0.0f32; BLOCK];
    let mut r = vec![0.0f32; BLOCK];

    for _ in 0..4 {
        let mut outs: [&mut [f32]; 2] = [&mut l, &mut r];
        runner.process_block(BLOCK, &ctx, &mut outs);
    }
    let res = catch_unwind(AssertUnwindSafe(|| {
        let _guard = AllocGuard::new();
        for _ in 0..8 {
            let mut outs: [&mut [f32]; 2] = [&mut l, &mut r];
            runner.process_block(BLOCK, &ctx, &mut outs);
        }
    }));
    res.map_err(|_| format!("{kind} allocates in process_block"))
}

#[test]
fn default_project_is_allocation_free() {
    let project = create_default_project("rt");
    let mut transport = Transport::new(44100, 120.0);
    transport.play();
    let ctx = ProcessContext::from_transport(&transport);
    let mut runner = GraphRunner::new(&project);
    let mut l = vec![0.0f32; BLOCK];
    let mut r = vec![0.0f32; BLOCK];
    for _ in 0..4 {
        let mut outs: [&mut [f32]; 2] = [&mut l, &mut r];
        runner.process_block(BLOCK, &ctx, &mut outs);
    }
    let _guard = AllocGuard::new();
    for _ in 0..16 {
        let mut outs: [&mut [f32]; 2] = [&mut l, &mut r];
        runner.process_block(BLOCK, &ctx, &mut outs);
    }
}

#[test]
fn every_factory_node_kind_is_allocation_free() {
    let kinds = factory_kinds();
    assert!(kinds.len() > 40, "kind discovery broke: {}", kinds.len());
    let mut failing = Vec::new();
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    for k in &kinds {
        if run_guarded(k).is_err() {
            failing.push(k.clone());
        }
    }
    std::panic::set_hook(prev_hook);
    let unexpected: Vec<_> = failing
        .iter()
        .filter(|k| !KNOWN_ALLOCATING.contains(&k.as_str()))
        .collect();
    let stale: Vec<_> = KNOWN_ALLOCATING
        .iter()
        .filter(|k| !failing.iter().any(|f| f == *k))
        .collect();
    assert!(
        unexpected.is_empty(),
        "allocating node kinds: {unexpected:?}"
    );
    assert!(
        stale.is_empty(),
        "stale KNOWN_ALLOCATING entries: {stale:?}"
    );
}

#[test]
fn guard_is_armed_in_this_binary() {
    // Negative control: proves the allocator hook is linked into this test binary, so the
    // passes above are not vacuous.
    let r = catch_unwind(|| {
        let _g = AllocGuard::new();
        let v: Vec<u8> = Vec::with_capacity(1024);
        std::hint::black_box(&v);
    });
    assert!(r.is_err());
}
