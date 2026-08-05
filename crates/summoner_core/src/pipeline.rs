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

//! Cloud-native multi-tenant headless render pipeline and batch job queue.

use crate::allocator::AllocGuard;
use crate::audio::{FixedAudioBuffer, Sample};
use crate::node::{AudioNode, GainNode, ProcessContext, SineOscillatorNode};
use crate::transport::Transport;
use blake3::Hasher;

/// Multi-tenant rendering job request descriptor.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderJob {
    pub job_id: String,
    pub tenant_id: String,
    pub num_frames: u64,
    pub sample_rate: u32,
    pub bpm: f64,
    pub synth_frequency: f32,
}

impl RenderJob {
    pub fn new(
        job_id: impl Into<String>,
        tenant_id: impl Into<String>,
        num_frames: u64,
        sample_rate: u32,
        bpm: f64,
        synth_frequency: f32,
    ) -> Self {
        Self {
            job_id: job_id.into(),
            tenant_id: tenant_id.into(),
            num_frames,
            sample_rate,
            bpm,
            synth_frequency,
        }
    }
}

/// Execution result metrics for a completed render job.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderResult {
    pub job_id: String,
    pub tenant_id: String,
    pub frames_processed: u64,
    pub sample_checksum: f64,
    pub hash_digest: String,
    pub success: bool,
}

/// Multi-tenant batch queue for deterministic headless audio rendering.
#[derive(Debug, Default, Clone)]
pub struct MultiTenantRenderQueue {
    jobs: Vec<RenderJob>,
}

impl MultiTenantRenderQueue {
    pub fn new() -> Self {
        Self { jobs: Vec::new() }
    }

    pub fn enqueue(&mut self, job: RenderJob) {
        self.jobs.push(job);
    }

    pub fn jobs(&self) -> &[RenderJob] {
        &self.jobs
    }

    /// Process all enqueued multi-tenant jobs sequentially with real-time zero-alloc safety.
    pub fn process_all(&self) -> Vec<RenderResult> {
        let mut results = Vec::with_capacity(self.jobs.len());

        for job in &self.jobs {
            let mut transport = Transport::new(job.sample_rate, job.bpm);
            transport.play();

            let mut sine_node = SineOscillatorNode::new(job.synth_frequency);
            let mut gain_node = GainNode::new(0.5);

            const CHANNELS: usize = 2;
            const BLOCK_SIZE: usize = 64;
            let mut mid_buffer = FixedAudioBuffer::<CHANNELS, BLOCK_SIZE>::new();
            let mut out_buffer = FixedAudioBuffer::<CHANNELS, BLOCK_SIZE>::new();

            let mut frames_processed: u64 = 0;
            let mut sample_sum: f64 = 0.0;
            let mut hasher = Hasher::new();

            {
                let _guard = AllocGuard::new();

                while frames_processed < job.num_frames {
                    let block_frames =
                        ((job.num_frames - frames_processed) as usize).min(BLOCK_SIZE);
                    mid_buffer.set_active_frames(block_frames);
                    out_buffer.set_active_frames(block_frames);

                    mid_buffer.clear();
                    out_buffer.clear();

                    let ctx = ProcessContext::from_transport(&transport);

                    let dummy_in: [&[Sample]; 0] = [];
                    let mut mid_slices = mid_buffer.channels_mut_2();
                    sine_node.process(&dummy_in, &mut mid_slices, &ctx);

                    let mid_ref = mid_buffer.channels_ref_2();
                    let mut out_slices = out_buffer.channels_mut_2();
                    gain_node.process(&mid_ref, &mut out_slices, &ctx);

                    for ch in 0..CHANNELS {
                        for s in out_buffer.channel(ch) {
                            sample_sum += s.abs() as f64;
                            hasher.update(&s.to_le_bytes());
                        }
                    }

                    transport.advance_frames(block_frames as u64);
                    frames_processed += block_frames as u64;
                }
            }

            let hash_digest = hasher.finalize().to_hex().to_string();
            results.push(RenderResult {
                job_id: job.job_id.clone(),
                tenant_id: job.tenant_id.clone(),
                frames_processed,
                sample_checksum: sample_sum,
                hash_digest,
                success: true,
            });
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_tenant_pipeline_determinism() {
        let mut queue1 = MultiTenantRenderQueue::new();
        queue1.enqueue(RenderJob::new(
            "job-101",
            "tenant-alpha",
            1024,
            44100,
            120.0,
            440.0,
        ));
        queue1.enqueue(RenderJob::new(
            "job-102",
            "tenant-beta",
            2048,
            48000,
            128.0,
            880.0,
        ));

        let res1 = queue1.process_all();
        assert_eq!(res1.len(), 2);
        assert!(res1[0].success);
        assert!(res1[1].success);

        // Verify deterministic bit-identical re-execution
        let res2 = queue1.process_all();
        assert_eq!(res1[0].hash_digest, res2[0].hash_digest);
        assert_eq!(res1[1].hash_digest, res2[1].hash_digest);
        assert_eq!(res1[0].sample_checksum, res2[0].sample_checksum);
    }
}
