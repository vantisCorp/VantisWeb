use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::collections::HashMap;
use std::path::PathBuf;

// Mock types for benchmarking
struct ProfileManager {
    profiles: HashMap<String, Profile>,
}

#[derive(Clone)]
struct Profile {
    name: String,
    settings: HashMap<String, String>,
}

impl ProfileManager {
    fn new(_base_path: PathBuf) -> Self {
        ProfileManager {
            profiles: HashMap::new(),
        }
    }

    fn load_profile(&mut self, name: &str) -> Result<Profile, String> {
        // Simulate loading
        std::thread::sleep(std::time::Duration::from_micros(50));
        let profile = Profile {
            name: name.to_string(),
            settings: HashMap::new(),
        };
        self.profiles.insert(name.to_string(), profile.clone());
        Ok(profile)
    }

    fn create_test_profile(&mut self, name: &str, size: usize) -> Profile {
        let mut settings = HashMap::new();
        for i in 0..size {
            settings.insert(format!("key{}", i), format!("value{}", i));
        }
        let profile = Profile {
            name: name.to_string(),
            settings,
        };
        self.profiles.insert(name.to_string(), profile.clone());
        profile
    }

    fn sync_profile(&self, _profile: &Profile) -> Result<(), String> {
        // Simulate sync
        std::thread::sleep(std::time::Duration::from_micros(200));
        Ok(())
    }
}

struct TemplateManager {
    templates: HashMap<String, Template>,
    loaded: bool,
}

#[derive(Clone)]
struct Template {
    name: String,
    settings: HashMap<String, String>,
}

impl TemplateManager {
    fn new(_base_path: PathBuf) -> Self {
        TemplateManager {
            templates: HashMap::new(),
            loaded: false,
        }
    }

    fn get_template(&mut self, name: &str) -> Result<Template, String> {
        if !self.loaded {
            self.load_all_templates();
        }
        self.templates.get(name).cloned().ok_or_else(|| "Template not found".to_string())
    }

    fn get_all_templates(&mut self) -> Vec<Template> {
        if !self.loaded {
            self.load_all_templates();
        }
        self.templates.values().cloned().collect()
    }

    fn load_all_templates(&mut self) {
        // Simulate loading templates
        std::thread::sleep(std::time::Duration::from_micros(100));
        
        let templates = vec!["work", "gaming", "privacy", "developer"];
        for name in templates {
            let template = Template {
                name: name.to_string(),
                settings: HashMap::new(),
            };
            self.templates.insert(name.to_string(), template);
        }
        self.loaded = true;
    }
}

fn bench_profile_loading(c: &mut Criterion) {
    let mut group = c.benchmark_group("profile_loading");
    
    group.bench_function("single_profile", |b| {
        let mut manager = ProfileManager::new(PathBuf::from("profiles"));
        b.iter(|| {
            manager.load_profile(black_box("default"))
        })
    });
    
    for profile_count in [1, 5, 10].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(profile_count),
            profile_count,
            |b, &profile_count| {
                let mut manager = ProfileManager::new(PathBuf::from("profiles"));
                b.iter(|| {
                    for i in 0..profile_count {
                        black_box(manager.load_profile(&format!("profile{}", i)));
                    }
                })
            }
        );
    }
    
    group.finish();
}

fn bench_template_loading(c: &mut Criterion) {
    let mut group = c.benchmark_group("template_loading");
    
    group.bench_function("single_template", |b| {
        let mut manager = TemplateManager::new(PathBuf::from("profiles/templates"));
        b.iter(|| {
            manager.get_template(black_box("work"))
        })
    });
    
    group.bench_function("all_templates", |b| {
        let mut manager = TemplateManager::new(PathBuf::from("profiles/templates"));
        b.iter(|| {
            manager.get_all_templates()
        })
    });
    
    group.finish();
}

fn bench_profile_sync(c: &mut Criterion) {
    let mut group = c.benchmark_group("profile_sync");
    
    group.bench_function("sync_small_profile", |b| {
        let mut manager = ProfileManager::new(PathBuf::from("profiles"));
        let profile = manager.create_test_profile("small", 100);
        b.iter(|| {
            manager.sync_profile(black_box(&profile))
        })
    });
    
    group.bench_function("sync_large_profile", |b| {
        let mut manager = ProfileManager::new(PathBuf::from("profiles"));
        let profile = manager.create_test_profile("large", 10000);
        b.iter(|| {
            manager.sync_profile(black_box(&profile))
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_profile_loading,
    bench_template_loading,
    bench_profile_sync
);
criterion_main!(benches);