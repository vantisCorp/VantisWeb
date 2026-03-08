//! Memory Analyzer for heap snapshots and memory leak detection
//! 
//! Provides comprehensive memory profiling capabilities including:
//! - Heap snapshot capture and comparison
//! - Memory leak detection
//! - Object retention analysis
//! - Memory usage trends

use crate::developer_tools::models::{
    HeapNode, HeapNodeType, HeapEdge, HeapEdgeType,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, BTreeMap};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

/// Memory snapshot identifier
pub type SnapshotId = String;

/// Memory statistics for a category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Total bytes used
    pub total_bytes: u64,
    /// Number of objects
    pub object_count: usize,
    /// Number of references
    pub reference_count: usize,
    /// Average object size
    pub avg_object_size: f64,
}

impl Default for MemoryStats {
    fn default() -> Self {
        Self {
            total_bytes: 0,
            object_count: 0,
            reference_count: 0,
            avg_object_size: 0.0,
        }
    }
}

/// Memory category for classification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryCategory {
    /// JavaScript objects
    JavaScript,
    /// DOM nodes
    DOM,
    /// CSS rules
    CSS,
    /// Network resources
    Network,
    /// Code (compiled scripts)
    Code,
    /// Typed arrays
    TypedArrays,
    /// Arrays
    Arrays,
    /// Strings
    Strings,
    /// System objects
    System,
    /// Other/unknown
    Other,
}

/// Retaining path for an object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetainingPath {
    /// Path nodes from GC root to object
    pub path: Vec<HeapNode>,
    /// Distance from root
    pub distance: usize,
    /// Retaining node that keeps this alive
    pub retained_by: Option<String>,
}

/// Memory leak candidate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryLeak {
    /// Node that might be leaking
    pub node: HeapNode,
    /// Estimated leaked bytes
    pub leaked_bytes: u64,
    /// Retaining paths
    pub retaining_paths: Vec<RetainingPath>,
    /// Confidence score (0-100)
    pub confidence: u8,
    /// Reason for suspicion
    pub reason: String,
}

/// Heap snapshot comparison result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotDiff {
    /// Snapshot ID being compared
    pub snapshot_before: SnapshotId,
    pub snapshot_after: SnapshotId,
    /// Added nodes
    pub added_nodes: Vec<HeapNode>,
    /// Removed nodes
    pub removed_nodes: Vec<String>,
    /// Modified nodes (size changed)
    pub modified_nodes: Vec<(HeapNode, u64)>, // (node, old_size)
    /// Statistics change
    pub stats_delta: MemoryStats,
    /// Detected leaks
    pub detected_leaks: Vec<MemoryLeak>,
}

/// Memory usage over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryTrend {
    /// Timestamp of measurement
    pub timestamp: DateTime<Utc>,
    /// Total heap size
    pub heap_size: u64,
    /// Used heap size
    pub used_heap: u64,
    /// Total objects
    pub object_count: usize,
    /// DOM node count
    pub dom_node_count: usize,
    /// Event listener count
    pub event_listener_count: usize,
}

/// Heap snapshot data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeapSnapshot {
    /// Unique snapshot ID
    pub id: SnapshotId,
    /// Creation timestamp
    pub timestamp: DateTime<Utc>,
    /// All nodes in the snapshot
    pub nodes: HashMap<String, HeapNode>,
    /// All edges (references)
    pub edges: Vec<HeapEdge>,
    /// Statistics by category
    pub stats_by_category: HashMap<MemoryCategory, MemoryStats>,
    /// Overall statistics
    pub total_stats: MemoryStats,
    /// GC roots
    pub gc_roots: Vec<String>,
}

/// Memory analyzer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAnalyzerConfig {
    /// Enable automatic leak detection
    pub auto_detect_leaks: bool,
    /// Minimum object size to consider for leaks (bytes)
    pub min_leak_size: u64,
    /// Minimum object count to consider as leak
    pub min_leak_count: usize,
    /// Enable retention tracking
    pub track_retention: bool,
    /// Maximum retaining paths to compute
    pub max_retaining_paths: usize,
    /// Sampling interval for trends (ms)
    pub sampling_interval_ms: u64,
}

impl Default for MemoryAnalyzerConfig {
    fn default() -> Self {
        Self {
            auto_detect_leaks: true,
            min_leak_size: 1024,        // 1KB
            min_leak_count: 10,
            track_retention: true,
            max_retaining_paths: 5,
            sampling_interval_ms: 1000,
        }
    }
}

/// Memory Analyzer for heap inspection
pub struct MemoryAnalyzer {
    /// Configuration
    config: MemoryAnalyzerConfig,
    /// Captured snapshots
    snapshots: Arc<RwLock<HashMap<SnapshotId, HeapSnapshot>>>,
    /// Memory usage trends
    trends: Arc<RwLock<Vec<MemoryTrend>>>,
    /// Current snapshot being built
    current_snapshot: Arc<RwLock<Option<HeapSnapshot>>>,
}

impl MemoryAnalyzer {
    /// Create a new memory analyzer
    pub fn new(config: MemoryAnalyzerConfig) -> Self {
        Self {
            config,
            snapshots: Arc::new(RwLock::new(HashMap::new())),
            trends: Arc::new(RwLock::new(Vec::new())),
            current_snapshot: Arc::new(RwLock::new(None)),
        }
    }

    /// Take a heap snapshot
    pub async fn take_snapshot(&self, label: Option<&str>) -> Result<SnapshotId, String> {
        let id = label
            .map(|l| format!("snapshot-{}-{}", l, Utc::now().timestamp()))
            .unwrap_or_else(|| format!("snapshot-{}", Utc::now().timestamp()));

        // In a real implementation, this would integrate with the JS engine
        // For now, we create an empty snapshot structure
        let snapshot = HeapSnapshot {
            id: id.clone(),
            timestamp: Utc::now(),
            nodes: HashMap::new(),
            edges: Vec::new(),
            stats_by_category: HashMap::new(),
            total_stats: MemoryStats::default(),
            gc_roots: Vec::new(),
        };

        let mut snapshots = self.snapshots.write().await;
        snapshots.insert(id.clone(), snapshot);

        Ok(id)
    }

    /// Get a snapshot by ID
    pub async fn get_snapshot(&self, id: &str) -> Option<HeapSnapshot> {
        let snapshots = self.snapshots.read().await;
        snapshots.get(id).cloned()
    }

    /// List all snapshots
    pub async fn list_snapshots(&self) -> Vec<(SnapshotId, DateTime<Utc>)> {
        let snapshots = self.snapshots.read().await;
        snapshots
            .values()
            .map(|s| (s.id.clone(), s.timestamp))
            .collect()
    }

    /// Delete a snapshot
    pub async fn delete_snapshot(&self, id: &str) -> bool {
        let mut snapshots = self.snapshots.write().await;
        snapshots.remove(id).is_some()
    }

    /// Clear all snapshots
    pub async fn clear_snapshots(&self) {
        let mut snapshots = self.snapshots.write().await;
        snapshots.clear();
    }

    /// Compare two snapshots
    pub async fn compare_snapshots(
        &self,
        before_id: &str,
        after_id: &str,
    ) -> Result<SnapshotDiff, String> {
        let snapshots = self.snapshots.read().await;

        let before = snapshots
            .get(before_id)
            .ok_or_else(|| format!("Snapshot {} not found", before_id))?;
        let after = snapshots
            .get(after_id)
            .ok_or_else(|| format!("Snapshot {} not found", after_id))?;

        // Find added and modified nodes
        let mut added_nodes = Vec::new();
        let mut modified_nodes = Vec::new();

        for (id, node) in &after.nodes {
            if let Some(before_node) = before.nodes.get(id) {
                if node.self_size != before_node.self_size {
                    modified_nodes.push((node.clone(), before_node.self_size));
                }
            } else {
                added_nodes.push(node.clone());
            }
        }

        // Find removed nodes
        let removed_nodes: Vec<String> = before
            .nodes
            .keys()
            .filter(|id| !after.nodes.contains_key(*id))
            .cloned()
            .collect();

        // Calculate stats delta
        let stats_delta = MemoryStats {
            total_bytes: after.total_stats.total_bytes.saturating_sub(before.total_stats.total_bytes),
            object_count: after.total_stats.object_count.saturating_sub(before.total_stats.object_count),
            reference_count: after.total_stats.reference_count.saturating_sub(before.total_stats.reference_count),
            avg_object_size: after.total_stats.avg_object_size - before.total_stats.avg_object_size,
        };

        // Detect leaks
        let detected_leaks = if self.config.auto_detect_leaks {
            self.detect_leaks_in_diff(&added_nodes, &modified_nodes).await
        } else {
            Vec::new()
        };

        Ok(SnapshotDiff {
            snapshot_before: before_id.to_string(),
            snapshot_after: after_id.to_string(),
            added_nodes,
            removed_nodes,
            modified_nodes,
            stats_delta,
            detected_leaks,
        })
    }

    /// Detect memory leaks in added nodes
    async fn detect_leaks_in_diff(
        &self,
        added_nodes: &[HeapNode],
        modified_nodes: &[(HeapNode, u64)],
    ) -> Vec<MemoryLeak> {
        let mut leaks = Vec::new();

        // Group nodes by type/name
        let mut type_counts: HashMap<String, (usize, u64)> = HashMap::new();
        for node in added_nodes {
            let key = format!("{:?}", node.node_type);
            let entry = type_counts.entry(key).or_insert((0, 0));
            entry.0 += 1;
            entry.1 += node.self_size as u64;
        }

        // Check for potential leaks (many objects of same type)
        for (type_name, (count, total_size)) in type_counts {
            if count >= self.config.min_leak_count && total_size >= self.config.min_leak_size {
                // Find representative node
                if let Some(node) = added_nodes.iter().find(|n| format!("{:?}", n.node_type) == type_name) {
                    leaks.push(MemoryLeak {
                        node: node.clone(),
                        leaked_bytes: total_size,
                        retaining_paths: Vec::new(), // Would compute in real impl
                        confidence: std::cmp::min(100, count as u8 * 10),
                        reason: format!("{} objects of type {} detected", count, type_name),
                    });
                }
            }
        }

        leaks
    }

    /// Find objects by name pattern
    pub async fn find_objects(&self, pattern: &str) -> Vec<HeapNode> {
        let snapshots = self.snapshots.read().await;
        let mut results = Vec::new();

        for snapshot in snapshots.values() {
            for node in snapshot.nodes.values() {
                if node.name.contains(pattern) {
                    results.push(node.clone());
                }
            }
        }

        results
    }

    /// Get retaining path for an object
    pub async fn get_retaining_path(&self, _node_id: &str) -> Option<RetainingPath> {
        // In a real implementation, this would traverse the heap graph
        // from the target node back to GC roots
        None
    }

    /// Record memory trend
    pub async fn record_trend(&self, trend: MemoryTrend) {
        let mut trends = self.trends.write().await;
        trends.push(trend);
        
        // Keep last 1000 trends
        if trends.len() > 1000 {
            trends.remove(0);
        }
    }

    /// Get memory trends
    pub async fn get_trends(&self) -> Vec<MemoryTrend> {
        let trends = self.trends.read().await;
        trends.clone()
    }

    /// Analyze memory distribution
    pub async fn analyze_distribution(&self) -> HashMap<MemoryCategory, MemoryStats> {
        let snapshots = self.snapshots.read().await;
        
        // Get latest snapshot
        let latest = snapshots
            .values()
            .max_by_key(|s| s.timestamp);

        match latest {
            Some(snapshot) => snapshot.stats_by_category.clone(),
            None => HashMap::new(),
        }
    }

    /// Force garbage collection (if supported)
    pub async fn force_gc(&self) -> Result<MemoryStats, String> {
        // In a real implementation, this would trigger GC
        // and return before/after statistics
        Ok(MemoryStats::default())
    }

    /// Get summary statistics
    pub async fn get_summary(&self) -> MemorySummary {
        let snapshots = self.snapshots.read().await;
        let trends = self.trends.read().await;

        let total_snapshots = snapshots.len();
        let latest_snapshot = snapshots.values().max_by_key(|s| s.timestamp);
        
        let latest_heap_size = latest_snapshot
            .map(|s| s.total_stats.total_bytes)
            .unwrap_or(0);

        let peak_heap_size = trends
            .iter()
            .map(|t| t.heap_size)
            .max()
            .unwrap_or(0);

        let trend_count = trends.len();

        MemorySummary {
            total_snapshots,
            latest_heap_size,
            peak_heap_size,
            trend_data_points: trend_count,
        }
    }
}

/// Summary of memory state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySummary {
    /// Total number of snapshots
    pub total_snapshots: usize,
    /// Latest heap size
    pub latest_heap_size: u64,
    /// Peak heap size recorded
    pub peak_heap_size: u64,
    /// Number of trend data points
    pub trend_data_points: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_take_snapshot() {
        let analyzer = MemoryAnalyzer::new(MemoryAnalyzerConfig::default());
        let id = analyzer.take_snapshot(Some("test")).await.unwrap();
        assert!(id.starts_with("snapshot-test-"));
    }

    #[tokio::test]
    async fn test_list_snapshots() {
        let analyzer = MemoryAnalyzer::new(MemoryAnalyzerConfig::default());
        analyzer.take_snapshot(None).await.unwrap();
        analyzer.take_snapshot(None).await.unwrap();
        
        let list = analyzer.list_snapshots().await;
        assert_eq!(list.len(), 2);
    }

    #[tokio::test]
    async fn test_get_summary() {
        let analyzer = MemoryAnalyzer::new(MemoryAnalyzerConfig::default());
        let summary = analyzer.get_summary().await;
        assert_eq!(summary.total_snapshots, 0);
    }
}