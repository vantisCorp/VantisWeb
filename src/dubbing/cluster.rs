//! Vantis Cluster - Distributed computing for video processing
//! 
//! This module provides distributed computing capabilities for
//! video dubbing, enabling device-to-device computation, load
//! balancing, and task distribution.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use super::models::*;
use super::mod::{ClusterJob, JobStatus};

/// Vantis Cluster for distributed computing
pub struct VantisCluster {
    /// Cluster nodes
    nodes: HashMap<String, ClusterNode>,
    /// Job queue
    job_queue: VecDeque<ClusterJob>,
    /// Active jobs
    active_jobs: HashMap<String, ClusterJob>,
    /// Completed jobs
    completed_jobs: HashMap<String, CompletedJob>,
    /// Cluster settings
    settings: ClusterSettings,
    /// Statistics
    stats: ClusterStats,
    /// Job counter
    job_counter: AtomicU64,
}

impl VantisCluster {
    /// Create a new Vantis Cluster
    pub fn new() -> Self {
        let mut cluster = Self {
            nodes: HashMap::new(),
            job_queue: VecDeque::new(),
            active_jobs: HashMap::new(),
            completed_jobs: HashMap::new(),
            settings: ClusterSettings::default(),
            stats: ClusterStats::default(),
            job_counter: AtomicU64::new(1),
        };
        cluster.initialize_local_node();
        cluster
    }

    /// Initialize local node
    fn initialize_local_node(&mut self) {
        let local_node = ClusterNode {
            id: "local".to_string(),
            name: "Local Device".to_string(),
            node_type: NodeType::Local,
            capacity: 100, // Base capacity
            load: 0,
            status: NodeStatus::Online,
            latency_ms: 0,
            features: vec![
                "translation".to_string(),
                "voice_synthesis".to_string(),
                "lip_sync".to_string(),
            ],
        };
        
        self.nodes.insert(local_node.id.clone(), local_node);
    }

    /// Register a new node
    pub fn register_node(&mut self, node: ClusterNode) {
        self.stats.total_nodes += 1;
        self.nodes.insert(node.id.clone(), node);
    }

    /// Unregister a node
    pub fn unregister_node(&mut self, node_id: &str) {
        if let Some(node) = self.nodes.remove(node_id) {
            if node.node_type != NodeType::Local {
                self.stats.total_nodes = self.stats.total_nodes.saturating_sub(1);
            }
        }
    }

    /// Get all nodes
    pub fn get_nodes(&self) -> Vec<&ClusterNode> {
        self.nodes.values().collect()
    }

    /// Get node by ID
    pub fn get_node(&self, id: &str) -> Option<&ClusterNode> {
        self.nodes.get(id)
    }

    /// Submit a new job
    pub fn submit_job(&mut self, video_id: &str, settings: super::mod::DubbingSettings) -> ClusterJob {
        let job_id = format!("job_{}", self.job_counter.fetch_add(1, Ordering::SeqCst));
        
        let job = ClusterJob {
            id: job_id.clone(),
            video_id: video_id.to_string(),
            status: JobStatus::Queued,
            progress: 0,
            assigned_nodes: Vec::new(),
            eta_seconds: None,
        };
        
        self.job_queue.push_back(job.clone());
        self.stats.total_jobs += 1;
        
        // Try to assign immediately if nodes available
        self.assign_jobs();
        
        job
    }

    /// Assign queued jobs to available nodes
    fn assign_jobs(&mut self) {
        while !self.job_queue.is_empty() {
            // Find best available node
            let best_node = self.find_best_node();
            
            if let Some(node_id) = best_node {
                if let Some(job) = self.job_queue.pop_front() {
                    let mut assigned_job = job.clone();
                    assigned_job.status = JobStatus::Processing;
                    assigned_job.assigned_nodes.push(node_id.clone());
                    assigned_job.eta_seconds = self.estimate_job_duration(&assigned_job);
                    
                    // Update node load
                    if let Some(node) = self.nodes.get_mut(&node_id) {
                        node.load = (node.load + 10).min(100);
                    }
                    
                    self.active_jobs.insert(job.id.clone(), assigned_job);
                }
            } else {
                break;
            }
        }
    }

    /// Find best available node
    fn find_best_node(&self) -> Option<String> {
        self.nodes.values()
            .filter(|n| n.status == NodeStatus::Online && n.load < 80)
            .min_by_key(|n| n.load)
            .map(|n| n.id.clone())
    }

    /// Estimate job duration
    fn estimate_job_duration(&self, job: &ClusterJob) -> Option<u32> {
        // Base estimation: 2x video duration
        // Would be more sophisticated in production
        Some(60) // Placeholder: 60 seconds
    }

    /// Get job status
    pub fn get_job_status(&self, job_id: &str) -> Option<JobStatus> {
        // Check active jobs
        if let Some(job) = self.active_jobs.get(job_id) {
            return Some(job.status);
        }
        
        // Check completed jobs
        if let Some(job) = self.completed_jobs.get(job_id) {
            return Some(job.status);
        }
        
        // Check queue
        for job in &self.job_queue {
            if job.id == job_id {
                return Some(job.status);
            }
        }
        
        None
    }

    /// Update job progress
    pub fn update_progress(&mut self, job_id: &str, progress: u8) {
        if let Some(job) = self.active_jobs.get_mut(job_id) {
            job.progress = progress.min(100);
        }
    }

    /// Complete a job
    pub fn complete_job(&mut self, job_id: &str, result: JobResult) {
        if let Some(job) = self.active_jobs.remove(job_id) {
            // Update node loads
            for node_id in &job.assigned_nodes {
                if let Some(node) = self.nodes.get_mut(node_id) {
                    node.load = node.load.saturating_sub(10);
                }
            }
            
            let completed = CompletedJob {
                job: ClusterJob {
                    status: JobStatus::Completed,
                    progress: 100,
                    ..job
                },
                result,
                completed_at: Utc::now(),
            };
            
            self.completed_jobs.insert(job_id.to_string(), completed);
            self.stats.completed_jobs += 1;
            
            // Try to assign more jobs
            self.assign_jobs();
        }
    }

    /// Cancel a job
    pub fn cancel_job(&mut self, job_id: &str) -> bool {
        // Remove from queue
        if let Some(pos) = self.job_queue.iter().position(|j| j.id == job_id) {
            self.job_queue.remove(pos);
            return true;
        }
        
        // Cancel active job
        if let Some(mut job) = self.active_jobs.remove(job_id) {
            job.status = JobStatus::Cancelled;
            
            // Update node loads
            for node_id in &job.assigned_nodes {
                if let Some(node) = self.nodes.get_mut(node_id) {
                    node.load = node.load.saturating_sub(10);
                }
            }
            
            self.completed_jobs.insert(job_id.to_string(), CompletedJob {
                job,
                result: JobResult::Cancelled,
                completed_at: Utc::now(),
            });
            
            return true;
        }
        
        false
    }

    /// Get queue length
    pub fn queue_length(&self) -> usize {
        self.job_queue.len()
    }

    /// Get active job count
    pub fn active_count(&self) -> usize {
        self.active_jobs.len()
    }

    /// Get cluster load (average of all nodes)
    pub fn cluster_load(&self) -> u8 {
        if self.nodes.is_empty() {
            return 0;
        }
        
        let total: u32 = self.nodes.values().map(|n| n.load as u32).sum();
        (total / self.nodes.len() as u32) as u8
    }

    /// Get cluster capacity
    pub fn cluster_capacity(&self) -> u32 {
        self.nodes.values().map(|n| n.capacity).sum()
    }

    /// Distribute tasks across nodes
    pub fn distribute_tasks(&mut self, tasks: Vec<ProcessingTask>) -> HashMap<String, String> {
        let mut assignments = HashMap::new();
        
        for task in tasks {
            if let Some(node_id) = self.find_best_node_for_task(&task) {
                assignments.insert(task.id.clone(), node_id.clone());
                
                // Update node load
                if let Some(node) = self.nodes.get_mut(&node_id) {
                    node.load = (node.load + 5).min(100);
                }
            }
        }
        
        assignments
    }

    /// Find best node for a specific task
    fn find_best_node_for_task(&self, task: &ProcessingTask) -> Option<String> {
        self.nodes.values()
            .filter(|n| {
                n.status == NodeStatus::Online && 
                n.load < 90 &&
                self.node_supports_task(n, task)
            })
            .min_by(|a, b| {
                // Prefer nodes with lower latency and load
                let score_a = a.load as u32 * 10 + a.latency_ms;
                let score_b = b.load as u32 * 10 + b.latency_ms;
                score_a.cmp(&score_b)
            })
            .map(|n| n.id.clone())
    }

    /// Check if node supports a task
    fn node_supports_task(&self, node: &ClusterNode, task: &ProcessingTask) -> bool {
        let required_feature = match task.task_type {
            TaskType::Translation => "translation",
            TaskType::VoiceSynthesis => "voice_synthesis",
            TaskType::LipSync => "lip_sync",
            TaskType::AudioProcessing => "audio_processing",
            TaskType::VideoProcessing => "video_processing",
            TaskType::FullDubbing => "full_dubbing",
        };
        
        node.features.contains(&required_feature.to_string())
    }

    /// Get settings
    pub fn settings(&self) -> &ClusterSettings {
        &self.settings
    }

    /// Update settings
    pub fn update_settings(&mut self, settings: ClusterSettings) {
        self.settings = settings;
    }

    /// Get statistics
    pub fn stats(&self) -> &ClusterStats {
        &self.stats
    }

    /// Prune old completed jobs
    pub fn prune_completed_jobs(&mut self, max_age_hours: u8) {
        let cutoff = Utc::now() - chrono::Duration::hours(max_age_hours as i64);
        
        self.completed_jobs.retain(|_, job| {
            job.completed_at > cutoff
        });
    }
}

impl Default for VantisCluster {
    fn default() -> Self {
        Self::new()
    }
}

/// Completed job record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedJob {
    /// The job
    pub job: ClusterJob,
    /// Job result
    pub result: JobResult,
    /// Completion time
    pub completed_at: DateTime<Utc>,
}

/// Job result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobResult {
    /// Success with output reference
    Success(String),
    /// Failure with error message
    Failed(String),
    /// Cancelled
    Cancelled,
}

/// Cluster settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterSettings {
    /// Maximum queue size
    pub max_queue_size: usize,
    /// Maximum concurrent jobs per node
    pub max_jobs_per_node: u8,
    /// Job timeout in seconds
    pub job_timeout_seconds: u32,
    /// Enable auto-scaling
    pub auto_scaling: bool,
    /// Preferred node types
    pub preferred_node_types: Vec<NodeType>,
    /// Enable load balancing
    pub load_balancing: bool,
    /// Maximum latency tolerance (ms)
    pub max_latency_ms: u32,
}

impl Default for ClusterSettings {
    fn default() -> Self {
        Self {
            max_queue_size: 100,
            max_jobs_per_node: 3,
            job_timeout_seconds: 3600,
            auto_scaling: false,
            preferred_node_types: vec![NodeType::Local, NodeType::Cloud],
            load_balancing: true,
            max_latency_ms: 500,
        }
    }
}

/// Cluster statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClusterStats {
    /// Total registered nodes
    pub total_nodes: u32,
    /// Total jobs submitted
    pub total_jobs: u64,
    /// Completed jobs
    pub completed_jobs: u64,
    /// Failed jobs
    pub failed_jobs: u64,
    /// Average job duration
    pub avg_job_duration_seconds: f32,
    /// Total processing time
    pub total_processing_seconds: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_creation() {
        let cluster = VantisCluster::new();
        assert!(cluster.get_nodes().len() >= 1); // At least local node
    }

    #[test]
    fn test_job_submission() {
        let mut cluster = VantisCluster::new();
        let job = cluster.submit_job("video_123", super::mod::DubbingSettings::default());
        
        assert!(!job.id.is_empty());
        assert!(matches!(job.status, JobStatus::Queued | JobStatus::Processing));
    }

    #[test]
    fn test_node_registration() {
        let mut cluster = VantisCluster::new();
        let node = ClusterNode {
            id: "remote_1".to_string(),
            name: "Remote Server".to_string(),
            node_type: NodeType::Remote,
            capacity: 500,
            load: 0,
            status: NodeStatus::Online,
            latency_ms: 50,
            features: vec!["translation".to_string()],
        };
        
        cluster.register_node(node);
        assert!(cluster.get_node("remote_1").is_some());
    }

    #[test]
    fn test_cluster_load() {
        let cluster = VantisCluster::new();
        let load = cluster.cluster_load();
        assert!(load <= 100);
    }

    #[test]
    fn test_job_cancellation() {
        let mut cluster = VantisCluster::new();
        let job = cluster.submit_job("video_123", super::mod::DubbingSettings::default());
        
        let cancelled = cluster.cancel_job(&job.id);
        assert!(cancelled);
        assert!(matches!(cluster.get_job_status(&job.id), Some(JobStatus::Cancelled)));
    }

    #[test]
    fn test_task_distribution() {
        let mut cluster = VantisCluster::new();
        let tasks = vec![
            ProcessingTask {
                id: "task_1".to_string(),
                task_type: TaskType::Translation,
                priority: 5,
                input_ref: "input_1".to_string(),
                output_ref: None,
                status: TaskStatus::Pending,
                progress: 0,
                created: 0,
                completed: None,
                error: None,
            },
        ];
        
        let assignments = cluster.distribute_tasks(tasks);
        assert!(!assignments.is_empty());
    }
}