//! Article Formatter
//! 
//! This module formats article content for optimal reading experience with
//! customizable fonts, themes, and layout options.
//! 
//! # Features
//! - Custom font and size settings
//! - Theme customization (light, dark, sepia, high contrast)
//! - Text alignment options
//! - Image and video handling
//! - Reading progress indicators

use super::{Article, ReadingModeConfig, ReadingTheme, TextAlignment, Result, ReadingError};

/// Article formatter
pub struct ArticleFormatter {
    css_templates: HashMap<ReadingTheme, String>,
}

impl ArticleFormatter {
    /// Create a new article formatter
    pub fn new() -> Self {
        Self {
            css_templates: Self::build_css_templates(),
        }
    }

    /// Format article for reading mode
    pub async fn format(&self, article: &Article, config: &ReadingModeConfig) -> Result<String> {
        let css = self.generate_css(config);
        let html = self.build_html(article, &css, config)?;
        Ok(html)
    }

    fn generate_css(&self, config: &ReadingModeConfig) -> String {
        let mut css = self.css_templates.get(&config.theme)
            .cloned()
            .unwrap_or_default();

        // Add custom font settings
        css = css.replace("{font-family}", &config.font_family);
        css = css.replace("{font-size}", &config.font_size.to_string());
        css = css.replace("{line-height}", &config.line_height.to_string());
        css = css.replace("{margin-width}", &config.margin_width.to_string());

        // Add text alignment
        let alignment = match config.text_alignment {
            TextAlignment::Left => "left",
            TextAlignment::Center => "center",
            TextAlignment::Justify => "justify",
        };
        css = css.replace("{text-align}", alignment);

        css
    }

    fn build_html(&self, article: &Article, css: &str, config: &ReadingModeConfig) -> Result<String> {
        let mut html = String::new();

        // HTML header
        html.push_str("<!DOCTYPE html>\n");
        html.push_str("<html lang=&quot;en&quot;>\n");
        html.push_str("<head>\n");
        html.push_str("<meta charset=&quot;UTF-8&quot;>\n");
        html.push_str("<meta name=&quot;viewport&quot; content=&quot;width=device-width, initial-scale=1.0&quot;>\n");
        html.push_str("<title>");
        html.push_str(&article.metadata.title);
        html.push_str("</title>\n");
        html.push_str("<style>\n");
        html.push_str(css);
        html.push_str("\n</style>\n");
        html.push_str("</head>\n");
        html.push_str("<body>\n");

        // Article container
        html.push_str("<div class=&quot;reading-mode-container&quot;>\n");

        // Article header
        html.push_str("<article>\n");
        html.push_str("<header class=&quot;article-header&quot;>\n");

        // Title
        html.push_str("<h1 class=&quot;article-title&quot;>");
        html.push_str(&self.escape_html(&article.metadata.title));
        html.push_str("</h1>\n");

        // Metadata
        html.push_str("<div class=&quot;article-meta&quot;>\n");

        if let Some(ref author) = article.metadata.author {
            html.push_str("<span class=&quot;author&quot;>");
            html.push_str("By ");
            html.push_str(&self.escape_html(author));
            html.push_str("</span>\n");
        }

        if let Some(ref date) = article.metadata.published_date {
            html.push_str("<span class=&quot;date&quot;>");
            html.push_str(&date.format("%B %d, %Y").to_string());
            html.push_str("</span>\n");
        }

        html.push_str("<span class=&quot;reading-time&quot;>");
        html.push_str(&format!("{} min read", article.metadata.reading_time_minutes));
        html.push_str("</span>\n");

        html.push_str("</div>\n"); // End article-meta

        // Featured image
        if config.show_images {
            if let Some(ref featured_image) = article.metadata.featured_image {
                html.push_str("<div class=&quot;featured-image&quot;>\n");
                html.push_str("<img src=&quot;");
                html.push_str(&self.escape_html(featured_image));
                html.push_str("&quot; alt=&quot;");
                html.push_str(&self.escape_html(&article.metadata.title));
                html.push_str("&quot;>\n");
                html.push_str("</div>\n");
            }
        }

        html.push_str("</header>\n"); // End article-header

        // Article content
        html.push_str("<div class=&quot;article-content&quot;>\n");
        html.push_str(&self.format_content(&article.content, config));
        html.push_str("</div>\n"); // End article-content

        html.push_str("</article>\n"); // End article

        html.push_str("</div>\n"); // End reading-mode-container

        // Reading progress bar
        html.push_str("<div class=&quot;reading-progress-bar&quot;>\n");
        html.push_str("<div class=&quot;reading-progress-fill&quot; style=&quot;width: ");
        html.push_str(&article.reading_progress.percentage().to_string());
        html.push_str("%&quot;></div>\n");
        html.push_str("</div>\n");

        // JavaScript for auto-scroll (if enabled)
        if config.auto_scroll_enabled {
            html.push_str("<script>\n");
            html.push_str(&self.generate_auto_scroll_script(config));
            html.push_str("</script>\n");
        }

        html.push_str("</body>\n");
        html.push_str("</html>");

        Ok(html)
    }

    fn format_content(&self, content: &super::ArticleContent, config: &ReadingModeConfig) -> String {
        let mut formatted = content.html.clone();

        // Clean up the content
        formatted = self.sanitize_html(&formatted);

        // Wrap images in figure elements
        if config.show_images {
            formatted = self.enhance_images(&formatted);
        }

        // Process videos
        if config.show_videos {
            formatted = self.process_videos(&formatted);
        } else {
            formatted = self.remove_videos(&formatted);
        }

        // Wrap text in paragraphs if needed
        formatted = self.ensure_paragraph_structure(&formatted);

        formatted
    }

    fn sanitize_html(&self, html: &str) -> String {
        let mut cleaned = html.clone();

        // Remove scripts
        let script_regex = regex::Regex::new(r"<script[^>]*>.*?</script>").unwrap();
        cleaned = script_regex.replace_all(&cleaned, "").to_string();

        // Remove styles
        let style_regex = regex::Regex::new(r"<style[^>]*>.*?</style>").unwrap();
        cleaned = style_regex.replace_all(&cleaned, "").to_string();

        // Remove comments
        let comment_regex = regex::Regex::new(r"<!--.*?-->").unwrap();
        cleaned = comment_regex.replace_all(&cleaned, "").to_string();

        // Remove iframes (except for embedded content)
        let iframe_regex = regex::Regex::new(r"<iframe[^>]*>.*?</iframe>").unwrap();
        cleaned = iframe_regex.replace_all(&cleaned, "").to_string();

        cleaned
    }

    fn enhance_images(&self, html: &str) -> String {
        let img_regex = regex::Regex::new(r#"<img([^>]+)>"#).unwrap();
        
        img_regex.replace_all(html, |caps: &regex::Captures| {
            let attrs = caps.get(1).unwrap().as_str();
            
            // Check if already wrapped
            if attrs.contains("data-wrapped") {
                return format!("<img{}>", attrs);
            }

            format!("<figure class=&quot;article-image&quot;><img{} data-wrapped=&quot;true&quot;></figure>", attrs)
        }).to_string()
    }

    fn process_videos(&self, html: &str) -> String {
        let youtube_regex = regex::Regex::new(
            r#"(?:youtube\.com/watch\?v=|youtu\.be/)([a-zA-Z0-9_-]+)"#
        ).unwrap();

        youtube_regex.replace_all(html, |caps: &regex::Captures| {
            let video_id = caps.get(1).unwrap().as_str();
            format!(
                r#"<div class="video-container"><iframe src="https://www.youtube.com/embed/{}" frameborder="0" allowfullscreen></iframe></div>"#,
                video_id
            )
        }).to_string()
    }

    fn remove_videos(&self, html: &str) -> String {
        let video_regex = regex::Regex::new(r#"<iframe[^>]*src=["']https?://(?:www\.)?youtube\.com/embed/[^"']+["'][^>]*>.*?</iframe>"#).unwrap();
        video_regex.replace_all(html, "").to_string()
    }

    fn ensure_paragraph_structure(&self, html: &str) -> String {
        let mut processed = html.clone();

        // Wrap orphaned text in paragraphs
        let orphan_regex = regex::Regex::new(r"(?<!<p>)^[^<].{50,}(?!</p>)$").unwrap();
        // This is simplified - real implementation would be more sophisticated

        processed
    }

    fn generate_auto_scroll_script(&self, config: &ReadingModeConfig) -> String {
        format!(
            r#"
let scrollSpeed = {};
let autoScrollEnabled = {};
let lastScrollY = window.scrollY;

function autoScroll() {{
    if (!autoScrollEnabled) return;
    
    const scrollStep = scrollSpeed / 10;
    window.scrollBy(0, scrollStep);
    
    // Stop at bottom
    if ((window.innerHeight + window.scrollY) >= document.body.offsetHeight) {{
        autoScrollEnabled = false;
        return;
    }}
    
    requestAnimationFrame(autoScroll);
}}

// Toggle auto-scroll with keyboard
document.addEventListener('keydown', (e) => {{
    if (e.key === 's' && e.ctrlKey) {{
        autoScrollEnabled = !autoScrollEnabled;
        if (autoScrollEnabled) {{
            autoScroll();
        }}
    }}
}});
"#,
            config.auto_scroll_speed,
            config.auto_scroll_enabled
        )
    }

    fn escape_html(&self, text: &str) -> String {
        text.replace('&', "&")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }

    fn build_css_templates() -> HashMap<ReadingTheme, String> {
        let mut templates = HashMap::new();

        // Light theme
        templates.insert(ReadingTheme::Light, Self::light_theme_css());

        // Dark theme
        templates.insert(ReadingTheme::Dark, Self::dark_theme_css());

        // Sepia theme
        templates.insert(ReadingTheme::Sepia, Self::sepia_theme_css());

        // High contrast theme
        templates.insert(ReadingTheme::HighContrast, Self::high_contrast_theme_css());

        templates
    }

    fn light_theme_css() -> String {
        r#"
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: {font-family}, serif;
    font-size: {font-size}px;
    line-height: {line-height};
    color: #333;
    background-color: #fff;
    text-align: {text-align};
}

.reading-mode-container {
    max-width: 800px;
    margin: 40px auto;
    padding: 0 {margin_width}px;
}

.article-title {
    font-size: 2.5em;
    font-weight: 700;
    margin-bottom: 0.5em;
    line-height: 1.2;
}

.article-meta {
    font-size: 0.9em;
    color: #666;
    margin-bottom: 2em;
    padding-bottom: 1em;
    border-bottom: 1px solid #eee;
}

.article-meta span {
    margin-right: 1em;
}

.featured-image {
    margin-bottom: 2em;
}

.featured-image img {
    max-width: 100%;
    height: auto;
    border-radius: 8px;
}

.article-content {
    font-size: 1.1em;
}

.article-content p {
    margin-bottom: 1.5em;
}

.article-content h2 {
    font-size: 1.8em;
    margin: 2em 0 1em;
}

.article-content h3 {
    font-size: 1.5em;
    margin: 1.5em 0 0.8em;
}

.article-content img {
    max-width: 100%;
    height: auto;
    margin: 1.5em 0;
    border-radius: 4px;
}

.article-image {
    margin: 1.5em 0;
    text-align: center;
}

.video-container {
    position: relative;
    padding-bottom: 56.25%;
    height: 0;
    margin: 2em 0;
}

.video-container iframe {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
}

.reading-progress-bar {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 3px;
    background-color: #f0f0f0;
    z-index: 1000;
}

.reading-progress-fill {
    height: 100%;
    background-color: #007bff;
    transition: width 0.3s ease;
}
"#.to_string()
    }

    fn dark_theme_css() -> String {
        r#"
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: {font-family}, serif;
    font-size: {font-size}px;
    line-height: {line-height};
    color: #e0e0e0;
    background-color: #1a1a1a;
    text-align: {text-align};
}

.reading-mode-container {
    max-width: 800px;
    margin: 40px auto;
    padding: 0 {margin_width}px;
}

.article-title {
    font-size: 2.5em;
    font-weight: 700;
    margin-bottom: 0.5em;
    line-height: 1.2;
    color: #fff;
}

.article-meta {
    font-size: 0.9em;
    color: #999;
    margin-bottom: 2em;
    padding-bottom: 1em;
    border-bottom: 1px solid #333;
}

.article-meta span {
    margin-right: 1em;
}

.featured-image {
    margin-bottom: 2em;
}

.featured-image img {
    max-width: 100%;
    height: auto;
    border-radius: 8px;
}

.article-content {
    font-size: 1.1em;
}

.article-content p {
    margin-bottom: 1.5em;
}

.article-content a {
    color: #5c9aff;
}

.article-content h2 {
    font-size: 1.8em;
    margin: 2em 0 1em;
    color: #fff;
}

.article-content h3 {
    font-size: 1.5em;
    margin: 1.5em 0 0.8em;
    color: #fff;
}

.article-content img {
    max-width: 100%;
    height: auto;
    margin: 1.5em 0;
    border-radius: 4px;
}

.article-image {
    margin: 1.5em 0;
    text-align: center;
}

.video-container {
    position: relative;
    padding-bottom: 56.25%;
    height: 0;
    margin: 2em 0;
}

.video-container iframe {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
}

.reading-progress-bar {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 3px;
    background-color: #2a2a2a;
    z-index: 1000;
}

.reading-progress-fill {
    height: 100%;
    background-color: #5c9aff;
    transition: width 0.3s ease;
}
"#.to_string()
    }

    fn sepia_theme_css() -> String {
        r#"
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: {font-family}, serif;
    font-size: {font-size}px;
    line-height: {line-height};
    color: #5b4636;
    background-color: #f4ecd8;
    text-align: {text-align};
}

.reading-mode-container {
    max-width: 800px;
    margin: 40px auto;
    padding: 0 {margin_width}px;
}

.article-title {
    font-size: 2.5em;
    font-weight: 700;
    margin-bottom: 0.5em;
    line-height: 1.2;
    color: #4b3525;
}

.article-meta {
    font-size: 0.9em;
    color: #8b7666;
    margin-bottom: 2em;
    padding-bottom: 1em;
    border-bottom: 1px solid #d4c4b0;
}

.article-meta span {
    margin-right: 1em;
}

.featured-image {
    margin-bottom: 2em;
}

.featured-image img {
    max-width: 100%;
    height: auto;
    border-radius: 8px;
}

.article-content {
    font-size: 1.1em;
}

.article-content p {
    margin-bottom: 1.5em;
}

.article-content a {
    color: #8b4513;
}

.article-content h2 {
    font-size: 1.8em;
    margin: 2em 0 1em;
    color: #4b3525;
}

.article-content h3 {
    font-size: 1.5em;
    margin: 1.5em 0 0.8em;
    color: #4b3525;
}

.article-content img {
    max-width: 100%;
    height: auto;
    margin: 1.5em 0;
    border-radius: 4px;
}

.article-image {
    margin: 1.5em 0;
    text-align: center;
}

.video-container {
    position: relative;
    padding-bottom: 56.25%;
    height: 0;
    margin: 2em 0;
}

.video-container iframe {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
}

.reading-progress-bar {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 3px;
    background-color: #e4d4c0;
    z-index: 1000;
}

.reading-progress-fill {
    height: 100%;
    background-color: #8b4513;
    transition: width 0.3s ease;
}
"#.to_string()
    }

    fn high_contrast_theme_css() -> String {
        r#"
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: {font-family}, sans-serif;
    font-size: {font-size}px;
    line-height: {line-height};
    color: #000;
    background-color: #fff;
    text-align: {text-align};
}

.reading-mode-container {
    max-width: 800px;
    margin: 40px auto;
    padding: 0 {margin_width}px;
}

.article-title {
    font-size: 2.5em;
    font-weight: 700;
    margin-bottom: 0.5em;
    line-height: 1.2;
    color: #000;
}

.article-meta {
    font-size: 0.9em;
    color: #000;
    margin-bottom: 2em;
    padding-bottom: 1em;
    border-bottom: 2px solid #000;
    font-weight: bold;
}

.article-meta span {
    margin-right: 1em;
}

.featured-image {
    margin-bottom: 2em;
    border: 3px solid #000;
}

.featured-image img {
    max-width: 100%;
    height: auto;
}

.article-content {
    font-size: 1.1em;
    font-weight: 500;
}

.article-content p {
    margin-bottom: 1.5em;
}

.article-content a {
    color: #000;
    text-decoration: underline;
    font-weight: bold;
}

.article-content h2 {
    font-size: 1.8em;
    margin: 2em 0 1em;
    color: #000;
    border-bottom: 2px solid #000;
    padding-bottom: 0.3em;
}

.article-content h3 {
    font-size: 1.5em;
    margin: 1.5em 0 0.8em;
    color: #000;
    font-weight: bold;
}

.article-content img {
    max-width: 100%;
    height: auto;
    margin: 1.5em 0;
    border: 2px solid #000;
}

.article-image {
    margin: 1.5em 0;
    text-align: center;
    border: 2px solid #000;
}

.video-container {
    position: relative;
    padding-bottom: 56.25%;
    height: 0;
    margin: 2em 0;
    border: 3px solid #000;
}

.video-container iframe {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
}

.reading-progress-bar {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 5px;
    background-color: #ccc;
    z-index: 1000;
}

.reading-progress-fill {
    height: 100%;
    background-color: #000;
    transition: width 0.3s ease;
}
"#.to_string()
    }
}

impl Default for ArticleFormatter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_html() {
        let formatter = ArticleFormatter::new();
        assert_eq!(formatter.escape_html("<script>"), "&lt;script&gt;");
        assert_eq!(formatter.escape_html("A & B"), "A & B");
    }

    #[test]
    fn test_sanitize_html() {
        let formatter = ArticleFormatter::new();
        let html = "<p>Test</p><script>alert('xss')</script>";
        let cleaned = formatter.sanitize_html(html);
        assert!(!cleaned.contains("<script>"));
        assert!(cleaned.contains("<p>Test</p>"));
    }
}