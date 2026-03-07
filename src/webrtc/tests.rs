/// # WebRTC Integration Tests
/// 
/// Comprehensive test suite for WebRTC functionality.

#[cfg(test)]
mod tests {
    use crate::webrtc::*;
    use crate::webrtc::connection::RTCPeerConnection;
    use crate::webrtc::media::{MediaStream, MediaTrack, MediaConstraints};
    use crate::webrtc::data_channel::{RTCDataChannel, RTCDataChannelInit};
    use crate::webrtc::ice::{IceAgent, IceConfig, RTCIceCandidate, RTCIceCandidateType};
    use crate::webrtc::signaling::{RTCSessionDescription, RTCSdpType, SignalingMessage};
    use crate::webrtc::stats::{StatsCollector, StatsType, StatsData, RtpStreamStats, MediaType};

    // ============= Configuration Tests =============

    #[test]
    fn test_default_configuration() {
        let config = RTCConfiguration::default();
        
        assert!(!config.ice_servers.is_empty());
        assert!(config.enable_ipv6);
    }

    #[test]
    fn test_custom_ice_server() {
        let config = RTCConfiguration {
            ice_servers: vec![crate::webrtc::ice::ICEServer {
                urls: vec!["stun:stun.example.com:3478".to_string()],
                username: Some("testuser".to_string()),
                credential: Some("testpass".to_string()),
            }],
            ..Default::default()
        };
        
        assert_eq!(config.ice_servers.len(), 1);
        assert_eq!(config.ice_servers[0].username, Some("testuser".to_string()));
    }

    // ============= Manager Tests =============

    #[tokio::test]
    async fn test_manager_creation() {
        let manager = WebRTCManager::default();
        let connections = manager.get_all_connections().await;
        
        assert!(connections.is_empty());
    }

    #[tokio::test]
    async fn test_create_peer_connection() {
        let manager = WebRTCManager::default();
        
        let result = manager.create_peer_connection("test-peer").await;
        assert!(result.is_ok());
        
        let connections = manager.get_all_connections().await;
        assert_eq!(connections.len(), 1);
    }

    #[tokio::test]
    async fn test_get_peer_connection() {
        let manager = WebRTCManager::default();
        
        manager.create_peer_connection("peer-1").await.unwrap();
        manager.create_peer_connection("peer-2").await.unwrap();
        
        let connection = manager.get_peer_connection("peer-1").await;
        assert!(connection.is_some());
        
        let connection = manager.get_peer_connection("nonexistent").await;
        assert!(connection.is_none());
    }

    #[tokio::test]
    async fn test_close_peer_connection() {
        let manager = WebRTCManager::default();
        
        manager.create_peer_connection("peer-1").await.unwrap();
        
        let result = manager.close_peer_connection("peer-1").await;
        assert!(result.is_ok());
        
        let connection = manager.get_peer_connection("peer-1").await;
        assert!(connection.is_none());
    }

    // ============= ICE Tests =============

    #[tokio::test]
    async fn test_ice_agent_creation() {
        let config = IceConfig::default();
        let agent = IceAgent::new(config);
        
        let state = agent.connection_state().await;
        assert!(matches!(state, crate::webrtc::ice::RTCIceConnectionState::New));
    }

    #[tokio::test]
    async fn test_ice_candidate_creation() {
        let candidate = RTCIceCandidate::new_host("192.168.1.1".to_string(), 50000, 1);
        
        assert_eq!(candidate.ip, "192.168.1.1");
        assert_eq!(candidate.port, 50000);
        assert_eq!(candidate.candidate_type, RTCIceCandidateType::Host);
    }

    #[tokio::test]
    async fn test_ice_candidate_sdp() {
        let candidate = RTCIceCandidate::new_host("192.168.1.1".to_string(), 50000, 1);
        let sdp = candidate.to_sdp();
        
        assert!(sdp.contains("192.168.1.1"));
        assert!(sdp.contains("host"));
        assert!(sdp.contains("50000"));
    }

    // ============= Signaling Tests =============

    #[test]
    fn test_session_description_offer() {
        let offer = RTCSessionDescription::offer("v=0\r\ns=test\r\n".to_string());
        
        assert_eq!(offer.sdp_type, RTCSdpType::Offer);
        assert!(offer.sdp.contains("s=test"));
    }

    #[test]
    fn test_session_description_answer() {
        let answer = RTCSessionDescription::answer("v=0\r\ns=test-answer\r\n".to_string());
        
        assert_eq!(answer.sdp_type, RTCSdpType::Answer);
    }

    #[test]
    fn test_signaling_message_serialization() {
        let offer = RTCSessionDescription::offer("test sdp".to_string());
        let message = SignalingMessage::Offer(offer);
        
        let json = serde_json::to_string(&message).unwrap();
        assert!(json.contains("Offer"));
    }

    // ============= Stats Tests =============

    #[tokio::test]
    async fn test_stats_collector() {
        let collector = StatsCollector::new(100);
        
        let stat = RTCStats {
            stats_type: StatsType::InboundRtp,
            timestamp: chrono::Utc::now(),
            id: "test-stat".to_string(),
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
        
        let result = collector.collect(stat).await;
        assert!(result.is_ok());
        
        let stats = collector.get_all().await;
        assert_eq!(stats.len(), 1);
    }

    #[tokio::test]
    async fn test_stats_quality_calculation() {
        let collector = StatsCollector::new(100);
        
        let quality = collector.calculate_quality().await;
        
        assert!(quality.overall_score >= 0.0 && quality.overall_score <= 1.0);
        assert!(quality.video_quality >= 0.0 && quality.video_quality <= 1.0);
        assert!(quality.audio_quality >= 0.0 && quality.audio_quality <= 1.0);
        assert!(quality.network_health >= 0.0 && quality.network_health <= 1.0);
    }

    // ============= Data Channel Tests =============

    #[tokio::test]
    async fn test_data_channel_creation() {
        let config = RTCDataChannelInit::default();
        let channel = RTCDataChannel::new("test-channel".to_string(), config);
        
        assert_eq!(channel.label(), "test-channel");
        
        let state = channel.state().await;
        assert!(matches!(state, crate::webrtc::data_channel::RTCDataChannelState::Connecting));
    }

    #[tokio::test]
    async fn test_data_channel_open_close() {
        let config = RTCDataChannelInit::default();
        let channel = RTCDataChannel::new("test".to_string(), config);
        
        channel.open().await;
        assert!(channel.is_open().await);
        
        channel.close().await;
        let state = channel.state().await;
        assert!(matches!(state, crate::webrtc::data_channel::RTCDataChannelState::Closed));
    }

    #[tokio::test]
    async fn test_data_channel_send_receive() {
        let config = RTCDataChannelInit::default();
        let channel = RTCDataChannel::new("test".to_string(), config);
        
        channel.open().await;
        
        let result = channel.send_text("Hello, WebRTC!".to_string()).await;
        assert!(result.is_ok());
        
        let data = vec![1u8, 2, 3, 4, 5];
        let result = channel.send_binary(data).await;
        assert!(result.is_ok());
    }
}