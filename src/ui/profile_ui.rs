//! Profile UI Components
//!
//! User interface for profile management:
//! - Profile manager UI
//! - Profile template selection
//! - Profile sync settings
//! - Profile analytics dashboard
//! - Profile security settings

use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::profiles::{
    ProfileConfig, ProfileType, ProfileTemplate, TemplateCategory,
    SyncConfig, SyncProvider, SyncStatus, ProfileAnalytics, UsageSummary,
    ProfileSecurity, SecurityLevel, AuthMethod
};

/// Profile UI component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileManagerUI {
    profiles: Vec<ProfileConfig>,
    active_profile_id: Option<String>,
    selected_profile_id: Option<String>,
    show_create_dialog: bool,
    show_delete_dialog: bool,
    show_template_dialog: bool,
    show_export_dialog: bool,
    show_import_dialog: bool,
    export_selected_profile: Option<String>,
    // Drag and drop state
    dragged_profile_id: Option<String>,
    drop_target_id: Option<String>,
    drag_over: bool,
}

impl ProfileManagerUI {
    /// Create a new profile manager UI
    pub fn new() -> Self {
        Self {
            profiles: Vec::new(),
            active_profile_id: None,
            selected_profile_id: None,
            show_create_dialog: false,
            show_delete_dialog: false,
            show_template_dialog: false,
            show_export_dialog: false,
            show_import_dialog: false,
            export_selected_profile: None,
            dragged_profile_id: None,
            drop_target_id: None,
            drag_over: false,
        }
    }

    /// Set profiles
    pub fn set_profiles(&mut self, profiles: Vec<ProfileConfig>) {
        self.profiles = profiles;
    }

    /// Set active profile
    pub fn set_active_profile(&mut self, profile_id: String) {
        self.active_profile_id = Some(profile_id);
    }

    /// Show create dialog
    pub fn show_create_dialog(&mut self) {
        self.show_create_dialog = true;
    }

    /// Hide create dialog
    pub fn hide_create_dialog(&mut self) {
        self.show_create_dialog = false;
    }

    /// Show delete dialog
    pub fn show_delete_dialog(&mut self, profile_id: String) {
        self.selected_profile_id = Some(profile_id);
        self.show_delete_dialog = true;
    }

    /// Hide delete dialog
    pub fn hide_delete_dialog(&mut self) {
        self.show_delete_dialog = false;
        self.selected_profile_id = None;
    }

    /// Show template dialog
    pub fn show_template_dialog(&mut self) {
        self.show_template_dialog = true;
    }

    /// Hide template dialog
    pub fn hide_template_dialog(&mut self) {
        self.show_template_dialog = false;
    }

    /// Show export dialog
    pub fn show_export_dialog(&mut self, profile_id: Option<String>) {
        self.export_selected_profile = profile_id;
        self.show_export_dialog = true;
    }

    /// Hide export dialog
    pub fn hide_export_dialog(&mut self) {
        self.show_export_dialog = false;
        self.export_selected_profile = None;
    }

    /// Show import dialog
    pub fn show_import_dialog(&mut self) {
        self.show_import_dialog = true;
    }

    /// Hide import dialog
    pub fn hide_import_dialog(&mut self) {
        self.show_import_dialog = false;
    }

    /// Start dragging a profile
    pub fn start_drag(&mut self, profile_id: String) {
        self.dragged_profile_id = Some(profile_id);
        self.drag_over = false;
    }

    /// End dragging
    pub fn end_drag(&mut self) {
        self.dragged_profile_id = None;
        self.drop_target_id = None;
        self.drag_over = false;
    }

    /// Set drop target
    pub fn set_drop_target(&mut self, target_id: Option<String>) {
        self.drop_target_id = target_id;
        self.drag_over = target_id.is_some();
    }

    /// Get dragged profile
    pub fn get_dragged_profile(&self) -> Option<&String> {
        self.dragged_profile_id.as_ref()
    }

    /// Get drop target
    pub fn get_drop_target(&self) -> Option<&String> {
        self.drop_target_id.as_ref()
    }

    /// Check if a profile is being dragged
    pub fn is_dragging(&self) -> bool {
        self.dragged_profile_id.is_some()
    }

    /// Check if a specific profile is being dragged
    pub fn is_dragging_profile(&self, profile_id: &str) -> bool {
        self.dragged_profile_id.as_ref().map_or(false, |id| id == profile_id)
    }

    /// Check if dragging over a specific profile
    pub fn is_dragging_over(&self, profile_id: &str) -> bool {
        self.drag_over && self.drop_target_id.as_ref().map_or(false, |id| id == profile_id)
    }

    /// Render profile manager UI
    pub fn render(&self) -> String {
        let mut html = String::with_capacity(5000);

        html.push_str(r#"
<div class="profile-manager">
    <div class="profile-manager-header">
        <h2>Profile Manager</h2>
        <div class="profile-manager-actions">
            <button id="create-profile-btn" class="button button-primary">Create Profile</button>
            <button id="import-profile-btn" class="button button-secondary">Import Profile</button>
        </div>
    </div>
    <div class="profile-list">
"#);

        for profile in &self.profiles {
            let is_active = self.active_profile_id.as_ref().map_or(false, |id| id == &profile.id);
            let is_dragging = self.is_dragging_profile(&profile.id);
            let is_drop_target = self.is_dragging_over(&profile.id);
            
            let mut classes = Vec::new();
            if is_active { classes.push("active"); }
            if is_dragging { classes.push("dragging"); }
            if is_drop_target { classes.push("drop-target"); }
            let class_str = classes.join(" ");

            html.push_str(&format!(r#"
        <div class="profile-card {}" data-profile-id="{}" data-profile-order="{}" draggable="true">
            <div class="profile-drag-handle" title="Drag to reorder">⋮⋮</div>
            <div class="profile-icon" style="background-color: {};">
                {}
            </div>
            <div class="profile-info">
                <h3 class="profile-name">{}</h3>
                <p class="profile-type">{:?}</p>
                <p class="profile-last-used">Last used: {}</p>
            </div>
            <div class="profile-actions">
                {}
                <button class="button button-secondary profile-settings-btn" data-profile-id="{}">Settings</button>
                <button class="button button-secondary profile-export-btn" data-profile-id="{}">Export</button>
                <button class="button button-danger profile-delete-btn" data-profile-id="{}">Delete</button>
            </div>
        </div>
"#,
                class_str,
                profile.id,
                profile.order,
                profile.color.as_ref().unwrap_or(&"#6366f1".to_string()),
                profile.icon.as_ref().unwrap_or(&"👤".to_string()),
                profile.name,
                profile.profile_type,
                format_timestamp(profile.last_used_at),
                if is_active { r#"<span class="active-badge">Active</span>"# } else { r#"<button class="button button-success profile-activate-btn" data-profile-id="{}">Activate</button>"# },
                profile.id,
                profile.id,
                profile.id
            ));
        }

        html.push_str(r#"
    </div>
</div>
"#);

        // Render dialogs
        if self.show_create_dialog {
            html.push_str(&self.render_create_dialog());
        }

        if self.show_delete_dialog {
            html.push_str(&self.render_delete_dialog());
        }

        if self.show_template_dialog {
            html.push_str(&self.render_template_dialog());
        }

        if self.show_export_dialog {
            html.push_str(&self.render_export_dialog());
        }

        if self.show_import_dialog {
            html.push_str(&self.render_import_dialog());
        }

        html
    }

    /// Render create profile dialog
    fn render_create_dialog(&self) -> String {
        format!(r#"
<div class="modal-overlay" id="create-profile-modal">
    <div class="modal">
        <div class="modal-header">
            <h3>Create New Profile</h3>
            <button class="modal-close" id="close-create-dialog">&times;</button>
        </div>
        <div class="modal-body">
            <div class="form-group">
                <label for="profile-name">Profile Name</label>
                <input type="text" id="profile-name" placeholder="Enter profile name">
            </div>
            <div class="form-group">
                <label for="profile-type">Profile Type</label>
                <select id="profile-type">
                    <option value="default">Default</option>
                    <option value="work">Work</option>
                    <option value="gaming">Gaming</option>
                    <option value="private">Private</option>
                    <option value="custom">Custom</option>
                </select>
            </div>
            <div class="form-group">
                <label for="profile-icon">Profile Icon</label>
                <input type="text" id="profile-icon" placeholder="👤">
            </div>
            <div class="form-group">
                <label for="profile-color">Profile Color</label>
                <input type="color" id="profile-color" value="#6366f1">
            </div>
            <div class="form-group">
                <label>Start from Template</label>
                <button id="select-template-btn" class="button button-secondary">Select Template</button>
            </div>
        </div>
        <div class="modal-footer">
            <button id="cancel-create-btn" class="button button-secondary">Cancel</button>
            <button id="confirm-create-btn" class="button button-primary">Create Profile</button>
        </div>
    </div>
</div>
"#)
    }

    /// Render delete profile dialog
    fn render_delete_dialog(&self) -> String {
        let profile_name = self.selected_profile_id.as_ref()
            .and_then(|id| self.profiles.iter().find(|p| &p.id == id))
            .map(|p| p.name.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        format!(r#"
<div class="modal-overlay" id="delete-profile-modal">
    <div class="modal">
        <div class="modal-header">
            <h3>Delete Profile</h3>
            <button class="modal-close" id="close-delete-dialog">&times;</button>
        </div>
        <div class="modal-body">
            <p>Are you sure you want to delete the profile <strong>"{}"</strong>?</p>
            <p class="warning">This action cannot be undone. All profile data will be permanently deleted.</p>
        </div>
        <div class="modal-footer">
            <button id="cancel-delete-btn" class="button button-secondary">Cancel</button>
            <button id="confirm-delete-btn" class="button button-danger">Delete Profile</button>
        </div>
    </div>
</div>
"#, profile_name)
    }

    /// Render template selection dialog
    fn render_template_dialog(&self) -> String {
        format!(r#"
<div class="modal-overlay" id="template-modal">
    <div class="modal modal-large">
        <div class="modal-header">
            <h3>Select Profile Template</h3>
            <button class="modal-close" id="close-template-dialog">&times;</button>
        </div>
        <div class="modal-body">
            <div class="template-grid">
                <div class="template-card" data-template="work">
                    <div class="template-icon">💼</div>
                    <h4>Work</h4>
                    <p>Productivity-focused with work-related bookmarks and settings</p>
                </div>
                <div class="template-card" data-template="gaming">
                    <div class="template-icon">🎮</div>
                    <h4>Gaming</h4>
                    <p>Optimized for gaming and streaming with low latency</p>
                </div>
                <div class="template-card" data-template="privacy">
                    <div class="template-icon">🔒</div>
                    <h4>Privacy</h4>
                    <p>Maximum privacy with enhanced tracking protection</p>
                </div>
                <div class="template-card" data-template="developer">
                    <div class="template-icon">💻</div>
                    <h4>Developer</h4>
                    <p>Web development tools and developer-friendly settings</p>
                </div>
            </div>
        </div>
        <div class="modal-footer">
            <button id="cancel-template-btn" class="button button-secondary">Cancel</button>
            <button id="confirm-template-btn" class="button button-primary" disabled>Apply Template</button>
        </div>
    </div>
</div>
"#)
    }

    /// Render export dialog
    fn render_export_dialog(&self) -> String {
        let is_all_profiles = self.export_selected_profile.is_none();
        let export_title = if is_all_profiles { "Export All Profiles" } else { "Export Profile" };
        let profile_name = self.export_selected_profile.as_ref()
            .and_then(|id| self.profiles.iter().find(|p| &p.id == id))
            .map(|p| p.name.clone())
            .unwrap_or_else(|| "".to_string());

        format!(r#"
<div class="modal-overlay" id="export-profile-modal">
    <div class="modal">
        <div class="modal-header">
            <h3>{}</h3>
            <button class="modal-close" id="close-export-dialog">&times;</button>
        </div>
        <div class="modal-body">
            <div class="form-group">
                <label>Export Name</label>
                <input type="text" id="export-name" value="{}" placeholder="Export name">
            </div>
            <div class="form-group">
                <label>Export Options</label>
                <label>
                    <input type="checkbox" id="export-bookmarks" checked>
                    Include bookmarks
                </label>
                <label>
                    <input type="checkbox" id="export-history" checked>
                    Include browsing history
                </label>
            </div>
            <div class="form-group">
                <label>Encryption</label>
                <label>
                    <input type="checkbox" id="export-encrypt">
                    Encrypt exported file
                </label>
                <div id="encryption-password-group" class="hidden">
                    <label for="export-password">Password</label>
                    <input type="password" id="export-password" placeholder="Enter encryption password">
                </div>
            </div>
        </div>
        <div class="modal-footer">
            <button id="cancel-export-btn" class="button button-secondary">Cancel</button>
            <button id="confirm-export-btn" class="button button-primary">Export</button>
        </div>
    </div>
</div>
"#, export_title, profile_name)
    }

    /// Render import dialog
    fn render_import_dialog(&self) -> String {
        format!(r#"
<div class="modal-overlay" id="import-profile-modal">
    <div class="modal">
        <div class="modal-header">
            <h3>Import Profile</h3>
            <button class="modal-close" id="close-import-dialog">&times;</button>
        </div>
        <div class="modal-body">
            <div class="form-group">
                <label for="import-file">Select Export File</label>
                <input type="file" id="import-file" accept=".json,.vantis">
            </div>
            <div class="form-group">
                <label for="import-password" id="import-password-label" class="hidden">Password</label>
                <input type="password" id="import-password" placeholder="Enter decryption password" class="hidden">
            </div>
            <div class="form-group">
                <label>Import Options</label>
                <label>
                    <input type="checkbox" id="import-overwrite" checked>
                    Overwrite existing profiles
                </label>
                <label>
                    <input type="checkbox" id="import-bookmarks" checked>
                    Import bookmarks
                </label>
                <label>
                    <input type="checkbox" id="import-history" checked>
                    Import history
                </label>
            </div>
            <div class="form-group">
                <label for="import-new-name">New Profile Name (optional)</label>
                <input type="text" id="import-new-name" placeholder="Rename imported profile">
            </div>
        </div>
        <div class="modal-footer">
            <button id="cancel-import-btn" class="button button-secondary">Cancel</button>
            <button id="confirm-import-btn" class="button button-primary">Import</button>
        </div>
    </div>
</div>
"#)
    }
}

/// Profile template selection UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileTemplateUI {
    templates: Vec<ProfileTemplate>,
    selected_template: Option<String>,
}

impl ProfileTemplateUI {
    /// Create a new profile template UI
    pub fn new() -> Self {
        Self {
            templates: Vec::new(),
            selected_template: None,
        }
    }

    /// Set templates
    pub fn set_templates(&mut self, templates: Vec<ProfileTemplate>) {
        self.templates = templates;
    }

    /// Select template
    pub fn select_template(&mut self, template_id: String) {
        self.selected_template = Some(template_id);
    }

    /// Get selected template
    pub fn get_selected_template(&self) -> Option<&ProfileTemplate> {
        self.selected_template.as_ref()
            .and_then(|id| self.templates.iter().find(|t| t.id == *id))
    }

    /// Render template selection UI
    pub fn render(&self) -> String {
        let mut html = String::with_capacity(3000);

        html.push_str(r#"
<div class="template-selection">
    <h2>Choose a Profile Template</h2>
    <div class="template-grid">
"#);

        for template in &self.templates {
            let is_selected = self.selected_template.as_ref().map_or(false, |id| id == &template.id);
            let selected_class = if is_selected { "selected" } else { "" };

            html.push_str(&format!(r#"
        <div class="template-card {}" data-template-id="{}">
            <div class="template-icon">{}</div>
            <h3>{}</h3>
            <p class="template-description">{}</p>
            <div class="template-features">
"#,
                selected_class,
                template.id,
                template.icon,
                template.name,
                template.description
            ));

            for feature in &template.features {
                html.push_str(&format!(r#"
                <span class="template-feature">{}</span>
"#, feature));
            }

            html.push_str(r#"
            </div>
            <div class="template-category">{:?}</div>
        </div>
"#);
        }

        html.push_str(r#"
    </div>
</div>
"#);

        html
    }
}

/// Profile sync settings UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSyncUI {
    sync_config: SyncConfig,
    sync_status: Option<SyncStatus>,
    show_sync_dialog: bool,
}

impl ProfileSyncUI {
    /// Create a new profile sync UI
    pub fn new(sync_config: SyncConfig) -> Self {
        Self {
            sync_config,
            sync_status: None,
            show_sync_dialog: false,
        }
    }

    /// Set sync status
    pub fn set_sync_status(&mut self, status: SyncStatus) {
        self.sync_status = Some(status);
    }

    /// Show sync dialog
    pub fn show_sync_dialog(&mut self) {
        self.show_sync_dialog = true;
    }

    /// Hide sync dialog
    pub fn hide_sync_dialog(&mut self) {
        self.show_sync_dialog = false;
    }

    /// Render sync settings UI
    pub fn render(&self) -> String {
        let mut html = String::with_capacity(2000);

        html.push_str(r#"
<div class="sync-settings">
    <div class="sync-header">
        <h2>Profile Synchronization</h2>
        <div class="sync-status">
"#);

        if let Some(status) = &self.sync_status {
            let status_class = match status {
                SyncStatus::Synced => "synced",
                SyncStatus::Syncing => "syncing",
                SyncStatus::Error(_) => "error",
                SyncStatus::Offline => "offline",
            };

            html.push_str(&format!(r#"
            <span class="sync-badge {}">{:?}</span>
"#, status_class, status));
        }

        html.push_str(r#"
        </div>
    </div>
    <div class="sync-config">
        <div class="form-group">
            <label for="sync-provider">Sync Provider</label>
            <select id="sync-provider">
"#);

        for provider in &[SyncProvider::Local, SyncProvider::Custom] {
            let selected = if self.sync_config.provider == *provider {
                "selected"
            } else {
                ""
            };

            html.push_str(&format!(r#"
                <option value="{:?}" {}>{:?}</option>
"#, provider, selected, provider));
        }

        html.push_str(r#"
            </select>
        </div>
        <div class="form-group">
            <label for="sync-interval">Sync Interval (minutes)</label>
            <input type="number" id="sync-interval" value="{}" min="1" max="1440">
        </div>
        <div class="form-group">
            <label>
                <input type="checkbox" id="auto-sync" {}>
                Enable automatic sync
            </label>
        </div>
        <div class="form-group">
            <label>
                <input type="checkbox" id="sync-on-startup" {}>
                Sync on startup
            </label>
        </div>
    </div>
    <div class="sync-actions">
        <button id="sync-now-btn" class="button button-primary">Sync Now</button>
        <button id="view-conflicts-btn" class="button button-secondary">View Conflicts</button>
        <button id="backup-profile-btn" class="button button-secondary">Backup Profile</button>
        <button id="restore-profile-btn" class="button button-secondary">Restore Profile</button>
    </div>
</div>
"#,
            self.sync_config.sync_interval,
            if self.sync_config.auto_sync { "checked" } else { "" },
            if self.sync_config.sync_on_startup { "checked" } else { "" }
        );

        if self.show_sync_dialog {
            html.push_str(&self.render_sync_dialog());
        }

        html
    }

    /// Render sync dialog
    fn render_sync_dialog(&self) -> String {
        format!(r#"
<div class="modal-overlay" id="sync-modal">
    <div class="modal">
        <div class="modal-header">
            <h3>Sync Profile</h3>
            <button class="modal-close" id="close-sync-dialog">&times;</button>
        </div>
        <div class="modal-body">
            <div class="sync-progress">
                <div class="progress-bar">
                    <div class="progress-fill" style="width: 0%"></div>
                </div>
                <p class="sync-status-text">Preparing to sync...</p>
            </div>
        </div>
        <div class="modal-footer">
            <button id="cancel-sync-btn" class="button button-secondary">Cancel</button>
        </div>
    </div>
</div>
"#)
    }
}

/// Profile analytics dashboard UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileAnalyticsUI {
    analytics: Option<ProfileAnalytics>,
    usage_summary: Option<UsageSummary>,
    time_range: String,
}

impl ProfileAnalyticsUI {
    /// Create a new profile analytics UI
    pub fn new() -> Self {
        Self {
            analytics: None,
            usage_summary: None,
            time_range: "7d".to_string(),
        }
    }

    /// Set analytics data
    pub fn set_analytics(&mut self, analytics: ProfileAnalytics) {
        self.analytics = Some(analytics);
    }

    /// Set usage summary
    pub fn set_usage_summary(&mut self, summary: UsageSummary) {
        self.usage_summary = Some(summary);
    }

    /// Set time range
    pub fn set_time_range(&mut self, range: String) {
        self.time_range = range;
    }

    /// Render analytics dashboard
    pub fn render(&self) -> String {
        let mut html = String::with_capacity(4000);

        html.push_str(r#"
<div class="analytics-dashboard">
    <div class="analytics-header">
        <h2>Profile Analytics</h2>
        <div class="time-range-selector">
            <button class="time-range-btn {}" data-range="1d">1 Day</button>
            <button class="time-range-btn {}" data-range="7d">7 Days</button>
            <button class="time-range-btn {}" data-range="30d">30 Days</button>
            <button class="time-range-btn {}" data-range="90d">90 Days</button>
        </div>
    </div>
"#,
            if self.time_range == "1d" { "active" } else { "" },
            if self.time_range == "7d" { "active" } else { "" },
            if self.time_range == "30d" { "active" } else { "" },
            if self.time_range == "90d" { "active" } else { "" }
        );

        if let Some(summary) = &self.usage_summary {
            html.push_str(r#"
    <div class="analytics-summary">
        <div class="summary-card">
            <div class="summary-icon">⏱️</div>
            <div class="summary-content">
                <h3>Total Time</h3>
                <p class="summary-value">{}</p>
            </div>
        </div>
        <div class="summary-card">
            <div class="summary-icon">🌐</div>
            <div class="summary-content">
                <h3>Websites Visited</h3>
                <p class="summary-value">{}</p>
            </div>
        </div>
        <div class="summary-card">
            <div class="summary-icon">📑</div>
            <div class="summary-content">
                <h3>Tabs Opened</h3>
                <p class="summary-value">{}</p>
            </div>
        </div>
        <div class="summary-card">
            <div class="summary-icon">⚡</div>
            <div class="summary-content">
                <h3>Avg Load Time</h3>
                <p class="summary-value">{:.2}s</p>
            </div>
        </div>
    </div>
"#,
                format_duration(summary.total_time),
                summary.websites_visited,
                summary.tabs_opened,
                summary.avg_page_load_time
            );
        }

        html.push_str(r#"
    <div class="analytics-charts">
        <div class="chart-container">
            <h3>Daily Usage</h3>
            <div class="chart" id="daily-usage-chart"></div>
        </div>
        <div class="chart-container">
            <h3>Top Websites</h3>
            <div class="chart" id="top-websites-chart"></div>
        </div>
    </div>
    <div class="analytics-details">
        <div class="details-section">
            <h3>Tab Statistics</h3>
            <table class="stats-table">
                <tr>
                    <th>Metric</th>
                    <th>Value</th>
                </tr>
"#);

        if let Some(analytics) = &self.analytics {
            html.push_str(&format!(r#"
                <tr>
                    <td>Total Tabs Opened</td>
                    <td>{}</td>
                </tr>
                <tr>
                    <td>Average Tabs per Session</td>
                    <td>{:.1}</td>
                </tr>
                <tr>
                    <td>Maximum Tabs Open</td>
                    <td>{}</td>
                </tr>
"#,
                analytics.tab_stats.total_opened,
                analytics.tab_stats.average_tabs,
                analytics.tab_stats.max_tabs
            ));
        }

        html.push_str(r#"
            </table>
        </div>
        <div class="details-section">
            <h3>Performance Metrics</h3>
            <table class="stats-table">
                <tr>
                    <th>Metric</th>
                    <th>Value</th>
                </tr>
"#);

        if let Some(analytics) = &self.analytics {
            html.push_str(&format!(r#"
                <tr>
                    <td>Average Page Load Time</td>
                    <td>{:.2}s</td>
                </tr>
                <tr>
                    <td>Crashes</td>
                    <td>{}</td>
                </tr>
                <tr>
                    <td>Average Memory Usage</td>
                    <td>{:.1} MB</td>
                </tr>
                <tr>
                    <td>Average CPU Usage</td>
                    <td>{:.1}%</td>
                </tr>
"#,
                analytics.performance.avg_page_load_time,
                analytics.performance.crashes,
                analytics.performance.avg_memory_usage,
                analytics.performance.avg_cpu_usage
            ));
        }

        html.push_str(r#"
            </table>
        </div>
    </div>
</div>
"#);

        html
    }
}

/// Profile security settings UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSecurityUI {
    security: Option<ProfileSecurity>,
    show_password_dialog: bool,
    show_biometric_dialog: bool,
}

impl ProfileSecurityUI {
    /// Create a new profile security UI
    pub fn new() -> Self {
        Self {
            security: None,
            show_password_dialog: false,
            show_biometric_dialog: false,
        }
    }

    /// Set security settings
    pub fn set_security(&mut self, security: ProfileSecurity) {
        self.security = Some(security);
    }

    /// Show password dialog
    pub fn show_password_dialog(&mut self) {
        self.show_password_dialog = true;
    }

    /// Hide password dialog
    pub fn hide_password_dialog(&mut self) {
        self.show_password_dialog = false;
    }

    /// Show biometric dialog
    pub fn show_biometric_dialog(&mut self) {
        self.show_biometric_dialog = true;
    }

    /// Hide biometric dialog
    pub fn hide_biometric_dialog(&mut self) {
        self.show_biometric_dialog = false;
    }

    /// Render security settings UI
    pub fn render(&self) -> String {
        let mut html = String::with_capacity(3000);

        html.push_str(r#"
<div class="security-settings">
    <div class="security-header">
        <h2>Profile Security</h2>
        <div class="security-status">
"#);

        if let Some(security) = &self.security {
            let status_class = match security.security_level {
                SecurityLevel::None => "none",
                SecurityLevel::Low => "low",
                SecurityLevel::Medium => "medium",
                SecurityLevel::High => "high",
            };

            html.push_str(&format!(r#"
            <span class="security-badge {}">{:?}</span>
"#, status_class, security.security_level));
        }

        html.push_str(r#"
        </div>
    </div>
    <div class="security-config">
        <div class="form-group">
            <label for="security-level">Security Level</label>
            <select id="security-level">
"#);

        for level in &[SecurityLevel::None, SecurityLevel::Low, SecurityLevel::Medium, SecurityLevel::High] {
            let selected = self.security.as_ref()
                .map_or(false, |s| s.security_level == *level);

            html.push_str(&format!(r#"
                <option value="{:?}" {}>{:?}</option>
"#, level, if selected { "selected" } else { "" }, level));
        }

        html.push_str(r#"
            </select>
        </div>
        <div class="form-group">
            <label for="auth-method">Authentication Method</label>
            <select id="auth-method">
"#);

        for method in &[AuthMethod::None, AuthMethod::Password, AuthMethod::Biometric, AuthMethod::TwoFactor] {
            let selected = self.security.as_ref()
                .map_or(false, |s| s.auth_method == *method);

            html.push_str(&format!(r#"
                <option value="{:?}" {}>{:?}</option>
"#, method, if selected { "selected" } else { "" }, method));
        }

        html.push_str(r#"
            </select>
        </div>
        <div class="form-group">
            <label>
                <input type="checkbox" id="auto-lock" {}>
                Auto-lock after inactivity
            </label>
        </div>
        <div class="form-group">
            <label for="lock-timeout">Lock Timeout (minutes)</label>
            <input type="number" id="lock-timeout" value="{}" min="1" max="60">
        </div>
        <div class="form-group">
            <label>
                <input type="checkbox" id="encrypt-data" {}>
                Encrypt profile data
            </label>
        </div>
    </div>
    <div class="security-actions">
        <button id="change-password-btn" class="button button-primary">Change Password</button>
        <button id="setup-biometric-btn" class="button button-secondary">Setup Biometric</button>
        <button id="view-attempts-btn" class="button button-secondary">View Failed Attempts</button>
    </div>
</div>
"#,
            self.security.as_ref().map_or(false, |s| s.auto_lock),
            self.security.as_ref().map_or(5, |s| s.lock_timeout),
            self.security.as_ref().map_or(false, |s| s.encrypt_data)
        );

        if self.show_password_dialog {
            html.push_str(&self.render_password_dialog());
        }

        if self.show_biometric_dialog {
            html.push_str(&self.render_biometric_dialog());
        }

        html
    }

    /// Render password change dialog
    fn render_password_dialog(&self) -> String {
        format!(r#"
<div class="modal-overlay" id="password-modal">
    <div class="modal">
        <div class="modal-header">
            <h3>Change Password</h3>
            <button class="modal-close" id="close-password-dialog">&times;</button>
        </div>
        <div class="modal-body">
            <div class="form-group">
                <label for="current-password">Current Password</label>
                <input type="password" id="current-password" placeholder="Enter current password">
            </div>
            <div class="form-group">
                <label for="new-password">New Password</label>
                <input type="password" id="new-password" placeholder="Enter new password">
            </div>
            <div class="form-group">
                <label for="confirm-password">Confirm Password</label>
                <input type="password" id="confirm-password" placeholder="Confirm new password">
            </div>
            <div class="password-strength">
                <div class="strength-bar">
                    <div class="strength-fill" style="width: 0%"></div>
                </div>
                <p class="strength-text">Password strength</p>
            </div>
        </div>
        <div class="modal-footer">
            <button id="cancel-password-btn" class="button button-secondary">Cancel</button>
            <button id="confirm-password-btn" class="button button-primary">Change Password</button>
        </div>
    </div>
</div>
"#)
    }

    /// Render biometric setup dialog
    fn render_biometric_dialog(&self) -> String {
        format!(r#"
<div class="modal-overlay" id="biometric-modal">
    <div class="modal">
        <div class="modal-header">
            <h3>Setup Biometric Authentication</h3>
            <button class="modal-close" id="close-biometric-dialog">&times;</button>
        </div>
        <div class="modal-body">
            <div class="biometric-setup">
                <div class="biometric-icon">👆</div>
                <p>Follow the instructions to set up biometric authentication</p>
                <div class="biometric-steps">
                    <div class="step">
                        <span class="step-number">1</span>
                        <p>Place your finger on the sensor</p>
                    </div>
                    <div class="step">
                        <span class="step-number">2</span>
                        <p>Lift and repeat 3 times</p>
                    </div>
                    <div class="step">
                        <span class="step-number">3</span>
                        <p>Wait for confirmation</p>
                    </div>
                </div>
                <div class="biometric-status">
                    <p class="status-text">Ready to scan</p>
                </div>
            </div>
        </div>
        <div class="modal-footer">
            <button id="cancel-biometric-btn" class="button button-secondary">Cancel</button>
            <button id="start-scan-btn" class="button button-primary">Start Scan</button>
        </div>
    </div>
</div>
"#)
    }
}

/// Format timestamp to human-readable string
fn format_timestamp(timestamp: i64) -> String {
    use chrono::{DateTime, Utc};
    
    let dt = DateTime::<Utc>::from_timestamp(timestamp / 1000, 0);
    match dt {
        Some(dt) => dt.format("%Y-%m-%d %H:%M").to_string(),
        None => "Unknown".to_string(),
    }
}

/// Format duration to human-readable string
fn format_duration(seconds: i64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    
    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_manager_ui() {
        let ui = ProfileManagerUI::new();
        let html = ui.render();
        assert!(html.contains("profile-manager"));
    }

    #[test]
    fn test_template_ui() {
        let ui = ProfileTemplateUI::new();
        let html = ui.render();
        assert!(html.contains("template-selection"));
    }

    #[test]
    fn test_sync_ui() {
        let config = SyncConfig {
            provider: SyncProvider::Local,
            sync_interval: 30,
            auto_sync: true,
            sync_on_startup: true,
        };
        let ui = ProfileSyncUI::new(config);
        let html = ui.render();
        assert!(html.contains("sync-settings"));
    }

    #[test]
    fn test_analytics_ui() {
        let ui = ProfileAnalyticsUI::new();
        let html = ui.render();
        assert!(html.contains("analytics-dashboard"));
    }

    #[test]
    fn test_security_ui() {
        let ui = ProfileSecurityUI::new();
        let html = ui.render();
        assert!(html.contains("security-settings"));
    }
}