//! Unit tests for core modules

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    #[test]
    fn test_history_manager_add_entry() {
        // Test adding history entry
        let mut history_manager = vanisweb::core::history::HistoryManager::new(100);
        
        history_manager.add_entry(
            "https://example.com".to_string(),
            "Example Site".to_string()
        ).unwrap();
        
        assert_eq!(history_manager.count(), 1);
    }

    #[test]
    fn test_history_manager_search() {
        // Test history search
        let mut history_manager = vanisweb::core::history::HistoryManager::new(100);
        
        history_manager.add_entry(
            "https://example.com".to_string(),
            "Example Site".to_string()
        ).unwrap();
        
        let results = history_manager.search("example");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_bookmark_manager_add() {
        // Test adding bookmark
        let mut bookmark_manager = vanisweb::core::bookmarks::BookmarkManager::new();
        
        bookmark_manager.add(
            "https://example.com".to_string(),
            "Example Site".to_string(),
            None
        ).unwrap();
        
        assert_eq!(bookmark_manager.count(), 1);
    }

    #[test]
    fn test_bookmark_manager_is_bookmarked() {
        // Test checking if URL is bookmarked
        let mut bookmark_manager = vanisweb::core::bookmarks::BookmarkManager::new();
        
        bookmark_manager.add(
            "https://example.com".to_string(),
            "Example Site".to_string(),
            None
        ).unwrap();
        
        assert!(bookmark_manager.is_bookmarked("https://example.com"));
    }

    #[test]
    fn test_download_manager_start() {
        // Test starting download
        // This is a placeholder test
        assert!(true);
    }
}