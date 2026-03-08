//! File Associations Module
//! 
//! Manages file associations and protocol handlers on Windows.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Supported file types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FileType {
    Html,
    Htm,
    Xhtml,
    Shtml,
    Pdf,
    Mhtml,
    Webp,
    Svg,
}

impl FileType {
    /// Get file extension
    pub fn extension(&self) -> &'static str {
        match self {
            FileType::Html => ".html",
            FileType::Htm => ".htm",
            FileType::Xhtml => ".xhtml",
            FileType::Shtml => ".shtml",
            FileType::Pdf => ".pdf",
            FileType::Mhtml => ".mhtml",
            FileType::Webp => ".webp",
            FileType::Svg => ".svg",
        }
    }
    
    /// Get MIME type
    pub fn mime_type(&self) -> &'static str {
        match self {
            FileType::Html | FileType::Htm | FileType::Shtml => "text/html",
            FileType::Xhtml => "application/xhtml+xml",
            FileType::Pdf => "application/pdf",
            FileType::Mhtml => "message/rfc822",
            FileType::Webp => "image/webp",
            FileType::Svg => "image/svg+xml",
        }
    }
    
    /// Get description
    pub fn description(&self) -> &'static str {
        match self {
            FileType::Html => "HTML Document",
            FileType::Htm => "HTM Document",
            FileType::Xhtml => "XHTML Document",
            FileType::Shtml => "SHTML Document",
            FileType::Pdf => "PDF Document",
            FileType::Mhtml => "MHTML Document",
            FileType::Webp => "WebP Image",
            FileType::Svg => "SVG Image",
        }
    }
    
    /// Get icon index
    pub fn icon_index(&self) -> i32 {
        match self {
            FileType::Html | FileType::Htm | FileType::Xhtml | FileType::Shtml => 0,
            FileType::Pdf => 1,
            FileType::Mhtml => 2,
            FileType::Webp | FileType::Svg => 3,
        }
    }
}

/// Protocol handler types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Protocol {
    Http,
    Https,
    Ftp,
    Mailto,
    Webcal,
    Magnet,
}

impl Protocol {
    /// Get protocol scheme
    pub fn scheme(&self) -> &'static str {
        match self {
            Protocol::Http => "http",
            Protocol::Https => "https",
            Protocol::Ftp => "ftp",
            Protocol::Mailto => "mailto",
            Protocol::Webcal => "webcal",
            Protocol::Magnet => "magnet",
        }
    }
    
    /// Get description
    pub fn description(&self) -> &'static str {
        match self {
            Protocol::Http => "URL:HTTP Protocol",
            Protocol::Https => "URL:HTTPS Protocol",
            Protocol::Ftp => "URL:FTP Protocol",
            Protocol::Mailto => "URL:Mailto Protocol",
            Protocol::Webcal => "URL:Webcal Protocol",
            Protocol::Magnet => "URL:Magnet Protocol",
        }
    }
}

/// Association result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssociationResult {
    /// File type or protocol
    pub target: String,
    /// Was successful
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// File associations manager
pub struct FileAssociations {
    /// Registered file types
    file_types: HashMap<FileType, bool>,
    /// Registered protocols
    protocols: HashMap<Protocol, bool>,
    /// Executable path
    exe_path: PathBuf,
    /// App name
    app_name: String,
    /// App ID for Windows
    app_id: String,
}

impl FileAssociations {
    /// Create a new file associations manager
    pub fn new(exe_path: PathBuf) -> Self {
        Self {
            file_types: HashMap::new(),
            protocols: HashMap::new(),
            exe_path,
            app_name: "VantisWeb".to_string(),
            app_id: "VantisCorp.VantisWeb".to_string(),
        }
    }
    
    /// Register file types
    #[cfg(target_os = "windows")]
    pub fn register(types: &[String]) -> Result<Vec<AssociationResult>, String> {
        let mut results = Vec::new();
        
        for type_str in types {
            let file_type = match type_str.as_str() {
                ".html" => FileType::Html,
                ".htm" => FileType::Htm,
                ".xhtml" => FileType::Xhtml,
                ".shtml" => FileType::Shtml,
                ".pdf" => FileType::Pdf,
                ".mhtml" => FileType::Mhtml,
                ".webp" => FileType::Webp,
                ".svg" => FileType::Svg,
                _ => {
                    results.push(AssociationResult {
                        target: type_str.clone(),
                        success: false,
                        error: Some("Unknown file type".to_string()),
                    });
                    continue;
                }
            };
            
            // In a real implementation, this would use winreg to modify registry
            results.push(AssociationResult {
                target: type_str.clone(),
                success: true,
                error: None,
            });
        }
        
        Ok(results)
    }
    
    /// Register protocol handlers
    #[cfg(target_os = "windows")]
    pub fn register_protocols(protocols: &[String]) -> Result<Vec<AssociationResult>, String> {
        let mut results = Vec::new();
        
        for protocol_str in protocols {
            let protocol = match protocol_str.to_lowercase().as_str() {
                "http" => Protocol::Http,
                "https" => Protocol::Https,
                "ftp" => Protocol::Ftp,
                "mailto" => Protocol::Mailto,
                "webcal" => Protocol::Webcal,
                "magnet" => Protocol::Magnet,
                _ => {
                    results.push(AssociationResult {
                        target: protocol_str.clone(),
                        success: false,
                        error: Some("Unknown protocol".to_string()),
                    });
                    continue;
                }
            };
            
            results.push(AssociationResult {
                target: protocol_str.clone(),
                success: true,
                error: None,
            });
        }
        
        Ok(results)
    }
    
    /// Unregister all associations
    #[cfg(target_os = "windows")]
    pub fn unregister_all() -> Result<(), String> {
        // Remove all registry entries
        Ok(())
    }
    
    /// Check if file type is registered
    pub fn is_registered(&self, file_type: &FileType) -> bool {
        self.file_types.get(file_type).copied().unwrap_or(false)
    }
    
    /// Check if protocol is registered
    pub fn is_protocol_registered(&self, protocol: &Protocol) -> bool {
        self.protocols.get(protocol).copied().unwrap_or(false)
    }
    
    /// Get all registered file types
    pub fn registered_file_types(&self) -> Vec<&FileType> {
        self.file_types.iter()
            .filter(|(_, &registered)| registered)
            .map(|(t, _)| t)
            .collect()
    }
    
    /// Get all registered protocols
    pub fn registered_protocols(&self) -> Vec<&Protocol> {
        self.protocols.iter()
            .filter(|(_, &registered)| registered)
            .map(|(p, _)| p)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_file_type_properties() {
        assert_eq!(FileType::Html.extension(), ".html");
        assert_eq!(FileType::Html.mime_type(), "text/html");
        assert_eq!(FileType::Pdf.description(), "PDF Document");
    }
    
    #[test]
    fn test_protocol_properties() {
        assert_eq!(Protocol::Https.scheme(), "https");
        assert!(!Protocol::Http.description().is_empty());
    }
    
    #[test]
    fn test_file_associations_creation() {
        let fa = FileAssociations::new(PathBuf::from("C:\\test\\VantisWeb.exe"));
        assert_eq!(fa.app_name, "VantisWeb");
    }
}