//! Collaborative Browsing Module
//! 
//! Real-time collaborative browsing sessions with shared tabs,
//! annotations, chat, and presence awareness.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast, mpsc};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::{CloudError, CloudProvider};
use super::models::*;

/// Session Manager for collaborative browsing
pub struct SessionManager {
    sessions: RwLock<HashMap<String, CollaborativeSession>>,
    active_session: RwLock<Option<String>>,
    config: SessionConfig,
    event_sender: broadcast::Sender<SessionEvent>,
}

/// Session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Maximum participants per session
    pub max_participants: usize,
    /// Session timeout in minutes
    pub session_timeout_minutes: u32,
    /// Enable chat
    pub chat_enabled: bool,
    /// Enable screen sharing
    pub screen_sharing: bool,
    /// Enable annotations
    pub annotations_enabled: bool,
    /// Enable cursor sharing
    pub cursor_sharing: bool,
    /// Enable voice chat
    pub voice_chat: bool,
    /// Auto-close when owner leaves
    pub auto_close: bool,
    /// Session history retention (days)
    pub history_retention_days: u32,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_participants: 10,
            session_timeout_minutes: 60,
            chat_enabled: true,
            screen_sharing: true,
            annotations_enabled: true,
            cursor_sharing: true,
            voice_chat: false,
            auto_close: true,
            history_retention_days: 7,
        }
    }
}

/// Collaborative Session
pub struct CollaborativeSession {
    /// Session info
    pub info: SessionInfo,
    /// Participants
    participants: RwLock<HashMap<String, Participant>>,
    /// Shared tabs
    tabs: RwLock<HashMap<String, SharedTab>>,
    /// Chat messages
    chat: RwLock<Vec<ChatMessage>>,
    /// Annotations
    annotations: RwLock<Vec<Annotation>>,
    /// Cursors
    cursors: RwLock<HashMap<String, CursorPosition>>,
    /// WebSocket connections
    connections: RwLock<HashMap<String, mpsc::Sender<WebSocketMessage>>>,
    /// Event sender
    event_sender: broadcast::Sender<SessionEvent>,
    /// State
    state: RwLock<SessionState>,
}

/// Session state
#[derive(Debug, Clone, PartialEq)]
pub enum SessionState {
    Active,
    Paused,
    Ending,
    Ended,
}

/// Session event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionEvent {
    ParticipantJoined { participant: ParticipantInfo },
    ParticipantLeft { participant_id: String },
    TabShared { tab: SharedTab },
    TabClosed { tab_id: String },
    ChatMessage { message: ChatMessage },
    AnnotationAdded { annotation: Annotation },
    AnnotationRemoved { annotation_id: String },
    CursorMoved { participant_id: String, position: CursorPosition },
    ScreenShareStarted { participant_id: String },
    ScreenShareStopped { participant_id: String },
    SessionEnding,
    SessionEnded,
    Error { message: String },
}

/// Participant in a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub info: ParticipantInfo,
    pub joined_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub permissions: ParticipantPermissions,
}

/// Participant permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantPermissions {
    pub can_share_tabs: bool,
    pub can_close_tabs: bool,
    pub can_annotate: bool,
    pub can_chat: bool,
    pub can_share_screen: bool,
    pub can_invite: bool,
    pub can_kick: bool,
}

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub sender_id: String,
    pub sender_name: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub message_type: ChatMessageType,
    pub reply_to: Option<String>,
    pub reactions: HashMap<String, Vec<String>>,
}

/// Chat message type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChatMessageType {
    Text,
    System,
    Action,
    File,
    Link,
    Code,
}

/// WebSocket message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebSocketMessage {
    Join { session_id: String, participant: ParticipantInfo },
    Leave { participant_id: String },
    TabShare { tab: SharedTab },
    TabClose { tab_id: String },
    CursorMove { position: CursorPosition },
    Annotation { annotation: Annotation },
    AnnotationRemove { annotation_id: String },
    Chat { message: ChatMessage },
    ScreenShare { participant_id: String, stream_id: String },
    ScreenShareStop { participant_id: String },
    Sync { data: serde_json::Value },
    Ping,
    Pong,
}

/// Presence manager
pub struct PresenceManager {
    presence: RwLock<HashMap<String, UserPresence>>,
}

/// User presence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPresence {
    pub user_id: String,
    pub status: PresenceStatus,
    pub current_url: Option<String>,
    pub current_tab_title: Option<String>,
    pub last_active: DateTime<Utc>,
    pub device: DeviceType,
    pub custom_status: Option<String>,
}

/// Presence status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PresenceStatus {
    Online,
    Away,
    Busy,
    Offline,
    Invisible,
}

/// Annotation manager
pub struct AnnotationManager {
    annotations: RwLock<HashMap<String, Vec<Annotation>>>,
    permissions: AnnotationPermissions,
}

#[derive(Debug, Clone)]
struct AnnotationPermissions {
    allow_guests: bool,
    max_per_page: usize,
    max_size_kb: usize,
}

/// Voice chat manager
pub struct VoiceChatManager {
    channels: RwLock<HashMap<String, VoiceChannel>>,
    config: VoiceConfig,
}

#[derive(Debug, Clone)]
pub struct VoiceChannel {
    pub id: String,
    pub participants: Vec<String>,
    pub muted: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConfig {
    pub enabled: bool,
    pub max_participants: usize,
    pub noise_suppression: bool,
    pub echo_cancellation: bool,
    pub auto_gain: bool,
}

impl SessionManager {
    pub fn new() -> Self {
        let (event_sender, _) = broadcast::channel(1000);
        
        Self {
            sessions: RwLock::new(HashMap::new()),
            active_session: RwLock::new(None),
            config: SessionConfig::default(),
            event_sender,
        }
    }

    /// Create a new collaborative session
    pub async fn create(&self, name: &str) -> Result<CollaborativeSession, CloudError> {
        let session_id = Uuid::new_v4().to_string();
        let invite_code = self.generate_invite_code();
        
        let info = SessionInfo {
            id: session_id.clone(),
            name: name.to_string(),
            owner_id: "current_user".to_string(),
            created_at: Utc::now(),
            expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
            participant_count: 1,
            shared_tabs: Vec::new(),
            settings: SessionSettings {
                open_join: false,
                max_participants: self.config.max_participants,
                screen_sharing: self.config.screen_sharing,
                annotations: self.config.annotations_enabled,
                chat_enabled: self.config.chat_enabled,
                auto_close: self.config.auto_close,
            },
            invite_code: invite_code.clone(),
        };
        
        let session = CollaborativeSession::new(info.clone());
        
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session);
        
        let mut active = self.active_session.write().await;
        *active = Some(session_id);
        
        // Return a clone
        Ok(CollaborativeSession::new(info))
    }

    /// Join an existing session
    pub async fn join(&self, session_id: &str) -> Result<CollaborativeSession, CloudError> {
        let sessions = self.sessions.read().await;
        
        if let Some(session) = sessions.get(session_id) {
            // Add participant
            session.add_participant(ParticipantInfo {
                id: Uuid::new_v4().to_string(),
                name: "Guest".to_string(),
                avatar: None,
                role: ParticipantRole::Viewer,
                joined_at: Utc::now(),
                last_active: Utc::now(),
                online: true,
                device_type: DeviceType::Desktop,
            }).await;
            
            return Ok(CollaborativeSession::new(session.info.clone()));
        }
        
        Err(CloudError::NotFound(format!("Session {} not found", session_id)))
    }

    /// Leave a session
    pub async fn leave(&self, session_id: &str) -> Result<(), CloudError> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(session_id) {
            session.remove_participant("current_user").await;
        }
        
        Ok(())
    }

    /// Leave all sessions
    pub async fn leave_all(&self) -> Result<(), CloudError> {
        let mut sessions = self.sessions.write().await;
        sessions.clear();
        
        let mut active = self.active_session.write().await;
        *active = None;
        
        Ok(())
    }

    /// Get active session
    pub async fn get_active(&self) -> Option<CollaborativeSession> {
        let active = self.active_session.read().await;
        if let Some(session_id) = active.as_ref() {
            let sessions = self.sessions.read().await;
            sessions.get(session_id).map(|s| CollaborativeSession::new(s.info.clone()))
        } else {
            None
        }
    }

    /// Get count of active sessions
    pub async fn active_count(&self) -> usize {
        self.sessions.read().await.len()
    }

    /// Subscribe to session events
    pub fn subscribe(&self) -> broadcast::Receiver<SessionEvent> {
        self.event_sender.subscribe()
    }

    fn generate_invite_code(&self) -> String {
        // Generate a 6-character invite code
        use std::iter;
        const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
        let mut rng = rand::thread_rng();
        (0..6)
            .map(|_| {
                let idx = (rng.next_u32() as usize) % CHARSET.len();
                CHARSET[idx] as char
            })
            .collect()
    }
}

impl CollaborativeSession {
    pub fn new(info: SessionInfo) -> Self {
        let (event_sender, _) = broadcast::channel(1000);
        
        Self {
            info,
            participants: RwLock::new(HashMap::new()),
            tabs: RwLock::new(HashMap::new()),
            chat: RwLock::new(Vec::new()),
            annotations: RwLock::new(Vec::new()),
            cursors: RwLock::new(HashMap::new()),
            connections: RwLock::new(HashMap::new()),
            event_sender,
            state: RwLock::new(SessionState::Active),
        }
    }

    /// Add a participant
    pub async fn add_participant(&self, participant: ParticipantInfo) {
        let p = Participant {
            info: participant.clone(),
            joined_at: Utc::now(),
            last_active: Utc::now(),
            permissions: ParticipantPermissions::default_for(&participant.role),
        };
        
        let mut participants = self.participants.write().await;
        participants.insert(participant.id.clone(), p);
        
        let _ = self.event_sender.send(SessionEvent::ParticipantJoined { participant });
    }

    /// Remove a participant
    pub async fn remove_participant(&self, participant_id: &str) {
        let mut participants = self.participants.write().await;
        participants.remove(participant_id);
        
        let _ = self.event_sender.send(SessionEvent::ParticipantLeft {
            participant_id: participant_id.to_string(),
        });
    }

    /// Share a tab
    pub async fn share_tab(&self, tab: SharedTab) -> Result<(), CloudError> {
        let mut tabs = self.tabs.write().await;
        tabs.insert(tab.id.clone(), tab.clone());
        
        let _ = self.event_sender.send(SessionEvent::TabShared { tab });
        
        Ok(())
    }

    /// Close a shared tab
    pub async fn close_tab(&self, tab_id: &str) -> Result<(), CloudError> {
        let mut tabs = self.tabs.write().await;
        tabs.remove(tab_id);
        
        let _ = self.event_sender.send(SessionEvent::TabClosed {
            tab_id: tab_id.to_string(),
        });
        
        Ok(())
    }

    /// Send a chat message
    pub async fn send_message(&self, message: ChatMessage) -> Result<(), CloudError> {
        let mut chat = self.chat.write().await;
        chat.push(message.clone());
        
        let _ = self.event_sender.send(SessionEvent::ChatMessage { message });
        
        Ok(())
    }

    /// Get chat history
    pub async fn get_chat_history(&self) -> Vec<ChatMessage> {
        self.chat.read().await.clone()
    }

    /// Add an annotation
    pub async fn add_annotation(&self, annotation: Annotation) -> Result<(), CloudError> {
        let mut annotations = self.annotations.write().await;
        annotations.push(annotation.clone());
        
        let _ = self.event_sender.send(SessionEvent::AnnotationAdded { annotation });
        
        Ok(())
    }

    /// Remove an annotation
    pub async fn remove_annotation(&self, annotation_id: &str) -> Result<(), CloudError> {
        let mut annotations = self.annotations.write().await;
        annotations.retain(|a| a.id != annotation_id);
        
        let _ = self.event_sender.send(SessionEvent::AnnotationRemoved {
            annotation_id: annotation_id.to_string(),
        });
        
        Ok(())
    }

    /// Update cursor position
    pub async fn update_cursor(&self, participant_id: &str, position: CursorPosition) {
        let mut cursors = self.cursors.write().await;
        cursors.insert(participant_id.to_string(), position.clone());
        
        let _ = self.event_sender.send(SessionEvent::CursorMoved {
            participant_id: participant_id.to_string(),
            position,
        });
    }

    /// Get all cursor positions
    pub async fn get_cursors(&self) -> HashMap<String, CursorPosition> {
        self.cursors.read().await.clone()
    }

    /// Get participants
    pub async fn get_participants(&self) -> Vec<ParticipantInfo> {
        self.participants.read().await.values().map(|p| p.info.clone()).collect()
    }

    /// End the session
    pub async fn end(&self) -> Result<(), CloudError> {
        let mut state = self.state.write().await;
        *state = SessionState::Ending;
        
        let _ = self.event_sender.send(SessionEvent::SessionEnding);
        
        // Notify all participants
        let connections = self.connections.read().await;
        for sender in connections.values() {
            let _ = sender.send(WebSocketMessage::Leave {
                participant_id: "session_ended".to_string(),
            }).await;
        }
        
        *state = SessionState::Ended;
        
        let _ = self.event_sender.send(SessionEvent::SessionEnded);
        
        Ok(())
    }

    /// Subscribe to session events
    pub fn subscribe(&self) -> broadcast::Receiver<SessionEvent> {
        self.event_sender.subscribe()
    }
}

impl ParticipantPermissions {
    fn default_for(role: &ParticipantRole) -> Self {
        match role {
            ParticipantRole::Owner => Self {
                can_share_tabs: true,
                can_close_tabs: true,
                can_annotate: true,
                can_chat: true,
                can_share_screen: true,
                can_invite: true,
                can_kick: true,
            },
            ParticipantRole::Admin => Self {
                can_share_tabs: true,
                can_close_tabs: true,
                can_annotate: true,
                can_chat: true,
                can_share_screen: true,
                can_invite: true,
                can_kick: false,
            },
            ParticipantRole::Editor => Self {
                can_share_tabs: true,
                can_close_tabs: false,
                can_annotate: true,
                can_chat: true,
                can_share_screen: true,
                can_invite: false,
                can_kick: false,
            },
            ParticipantRole::Viewer => Self {
                can_share_tabs: false,
                can_close_tabs: false,
                can_annotate: false,
                can_chat: true,
                can_share_screen: false,
                can_invite: false,
                can_kick: false,
            },
        }
    }
}

impl PresenceManager {
    pub fn new() -> Self {
        Self {
            presence: RwLock::new(HashMap::new()),
        }
    }

    pub async fn update_presence(&self, user_id: &str, presence: UserPresence) {
        let mut presences = self.presence.write().await;
        presences.insert(user_id.to_string(), presence);
    }

    pub async fn get_presence(&self, user_id: &str) -> Option<UserPresence> {
        self.presence.read().await.get(user_id).cloned()
    }

    pub async fn get_all_online(&self) -> Vec<UserPresence> {
        self.presence.read().await
            .values()
            .filter(|p| p.status == PresenceStatus::Online)
            .cloned()
            .collect()
    }
}

impl AnnotationManager {
    pub fn new() -> Self {
        Self {
            annotations: RwLock::new(HashMap::new()),
            permissions: AnnotationPermissions {
                allow_guests: true,
                max_per_page: 100,
                max_size_kb: 500,
            },
        }
    }
}

impl VoiceChatManager {
    pub fn new(config: VoiceConfig) -> Self {
        Self {
            channels: RwLock::new(HashMap::new()),
            config,
        }
    }

    pub async fn create_channel(&self, session_id: &str) -> VoiceChannel {
        let channel = VoiceChannel {
            id: Uuid::new_v4().to_string(),
            participants: Vec::new(),
            muted: Vec::new(),
            created_at: Utc::now(),
        };
        
        let mut channels = self.channels.write().await;
        channels.insert(session_id.to_string(), channel.clone());
        
        channel
    }

    pub async fn join_channel(&self, session_id: &str, participant_id: &str) -> Result<(), CloudError> {
        let mut channels = self.channels.write().await;
        
        if let Some(channel) = channels.get_mut(session_id) {
            if channel.participants.len() >= self.config.max_participants {
                return Err(CloudError::SessionError("Voice channel full".to_string()));
            }
            channel.participants.push(participant_id.to_string());
        }
        
        Ok(())
    }

    pub async fn leave_channel(&self, session_id: &str, participant_id: &str) {
        let mut channels = self.channels.write().await;
        
        if let Some(channel) = channels.get_mut(session_id) {
            channel.participants.retain(|p| p != participant_id);
            channel.muted.retain(|p| p != participant_id);
        }
    }

    pub async fn toggle_mute(&self, session_id: &str, participant_id: &str) -> bool {
        let mut channels = self.channels.write().await;
        
        if let Some(channel) = channels.get_mut(session_id) {
            if channel.muted.contains(&participant_id.to_string()) {
                channel.muted.retain(|p| p != participant_id);
                return false;
            } else {
                channel.muted.push(participant_id.to_string());
                return true;
            }
        }
        
        false
    }
}

// Need rand for invite code generation
mod rand {
    use std::cell::Cell;
    
    pub struct ThreadRng(Cell<u32>);
    
    pub fn thread_rng() -> ThreadRng {
        ThreadRng(Cell::new(1))
    }
    
    impl ThreadRng {
        pub fn next_u32(&self) -> u32 {
            // Simple LCG for placeholder
            let mut state = self.0.get();
            state = state.wrapping_mul(1103515245).wrapping_add(12345);
            self.0.set(state);
            state
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_config_defaults() {
        let config = SessionConfig::default();
        assert_eq!(config.max_participants, 10);
        assert!(config.chat_enabled);
        assert!(config.annotations_enabled);
    }

    #[tokio::test]
    async fn test_create_session() {
        let manager = SessionManager::new();
        let session = manager.create("Test Session").await.unwrap();
        
        assert_eq!(session.info.name, "Test Session");
        assert!(!session.info.invite_code.is_empty());
    }
}