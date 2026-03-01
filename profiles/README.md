# VantisWeb Profiles

This directory contains profile templates and configuration for VantisWeb.

## Profile Templates

Predefined profile templates for different use cases:

### Work Profile
- Optimized for productivity
- Google search engine
- Start pages: Google, GitHub, Stack Overflow
- Tracker and ad blocking enabled
- Light theme
- Extensions: Password Manager, Productivity Tracker

### Gaming Profile
- Optimized for gaming and streaming
- High CPU priority
- 4GB memory limit
- Start pages: Twitch, YouTube, Discord
- Dark theme
- Extensions: Twitch Enhancer, Discord RPC

### Privacy Profile
- Maximum privacy and security
- DuckDuckGo search engine
- Clear cookies and history on exit
- Private mode by default
- Tracker and ad blocking enabled
- Dark theme
- Extensions: Privacy Badger, HTTPS Everywhere, uBlock Origin

### Developer Profile
- Optimized for web development
- High CPU priority
- 8GB memory limit
- Custom keyboard shortcuts for DevTools
- Start pages: GitHub, Stack Overflow, MDN
- Dark theme
- Extensions: React DevTools, Vue DevTools, Redux DevTools, Lighthouse

## Profile Features

### Templates
- Predefined configurations for common use cases
- Customizable settings
- Import/export functionality

### Synchronization
- Cloud sync across devices
- Local backup/restore
- Conflict resolution

### Analytics
- Usage statistics
- Time tracking
- Top websites
- Performance metrics

### Security
- Password protection
- Biometric authentication
- Profile encryption
- Auto-lock functionality

## Creating Custom Profiles

1. Use the ProfileManager API
2. Start from a template or create from scratch
3. Customize settings as needed
4. Save and activate the profile

## Profile Structure

```
profile-id/
├── profile.json          # Profile configuration
├── bookmarks.json        # Profile bookmarks
├── history.json          # Browsing history
├── settings.json         # Profile settings
├── extensions/           # Profile-specific extensions
└── data/                 # Profile data
```

## API Usage

### Creating a Profile from Template

```rust
use vanisweb::profiles::{ProfileManager, TemplateManager};

let template_manager = TemplateManager::new();
let template = template_manager.get("work").unwrap();

// Create profile from template
let profile = ProfileConfig::new("My Work Profile".to_string(), ProfileType::Work);
profile.settings = template.settings.clone();
profile.extensions = template.extensions.clone();
profile.theme = template.theme.clone();

manager.add_profile(profile).await?;
```

### Syncing Profiles

```rust
use vanisweb::profiles::ProfileSyncManager;

let mut sync_manager = ProfileSyncManager::new(profiles_dir);
sync_manager.enable(SyncProvider::Local)?;
sync_manager.sync()?;
```

### Profile Analytics

```rust
use vanisweb::profiles::AnalyticsManager;

let mut analytics_manager = AnalyticsManager::new();
analytics_manager.start_session(profile_id)?;
// ... use profile ...
analytics_manager.end_session(profile_id)?;

let summary = analytics_manager.get_usage_summary(profile_id)?;
println!("Total time: {} seconds", summary.total_time);
```

### Profile Security

```rust
use vanisweb::profiles::{ProfileSecurityManager, SecurityLevel};

let mut security_manager = ProfileSecurityManager::new()?;
security_manager.create_security(profile_id, SecurityLevel::Medium)?;
security_manager.set_password(profile_id, "secure_password")?;

// Authenticate
let verified = security_manager.authenticate(
    profile_id,
    &AuthMethod::Password,
    "secure_password"
)?;
```

## Security Best Practices

1. Use strong passwords for profile protection
2. Enable biometric authentication when available
3. Use appropriate security levels:
   - None: No security (not recommended)
   - Low: Password only
   - Medium: Password + encryption
   - High: Password + encryption + biometric
4. Enable auto-lock for sensitive profiles
5. Regularly backup profiles

## Troubleshooting

### Profile Not Loading
- Check profile.json syntax
- Verify profile ID is correct
- Check file permissions

### Sync Issues
- Verify sync provider configuration
- Check network connection
- Resolve conflicts manually

### Security Issues
- Reset password if forgotten
- Clear failed attempts after lockout
- Re-enable security if needed

## Support

For more information, see:
- [API Reference](../docs/API_REFERENCE.md)
- [GitHub Issues](https://github.com/vantisCorp/VantisWeb/issues)
- [Documentation](https://github.com/vantisCorp/VantisWeb/wiki)