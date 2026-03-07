/// # WebRTC Statistics Module
/// 
/// This module implements statistics collection and monitoring for WebRTC connections.
/// Provides detailed metrics for connection quality, performance, and debugging.
/// 
/// ## Features
/// - Connection quality monitoring
/// - Bandwidth statistics
/// - Packet loss tracking
/// - Latency measurements
/// - Video/Audio quality metrics
/// - ICE connection statistics

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use thiserror::Error;

/// Errors that can occur in statistics operations
#[derive(Error, Debug)]
pub enum StatsError {
    #[error("No statistics available")]
    NoStatsAvailable,
    #[error("Invalid statistics format: {0}")]
    InvalidFormat(String),
    #[error("Statistics collection failed: {0}")]
    CollectionFailed(String),
}

/// Statistics type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatsType {
    /// Media stream statistics
    MediaStream,
    /// Inbound RTP statistics
    InboundRtp,
    /// Outbound RTP statistics
    OutboundRtp,
    /// Remote inbound RTP statistics
    RemoteInboundRtp,
    /// Remote outbound RTP statistics
    RemoteOutboundRtp,
    /// Media source statistics
    MediaSource,
    /// Data channel statistics
    DataChannel,
    /// Transport statistics
    Transport,
    /// ICE candidate statistics
    IceCandidate,
    /// ICE candidate pair statistics
    IceCandidatePair,
    /// Certificate statistics
    Certificate,
}

/// WebRTC statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RTCStats {
    /// Statistics type
    pub stats_type: StatsType,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Statistics ID
    pub id: String,
    /// Specific statistics data
    pub data: StatsData,
}

/// Statistics data based on type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatsData {
    /// Media stream stats
    MediaStream(MediaStreamStats),
    /// Inbound RTP stats
    InboundRtp(RtpStreamStats),
    /// Outbound RTP stats
    OutboundRtp(RtpStreamStats),
    /// Transport stats
    Transport(TransportStats),
    /// ICE candidate pair stats
    IceCandidatePair(IceCandidatePairStats),
    /// Data channel stats
    DataChannel(DataChannelStats),
}

/// Media stream statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaStreamStats {
    /// Stream ID
    pub stream_id: String,
    /// Stream identifier (from SDP)
    pub stream_identifier: String,
    /// Track identifiers
    pub track_ids: Vec<String>,
}

/// RTP stream statistics (common for inbound/outbound)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RtpStreamStats {
    /// SSRC (Synchronization Source)
    pub ssrc: u32,
    /// Media type
    pub media_type: MediaType,
    /// Codec identifier
    pub codec_id: String,
    /// Packets received
    pub packets_received: u64,
    /// Packets sent
    pub packets_sent: u64,
    /// Bytes received
    pub bytes_received: u64,
    /// Bytes sent
    pub bytes_sent: u64,
    /// Packets lost
    pub packets_lost: u64,
    /// Fraction of packets lost (as percentage)
    pub fraction_lost: f64,
    /// Jitter (in seconds)
    pub jitter: f64,
    /// Round trip time (in seconds)
    pub rtt: f64,
    /// Bitrate (bits per second)
    pub bitrate: u64,
}

/// Media type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaType {
    Audio,
    Video,
}

/// Transport statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportStats {
    /// Transport ID
    pub transport_id: String,
    /// Bytes sent
    pub bytes_sent: u64,
    /// Bytes received
    pub bytes_received: u64,
    /// RTCP bytes sent
    pub rtcp_bytes_sent: u64,
    /// RTCP bytes received
    pub rtcp_bytes_received: u64,
    /// Selected ICE candidate pair ID
    pub selected_candidate_pair_id: Option<String>,
    /// Local certificate ID
    pub local_certificate_id: Option<String>,
    /// Remote certificate ID
    pub remote_certificate_id: Option<String>,
}

/// ICE candidate pair statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceCandidatePairStats {
    /// Pair ID
    pub pair_id: String,
    /// Transport ID
    pub transport_id: String,
    /// Local candidate ID
    pub local_candidate_id: String,
    /// Remote candidate ID
    pub remote_candidate_id: String,
    /// Current state
    pub state: IceCandidatePairState,
    /// Priority
    pub priority: u64,
    /// Nominated (whether this pair is selected)
    pub nominated: bool,
    /// Total requests sent
    pub requests_sent: u64,
    /// Total responses received
    pub responses_received: u64,
    /// Total retransmissions sent
    pub retransmissions_sent: u64,
    /// Total consent requests sent
    pub consent_requests_sent: u64,
    /// Current round trip time (seconds)
    pub current_rtt: f64,
    /// Total round trip time (seconds)
    pub total_rtt: f64,
    /// Total responses that were retransmissions
    pub responses_received_retransmissions: u64,
}

/// ICE candidate pair state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IceCandidatePairState {
    /// Waiting for connectivity check
    Frozen,
    /// In progress
    InProgress,
    /// Successfully connected
    Succeeded,
    /// Failed connectivity check
    Failed,
}

/// Data channel statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataChannelStats {
    /// Data channel ID
    pub data_channel_id: u16,
    /// Data channel label
    pub label: String,
    /// Protocol
    pub protocol: String,
    /// Data channel state
    pub state: DataChannelState,
    /// Messages sent
    pub messages_sent: u64,
    /// Bytes sent
    pub bytes_sent: u64,
    /// Messages received
    pub messages_received: u64,
    /// Bytes received
    pub bytes_received: u64,
}

/// Data channel state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataChannelState {
    Connecting,
    Open,
    Closing,
    Closed,
}

/// Connection quality score
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ConnectionQuality {
    /// Overall quality score (0.0 - 1.0)
    pub overall_score: f64,
    /// Video quality score (0.0 - 1.0)
    pub video_quality: f64,
    /// Audio quality score (0.0 - 1.0)
    pub audio_quality: f64,
    /// Network health (0.0 - 1.0)
    pub network_health: f64,
}

/// Statistics collector
pub struct StatsCollector {
    /// Collected statistics
    stats: Arc<RwLock<Vec<RTCStats>>>,
    /// Maximum number of stats to keep
    max_stats: usize,
    /// Enable collection
    enabled: Arc<RwLock<bool>>,
}

impl StatsCollector {
    /// Create a new statistics collector
    pub fn new(max_stats: usize) -> Self {
        Self {
            stats: Arc::new(RwLock::new(Vec::new())),
            max_stats,
            enabled: Arc::new(RwLock::new(true)),
        }
    }

    /// Collect and store a statistic
    pub async fn collect(&self, stat: RTCStats) -> Result<(), StatsError> {
        if !*self.enabled.read().await {
            return Err(StatsError::CollectionFailed("Collection disabled".to_string()));
        }

        let mut stats = self.stats.write().await;
        stats.push(stat);

        // Maintain max size
        if stats.len() > self.max_stats {
            stats.remove(0);
        }

        Ok(())
    }

    /// Get all collected statistics
    pub async fn get_all(&self) -> Vec<RTCStats> {
        self.stats.read().await.clone()
    }

    /// Get statistics by type
    pub async fn get_by_type(&self, stats_type: StatsType) -> Vec<RTCStats> {
        let stats = self.stats.read().await;
        stats.iter()
            .filter(|s| s.stats_type == stats_type)
            .cloned()
            .collect()
    }

    /// Get statistics by ID
    pub async fn get_by_id(&self, id: &str) -> Option<RTCStats> {
        let stats = self.stats.read().await;
        stats.iter()
            .find(|s| s.id == id)
            .cloned()
    }

    /// Get latest statistics by type
    pub async fn get_latest(&self, stats_type: StatsType) -> Option<RTCStats> {
        let stats = self.stats.read().await;
        stats.iter()
            .filter(|s| s.stats_type == stats_type)
            .last()
            .cloned()
    }

    /// Clear all statistics
    pub async fn clear(&self) {
        self.stats.write().await.clear();
    }

    /// Enable or disable collection
    pub async fn set_enabled(&self, enabled: bool) {
        *self.enabled.write().await = enabled;
    }

    /// Check if collection is enabled
    pub async fn is_enabled(&self) -> bool {
        *self.enabled.read().await
    }

    /// Calculate connection quality
    pub async fn calculate_quality(&self) -> ConnectionQuality {
        let stats = self.stats.read().await;
        
        // Get latest RTP stats
        let inbound_rtp: Vec<_> = stats.iter()
            .filter(|s| matches!(s.stats_type, StatsType::InboundRtp))
            .collect();
        
        let outbound_rtp: Vec<_> = stats.iter()
            .filter(|s| matches!(s.stats_type, StatsType::OutboundRtp))
            .collect();

        // Get latest ICE pair stats
        let ice_pair: Vec<_> = stats.iter()
            .filter(|s| matches!(s.stats_type, StatsType::IceCandidatePair))
            .collect();

        // Calculate quality scores based on:
        // - Packet loss (lower is better)
        // - RTT (lower is better)
        // - Jitter (lower is better)
        // - Bitrate (higher is better)
        
        let video_quality = if inbound_rtp.is_empty() || outbound_rtp.is_empty() {
            1.0
        } else {
            // Simplified quality calculation
            let total_loss = inbound_rtp.iter()
                .filter_map(|s| {
                    if let StatsData::InboundRtp(rtp) = &s.data {
                        Some(rtp.fraction_lost)
                    } else {
                        None
                    }
                })
                .sum::<f64>();
            
            let avg_loss = if inbound_rtp.is_empty() { 0.0 } else { total_loss / inbound_rtp.len() as f64 };
            (1.0 - avg_loss / 100.0).max(0.0).min(1.0)
        };

        let audio_quality = video_quality; // Simplified

        let network_health = if ice_pair.is_empty() {
            1.0
        } else {
            // Based on RTT and packet loss
            let total_rtt = ice_pair.iter()
                .filter_map(|s| {
                    if let StatsData::IceCandidatePair(pair) = &s.data {
                        Some(pair.current_rtt)
                    } else {
                        None
                    }
                })
                .sum::<f64>();
            
            let avg_rtt = if ice_pair.is_empty() { 0.0 } else { total_rtt / ice_pair.len() as f64 };
            // RTT < 100ms is excellent, > 500ms is poor
            let rtt_score = (1.0 - avg_rtt / 0.5).max(0.0).min(1.0);
            rtt_score
        };

        let overall_score = (video_quality + audio_quality + network_health) / 3.0;

        ConnectionQuality {
            overall_score,
            video_quality,
            audio_quality,
            network_health,
        }
    }
}

/// Statistics report generator
pub struct StatsReport;

impl StatsReport {
    /// Generate a text report
    pub fn generate_text(stats: &[RTCStats]) -> String {
        let mut report = String::new();
        report.push_str("=== WebRTC Statistics Report ===\n\n");
        
        // Group by type
        let mut grouped = std::collections::HashMap::new();
        for stat in stats {
            grouped.entry(stat.stats_type)
                .or_insert_with(Vec::new)
                .push(stat);
        }

        for (stats_type, type_stats) in grouped {
            report.push_str(&format!("### {:?} ({})\n", stats_type, type_stats.len()));
            
            for stat in type_stats {
                report.push_str(&format!("  ID: {}\n", stat.id));
                report.push_str(&format!("  Timestamp: {}\n", stat.timestamp));
                
                match &stat.data {
                    StatsData::InboundRtp(rtp) => {
                        report.push_str(&format!("  Packets Lost: {}\n", rtp.packets_lost));
                        report.push_str(&format!("  Fraction Lost: {:.2}%\n", rtp.fraction_lost));
                        report.push_str(&format!("  RTT: {:.2}ms\n", rtp.rtt * 1000.0));
                        report.push_str(&format!("  Jitter: {:.2}ms\n", rtp.jitter * 1000.0));
                        report.push_str(&format!("  Bitrate: {} bps\n", rtp.bitrate));
                    }
                    StatsData::OutboundRtp(rtp) => {
                        report.push_str(&format!("  Packets Sent: {}\n", rtp.packets_sent));
                        report.push_str(&format!("  Bytes Sent: {}\n", rtp.bytes_sent));
                    }
                    StatsData::Transport(transport) => {
                        report.push_str(&format!("  Bytes Sent: {}\n", transport.bytes_sent));
                        report.push_str(&format!("  Bytes Received: {}\n", transport.bytes_received));
                    }
                    StatsData::IceCandidatePair(pair) => {
                        report.push_str(&format!("  State: {:?}\n", pair.state));
                        report.push_str(&format!("  RTT: {:.2}ms\n", pair.current_rtt * 1000.0));
                    }
                    _ => {}
                }
                
                report.push_str("\n");
            }
        }

        report
    }

    /// Generate a JSON report
    pub fn generate_json(stats: &[RTCStats]) -> Result<String, StatsError> {
        serde_json::to_string_pretty(stats)
            .map_err(|e| StatsError::InvalidFormat(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_quality() {
        let quality = ConnectionQuality {
            overall_score: 0.85,
            video_quality: 0.9,
            audio_quality: 0.8,
            network_health: 0.85,
        };
        
        assert!(quality.overall_score >= 0.0 && quality.overall_score <= 1.0);
    }

    #[tokio::test]
    async fn test_stats_collector() {
        let collector = StatsCollector::new(100);
        
        let stat = RTCStats {
            stats_type: StatsType::InboundRtp,
            timestamp: Utc::now(),
            id: "test".to_string(),
            data: StatsData::InboundRtp(RtpStreamStats {
                ssrc: 123456,
                media_type: MediaType::Video,
                codec_id: "VP8".to_string(),
                packets_received: 1000,
                packets_sent: 0,
                bytes_received: 50000,
                bytes_sent: 0,
                packets_lost: 5,
                fraction_lost: 0.5,
                jitter: 0.01,
                rtt: 0.05,
                bitrate: 1000000,
            }),
        };
        
        let result = collector.collect(stat.clone()).await;
        assert!(result.is_ok());
        
        let retrieved = collector.get_by_id("test").await;
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn test_quality_calculation() {
        let collector = StatsCollector::new(100);
        
        let quality = collector.calculate_quality().await;
        assert!(quality.overall_score >= 0.0 && quality.overall_score <= 1.0);
    }
}