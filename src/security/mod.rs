//! Vantis Security Module
//! 
//! Comprehensive security system:
//! - Digital Immune System
//! - Post-quantum cryptography
//! - Polymorphic Code Engine
//! - Merkle Tree verification
//! - Sandbox isolation

pub mod manager;
pub mod crypto;
pub mod immune_system;
pub mod sandbox;

pub use manager::SecurityManager;
pub use crypto::{CryptoEngine, PostQuantumCrypto};
pub use immune_system::DigitalImmuneSystem;
pub use sandbox::Sandbox;