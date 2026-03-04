//! Unit tests for the Profile Manager module

use vantisweb::profiles::profile_manager::ProfileManager;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_manager_creation() {
        let manager = ProfileManager::new();
        assert_eq!(manager.profile_count(), 0);
    }

    #[test]
    fn test_create_profile() {
        let mut manager = ProfileManager::new();
        let profile = manager.create_profile("Test Profile");
        assert_eq!(profile.name, "Test Profile");
        assert_eq!(manager.profile_count(), 1);
    }

    #[test]
    fn test_delete_profile() {
        let mut manager = ProfileManager::new();
        let profile = manager.create_profile("Test Profile");
        assert!(manager.delete_profile(&profile.id));
        assert_eq!(manager.profile_count(), 0);
    }

    #[test]
    fn test_get_profile() {
        let mut manager = ProfileManager::new();
        let profile = manager.create_profile("Test Profile");
        let retrieved = manager.get_profile(&profile.id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test Profile");
    }

    #[test]
    fn test_list_profiles() {
        let mut manager = ProfileManager::new();
        manager.create_profile("Profile 1");
        manager.create_profile("Profile 2");
        manager.create_profile("Profile 3");
        let profiles = manager.list_profiles();
        assert_eq!(profiles.len(), 3);
    }

    #[test]
    fn test_set_default_profile() {
        let mut manager = ProfileManager::new();
        let profile1 = manager.create_profile("Profile 1");
        let profile2 = manager.create_profile("Profile 2");
        manager.set_default_profile(&profile1.id);
        assert_eq!(manager.get_default_profile_id(), Some(profile1.id));
        manager.set_default_profile(&profile2.id);
        assert_eq!(manager.get_default_profile_id(), Some(profile2.id));
    }

    #[test]
    fn test_clone_profile() {
        let mut manager = ProfileManager::new();
        let profile = manager.create_profile("Original Profile");
        let cloned = manager.clone_profile(&profile.id);
        assert!(cloned.is_some());
        assert_eq!(manager.profile_count(), 2);
    }

    #[test]
    fn test_update_profile() {
        let mut manager = ProfileManager::new();
        let profile = manager.create_profile("Old Name");
        manager.update_profile_name(&profile.id, "New Name");
        let updated = manager.get_profile(&profile.id);
        assert_eq!(updated.unwrap().name, "New Name");
    }

    #[test]
    fn test_profile_search() {
        let mut manager = ProfileManager::new();
        manager.create_profile("Work Profile");
        manager.create_profile("Personal Profile");
        manager.create_profile("Gaming Profile");
        let results = manager.search_profiles("Profile");
        assert_eq!(results.len(), 3);
    }
}