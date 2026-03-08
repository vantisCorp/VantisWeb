//! Build script for VantisWeb
//! Generates Windows resources and metadata

#[cfg(windows)]
fn main() {
    use std::env;
    use std::fs;
    use std::path::Path;
    
    let out_dir = env::var("OUT_DIR").unwrap();
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    
    // Check if icon exists
    let icon_path = Path::new(&manifest_dir).join("assets/icons/icon.ico");
    let icon_arg = if icon_path.exists() {
        format!("--icon {}", icon_path.display())
    } else {
        String::new()
    };
    
    // Embed Windows manifest for proper DPI awareness and admin rights
    let manifest_content = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <assemblyIdentity
    version="1.5.0.0"
    processorArchitecture="amd64"
    name="VantisCorp.VantisWeb"
    type="win32"/>
  <description>VantisWeb Browser</description>
  
  <!-- Windows Vista+ compatibility -->
  <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
      <!-- Windows 10 and Windows 11 -->
      <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}"/>
      <!-- Windows 8.1 -->
      <supportedOS Id="{1f676c76-80e1-4239-95bb-83d0f6d0da78}"/>
      <!-- Windows 8 -->
      <supportedOS Id="{4a2f28e3-53b9-4441-ba9c-d69d4a4a6e38}"/>
      <!-- Windows 7 -->
      <supportedOS Id="{35138b9a-5d96-4fbd-8e2d-a2440225f93a}"/>
    </application>
  </compatibility>
  
  <!-- DPI awareness -->
  <asmv3:application xmlns:asmv3="urn:schemas-microsoft-com:asm.v3">
    <asmv3:windowsSettings xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">
      <dpiAware>true/PM</dpiAware>
      <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2</dpiAwareness>
    </asmv3:windowsSettings>
  </asmv3:application>
  
  <!-- Request admin rights for installer functionality -->
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="asInvoker" uiAccess="false"/>
      </requestedPrivileges>
    </security>
  </trustInfo>
</assembly>"#;
    
    let manifest_path = Path::new(&out_dir).join("vantisweb.manifest");
    fs::write(&manifest_path, manifest_content).unwrap();
    
    // Use winres to embed resources
    let mut res = winres::WindowsResource::new();
    
    res.set("ProductName", "VantisWeb Browser")
       .set("FileDescription", "VantisWeb - Next-generation Web Browser")
       .set("LegalCopyright", "Copyright © 2024-2026 Vantis Corp")
       .set("OriginalFilename", "VantisWeb.exe")
       .set("CompanyName", "Vantis Corp")
       .set("InternalName", "VantisWeb")
       .set("ProductVersion", "1.5.0")
       .set("FileVersion", "1.5.0");
    
    if icon_path.exists() {
        res.set_icon_with_id(&icon_path.display().to_string(), "MAINICON");
    }
    
    res.set_manifest(manifest_content);
    
    if let Err(e) = res.compile() {
        eprintln!("cargo:warning=Failed to compile Windows resources: {}", e);
    }
    
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets/icons/icon.ico");
}

#[cfg(not(windows))]
fn main() {
    // No build script needed for non-Windows platforms
    println!("cargo:rerun-if-changed=build.rs");
}