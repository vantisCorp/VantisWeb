# Profile UI Components Guide

## Overview

The Profile UI components provide a comprehensive user interface for managing user profiles in the VantisWeb browser. These components offer intuitive interfaces for profile creation, template selection, synchronization, analytics, and security settings.

## Components

### 1. ProfileManagerUI

The main profile management interface that allows users to create, view, activate, and delete profiles.

#### Features
- Display all user profiles in a card-based layout
- Show active profile with visual indicator
- Create new profiles with custom settings
- Delete profiles (except active profile)
- Activate profiles with one click
- Profile icons and colors for easy identification
- Last used timestamp for each profile

#### Usage Example

```rust
use vantisweb::ui::ProfileManagerUI;
use vantisweb::profiles::{ProfileManager, ProfileConfig, ProfileType};

// Create profile manager UI
let mut profile_ui = ProfileManagerUI::new();

// Load profiles from profile manager
let profiles = profile_manager.get_all_profiles().await;
profile_ui.set_profiles(profiles);

// Set active profile
if let Some(active) = profile_manager.get_active_profile().await {
    profile_ui.set_active_profile(active.id);
}

// Render UI
let html = profile_ui.render();
```

#### UI Elements

**Profile Card**
- Profile icon (emoji or custom)
- Profile name
- Profile type (Default, Work, Gaming, Private, Custom)
- Last used timestamp
- Active badge (if active)
- Activate button (if not active)
- Settings button
- Delete button

**Create Profile Dialog**
- Profile name input
- Profile type selector
- Profile icon input
- Profile color picker
- Template selection button

**Delete Profile Dialog**
- Confirmation message
- Warning about permanent deletion
- Cancel and confirm buttons

**Template Selection Dialog**
- Grid of available templates
- Template icons and descriptions
- Feature lists for each template
- Apply template button

---

### 2. ProfileTemplateUI

Interface for selecting profile templates when creating new profiles.

#### Features
- Display available profile templates
- Visual template cards with icons
- Template descriptions and features
- Category filtering
- Single template selection
- Selected template highlighting

#### Available Templates

**Work Template**
- Icon: 💼
- Category: Productivity
- Features: Work bookmarks, productivity tools, work-focused settings

**Gaming Template**
- Icon: 🎮
- Category: Entertainment
- Features: Gaming bookmarks, low latency settings, streaming optimization

**Privacy Template**
- Icon: 🔒
- Category: Security
- Features: Enhanced tracking protection, privacy-focused settings, secure defaults

**Developer Template**
- Icon: 💻
- Category: Development
- Features: Developer tools, web development bookmarks, debugging settings

#### Usage Example

```rust
use vantisweb::ui::ProfileTemplateUI;
use vantisweb::profiles::TemplateManager;

// Create template UI
let mut template_ui = ProfileTemplateUI::new();

// Load templates from template manager
let templates = template_manager.get_all_templates().await;
template_ui.set_templates(templates);

// Select a template
template_ui.select_template("work-template-id".to_string());

// Get selected template
if let Some(template) = template_ui.get_selected_template() {
    println!("Selected template: {}", template.name);
}

// Render UI
let html = template_ui.render();
```

---

### 3. ProfileSyncUI

Interface for configuring profile synchronization across devices.

#### Features
- Sync provider selection (Local, Custom)
- Sync interval configuration
- Auto-sync toggle
- Sync on startup option
- Real-time sync status display
- Manual sync trigger
- Conflict resolution interface
- Backup and restore functionality

#### Sync Providers

**Local Provider**
- Syncs profiles on the same device
- No external dependencies
- Fast and reliable

**Custom Provider**
- Supports custom sync endpoints
- Cloud synchronization
- Cross-device sync

#### Usage Example

```rust
use vantisweb::ui::ProfileSyncUI;
use vantisweb::profiles::{SyncConfig, SyncProvider};

// Create sync configuration
let sync_config = SyncConfig {
    provider: SyncProvider::Local,
    sync_interval: 30,
    auto_sync: true,
    sync_on_startup: true,
};

// Create sync UI
let mut sync_ui = ProfileSyncUI::new(sync_config);

// Update sync status
sync_ui.set_sync_status(SyncStatus::Synced);

// Show sync dialog
sync_ui.show_sync_dialog();

// Render UI
let html = sync_ui.render();
```

#### UI Elements

**Sync Status Badge**
- Synced (green)
- Syncing (blue)
- Error (red)
- Offline (gray)

**Sync Configuration**
- Provider dropdown
- Sync interval input (minutes)
- Auto-sync checkbox
- Sync on startup checkbox

**Sync Actions**
- Sync Now button
- View Conflicts button
- Backup Profile button
- Restore Profile button

**Sync Progress Dialog**
- Progress bar
- Status text
- Cancel button

---

### 4. ProfileAnalyticsUI

Comprehensive analytics dashboard for profile usage statistics.

#### Features
- Time range selection (1 day, 7 days, 30 days, 90 days)
- Summary cards with key metrics
- Daily usage chart
- Top websites chart
- Tab statistics table
- Performance metrics table
- Visual data visualization

#### Metrics Tracked

**Summary Metrics**
- Total time spent
- Websites visited
- Tabs opened
- Average page load time

**Tab Statistics**
- Total tabs opened
- Average tabs per session
- Maximum tabs open

**Performance Metrics**
- Average page load time
- Number of crashes
- Average memory usage
- Average CPU usage

#### Usage Example

```rust
use vantisweb::ui::ProfileAnalyticsUI;
use vantisweb::profiles::{AnalyticsManager, UsageSummary};

// Create analytics UI
let mut analytics_ui = ProfileAnalyticsUI::new();

// Load analytics data
let analytics = analytics_manager.get_analytics("profile-id").await;
analytics_ui.set_analytics(analytics);

// Load usage summary
let summary = analytics_manager.get_usage_summary("profile-id", 7).await;
analytics_ui.set_usage_summary(summary);

// Set time range
analytics_ui.set_time_range("7d".to_string());

// Render UI
let html = analytics_ui.render();
```

#### UI Elements

**Time Range Selector**
- 1 Day button
- 7 Days button
- 30 Days button
- 90 Days button

**Summary Cards**
- Total Time (with icon)
- Websites Visited (with icon)
- Tabs Opened (with icon)
- Average Load Time (with icon)

**Charts**
- Daily Usage chart
- Top Websites chart

**Details Tables**
- Tab Statistics table
- Performance Metrics table

---

### 5. ProfileSecurityUI

Interface for configuring profile security settings.

#### Features
- Security level selection (None, Low, Medium, High)
- Authentication method configuration
- Auto-lock settings
- Lock timeout configuration
- Profile data encryption
- Password change dialog
- Biometric authentication setup
- Failed attempts viewing

#### Security Levels

**None**
- No security restrictions
- No authentication required
- No data encryption

**Low**
- Basic password protection
- Optional data encryption
- No auto-lock

**Medium**
- Password authentication required
- Data encryption enabled
- Auto-lock after inactivity

**High**
- Two-factor authentication
- Full data encryption
- Strict auto-lock settings
- Failed attempt tracking

#### Authentication Methods

**None**
- No authentication

**Password**
- Password-based authentication
- BLAKE3 hashing

**Biometric**
- Fingerprint or face recognition
- Encrypted biometric data

**TwoFactor**
- Password + second factor
- Enhanced security

#### Usage Example

```rust
use vantisweb::ui::ProfileSecurityUI;
use vantisweb::profiles::{ProfileSecurity, SecurityLevel, AuthMethod};

// Create security configuration
let security = ProfileSecurity {
    security_level: SecurityLevel::Medium,
    auth_method: AuthMethod::Password,
    auto_lock: true,
    lock_timeout: 5,
    encrypt_data: true,
    failed_attempts: 0,
    max_attempts: 5,
    locked_until: None,
};

// Create security UI
let mut security_ui = ProfileSecurityUI::new();
security_ui.set_security(security);

// Show password dialog
security_ui.show_password_dialog();

// Render UI
let html = security_ui.render();
```

#### UI Elements

**Security Status Badge**
- None (gray)
- Low (yellow)
- Medium (blue)
- High (green)

**Security Configuration**
- Security level dropdown
- Authentication method dropdown
- Auto-lock checkbox
- Lock timeout input (minutes)
- Encrypt data checkbox

**Security Actions**
- Change Password button
- Setup Biometric button
- View Failed Attempts button

**Password Change Dialog**
- Current password input
- New password input
- Confirm password input
- Password strength indicator
- Cancel and confirm buttons

**Biometric Setup Dialog**
- Setup instructions
- Step-by-step guide
- Scan status
- Start scan button

---

## Integration with Backend

### Connecting to ProfileManager

```rust
use vantisweb::ui::ProfileManagerUI;
use vantisweb::profiles::ProfileManager;

// Initialize profile manager
let kernel = Arc::new(VantisKernel::new().await?);
let profile_manager = Arc::new(ProfileManager::new(kernel)?);
profile_manager.initialize().await?;

// Create UI
let mut profile_ui = ProfileManagerUI::new();

// Load profiles
let profiles = profile_manager.get_all_profiles().await;
profile_ui.set_profiles(profiles);

// Set active profile
if let Some(active) = profile_manager.get_active_profile().await {
    profile_ui.set_active_profile(active.id);
}
```

### Handling UI Events

```rust
// Example: Create profile
async fn handle_create_profile(
    profile_manager: &ProfileManager,
    name: String,
    profile_type: ProfileType,
    icon: Option<String>,
    color: Option<String>,
) -> Result<()> {
    let mut profile = ProfileConfig::new(name, profile_type);
    profile.icon = icon;
    profile.color = color;
    
    profile_manager.add_profile(profile).await?;
    Ok(())
}

// Example: Activate profile
async fn handle_activate_profile(
    profile_manager: &ProfileManager,
    profile_id: String,
) -> Result<()> {
    profile_manager.set_active_profile(&profile_id).await?;
    Ok(())
}

// Example: Delete profile
async fn handle_delete_profile(
    profile_manager: &ProfileManager,
    profile_id: String,
) -> Result<()> {
    profile_manager.delete_profile(&profile_id).await?;
    Ok(())
}
```

---

## Styling

The Profile UI components use the CSS file `profile_ui.css` which includes:

- Modern, clean design
- Responsive layout
- Smooth animations and transitions
- Card-based components
- Modal dialogs
- Form elements
- Button styles
- Color-coded status indicators

### Customization

You can customize the appearance by modifying the CSS variables and styles:

```css
/* Primary color */
--primary-color: #6366f1;

/* Success color */
--success-color: #10b981;

/* Danger color */
--danger-color: #ef4444;

/* Warning color */
--warning-color: #f59e0b;

/* Border radius */
--border-radius: 12px;

/* Box shadow */
--box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
```

---

## Testing

All Profile UI components include unit tests:

```rust
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
```

Run tests with:
```bash
cargo test --lib ui::profile_ui
```

---

## Best Practices

1. **Always load profiles from ProfileManager** before rendering UI
2. **Handle errors gracefully** when profile operations fail
3. **Update UI state** after backend operations complete
4. **Use async/await** for all profile manager operations
5. **Validate user input** before creating or updating profiles
6. **Show loading states** during async operations
7. **Provide feedback** for user actions (success/error messages)
8. **Use confirmation dialogs** for destructive operations (delete)
9. **Keep UI responsive** by using non-blocking operations
10. **Test thoroughly** with different profile configurations

---

## Future Enhancements

Potential improvements for the Profile UI:

1. **Drag and Drop** profile reordering
2. **Profile Import/Export** from files
3. **Profile Cloning** with settings
4. **Profile Merging** functionality
5. **Advanced Analytics** with more charts
6. **Profile Comparison** tool
7. **Bulk Operations** on multiple profiles
8. **Profile Templates** from community
9. **Profile Sharing** between users
10. **AI-powered** profile recommendations

---

## Support

For issues, questions, or contributions related to Profile UI components, please refer to the main VantisWeb documentation or open an issue on GitHub.