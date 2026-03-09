use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use super::Bookmark;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub total_processed: usize,
    pub imported: usize,
    pub skipped: usize,
    pub failed: usize,
    pub errors: Vec<ImportError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportError {
    pub line_number: Option<usize>,
    pub message: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub exported_count: usize,
    pub file_path: String,
    pub format: ExportFormat,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Html,
    Csv,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetscapeBookmark {
    pub url: String,
    pub title: String,
    pub add_date: i64,
    pub last_modified: Option<i64>,
    pub tags: Vec<String>,
    pub icon: Option<String>,
    pub folder: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChromeBookmark {
    pub url: String,
    pub name: String,
    pub date_added: Option<String>,
    pub id: Option<String>,
    pub meta_info: Option<ChromeMetaInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChromeMetaInfo {
    pub last_visited_desktop: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirefoxBookmark {
    pub uri: String,
    pub title: String,
    pub tags: Option<Vec<String>>,
    pub dateAdded: Option<i64>,
    pub lastModified: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafariBookmark {
    pub URLString: String,
    pub URIDictionary: SafariURIDictionary,
    pub Children: Option<Vec<SafariBookmark>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafariURIDictionary {
    pub title: String,
}

#[derive(Debug, Clone)]
pub struct DuplicateDetection {
    mode: DuplicateMode,
    threshold: f64,
}

#[derive(Debug, Clone, Copy)]
pub enum DuplicateMode {
    ExactUrl,
    SimilarUrl,
    TitleAndUrl,
    None,
}

pub struct BookmarkImporter {
    existing_urls: Arc<RwLock<HashMap<String, Uuid>>>,
    duplicate_detection: DuplicateDetection,
}

impl BookmarkImporter {
    pub fn new(duplicate_mode: DuplicateMode) -> Self {
        Self {
            existing_urls: Arc::new(RwLock::new(HashMap::new())),
            duplicate_detection: DuplicateDetection {
                mode: duplicate_mode,
                threshold: 0.9,
            },
        }
    }

    pub async fn load_existing_urls(&self, bookmarks: &[Bookmark]) {
        let mut urls = self.existing_urls.write().await;
        for bookmark in bookmarks {
            urls.insert(bookmark.url.clone(), bookmark.id);
        }
    }

    pub async fn import_from_json(&self, json: &str) -> ImportResult {
        let mut result = ImportResult {
            total_processed: 0,
            imported: 0,
            skipped: 0,
            failed: 0,
            errors: Vec::new(),
        };

        let bookmarks: Vec<Bookmark> = match serde_json::from_str(json) {
            Ok(b) => b,
            Err(e) => {
                result.failed = 1;
                result.errors.push(ImportError {
                    line_number: None,
                    message: format!("Failed to parse JSON: {}", e),
                    url: None,
                });
                return result;
            }
        };

        result.total_processed = bookmarks.len();
        
        for bookmark in bookmarks {
            match self.import_bookmark(bookmark).await {
                Ok(imported) => {
                    if imported {
                        result.imported += 1;
                    } else {
                        result.skipped += 1;
                    }
                }
                Err(e) => {
                    result.failed += 1;
                    result.errors.push(ImportError {
                        line_number: None,
                        message: e,
                        url: None,
                    });
                }
            }
        }

        result
    }

    pub async fn import_from_html(&self, html: &str) -> ImportResult {
        let mut result = ImportResult {
            total_processed: 0,
            imported: 0,
            skipped: 0,
            failed: 0,
            errors: Vec::new(),
        };

        let bookmarks = self.parse_netscape_html(html);
        result.total_processed = bookmarks.len();

        for bookmark in bookmarks {
            match self.import_bookmark(bookmark).await {
                Ok(imported) => {
                    if imported {
                        result.imported += 1;
                    } else {
                        result.skipped += 1;
                    }
                }
                Err(e) => {
                    result.failed += 1;
                    result.errors.push(ImportError {
                        line_number: None,
                        message: e,
                        url: Some(bookmark.url.clone()),
                    });
                }
            }
        }

        result
    }

    pub async fn import_from_csv(&self, csv: &str) -> ImportResult {
        let mut result = ImportResult {
            total_processed: 0,
            imported: 0,
            skipped: 0,
            failed: 0,
            errors: Vec::new(),
        };

        let mut rdr = csv::ReaderBuilder::new().from_reader(csv.as_bytes());
        let headers = rdr.headers().ok().map(|h| h.clone());

        for (line_num, record_result) in rdr.records().enumerate() {
            result.total_processed += 1;
            
            let record = match record_result {
                Ok(r) => r,
                Err(e) => {
                    result.failed += 1;
                    result.errors.push(ImportError {
                        line_number: Some(line_num + 2),
                        message: format!("Failed to parse CSV record: {}", e),
                        url: None,
                    });
                    continue;
                }
            };

            let bookmark = match self.parse_csv_record(record, &headers) {
                Ok(b) => b,
                Err(e) => {
                    result.failed += 1;
                    result.errors.push(ImportError {
                        line_number: Some(line_num + 2),
                        message: e,
                        url: None,
                    });
                    continue;
                }
            };

            match self.import_bookmark(bookmark).await {
                Ok(imported) => {
                    if imported {
                        result.imported += 1;
                    } else {
                        result.skipped += 1;
                    }
                }
                Err(e) => {
                    result.failed += 1;
                    result.errors.push(ImportError {
                        line_number: Some(line_num + 2),
                        message: e,
                        url: None,
                    });
                }
            }
        }

        result
    }

    fn parse_netscape_html(&self, html: &str) -> Vec<Bookmark> {
        let mut bookmarks = Vec::new();
        
        // Simple HTML parser for Netscape bookmark format
        for line in html.lines() {
            if line.contains("<DT><A") || line.contains("<DT><H3") {
                if let Some(start) = line.find("HREF=&quot;") {
                    if let Some(end) = line[start + 6..].find("&quot;") {
                        let url = line[start + 6..start + 6 + end].to_string();
                        
                        let title = if let Some(title_start) = line.find(">") {
                            if let Some(title_end) = line[title_start + 1..].find("<") {
                                line[title_start + 1..title_start + 1 + title_end].to_string()
                            } else {
                                url.clone()
                            }
                        } else {
                            url.clone()
                        };
                        
                        let add_date = if let Some(date_start) = line.find("ADD_DATE=&quot;") {
                            if let Some(date_end) = line[date_start + 10..].find("&quot;") {
                                line[date_start + 10..date_start + 10 + date_end]
                                    .parse()
                                    .unwrap_or(0)
                            } else {
                                0
                            }
                        } else {
                            0
                        };
                        
                        let tags = if let Some(tags_start) = line.find("TAGS=&quot;") {
                            if let Some(tags_end) = line[tags_start + 6..].find("&quot;") {
                                line[tags_start + 6..tags_start + 6 + tags_end]
                                    .split(',')
                                    .filter(|s| !s.is_empty())
                                    .map(|s| s.to_string())
                                    .collect()
                            } else {
                                Vec::new()
                            }
                        } else {
                            Vec::new()
                        };
                        
                        let created_at = if add_date > 0 {
                            DateTime::from_timestamp(add_date, 0).unwrap_or_else(Utc::now)
                        } else {
                            Utc::now()
                        };
                        
                        bookmarks.push(Bookmark {
                            id: Uuid::new_v4(),
                            url,
                            title,
                            description: None,
                            favicon: None,
                            folder_id: None,
                            tags,
                            is_favorite: false,
                            is_read_later: false,
                            visit_count: 0,
                            last_accessed: None,
                            created_at,
                            updated_at: Utc::now(),
                        });
                    }
                }
            }
        }
        
        bookmarks
    }

    fn parse_csv_record(&self, record: csv::StringRecord, headers: &Option<csv::StringRecord>) -> Result<Bookmark, String> {
        let url = record.get(0).ok_or("Missing URL column")?.to_string();
        let title = record.get(1).unwrap_or(&url).to_string();
        let description = record.get(2).map(|s| s.to_string());
        let tags = record.get(3)
            .map(|s| s.split(',').filter(|t| !t.is_empty()).map(|t| t.trim().to_string()).collect())
            .unwrap_or_default();
        
        Ok(Bookmark {
            id: Uuid::new_v4(),
            url,
            title,
            description,
            favicon: None,
            folder_id: None,
            tags,
            is_favorite: false,
            is_read_later: false,
            visit_count: 0,
            last_accessed: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    async fn import_bookmark(&self, bookmark: Bookmark) -> Result<bool, String> {
        let urls = self.existing_urls.read().await;
        
        match self.duplicate_detection.mode {
            DuplicateMode::ExactUrl => {
                if urls.contains_key(&bookmark.url) {
                    return Ok(false); // Skip duplicate
                }
            }
            DuplicateMode::SimilarUrl => {
                for existing_url in urls.keys() {
                    if self.url_similarity(&bookmark.url, existing_url) > self.duplicate_detection.threshold {
                        return Ok(false);
                    }
                }
            }
            DuplicateMode::TitleAndUrl => {
                for (existing_url, _) in urls.iter() {
                    if self.url_similarity(&bookmark.url, existing_url) > self.duplicate_detection.threshold {
                        return Ok(false);
                    }
                }
            }
            DuplicateMode::None => {}
        }
        
        // Mark as imported (would normally add to storage)
        let mut urls = self.existing_urls.write().await;
        urls.insert(bookmark.url.clone(), bookmark.id);
        
        Ok(true)
    }

    fn url_similarity(&self, url1: &str, url2: &str) -> f64 {
        // Simple similarity check
        if url1 == url2 {
            return 1.0;
        }
        
        let url1_lower = url1.to_lowercase();
        let url2_lower = url2.to_lowercase();
        
        if url1_lower == url2_lower {
            return 0.95;
        }
        
        // Check domain
        let domain1 = url1_lower.split('/').next().unwrap_or("");
        let domain2 = url2_lower.split('/').next().unwrap_or("");
        
        if domain1 == domain2 {
            return 0.7;
        }
        
        0.0
    }
}

pub struct BookmarkExporter;

impl BookmarkExporter {
    pub async fn export_to_json(bookmarks: &[Bookmark]) -> Result<ExportResult, String> {
        let start = std::time::Instant::now();
        
        let json = serde_json::to_string_pretty(bookmarks)
            .map_err(|e| format!("Failed to serialize bookmarks: {}", e))?;
        
        let file_path = format!("bookmarks_export_{}.json", Utc::now().timestamp());
        tokio::fs::write(&file_path, json).await
            .map_err(|e| format!("Failed to write export file: {}", e))?;
        
        Ok(ExportResult {
            exported_count: bookmarks.len(),
            file_path,
            format: ExportFormat::Json,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    pub async fn export_to_html(bookmarks: &[Bookmark]) -> Result<ExportResult, String> {
        let start = std::time::Instant::now();
        
        let mut html = String::from(
            r##"<!DOCTYPE NETSCAPE-Bookmark-file-1>
<!-- This is an automatically generated file.
     It will be read and overwritten.
     DO NOT EDIT! -->
<META HTTP-EQUIV="Content-Type" CONTENT="text/html; charset=UTF-8">
<TITLE>Bookmarks</TITLE>
<H1>Bookmarks</H1>
<DL><p>
"#
        );
        
        for bookmark in bookmarks {
            let tags_csv = bookmark.tags.join(",");
            let add_date = bookmark.created_at.timestamp();
            let desc_attr = if let Some(ref desc) = bookmark.description {
                format!("SHORTCUTURL=&quot;{}&quot; LAST_MODIFIED=&quot;{}&quot;", desc, bookmark.updated_at.timestamp())
            } else {
                format!("LAST_MODIFIED=&quot;{}&quot;", bookmark.updated_at.timestamp())
            };
            
            html.push_str(&format!(
                r##"    <DT><A HREF="{}" ADD_DATE="{}" {} TAGS="{}">{}</A>"##,
                bookmark.url, add_date, desc_attr, tags_csv, bookmark.title
            ));
        }
        
        html.push_str("\n</DL><p>");
        
        let file_path = format!("bookmarks_export_{}.html", Utc::now().timestamp());
        tokio::fs::write(&file_path, html).await
            .map_err(|e| format!("Failed to write export file: {}", e))?;
        
        Ok(ExportResult {
            exported_count: bookmarks.len(),
            file_path,
            format: ExportFormat::Html,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    pub async fn export_to_csv(bookmarks: &[Bookmark]) -> Result<ExportResult, String> {
        let start = std::time::Instant::now();
        
        let mut csv = String::from("URL,Title,Description,Tags,IsFavorite,IsReadLater,CreatedAt\n");
        
        for bookmark in bookmarks {
            let tags = bookmark.tags.join(";");
            let description = bookmark.description.as_deref().unwrap_or("");
            let created_at = bookmark.created_at.to_rfc3339();
            
            csv.push_str(&format!(
                "{},{},{},{},{},{},{}\n",
                bookmark.url,
                bookmark.title,
                description,
                tags,
                bookmark.is_favorite,
                bookmark.is_read_later,
                created_at
            ));
        }
        
        let file_path = format!("bookmarks_export_{}.csv", Utc::now().timestamp());
        tokio::fs::write(&file_path, csv).await
            .map_err(|e| format!("Failed to write export file: {}", e))?;
        
        Ok(ExportResult {
            exported_count: bookmarks.len(),
            file_path,
            format: ExportFormat::Csv,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_import_from_json() {
        let importer = BookmarkImporter::new(DuplicateMode::ExactUrl);
        
        let json = r##"[{
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "url": "https://example.com",
            "title": "Example",
            "description": null,
            "favicon": null,
            "folder_id": null,
            "tags": [],
            "is_favorite": false,
            "is_read_later": false,
            "visit_count": 0,
            "last_accessed": null,
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z"
        }]"##;
        
        let result = importer.import_from_json(json).await;
        assert_eq!(result.total_processed, 1);
        assert_eq!(result.imported, 1);
    }

    #[tokio::test]
    async fn test_duplicate_detection() {
        let importer = BookmarkImporter::new(DuplicateMode::ExactUrl);
        
        let bookmark = Bookmark {
            id: Uuid::new_v4(),
            url: "https://example.com".to_string(),
            title: "Example".to_string(),
            description: None,
            favicon: None,
            folder_id: None,
            tags: Vec::new(),
            is_favorite: false,
            is_read_later: false,
            visit_count: 0,
            last_accessed: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        importer.load_existing_urls(&[bookmark.clone()]).await;
        
        let result = importer.import_bookmark(bookmark).await;
        assert_eq!(result.unwrap(), false); // Should be skipped as duplicate
    }

    #[tokio::test]
    async fn test_export_to_json() {
        let bookmarks = vec![Bookmark {
            id: Uuid::new_v4(),
            url: "https://example.com".to_string(),
            title: "Example".to_string(),
            description: None,
            favicon: None,
            folder_id: None,
            tags: Vec::new(),
            is_favorite: false,
            is_read_later: false,
            visit_count: 0,
            last_accessed: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }];
        
        let result = BookmarkExporter::export_to_json(&bookmarks).await.unwrap();
        assert_eq!(result.exported_count, 1);
        assert_eq!(result.format, ExportFormat::Json);
    }
}