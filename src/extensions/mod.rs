//! Extensions System
//!
//! Plugin/add-on system for extending browser functionality:
//! - Extension loading and management
//! - Extension API (browser, storage, messaging, tabs, requests)
//! - Security and sandboxing
//! - Manifest-based configuration
//! - Extension marketplace
//! - Permission management
//! - Content script management
//! - Runtime API
//! - Advanced security features

pub mod extension;
pub mod manager;
pub mod registry;
pub mod loader;
pub mod manifest;
pub mod marketplace;
pub mod permissions;
pub mod sandbox;

// Advanced modules
pub mod content;
pub mod runtime;
pub mod security;

pub mod api {
    pub mod browser;
    pub mod storage;
    pub mod messaging;
    pub mod tabs;
    pub mod requests;
}

pub use extension::{Extension, ExtensionInfo, ExtensionState, ExtensionType};
pub use manager::ExtensionManager;
pub use registry::ExtensionRegistry;
pub use loader::ExtensionLoader;
pub use manifest::{Manifest, ManifestParser};
pub use marketplace::{ExtensionMarketplace, ExtensionListing, MarketplaceConfig};
pub use permissions::{Permission, PermissionManager, PermissionConfig};
pub use sandbox::{ExtensionSandbox, SandboxConfig, SandboxContext};

// Advanced modules exports
pub use content::{
    ContentScriptManager, ContentScript, ScriptType, RunAt,
    InjectionContext, ExecutionResult, UserScriptMetadata
};
pub use runtime::{
    RuntimeAPI, RuntimeMessage, MessageResponse, Port,
    RuntimeEventType, ExtensionEvent, InstallReason, PlatformInfo
};
pub use security::{
    SecurityManager, PermissionValidationResult, ThreatDetectionResult,
    ExtensionSecurityPolicy, TrustLevel, SecurityAuditEntry
};

pub use api::browser::BrowserAPI;
pub use api::storage::StorageAPI;
pub use api::messaging::MessagingAPI;
pub use api::tabs::TabsAPI;
pub use api::requests::RequestsAPI;
