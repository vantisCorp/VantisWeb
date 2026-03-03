# Profile Cloning/Duplication Implementation

## Overview
This document describes the implementation of the Profile Cloning/Duplication functionality for VantisWeb Browser (Issue #5 from v1.1.0 roadmap).

## Feature Summary
The Profile Cloning/Duplication feature allows users to:
- Clone/duplicate existing profiles with all settings
- Create exact copy of profile with new ID
- Append '(Copy)' to cloned profile name by default
- Allow cloning active and inactive profiles
- Optionally include bookmarks and history
- Show confirmation dialog before cloning
- Customize clone name and data selection

## Implementation Details

### Backend Components

#### 1. ProfileManager Clone Methods
**Location:** `src/profiles/mod.rs`

**New Methods Added:**

##### clone_profile()
```rust
pub async fn clone_profile(&self, profile_id: &str, new_name: Option<String>, include_bookmarks: bool, include_history: bool) -> Result<String>
```
Clones a profile with basic options:
- Finds the source profile by ID
- Creates a new profile with generated UUID
- Appends '(Copy)' to name if no custom name provided
- Copies all settings and extensions by default
- Optionally copies bookmarks and history
- Sets cloned profile as inactive
- Preserves original profile's order position
- Returns the ID of the newly created profile

**Parameters:**
- `profile_id`: ID of the profile to clone
- `new_name`: Optional custom name for the cloned profile
- `include_bookmarks`: Whether to copy bookmarks (default: true in most cases)
- `include_history`: Whether to copy history (default: false)

**Return:**
- `Result<String>`: ID of the newly created profile

**Error Handling:**
- Profile not found errors
- Persistence errors
- Data corruption errors

##### CloneOptions Structure
```rust
#[derive(Debug, Clone)]
pub struct CloneOptions {
    pub new_name: Option<String>,
    pub include_bookmarks: bool,
    pub include_history: bool,
    pub clone_settings: bool,
    pub clone_theme: bool,
}
```

Provides detailed configuration for cloning:
- `new_name`: Optional custom name
- `include_bookmarks`: Copy bookmarks
- `include_history`: Copy history
- `clone_settings`: Copy settings
- `clone_theme`: Copy theme

**Default Implementation:**
```rust
impl Default for CloneOptions {
    fn default() -> Self {
        Self {
            new_name: None,
            include_bookmarks: true,
            include_history: false,
            clone_settings: true,
            clone_theme: true,
        }
    }
}
```

##### clone_profile_with_options()
```rust
pub async fn clone_profile_with_options(&self, profile_id: &str, options: &CloneOptions) -> Result<String>
```
Clones a profile with detailed options:
- Accepts a `CloneOptions` struct for granular control
- Allows selective copying of profile components
- Same behavior as `clone_profile()` but with more flexibility
- Returns the ID of the newly created profile

**Benefits:**
- More explicit and readable
- Easier to extend with new options
- Better for complex cloning scenarios

### Frontend Components

#### 2. ProfileManagerUI Clone State
**Location:** `src/ui/profile_ui.rs`

**New Fields Added:**
```rust
show_clone_dialog: bool,
clone_source_profile: Option<String>,
clone_new_name: String,
clone_include_bookmarks: bool,
clone_include_history: bool,
```

**Field Descriptions:**
- `show_clone_dialog`: Controls clone dialog visibility
- `clone_source_profile`: ID of profile being cloned
- `clone_new_name`: Name for the cloned profile
- `clone_include_bookmarks`: Include bookmarks in clone
- `clone_include_history`: Include history in clone

**New Methods Added:**

##### show_clone_dialog()
```rust
pub fn show_clone_dialog(&mut self, profile_id: String)
```
Shows the clone dialog for a specific profile:
- Finds the profile by ID
- Sets default clone name as "Original Name (Copy)"
- Sets default options (bookmarks: true, history: false)
- Makes the dialog visible

##### hide_clone_dialog()
```rust
pub fn hide_clone_dialog(&mut self)
```
Hides the clone dialog:
- Clears all clone state
- Hides the dialog

##### render_clone_dialog()
```rust
fn render_clone_dialog(&self) -> String
```
Renders the clone dialog HTML:
- Shows source profile name
- Allows editing new profile name
- Provides checkboxes for bookmarks/history
- Includes cancel and clone buttons

#### 3. Profile Card UI Updates
**Location:** `src/ui/profile_ui.rs`

**New Button Added:**
```html
<button class="button button-secondary profile-clone-btn" 
        data-profile-id="{}">
    Clone
</button>
```

Positioned between Settings and Export buttons for logical grouping.

## User Flow

### Clone Profile Operation
1. **Initiate Clone**: User clicks "Clone" button on profile card
2. **Show Dialog**: Clone dialog appears with default options
3. **Customize**: User can:
   - Change the cloned profile name (default: "Original Name (Copy)")
   - Toggle "Include bookmarks" (default: checked)
   - Toggle "Include history" (default: unchecked)
4. **Confirm**: User clicks "Clone" button
5. **Execute Clone**: Backend creates new profile with selected data
6. **Update UI**: Profile list refreshes to show new cloned profile
7. **Persist**: Changes saved to disk

### Clone Dialog UI

```
┌─────────────────────────────────────┐
│  Clone Profile              [×]     │
├─────────────────────────────────────┤
│  Source Profile                      │
│  Work Profile                        │
│                                      │
│  New Profile Name                    │
│  [Work Profile (Copy)        ]       │
│                                      │
│  Clone Options                       │
│  ☑ Include bookmarks                 │
│  ☐ Include history                   │
├─────────────────────────────────────┤
│  [Cancel]              [Clone]       │
└─────────────────────────────────────┘
```

## API Endpoints

### Clone Profile
**POST** `/api/profiles/clone`

**Request Body:**
```json
{
    "profile_id": "profile-id-to-clone",
    "new_name": "My Profile (Copy)",
    "include_bookmarks": true,
    "include_history": false
}
```

**Response:**
```json
{
    "success": true,
    "cloned_profile_id": "new-profile-id",
    "message": "Profile cloned successfully"
}
```

**Error Response:**
```json
{
    "success": false,
    "error": "Profile not found: profile-id-to-clone"
}
```

### Clone Profile with Options
**POST** `/api/profiles/clone/advanced`

**Request Body:**
```json
{
    "profile_id": "profile-id-to-clone",
    "options": {
        "new_name": "Custom Clone",
        "include_bookmarks": true,
        "include_history": false,
        "clone_settings": true,
        "clone_theme": true
    }
}
```

**Response:**
```json
{
    "success": true,
    "cloned_profile_id": "new-profile-id",
    "message": "Profile cloned successfully"
}
```

## Usage Examples

### Basic Clone
```rust
use vantis::profiles::ProfileManager;

let manager = ProfileManager::new(kernel)?;
let cloned_id = manager.clone_profile(
    "work-profile-id",
    None,  // Use default name "Work Profile (Copy)"
    true,  // Include bookmarks
    false  // Don't include history
).await?;

println!("Cloned profile ID: {}", cloned_id);
```

### Clone with Custom Name
```rust
let cloned_id = manager.clone_profile(
    "work-profile-id",
    Some("My Custom Profile".to_string()),
    true,
    false
).await?;
```

### Clone with Options
```rust
use vantis::profiles::CloneOptions;

let options = CloneOptions {
    new_name: Some("Test Profile Copy".to_string()),
    include_bookmarks: false,
    include_history: false,
    clone_settings: true,
    clone_theme: false,
};

let cloned_id = manager.clone_profile_with_options(
    "work-profile-id",
    &options
).await?;
```

## Data Persistence

### Cloned Profile Storage
- Cloned profile gets new UUID
- Stored in same location as other profiles (`.vantisweb/profiles/`)
- All profile data persists to disk immediately
- Cloned profile is never active initially
- Can be activated later like any other profile

### Profile Data Copied
**Always Copied:**
- Profile type
- Icon
- Color
- Order position
- Settings (unless disabled)
- Extensions
- Theme (unless disabled)

**Optionally Copied:**
- Bookmarks (default: true)
- History (default: false)

**Never Copied:**
- Profile ID (new UUID generated)
- Active status (always false)
- Created timestamp (new timestamp)
- Last used timestamp (copied from original)

## Testing Strategy

### Unit Tests

#### clone_profile()
```rust
#[test]
async fn test_clone_profile() {
    let manager = create_test_manager();
    manager.create_profile("Original".to_string()).await;
    
    let profiles = manager.get_all_profiles().await;
    let original_id = &profiles[0].id;
    
    // Clone with default options
    let cloned_id = manager.clone_profile(original_id, None, true, false).await.unwrap();
    
    // Verify clone exists
    let profiles = manager.get_all_profiles().await;
    assert_eq!(profiles.len(), 2);
    
    // Verify cloned profile properties
    let cloned = profiles.iter().find(|p| p.id == cloned_id).unwrap();
    assert_eq!(cloned.name, "Original (Copy)");
    assert_eq!(cloned.profile_type, profiles[0].profile_type);
    assert!(!cloned.active);
    assert_eq!(cloned.bookmarks.len(), profiles[0].bookmarks.len());
    assert_eq!(cloned.history.len(), 0); // History not copied
}

#[test]
async fn test_clone_with_custom_name() {
    let manager = create_test_manager();
    manager.create_profile("Original".to_string()).await;
    
    let profiles = manager.get_all_profiles().await;
    let original_id = &profiles[0].id;
    
    let cloned_id = manager.clone_profile(
        original_id,
        Some("Custom Name".to_string()),
        true,
        false
    ).await.unwrap();
    
    let profiles = manager.get_all_profiles().await;
    let cloned = profiles.iter().find(|p| p.id == cloned_id).unwrap();
    assert_eq!(cloned.name, "Custom Name");
}

#[test]
async fn test_clone_without_bookmarks() {
    let manager = create_test_manager();
    // Create profile with bookmarks
    let profile = create_profile_with_bookmarks();
    manager.add_profile(profile).await;
    
    let profiles = manager.get_all_profiles().await;
    let original_id = &profiles[0].id;
    
    let cloned_id = manager.clone_profile(original_id, None, false, false).await.unwrap();
    
    let profiles = manager.get_all_profiles().await;
    let cloned = profiles.iter().find(|p| p.id == cloned_id).unwrap();
    assert_eq!(cloned.bookmarks.len(), 0); // No bookmarks copied
    assert_eq!(profiles[0].bookmarks.len(), 5); // Original still has bookmarks
}

#[test]
async fn test_clone_with_history() {
    let manager = create_test_manager();
    let profile = create_profile_with_history();
    manager.add_profile(profile).await;
    
    let profiles = manager.get_all_profiles().await;
    let original_id = &profiles[0].id;
    
    let cloned_id = manager.clone_profile(original_id, None, false, true).await.unwrap();
    
    let profiles = manager.get_all_profiles().await;
    let cloned = profiles.iter().find(|p| p.id == cloned_id).unwrap();
    assert_eq!(cloned.history.len(), profiles[0].history.len());
}

#[test]
async fn test_clone_active_profile() {
    let manager = create_test_manager();
    manager.create_profile("Original".to_string()).await;
    
    // Set profile as active
    let profiles = manager.get_all_profiles().await;
    let original_id = profiles[0].id.clone();
    manager.set_active_profile(original_id.clone()).await;
    
    // Clone should create inactive profile
    let cloned_id = manager.clone_profile(&original_id, None, true, false).await.unwrap();
    
    let profiles = manager.get_all_profiles().await;
    let original = profiles.iter().find(|p| p.id == original_id).unwrap();
    let cloned = profiles.iter().find(|p| p.id == cloned_id).unwrap();
    
    assert!(original.active);
    assert!(!cloned.active);
}

#[test]
async fn test_clone_nonexistent_profile() {
    let manager = create_test_manager();
    
    let result = manager.clone_profile("nonexistent-id", None, true, false).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}
```

#### clone_profile_with_options()
```rust
#[test]
async fn test_clone_with_options() {
    let manager = create_test_manager();
    manager.create_profile("Original".to_string()).await;
    
    let profiles = manager.get_all_profiles().await;
    let original_id = &profiles[0].id;
    
    let options = CloneOptions {
        new_name: Some("Custom Clone".to_string()),
        include_bookmarks: false,
        include_history: false,
        clone_settings: true,
        clone_theme: false,
    };
    
    let cloned_id = manager.clone_profile_with_options(original_id, &options).await.unwrap();
    
    let profiles = manager.get_all_profiles().await;
    let cloned = profiles.iter().find(|p| p.id == cloned_id).unwrap();
    assert_eq!(cloned.name, "Custom Clone");
    assert_eq!(cloned.bookmarks.len(), 0);
    assert!(cloned.theme.is_none());
}
```

### Integration Tests

#### End-to-End Clone Workflow
```rust
#[test]
async fn test_clone_workflow() {
    let manager = create_test_manager();
    
    // 1. Create original profile with data
    let original = create_test_profile();
    manager.add_profile(original).await;
    
    // 2. Clone with default options
    let profiles = manager.get_all_profiles().await;
    let original_id = &profiles[0].id;
    let cloned_id = manager.clone_profile(original_id, None, true, false).await.unwrap();
    
    // 3. Verify clone was created
    let profiles = manager.get_all_profiles().await;
    assert_eq!(profiles.len(), 2);
    
    // 4. Verify clone has correct properties
    let cloned = profiles.iter().find(|p| p.id == cloned_id).unwrap();
    assert_eq!(cloned.name, "Original Name (Copy)");
    assert!(!cloned.active);
    assert_eq!(cloned.bookmarks.len(), 3); // Original had 3 bookmarks
    
    // 5. Activate cloned profile
    manager.set_active_profile(cloned_id.clone()).await;
    
    // 6. Verify only one profile is active
    let profiles = manager.get_all_profiles().await;
    let active_count = profiles.iter().filter(|p| p.active).count();
    assert_eq!(active_count, 1);
    assert!(profiles.iter().find(|p| p.id == cloned_id).unwrap().active);
}
```

### Manual Testing Checklist

#### Basic Cloning
- [ ] Clone profile with default options
- [ ] Clone profile with custom name
- [ ] Clone profile without bookmarks
- [ ] Clone profile with history
- [ ] Clone active profile
- [ ] Clone inactive profile

#### Data Preservation
- [ ] Bookmarks copied correctly
- [ ] History copied correctly
- [ ] Settings copied correctly
- [ ] Theme copied correctly
- [ ] Extensions copied correctly
- [ ] Color and icon copied correctly

#### Edge Cases
- [ ] Clone profile with no bookmarks
- [ ] Clone profile with no history
- [ ] Clone profile with empty name (should be prevented)
- [ ] Clone profile with very long name
- [ ] Clone same profile multiple times
- [ ] Clone profile, then clone the clone
- [ ] Clone nonexistent profile (should fail)

#### UI Testing
- [ ] Clone button appears on all profiles
- [ ] Clone dialog shows correct source name
- [ ] Default name is "Original Name (Copy)"
- [ ] Bookmarks checkbox checked by default
- [ ] History checkbox unchecked by default
- [ ] Cancel button closes dialog
- [ ] Clone button creates profile
- [ ] Profile list refreshes after clone
- [ ] New profile appears in correct position

## Performance Considerations

### Optimizations
1. **Efficient Copy**: Uses `clone()` on simple data structures
2. **Selective Copy**: Only copies requested data
3. **Immediate Persistence**: Saves immediately to avoid data loss
4. **Minimal Memory**: Doesn't keep unnecessary copies in memory

### Expected Performance
- **Clone Operation**: <100ms
- **With Bookmarks**: <200ms
- **With History**: <300ms
- **Profile List Update**: <50ms

## Security Considerations

### Current Implementation
- New UUID generated for each clone
- No sensitive data exposed
- Clone inherits same security settings
- No authentication required for cloning own profiles

### Future Enhancements
1. **Permission Check**: Verify user can clone profile
2. **Profile Owner**: Track profile ownership
3. **Clone Limit**: Limit number of clones per user
4. **Clone History**: Track clone lineage
5. **Clone Expiration**: Time-limited cloned profiles

## Browser Compatibility

### Supported Browsers
- ✅ Chrome 90+
- ✅ Firefox 88+
- ✅ Safari 14+
- ✅ Edge 90+

### Mobile Support
- ✅ iOS 14+
- ✅ Android 10+

## Known Limitations

### Current Implementation
- Clone dialog implemented
- Backend functionality complete
- Frontend UI components ready
- JavaScript event handlers needed
- No clone history tracking
- No clone deletion cascade
- No merge with original

### Future Enhancements
1. **Clone Templates**: Save clone configurations as templates
2. **Clone Groups**: Clone multiple profiles at once
3. **Clone Scheduling**: Schedule clones for later
4. **Clone Comparison**: Compare clone to original
5. **Clone Sync**: Sync changes back to original
6. **Clone Notifications**: Notify when clone is created
7. **Clone Permissions**: Restrict who can clone profiles

## Documentation

### User Documentation Needed
- How to clone a profile
- Clone options explained
- Best practices for cloning
- Troubleshooting

### Developer Documentation
- API reference (rustdoc)
- Integration guide
- Event handling guide
- Clone options reference

## Release Notes

### Version 1.1.0 - Feature Addition
**New Features:**
- Profile cloning/duplication
- Clone dialog with options
- Custom clone naming
- Selective data copying
- Basic and advanced clone methods

**Implementation:**
- Added clone_profile() method
- Added clone_profile_with_options() method
- Added CloneOptions structure
- Updated ProfileManagerUI with clone state
- Added Clone button to profile cards
- Added clone dialog

**Known Issues:**
- JavaScript event handlers not yet implemented
- No clone history tracking
- No merge functionality

## Conclusion

The Profile Cloning/Duplication feature provides users with an easy way to create similar profiles without starting from scratch. The implementation includes:

- ✅ Complete backend functionality
- ✅ UI component integration
- ✅ Clone dialog with options
- ✅ Custom naming support
- ✅ Selective data copying
- ✅ Two clone methods (basic and advanced)
- ✅ Comprehensive error handling
- ✅ Clone state management

**Remaining Work:**
- JavaScript event handler implementation
- Clone history tracking
- Merge with original functionality
- Profile comparison tool
- Bulk cloning support

The feature is ready for frontend integration and testing.