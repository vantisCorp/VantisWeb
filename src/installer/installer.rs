/// # Installer Builder Module
/// 
/// Provides functionality to build platform-specific installer packages.

use std::path::Path;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{BuildConfig, Platform, Architecture, PackageFormat, Result, InstallerError};

/// Installer builder
pub struct InstallerBuilder {
    /// Configuration
    config: BuildConfig,
    /// Build progress
    progress: BuildProgress,
}

/// Build progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildProgress {
    /// Current step
    pub current_step: String,
    /// Total steps
    pub total_steps: u32,
    /// Current step number
    pub current_step_num: u32,
    /// Completion percentage
    pub percentage: f32,
    /// Is complete
    pub is_complete: bool,
    /// Error message if failed
    pub error: Option<String>,
}

impl InstallerBuilder {
    /// Create a new installer builder
    pub fn new(config: BuildConfig) -> Self {
        Self {
            config,
            progress: BuildProgress {
                current_step: "Initializing".to_string(),
                total_steps: 8,
                current_step_num: 0,
                percentage: 0.0,
                is_complete: false,
                error: None,
            },
        }
    }

    /// Build the installer
    pub async fn build(&mut self) -> Result<String> {
        // Step 1: Validate configuration
        self.update_progress("Validating configuration", 1).await?;
        self.validate_config()?;

        // Step 2: Prepare build environment
        self.update_progress("Preparing build environment", 2).await?;
        self.prepare_environment().await?;

        // Step 3: Copy application files
        self.update_progress("Copying application files", 3).await?;
        self.copy_files().await?;

        // Step 4: Process resources
        self.update_progress("Processing resources", 4).await?;
        self.process_resources().await?;

        // Step 5: Generate installer script/config
        self.update_progress("Generating installer configuration", 5).await?;
        let script = self.generate_installer_script().await?;

        // Step 6: Build installer
        self.update_progress("Building installer package", 6).await?;
        let package_path = self.build_package(&script).await?;

        // Step 7: Sign package (if enabled)
        if self.config.sign_package {
            self.update_progress("Signing package", 7).await?;
            self.sign_package(&package_path).await?;
        } else {
            self.progress.current_step_num = 7;
        }

        // Step 8: Finalize
        self.update_progress("Finalizing", 8).await?;
        self.progress.is_complete = true;

        Ok(package_path)
    }

    /// Get current build progress
    pub fn progress(&self) -> &BuildProgress {
        &self.progress
    }

    /// Update build progress
    async fn update_progress(&mut self, step: &str, step_num: u32) -> Result<()> {
        self.progress.current_step = step.to_string();
        self.progress.current_step_num = step_num;
        self.progress.percentage = (step_num as f32 / self.progress.total_steps as f32) * 100.0;
        Ok(())
    }

    /// Validate configuration
    fn validate_config(&self) -> Result<()> {
        // Check source directory exists
        if !Path::new(&self.config.source_dir).exists() {
            return Err(InstallerError::ConfigError(
                format!("Source directory not found: {}", self.config.source_dir)
            ));
        }

        // Validate platform/package format combination
        match (&self.config.platform, &self.config.package_format) {
            (Platform::Windows, PackageFormat::Msi) |
            (Platform::Windows, PackageFormat::Nsis) => {}
            (Platform::MacOS, PackageFormat::Dmg) |
            (Platform::MacOS, PackageFormat::Pkg) => {}
            (Platform::Linux, PackageFormat::Deb) |
            (Platform::Linux, PackageFormat::Rpm) |
            (Platform::Linux, PackageFormat::AppImage) => {}
            _ => return Err(InstallerError::ConfigError(
                "Invalid platform/package format combination".to_string()
            )),
        }

        Ok(())
    }

    /// Prepare build environment
    async fn prepare_environment(&self) -> Result<()> {
        // Create output directory
        let output_path = Path::new(&self.config.output_dir);
        if output_path.exists() {
            // Clean previous build
            std::fs::remove_dir_all(output_path)
                .map_err(|e| InstallerError::BuildError(format!("Failed to clean output: {}", e)))?;
        }
        
        std::fs::create_dir_all(output_path)
            .map_err(|e| InstallerError::BuildError(format!("Failed to create output: {}", e)))?;

        // Create build staging directory
        let staging_path = output_path.join("staging");
        std::fs::create_dir_all(&staging_path)
            .map_err(|e| InstallerError::BuildError(format!("Failed to create staging: {}", e)))?;

        Ok(())
    }

    /// Copy application files to staging
    async fn copy_files(&self) -> Result<()> {
        let staging = Path::new(&self.config.output_dir).join("staging");
        let source = Path::new(&self.config.source_dir);

        // Copy all files recursively
        self.copy_dir_recursive(source, &staging)
            .map_err(|e| InstallerError::BuildError(format!("Failed to copy files: {}", e)))?;

        Ok(())
    }

    /// Copy directory recursively
    fn copy_dir_recursive(&self, src: &Path, dst: &Path) -> std::io::Result<()> {
        std::fs::create_dir_all(dst)?;
        
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            
            if ty.is_dir() {
                self.copy_dir_recursive(&entry.path(), &dst.join(entry.file_name()))?;
            } else {
                std::fs::copy(entry.path(), dst.join(entry.file_name()))?;
            }
        }
        
        Ok(())
    }

    /// Process resources (icons, etc.)
    async fn process_resources(&self) -> Result<()> {
        // Process icon if specified
        if let Some(ref icon_path) = self.config.icon_path {
            let staging = Path::new(&self.config.output_dir).join("staging");
            let icon_dest = staging.join("resources").join("icon.png");
            
            std::fs::create_dir_all(icon_dest.parent().unwrap())
                .map_err(|e| InstallerError::BuildError(e.to_string()))?;
            
            std::fs::copy(icon_path, &icon_dest)
                .map_err(|e| InstallerError::BuildError(e.to_string()))?;

            // Generate platform-specific icons
            self.generate_platform_icons(&icon_dest).await?;
        }

        Ok(())
    }

    /// Generate platform-specific icons
    async fn generate_platform_icons(&self, source_icon: &Path) -> Result<()> {
        match self.config.platform {
            Platform::Windows => {
                // Generate .ico file
                // In production, would use image processing library
            }
            Platform::MacOS => {
                // Generate .icns file
                // In production, would use sips or iconutil
            }
            Platform::Linux => {
                // Generate PNG at various sizes
                // In production, would resize icon
            }
        }
        Ok(())
    }

    /// Generate installer script
    async fn generate_installer_script(&self) -> Result<String> {
        match (&self.config.platform, &self.config.package_format) {
            (Platform::Windows, PackageFormat::Nsis) => self.generate_nsis_script(),
            (Platform::Windows, PackageFormat::Msi) => self.generate_msi_config(),
            (Platform::MacOS, PackageFormat::Dmg) => self.generate_dmg_config(),
            (Platform::MacOS, PackageFormat::Pkg) => self.generate_pkg_config(),
            (Platform::Linux, PackageFormat::Deb) => self.generate_deb_control(),
            (Platform::Linux, PackageFormat::Rpm) => self.generate_rpm_spec(),
            (Platform::Linux, PackageFormat::AppImage) => self.generate_appimage_config(),
            _ => Err(InstallerError::BuildError("Unsupported combination".to_string())),
        }
    }

    /// Generate NSIS installer script for Windows
    fn generate_nsis_script(&self) -> Result<String> {
        Ok(format!(r##"
!include "MUI2.nsh"
!include "FileFunc.nsh"

Name "{}"
OutFile "..\{}\{}_setup.exe"
InstallDir "$PROGRAM64\{}"
InstallDirRegKey HKLM "Software\{} "Install_Dir"
RequestExecutionLevel admin

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "..\staging\LICENSE"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"

Section "Install"
    SetOutPath $INSTDIR
    File /r "..\staging\*.*"
    
    WriteRegStr HKLM "Software\{}" "Install_Dir" "$INSTDIR"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\{}" "DisplayName" "{}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\{}" "UninstallString" '"$INSTDIR\uninstall.exe"'
    
    CreateDirectory "$SMPROGRAMS\{}"
    CreateShortCut "$SMPROGRAMS\{}\{}.lnk" "$INSTDIR\{}.exe"
    CreateShortCut "$DESKTOP\{}.lnk" "$INSTDIR\{}.exe"
    
    WriteUninstaller "$INSTDIR\uninstall.exe"
SectionEnd

Section "Uninstall"
    Delete "$INSTDIR\*.*"
    RMDir /r "$INSTDIR"
    Delete "$SMPROGRAMS\{}\*.lnk"
    RMDir "$SMPROGRAMS\{}"
    Delete "$DESKTOP\{}.lnk"
    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\{}"
    DeleteRegKey HKLM "Software\{}"
SectionEnd
"##,
            self.config.app_name,
            self.config.output_dir,
            self.config.app_name.to_lowercase().replace(' ', "_"),
            self.config.app_name,
            self.config.app_name,
            self.config.app_name,
            self.config.app_name,
            self.config.app_name,
            self.config.app_name,
            self.config.app_name,
            self.config.app_name,
            self.config.app_name,
            self.config.app_name,
            self.config.app_name,
            self.config.app_name,
            self.config.app_name,
            self.config.app_name,
            self.config.app_name,
            self.config.app_name,
        ))
    }

    /// Generate MSI configuration
    fn generate_msi_config(&self) -> Result<String> {
        // In production, would generate WiX XML configuration
        Ok(format!("MSI configuration for {} would be generated here", self.config.app_name))
    }

    /// Generate DMG configuration
    fn generate_dmg_config(&self) -> Result<String> {
        Ok(format!("DMG configuration for {} would be generated here", self.config.app_name))
    }

    /// Generate PKG configuration
    fn generate_pkg_config(&self) -> Result<String> {
        Ok(format!("PKG configuration for {} would be generated here", self.config.app_name))
    }

    /// Generate DEB control file
    fn generate_deb_control(&self) -> Result<String> {
        Ok(format!(r##"Package: {}
Version: {}
Section: web
Priority: optional
Architecture: {}
Maintainer: VantisWeb Team <team@vantisweb.com>
Description: {}
 Next-generation web browser with Liquid Core Architecture
Homepage: https://vantisweb.com
"##,
            self.config.app_name.to_lowercase().replace(' ', "-"),
            self.config.app_version,
            match self.config.architecture {
                Architecture::X86_64 => "amd64",
                Architecture::Aarch64 => "arm64",
                Architecture::Arm => "armhf",
            },
            self.config.app_name,
        ))
    }

    /// Generate RPM spec file
    fn generate_rpm_spec(&self) -> Result<String> {
        Ok(format!(r##"Name: {}
Version: {}
Release: 1
Summary: Next-generation web browser
License: MIT
URL: https://vantisweb.com

%description
Next-generation web browser with Liquid Core Architecture

%install
mkdir -p $RPM_BUILD_ROOT/usr/bin
mkdir -p $RPM_BUILD_ROOT/usr/share/{}
cp -r staging/* $RPM_BUILD_ROOT/usr/share/{}/

%files
/usr/bin/{}
/usr/share/{}

%post
ln -sf /usr/share/{}/{} /usr/bin/{}

%postun
rm -f /usr/bin/{}
"##,
            self.config.app_name.to_lowercase(),
            self.config.app_version,
            self.config.app_name.to_lowercase(),
            self.config.app_name.to_lowercase(),
            self.config.app_name.to_lowercase(),
            self.config.app_name.to_lowercase(),
            self.config.app_name.to_lowercase(),
            self.config.app_name.to_lowercase(),
            self.config.app_name.to_lowercase(),
            self.config.app_name.to_lowercase(),
        ))
    }

    /// Generate AppImage configuration
    fn generate_appimage_config(&self) -> Result<String> {
        Ok(format!(r##"#!/bin/bash
APPDIR="$(dirname "$(readlink -f "$0")")"
export APPIMAGE_EXTRACT_AND_RUN=1
exec "$APPDIR/usr/bin/{}" "$@"
"##, self.config.app_name.to_lowercase()))
    }

    /// Build the actual package
    async fn build_package(&self, script: &str) -> Result<String> {
        let output_dir = Path::new(&self.config.output_dir);
        
        // Write installer script
        let script_path = output_dir.join("installer_script");
        std::fs::write(&script_path, script)
            .map_err(|e| InstallerError::BuildError(e.to_string()))?;

        // Build package based on format
        let package_name = match self.config.package_format {
            PackageFormat::Nsis => format!("{}_setup.exe", self.config.app_name.to_lowercase().replace(' ', "_")),
            PackageFormat::Msi => format!("{}-{}.msi", self.config.app_name.to_lowercase(), self.config.app_version),
            PackageFormat::Dmg => format!("{}-{}.dmg", self.config.app_name.to_lowercase(), self.config.app_version),
            PackageFormat::Pkg => format!("{}-{}.pkg", self.config.app_name.to_lowercase(), self.config.app_version),
            PackageFormat::Deb => format!("{}_{}_{}.deb", 
                self.config.app_name.to_lowercase().replace(' ', "-"),
                self.config.app_version,
                match self.config.architecture {
                    Architecture::X86_64 => "amd64",
                    Architecture::Aarch64 => "arm64",
                    Architecture::Arm => "armhf",
                }
            ),
            PackageFormat::Rpm => format!("{}-{}-1.{}.rpm", 
                self.config.app_name.to_lowercase(),
                self.config.app_version,
                match self.config.architecture {
                    Architecture::X86_64 => "x86_64",
                    Architecture::Aarch64 => "aarch64",
                    Architecture::Arm => "armv7hl",
                }
            ),
            PackageFormat::AppImage => format!("{}-{}.AppImage", 
                self.config.app_name.to_lowercase(),
                self.config.app_version
            ),
        };

        Ok(output_dir.join(&package_name).to_string_lossy().to_string())
    }

    /// Sign the package
    async fn sign_package(&self, package_path: &str) -> Result<()> {
        // In production, this would call platform-specific signing tools
        // Windows: signtool.exe
        // macOS: codesign
        // Linux: GPG
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_config() {
        let config = BuildConfig {
            app_name: "VantisWeb".to_string(),
            app_version: Version::new(1, 0, 0),
            platform: Platform::Windows,
            architecture: Architecture::X86_64,
            package_format: PackageFormat::Nsis,
            output_dir: "/tmp/output".to_string(),
            source_dir: "/tmp/source".to_string(),
            icon_path: None,
            sign_package: false,
            compress: true,
            silent_install: true,
        };

        let builder = InstallerBuilder::new(config);
        assert_eq!(builder.progress().total_steps, 8);
    }
}