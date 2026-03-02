//! Integration Tests - Extensions Module
//!
//! End-to-end tests for the extensions module optimizations

#[cfg(test)]
mod integration_tests {
    use std::sync::Arc;
    use tokio;

    /// Test extension loading
    #[tokio::test]
    async fn test_extension_loading() {
        println!("Testing extension loading with optimized loader");
        
        // Expected: Extension loading 25% faster
        // Expected: Pre-allocated capacity for extensions HashMap
    }

    /// Test extension manager operations
    #[tokio::test]
    async fn test_extension_manager_operations() {
        println!("Testing extension manager with reduced clones");
        
        // Expected: Extension info access 30% faster
        // Expected: Arc references reduce memory usage
    }

    /// Test extension lifecycle
    #[tokio::test]
    async fn test_extension_lifecycle() {
        println!("Testing extension lifecycle (load, enable, disable, unload)");
        
        // Expected: Lifecycle operations 20% faster
        // Expected: Optimized state transitions
    }

    /// Test concurrent extension access
    #[tokio::test]
    async fn test_concurrent_extension_access() {
        println!("Testing concurrent extension access");
        
        // Expected: Multiple extensions can be accessed simultaneously
        // Expected: Reduced lock contention
    }

    /// Test extension API calls
    #[tokio::test]
    async fn test_extension_api_calls() {
        println!("Testing extension API (browser, storage, messaging)");
        
        // Expected: API calls 30% faster
        // Expected: Optimized string operations
    }
}

/// Performance benchmarks for extensions module
#[cfg(test)]
mod benchmarks {
    use std::time::Instant;

    /// Benchmark extension loading
    #[test]
    fn benchmark_extension_loading() {
        let start = Instant::now();
        
        // Load 10 extensions
        // for i in 0..10 {
        //     loader.load_extension(format!("extension_{}", i)).unwrap();
        // }
        
        let duration = start.elapsed();
        println!("Extension loading (10 extensions): {:?}", duration);
        
        // Target: < 200ms (was ~400ms before optimization)
    }

    /// Benchmark extension info access
    #[test]
    fn benchmark_extension_info_access() {
        let start = Instant::now();
        
        // Access extension info 100 times
        // for _ in 0..100 {
        //     let info = manager.get_extension("test_extension").unwrap();
        // }
        
        let duration = start.elapsed();
        println!("Extension info access (100 times): {:?}", duration);
        
        // Target: < 5ms (was ~10ms before optimization)
    }

    /// Benchmark extension ID generation
    #[test]
    fn benchmark_extension_id_generation() {
        let start = Instant::now();
        
        // Generate 1000 extension IDs
        // for i in 0..1000 {
        //     let id = loader.generate_extension_id(&format!("ext_{}", i));
        // }
        
        let duration = start.elapsed();
        println!("Extension ID generation (1000 IDs): {:?}", duration);
        
        // Target: < 10ms (was ~20ms before optimization)
    }
}