use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::collections::HashMap;
use std::path::PathBuf;

// Mock types for benchmarking
struct ExtensionManager {
    storage: HashMap<String, HashMap<String, String>>,
}

impl ExtensionManager {
    fn new() -> Self {
        ExtensionManager {
            storage: HashMap::new(),
        }
    }

    fn get_extension_storage(&amp;self, ext_id: &amp;str, key: &amp;str) -> Option<&amp;String> {
        self.storage.get(ext_id)?.get(key)
    }

    fn set_extension_storage(&amp;mut self, ext_id: &amp;str, key: &amp;str, value: &amp;str) {
        self.storage
            .entry(ext_id.to_string())
            .or_insert_with(HashMap::new)
            .insert(key.to_string(), value.to_string());
    }

    fn send_message(&amp;self, _from: &amp;str, _to: &amp;str, _msg: &amp;str) -> Result<(), String> {
        // Simulate message sending
        std::thread::sleep(std::time::Duration::from_micros(5));
        Ok(())
    }
}

struct ExtensionLoader {
    cache: HashMap<PathBuf, String>,
}

impl ExtensionLoader {
    fn new() -> Self {
        ExtensionLoader {
            cache: HashMap::new(),
        }
    }

    fn load_extension(&amp;mut self, path: &amp;PathBuf) -> Result<String, String> {
        // Simulate loading with cache
        if let Some(cached) = self.cache.get(path) {
            return Ok(cached.clone());
        }
        
        // Simulate file loading
        std::thread::sleep(std::time::Duration::from_micros(100));
        let manifest = format!("{{&quot;name&quot;: &quot;{}&quot;}}", path.display());
        self.cache.insert(path.clone(), manifest.clone());
        Ok(manifest)
    }
}

fn bench_extension_loading(c: &amp;mut Criterion) {
    let mut group = c.benchmark_group("extension_loading");
    
    group.bench_function("single_extension", |b| {
        let mut loader = ExtensionLoader::new();
        let path = PathBuf::from("extensions/example-extension");
        b.iter(|| {
            loader.load_extension(black_box(&amp;path))
        })
    });
    
    for ext_count in [1, 5, 10].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(ext_count),
            ext_count,
            |b, &amp;ext_count| {
                let mut loader = ExtensionLoader::new();
                let paths: Vec<_> = (0..ext_count)
                    .map(|i| PathBuf::from(format!("extensions/ext{}", i)))
                    .collect();
                b.iter(|| {
                    for path in &amp;paths {
                        black_box(loader.load_extension(path));
                    }
                })
            }
        );
    }
    
    group.finish();
}

fn bench_extension_api_calls(c: &amp;mut Criterion) {
    let mut group = c.benchmark_group("extension_api");
    
    group.bench_function("storage_get", |b| {
        let mut manager = ExtensionManager::new();
        manager.set_extension_storage("test_ext", "key", "value");
        b.iter(|| {
            manager.get_extension_storage(black_box("test_ext"), black_box("key"))
        })
    });
    
    group.bench_function("storage_set", |b| {
        let mut manager = ExtensionManager::new();
        b.iter(|| {
            manager.set_extension_storage(
                black_box("test_ext"),
                black_box("key"),
                black_box("value")
            )
        })
    });
    
    group.bench_function("messaging_send", |b| {
        let manager = ExtensionManager::new();
        b.iter(|| {
            manager.send_message(
                black_box("ext1"),
                black_box("ext2"),
                black_box("test_message")
            )
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_extension_loading,
    bench_extension_api_calls
);
criterion_main!(benches);