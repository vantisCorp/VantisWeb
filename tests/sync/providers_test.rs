//! Unit tests for Sync Providers

#[cfg(test)]
mod tests {
    use vantisweb::sync::providers::{SyncProviderType, SyncConfig};

    #[test]
    fn test_sync_config_default() {
        let config = SyncConfig::default();
        assert_eq!(config.timeout_ms, 30000);
        assert!(config.auto_sync);
        assert!(config.conflict_resolution.is_some());
    }

    #[test]
    fn test_sync_provider_type_values() {
        assert_eq!(SyncProviderType::GoogleDrive.to_string(), "Google Drive");
        assert_eq!(SyncProviderType::Dropbox.to_string(), "Dropbox");
        assert_eq!(SyncProviderType::iCloud.to_string(), "iCloud");
        assert_eq!(SyncProviderType::WebDAV.to_string(), "WebDAV");
    }

    #[test]
    fn test_sync_provider_type_from_str() {
        use std::str::FromStr;
        assert_eq!(
            SyncProviderType::from_str("google_drive").unwrap(),
            SyncProviderType::GoogleDrive
        );
        assert_eq!(
            SyncProviderType::from_str("dropbox").unwrap(),
            SyncProviderType::Dropbox
        );
    }
}