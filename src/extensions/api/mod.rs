//! Extension API
//!
//! Provides APIs for extensions to interact with the browser.

pub mod browser;
pub mod storage;
pub mod messaging;
pub mod tabs;
pub mod requests;

pub use browser::BrowserAPI;
pub use storage::StorageAPI;
pub use messaging::MessagingAPI;
pub use tabs::TabsAPI;
pub use requests::RequestsAPI;