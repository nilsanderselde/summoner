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

//! Distributed Cloud Rendering Farm for Stem Generation and High-Order Ambisonic Encodes (Task L2).
//!
//! Provides parallel distributed rendering coordination across cluster worker nodes:
//! - Multi-track stem batch partitioning
//! - Higher-Order Ambisonic (HOA 1st, 2nd, 3rd order) spatial rendering distribution
//! - Worker node registration, heartbeat monitoring, and automated failover/rescheduling
//! - Final package bundle assembly and delivery manifest generation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Target render format delivery profiles.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RenderTargetFormat {
    StemsArchiveZip,
    StereoMasterWav,
    StereoMasterFlac,
    AmbisonicFirstOrderBFormat,  // 4 channels (W, X, Y, Z)
    AmbisonicSecondOrderBFormat, // 9 channels
    AmbisonicThirdOrderBFormat,  // 16 channels
    AdmBwfDolbyAtmos,            // 128 bed/object channels
}

/// Operational status of a worker rendering node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkerNodeStatus {
    Idle,
    Busy,
    Draining,
    Offline,
}

/// Worker node specification in the distributed cluster.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderWorkerNode {
    pub worker_id: String,
    pub hostname: String,
    pub cpu_cores: usize,
    pub max_parallel_stems: usize,
    pub status: WorkerNodeStatus,
    pub last_heartbeat_timestamp: u64,
    pub active_job_ids: Vec<String>,
}

impl RenderWorkerNode {
    pub fn new(worker_id: &str, hostname: &str, cpu_cores: usize) -> Self {
        Self {
            worker_id: worker_id.to_string(),
            hostname: hostname.to_string(),
            cpu_cores,
            max_parallel_stems: cpu_cores.max(1),
            status: WorkerNodeStatus::Idle,
            last_heartbeat_timestamp: 0,
            active_job_ids: Vec::new(),
        }
    }

    pub fn available_slots(&self) -> usize {
        if self.status == WorkerNodeStatus::Idle {
            self.max_parallel_stems.saturating_sub(self.active_job_ids.len())
        } else {
            0
        }
    }
}

/// Individual stem render task allocated to a worker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StemRenderTask {
    pub task_id: String,
    pub track_id: u64,
    pub track_name: String,
    pub channel_count: usize,
    pub is_bus: bool,
    pub duration_seconds: f64,
    pub assigned_worker_id: Option<String>,
    pub is_completed: bool,
    pub output_file_name: Option<String>,
    pub output_byte_size: Option<usize>,
}

/// Status of an entire distributed render job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderJobStatus {
    Queued,
    Partitioned,
    Rendering,
    Aggregating,
    Completed,
    Failed,
}

/// A complete multi-stem or immersive ambisonic render job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedRenderJob {
    pub job_id: String,
    pub project_name: String,
    pub target_format: RenderTargetFormat,
    pub sample_rate: u32,
    pub bit_depth: u16,
    pub normalize_lufs: Option<f32>,
    pub status: RenderJobStatus,
    pub stem_tasks: Vec<StemRenderTask>,
    pub created_at_timestamp: u64,
    pub completed_at_timestamp: Option<u64>,
    pub final_delivery_manifest: Option<String>,
}

impl DistributedRenderJob {
    pub fn new(
        job_id: &str,
        project_name: &str,
        target_format: RenderTargetFormat,
        sample_rate: u32,
    ) -> Self {
        Self {
            job_id: job_id.to_string(),
            project_name: project_name.to_string(),
            target_format,
            sample_rate,
            bit_depth: 24,
            normalize_lufs: Some(-14.0),
            status: RenderJobStatus::Queued,
            stem_tasks: Vec::new(),
            created_at_timestamp: 1700000000,
            completed_at_timestamp: None,
            final_delivery_manifest: None,
        }
    }

    pub fn add_stem_task(
        &mut self,
        track_id: u64,
        track_name: &str,
        channels: usize,
        duration_seconds: f64,
    ) {
        let task_id = format!("{}_stem_{}", self.job_id, track_id);
        self.stem_tasks.push(StemRenderTask {
            task_id,
            track_id,
            track_name: track_name.to_string(),
            channel_count: channels,
            is_bus: false,
            duration_seconds,
            assigned_worker_id: None,
            is_completed: false,
            output_file_name: None,
            output_byte_size: None,
        });
    }

    pub fn completion_percentage(&self) -> f32 {
        if self.stem_tasks.is_empty() {
            return 100.0;
        }
        let completed = self.stem_tasks.iter().filter(|t| t.is_completed).count();
        (completed as f32 / self.stem_tasks.len() as f32) * 100.0
    }
}

/// Central Coordinator managing distributed cloud render workers and jobs.
#[derive(Debug, Clone)]
pub struct CloudRenderFarmCoordinator {
    workers: HashMap<String, RenderWorkerNode>,
    jobs: HashMap<String, DistributedRenderJob>,
    heartbeat_timeout_seconds: u64,
}

impl Default for CloudRenderFarmCoordinator {
    fn default() -> Self {
        Self {
            workers: HashMap::new(),
            jobs: HashMap::new(),
            heartbeat_timeout_seconds: 30,
        }
    }
}

impl CloudRenderFarmCoordinator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_worker(&mut self, worker: RenderWorkerNode) {
        self.workers.insert(worker.worker_id.clone(), worker);
    }

    pub fn unregister_worker(&mut self, worker_id: &str) {
        self.workers.remove(worker_id);
    }

    pub fn record_heartbeat(&mut self, worker_id: &str, timestamp: u64) -> bool {
        if let Some(worker) = self.workers.get_mut(worker_id) {
            worker.last_heartbeat_timestamp = timestamp;
            if worker.status == WorkerNodeStatus::Offline {
                worker.status = WorkerNodeStatus::Idle;
            }
            true
        } else {
            false
        }
    }

    pub fn submit_job(&mut self, job: DistributedRenderJob) {
        self.jobs.insert(job.job_id.clone(), job);
    }

    pub fn get_job(&self, job_id: &str) -> Option<&DistributedRenderJob> {
        self.jobs.get(job_id)
    }

    /// Partitions pending stem tasks across available idle worker nodes.
    pub fn schedule_pending_tasks(&mut self) -> usize {
        let mut scheduled_count = 0;

        // Collect available worker slot capacities
        let mut available_workers: Vec<(String, usize)> = self
            .workers
            .values()
            .filter(|w| w.status == WorkerNodeStatus::Idle && w.available_slots() > 0)
            .map(|w| (w.worker_id.clone(), w.available_slots()))
            .collect();

        if available_workers.is_empty() {
            return 0;
        }

        for job in self.jobs.values_mut() {
            if job.status != RenderJobStatus::Queued && job.status != RenderJobStatus::Rendering {
                continue;
            }

            for task in job.stem_tasks.iter_mut().filter(|t| !t.is_completed && t.assigned_worker_id.is_none()) {
                // Find worker with highest remaining capacity
                available_workers.sort_by_key(|b| std::cmp::Reverse(b.1));
                if let Some((worker_id, ref mut slots)) = available_workers.first_mut() {
                    if *slots > 0 {
                        task.assigned_worker_id = Some(worker_id.clone());
                        *slots -= 1;
                        scheduled_count += 1;
                    }
                }
            }

            if job.stem_tasks.iter().any(|t| t.assigned_worker_id.is_some() && !t.is_completed) {
                job.status = RenderJobStatus::Rendering;
            }
        }

        scheduled_count
    }

    /// Marks a specific stem task as completed by a worker node.
    pub fn report_task_completion(
        &mut self,
        job_id: &str,
        task_id: &str,
        output_file: &str,
        byte_size: usize,
    ) -> Result<(), String> {
        let job = self.jobs.get_mut(job_id).ok_or_else(|| format!("Job {} not found", job_id))?;

        let task = job
            .stem_tasks
            .iter_mut()
            .find(|t| t.task_id == task_id)
            .ok_or_else(|| format!("Task {} not found", task_id))?;

        task.is_completed = true;
        task.output_file_name = Some(output_file.to_string());
        task.output_byte_size = Some(byte_size);

        // Check if all stems for this job are complete
        if job.stem_tasks.iter().all(|t| t.is_completed) {
            job.status = RenderJobStatus::Completed;
            job.completed_at_timestamp = Some(1700000100);

            // Generate delivery manifest
            let mut manifest = format!("DELIVERY MANIFEST: {}\nFORMAT: {:?}\nSTEMS:\n", job.project_name, job.target_format);
            for t in &job.stem_tasks {
                manifest.push_str(&format!(
                    " - {} [{} ch, {} bytes] -> {}\n",
                    t.track_name,
                    t.channel_count,
                    t.output_byte_size.unwrap_or(0),
                    t.output_file_name.as_deref().unwrap_or("none")
                ));
            }
            job.final_delivery_manifest = Some(manifest);
        }

        Ok(())
    }

    /// Healthcheck sweep: identifies unresponsive workers and re-queues their tasks.
    pub fn prune_dead_workers(&mut self, current_timestamp: u64) -> usize {
        let timeout = self.heartbeat_timeout_seconds;
        let mut dead_worker_ids = Vec::new();

        for worker in self.workers.values_mut() {
            if worker.status != WorkerNodeStatus::Offline
                && current_timestamp.saturating_sub(worker.last_heartbeat_timestamp) > timeout
            {
                worker.status = WorkerNodeStatus::Offline;
                dead_worker_ids.push(worker.worker_id.clone());
            }
        }

        let mut rescheduled_count = 0;
        for dead_id in &dead_worker_ids {
            for job in self.jobs.values_mut() {
                for task in job.stem_tasks.iter_mut() {
                    if task.assigned_worker_id.as_deref() == Some(dead_id) && !task.is_completed {
                        task.assigned_worker_id = None; // Reset assignment for re-scheduling
                        rescheduled_count += 1;
                    }
                }
            }
        }

        rescheduled_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloud_render_job_initialization_and_tasks() {
        let mut job = DistributedRenderJob::new(
            "job_001",
            "Symphonic Ambience",
            RenderTargetFormat::AmbisonicThirdOrderBFormat,
            48000,
        );

        job.add_stem_task(1, "Strings Bed", 4, 180.0);
        job.add_stem_task(2, "Brass Fanfare", 4, 180.0);
        job.add_stem_task(3, "Spatial Reverb Return", 16, 180.0); // 16-channel HOA 3rd order

        assert_eq!(job.stem_tasks.len(), 3);
        assert_eq!(job.completion_percentage(), 0.0);
        assert_eq!(job.status, RenderJobStatus::Queued);
    }

    #[test]
    fn test_worker_scheduling_and_load_distribution() {
        let mut coordinator = CloudRenderFarmCoordinator::new();

        let w1 = RenderWorkerNode::new("worker_1", "render-node-01.local", 4);
        let w2 = RenderWorkerNode::new("worker_2", "render-node-02.local", 4);
        coordinator.register_worker(w1);
        coordinator.register_worker(w2);

        let mut job = DistributedRenderJob::new("job_002", "Electronic Stems", RenderTargetFormat::StemsArchiveZip, 44100);
        for i in 1..=6 {
            job.add_stem_task(i, &format!("Track {}", i), 2, 120.0);
        }
        coordinator.submit_job(job);

        let scheduled = coordinator.schedule_pending_tasks();
        assert_eq!(scheduled, 6);

        let job_ref = coordinator.get_job("job_002").unwrap();
        assert_eq!(job_ref.status, RenderJobStatus::Rendering);
        for task in &job_ref.stem_tasks {
            assert!(task.assigned_worker_id.is_some());
        }
    }

    #[test]
    fn test_task_completion_and_delivery_manifest() {
        let mut coordinator = CloudRenderFarmCoordinator::new();
        let w = RenderWorkerNode::new("w_single", "node.local", 8);
        coordinator.register_worker(w);

        let mut job = DistributedRenderJob::new("job_003", "Mini Mix", RenderTargetFormat::StereoMasterWav, 96000);
        job.add_stem_task(1, "Master Stereo", 2, 60.0);
        coordinator.submit_job(job);

        coordinator.schedule_pending_tasks();
        let task_id = "job_003_stem_1";

        coordinator
            .report_task_completion("job_003", task_id, "master_96k.wav", 23040000)
            .unwrap();

        let finished_job = coordinator.get_job("job_003").unwrap();
        assert_eq!(finished_job.status, RenderJobStatus::Completed);
        assert_eq!(finished_job.completion_percentage(), 100.0);

        let manifest = finished_job.final_delivery_manifest.as_ref().unwrap();
        assert!(manifest.contains("DELIVERY MANIFEST: Mini Mix"));
        assert!(manifest.contains("master_96k.wav"));
    }

    #[test]
    fn test_dead_worker_detection_and_failover() {
        let mut coordinator = CloudRenderFarmCoordinator::new();
        let mut w1 = RenderWorkerNode::new("w_flake", "flaky.node", 4);
        w1.last_heartbeat_timestamp = 100;
        coordinator.register_worker(w1);

        let mut job = DistributedRenderJob::new("job_failover", "Failover Test", RenderTargetFormat::StemsArchiveZip, 48000);
        job.add_stem_task(10, "Synths", 2, 60.0);
        coordinator.submit_job(job);

        coordinator.schedule_pending_tasks();
        assert!(coordinator.get_job("job_failover").unwrap().stem_tasks[0].assigned_worker_id.is_some());

        // Heartbeat timeout occurs at timestamp 200 (> 30s timeout)
        let rescheduled = coordinator.prune_dead_workers(200);
        assert_eq!(rescheduled, 1);

        // Task should now be unassigned ready for healthy worker
        let task = &coordinator.get_job("job_failover").unwrap().stem_tasks[0];
        assert!(task.assigned_worker_id.is_none());
        assert!(!task.is_completed);
    }
}

