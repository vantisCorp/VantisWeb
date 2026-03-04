//! Extensions System
//!
//! Plugin/add-on system for extending browser functionality:
//! - Extension loading and management
//! - Extension API (browser, storage, messaging, tabs, requests)
//! - Security and sandboxing
//! - Manifest-based configuration
//! - Extension marketplace
//! - Permission management

pub mod extension;
pub mod manager;
pub mod registry;
pub mod loader;
pub mod manifest;
pub mod marketplace;
pub mod permissions;
pub mod sandbox;

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

pub use api::browser::BrowserAPI;
pub use api::storage::StorageAPI;
pub use api::messaging::MessagingAPI;
pub use api::tabs::TabsAPI;
pub use api::requests::RequestsAPI;