//! Unit tests for Session Management

use vantisweb::auth::session::{SessionManager, DeviceType};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_manager_creation() {
        let manager = SessionManager::new();
        assert_eq!(manager.session_count(), 0);
    }

    #[test]
    fn test_create_session() {
        let mut manager = SessionManager::new();
        let session = manager.create_session(
            "user123",
            "device001",
            "Desktop",
            DeviceType::Desktop,
            "192.168.1.1",
        );
        assert!(session.is_some());
        assert_eq!(manager.session_count(), 1);
    }

    #[test]
    fn test_validate_session() {
        let mut manager = SessionManager::new();
        let session = manager.create_session(
            "user123",
            "device001",
            "Desktop",
            DeviceType::Desktop,
            "192.168.1.1",
        ).unwrap();
        assert!(manager.validate_session(&session.id));
    }

    #[test]
    fn test_revoke_session() {
        let mut manager = SessionManager::new();
        let session = manager.create_session(
            "user123",
            "device001",
            "Desktop",
            DeviceType::Desktop,
            "192.168.1.1",
        ).unwrap();
        assert!(manager.revoke_session(&session.id));
        assert!(!manager.validate_session(&session.id));
    }

    #[test]
    fn test_revoke_all_sessions() {
        let mut manager = SessionManager::new();
        manager.create_session("user1", "d1", "D1", DeviceType::Desktop, "ip1");
        manager.create_session("user1", "d2", "D2", DeviceType::Mobile, "ip2");
        manager.revoke_all_sessions("user1");
        assert_eq!(manager.session_count(), 0);
    }

    #[test]
    fn test_get_user_sessions() {
        let mut manager = SessionManager::new();
        manager.create_session("user1", "d1", "D1", DeviceType::Desktop, "ip1");
        manager.create_session("user1", "d2", "D2", DeviceType::Mobile, "ip2");
        manager.create_session("user2", "d3", "D3", DeviceType::Desktop, "ip3");
        let user_sessions = manager.get_user_sessions("user1");
        assert_eq!(user_sessions.len(), 2);
    }
}