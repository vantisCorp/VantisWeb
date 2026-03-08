//! VantisWeb Browser Library
//! 
//! Next-generation web browser with Liquid Core Architecture

pub mod core;
pub mod ui;
pub mod engine;
pub mod extensions;
pub mod security;
pub mod modules;
pub mod profiles;
pub mod network;
pub mod ai;
pub mod utils;
pub mod sync;
pub mod auth;
pub mod voice;
pub mod webrtc;
pub mod history;
pub mod installer;
pub mod profiling;
pub mod reading;
pub mod adblock;
pub mod password;
pub mod vpn;
pub mod capture;
pub mod bookmarks;
pub mod downloads;
pub mod devtools;
pub mod analytics;
pub mod pdf;
pub mod tabs;
pub mod accessibility;
pub mod health;
pub mod dubbing;
pub mod visual_regression;
pub mod performance;
pub mod ai_enhanced;
pub mod cloud_integration;

pub use core::kernel::VantisKernel;
pub use security::SecurityManager;
pub use ui::VantisUI;
pub use sync::{CloudSyncManager, SyncError, SyncProvider, SyncProviderType};
pub use auth::{AuthManager, AuthConfig, AuthError};
pub use voice::{VoiceRecognizer, VoiceProcessor, CommandRegistry};
pub use webrtc::{WebRTCManager, RTCConfiguration};
pub use history::{HistoryManager, HistoryEntry, HistoryQuery};
pub use password::{PasswordManager, PasswordConfig, PasswordEntry, PasswordStrength};
pub use vpn::{VPNManager, VPNConfig, VPNState, VPNProtocol, VPNServer, VPNStats};
pub use capture::{CaptureManager, CaptureConfig, CaptureResult, CaptureError};
pub use pdf::{PDFManager, PDFConfig, PDFDocument, PDFError, PDFAnnotation, FormField};
pub use tabs::{TabManager, TabGroupManager, WorkspaceManager, TabHibernationManager, TabSyncManager, TabSearchIndex};
pub use bookmarks::{BookmarkManager, BookmarkFolderManager, BookmarkSearchEngine, BookmarkSyncManager, BookmarkImporter, BookmarkExporter};
pub use downloads::{DownloadManager, DownloadScheduler, DownloadAccelerator, DownloadOrganizer, DownloadHistory, BrowserIntegration};
pub use devtools::{DevToolsManager, ElementsInspector, NetworkMonitor, JSConsole, PerformanceProfiler, StorageInspector};
pub use analytics::{AnalyticsEngine, AnalyticsConfig, AnalyticsError, AnalyticsSnapshot};
pub use accessibility::{AccessibilityManager, AccessibilityConfig, AccessibilityProfile, AccessibilityLevel};
pub use health::{HealthModule, NutriScanner, IngredX, VisualCalorieCounter, BioSync};
pub use dubbing::{LiveTranslator, VoiceCloner, LipSyncProcessor, VantisCluster};
pub use visual_regression::{VisualRegressionManager, ScreenshotCapture, DiffEngine, BaselineManager};
pub use performance::{PerformanceManager, LoadTester, StressTester, BenchmarkRunner, PerformanceMonitor};
pub use ai_enhanced::{AIEnhancedManager, AIAssistant, SmartSearch, ContentAnalyzer, PredictiveNavigator, BookmarkAI};
pub use cloud_integration::{CloudManager, CloudConfig, sync::RealTimeSync, collaboration::SessionManager, backup::CloudBackup, cross_device::CrossDeviceManager};

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = "VantisWeb Browser";