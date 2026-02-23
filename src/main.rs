//! VantisWeb Browser - Next Generation Web Experience
//! 
//! Copyright © 2024 Vantis Corp
//! Licensed under MIT License

mod core;
mod ui;
mod engine;
mod security;
mod modules;
mod profiles;
mod network;
mod ai;
mod utils;

use anyhow::Result;
use log::{info, error};
use tokio::runtime::Runtime;

use core::settings::SettingsManager;
use core::private_mode::PrivateModeManager;

fn main() -> Result<()> {
    // Initialize logger
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("╔══════════════════════════════════════════════╗");
    info!("║     VANTISWEB BROWSER v0.1.0               ║");
    info!("║     Liquid Core Architecture                ║");
    info!("║     © 2024 Vantis Corp                      ║");
    info!("╚══════════════════════════════════════════════╝");
    
    // Create async runtime
    let rt = Runtime::new()?;
    
    rt.block_on(async {
        run_browser().await
    })
}

async fn run_browser() -> Result<()> {
    info!("Initializing VantisWeb Core...");
    
    // Initialize Vantis Kernel
    let kernel = core::kernel::VantisKernel::new().await?;
    info!("✓ Vantis Kernel initialized");
    
    // Initialize Settings Manager
    let config_dir = kernel.get_config().await.get_config_dir();
    let settings_file = config_dir.join("settings.toml");
    let mut settings_manager = SettingsManager::new(settings_file.to_string_lossy().to_string())?;
    info!("✓ Settings Manager initialized");
    
    // Initialize Private Mode Manager
    let mut private_mode = PrivateModeManager::new();
    info!("✓ Private Mode Manager initialized");
    
    // Initialize Security Module
    let security = security::SecurityManager::new().await?;
    info!("✓ Security Manager initialized");
    
    // Initialize UI
    let ui = ui::VantisUI::new(kernel.clone()).await?;
    info!("✓ VantisUI initialized");
    
    // Main event loop
    info!("Starting main event loop...");
    ui.run().await?;
    
    // Cleanup
    info!("Cleaning up...");
    settings_manager.save()?;
    
    Ok(())
}