//! Unit tests for the Storage module

use vantisweb::core::storage::{Storage, StorageConfig};

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_storage() -> Storage {
        let config = StorageConfig {
            max_size: 1024 * 1024, // 1MB
            cache_enabled: true,
            persistence: true,
        };
        Storage::new(config)
    }

    #[test]
    fn test_storage_creation() {
        let storage = create_test_storage();
        assert!(storage.is_ready());
    }

    #[test]
    fn test_storage_set_get() {
        let mut storage = create_test_storage();
        storage.set("test_key", "test_value");
        assert_eq!(storage.get("test_key"), Some(&"test_value".to_string()));
    }

    #[test]
    fn test_storage_delete() {
        let mut storage = create_test_storage();
        storage.set("test_key", "test_value");
        assert!(storage.delete("test_key"));
        assert_eq!(storage.get("test_key"), None);
    }

    #[test]
    fn test_storage_clear() {
        let mut storage = create_test_storage();
        storage.set("key1", "value1");
        storage.set("key2", "value2");
        storage.clear();
        assert_eq!(storage.len(), 0);
    }

    #[test]
    fn test_storage_keys() {
        let mut storage = create_test_storage();
        storage.set("key1", "value1");
        storage.set("key2", "value2");
        let keys = storage.keys();
        assert_eq!(keys.len(), 2);
    }

    #[test]
    fn test_storage_size() {
        let mut storage = create_test_storage();
        storage.set("key", "value");
        let size = storage.size();
        assert!(size > 0);
    }

    #[test]
    fn test_storage_exists() {
        let mut storage = create_test_storage();
        storage.set("test_key", "test_value");
        assert!(storage.exists("test_key"));
        assert!(!storage.exists("non_existent"));
    }

    #[test]
    fn test_storage_batch_operations() {
        let mut storage = create_test_storage();
        let items = vec![
            ("key1", "value1"),
            ("key2", "value2"),
            ("key3", "value3"),
        ];
        storage.set_batch(&items);
        assert_eq!(storage.len(), 3);
    }
}