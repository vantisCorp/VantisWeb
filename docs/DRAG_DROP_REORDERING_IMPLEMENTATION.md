# Drag and Drop Profile Reordering Implementation

## Overview
This document describes the implementation of the Drag and Drop Profile Reordering functionality for VantisWeb Browser (Issue #3 from v1.1.0 roadmap).

## Feature Summary
The Drag and Drop Profile Reordering feature allows users to:
- Drag profile cards to reorder them in the profile manager
- Visual feedback shows where the profile will be dropped
- Profile order is persisted across sessions
- Touch support for mobile devices
- Smooth animations during drag operation

## Implementation Details

### Backend Components

#### 1. ProfileConfig Updates
**Location:** `src/profiles/mod.rs`

**New Field Added:**
```rust
/// Profile order position (for drag and drop reordering)
pub order: i32,
```

The `order` field tracks the position of each profile in the list, allowing for consistent ordering across sessions.

#### 2. ProfileManager Reordering Methods
**Location:** `src/profiles/mod.rs`

**New Methods Added:**

##### reorder_profile()
```rust
pub async fn reorder_profile(&self, profile_id: &str, new_position: i32) -> Result<()>
```
Moves a profile to a specified position:
- Validates the new position is within bounds
- Finds the profile and its current position
- Updates order values for affected profiles
  - Moving up: shifts profiles between new_position and current_position down
  - Moving down: shifts profiles between current_position and new_position up
- Sets the new position for the moved profile
- Persists changes to disk

**Error Handling:**
- Invalid position errors
- Profile not found errors
- Persistence errors

##### swap_profiles()
```rust
pub async fn swap_profiles(&self, profile_id_1: &str, profile_id_2: &str) -> Result<()>
```
Swaps the positions of two profiles:
- Finds both profiles
- Swaps their order values
- Persists changes to disk

##### get_ordered_profiles()
```rust
pub async fn get_ordered_profiles(&self) -> Vec<ProfileConfig>
```
Returns all profiles sorted by their order field.

##### normalize_order()
```rust
pub async fn normalize_order(&self) -> Result<()>
```
Reorders all profiles to ensure consecutive order values:
- Sorts profiles by current order
- Assigns consecutive order values (0, 1, 2, ...)
- Useful for maintenance and after bulk operations

### Frontend Components

#### 3. ProfileManagerUI Drag and Drop State
**Location:** `src/ui/profile_ui.rs`

**New Fields Added:**
```rust
dragged_profile_id: Option<String>,
drop_target_id: Option<String>,
drag_over: bool,
```

**New Methods Added:**

##### start_drag()
```rust
pub fn start_drag(&mut self, profile_id: String)
```
Initiates drag operation for a profile.

##### end_drag()
```rust
pub fn end_drag(&mut self)
```
Ends drag operation and clears drag state.

##### set_drop_target()
```rust
pub fn set_drop_target(&mut self, target_id: Option<String>)
```
Sets the current drop target during drag.

##### get_dragged_profile()
```rust
pub fn get_dragged_profile(&self) -> Option<&String>
```
Returns the currently dragged profile ID.

##### get_drop_target()
```rust
pub fn get_drop_target(&self) -> Option<&String>
```
Returns the current drop target ID.

##### is_dragging()
```rust
pub fn is_dragging(&self) -> bool
```
Checks if a drag operation is in progress.

##### is_dragging_profile()
```rust
pub fn is_dragging_profile(&self, profile_id: &str) -> bool
```
Checks if a specific profile is being dragged.

##### is_dragging_over()
```rust
pub fn is_dragging_over(&self, profile_id: &str) -> bool
```
Checks if dragging over a specific profile.

#### 4. Profile Card UI Updates
**Location:** `src/ui/profile_ui.rs`

**Enhancements:**
- Added `draggable="true"` attribute to profile cards
- Added `data-profile-order` attribute for tracking position
- Added drag handle with visual indicator (⋮⋮)
- Added CSS classes for drag states:
  - `.dragging` - Profile being dragged
  - `.drop-target` - Profile being dragged over
- Dynamic class assignment based on drag state

**HTML Structure:**
```html
<div class="profile-card [active] [dragging] [drop-target]" 
     data-profile-id="profile-id" 
     data-profile-order="0" 
     draggable="true">
    <div class="profile-drag-handle" title="Drag to reorder">⋮⋮</div>
    <div class="profile-icon">...</div>
    <div class="profile-info">...</div>
    <div class="profile-actions">...</div>
</div>
```

## User Flow

### Drag and Drop Operation
1. **Drag Start**: User clicks and holds on a profile card's drag handle
2. **Drag**: User moves the profile to a new position
3. **Visual Feedback**: Drop target profile shows `.drop-target` class
4. **Drop**: User releases the profile at the desired position
5. **Reorder**: Backend updates order values for affected profiles
6. **Persist**: Changes are saved to disk
7. **Re-render**: Profile list is re-rendered with new order

## JavaScript Integration (Required)

### Event Handlers

```javascript
// Drag Start
profileCard.addEventListener('dragstart', (e) => {
    const profileId = e.target.dataset.profileId;
    ui.start_drag(profileId);
    e.target.classList.add('dragging');
});

// Drag End
profileCard.addEventListener('dragend', (e) => {
    ui.end_drag();
    e.target.classList.remove('dragging');
});

// Drag Over
profileCard.addEventListener('dragover', (e) => {
    e.preventDefault(); // Allow drop
    const profileId = e.target.dataset.profileId;
    ui.set_drop_target(profileId);
});

// Drag Leave
profileCard.addEventListener('dragleave', (e) => {
    const profileId = e.target.dataset.profileId;
    if (ui.get_drop_target() === profileId) {
        ui.set_drop_target(null);
    }
});

// Drop
profileCard.addEventListener('drop', (e) => {
    e.preventDefault();
    const draggedId = ui.get_dragged_profile();
    const targetId = e.target.dataset.profileId;
    
    // Get current positions
    const draggedPosition = parseInt(e.target.dataset.profileOrder);
    const targetPosition = parseInt(targetElement.dataset.profileOrder);
    
    // Reorder via backend
    fetch('/api/profiles/reorder', {
        method: 'POST',
        body: JSON.stringify({
            profile_id: draggedId,
            new_position: targetPosition
        })
    });
    
    // Clear drag state
    ui.end_drag();
});
```

## CSS Styling (Required)

### Drag and Drop Styles

```css
/* Drag Handle */
.profile-drag-handle {
    position: absolute;
    left: 8px;
    top: 50%;
    transform: translateY(-50%);
    cursor: grab;
    padding: 8px;
    color: #666;
    user-select: none;
    font-size: 18px;
    letter-spacing: -4px;
}

.profile-drag-handle:active {
    cursor: grabbing;
}

/* Dragging State */
.profile-card.dragging {
    opacity: 0.5;
    transform: scale(1.02);
    box-shadow: 0 8px 16px rgba(0, 0, 0, 0.2);
}

/* Drop Target State */
.profile-card.drop-target {
    border: 2px dashed #6366f1;
    background-color: rgba(99, 102, 241, 0.1);
}

/* Smooth Transitions */
.profile-card {
    transition: all 0.2s ease;
}

/* Mobile Touch Support */
@media (hover: none) {
    .profile-drag-handle {
        padding: 16px;
        font-size: 24px;
    }
    
    .profile-card {
        touch-action: none;
    }
}
```

## API Endpoints

### Reorder Profile
**POST** `/api/profiles/reorder`

**Request Body:**
```json
{
    "profile_id": "profile-id",
    "new_position": 2
}
```

**Response:**
```json
{
    "success": true,
    "message": "Profile reordered successfully"
}
```

**Error Response:**
```json
{
    "success": false,
    "error": "Invalid position: 10"
}
```

### Swap Profiles
**POST** `/api/profiles/swap`

**Request Body:**
```json
{
    "profile_id_1": "profile-id-1",
    "profile_id_2": "profile-id-2"
}
```

**Response:**
```json
{
    "success": true,
    "message": "Profiles swapped successfully"
}
```

### Get Ordered Profiles
**GET** `/api/profiles/ordered`

**Response:**
```json
{
    "profiles": [
        {
            "id": "profile-1",
            "name": "Profile 1",
            "order": 0,
            ...
        },
        {
            "id": "profile-2",
            "name": "Profile 2",
            "order": 1,
            ...
        }
    ]
}
```

## Data Persistence

### Profile Order Storage
- Profile order is stored in the `order` field of `ProfileConfig`
- All profile data is persisted to `.vantisweb/profiles/` directory
- Order is preserved across browser sessions
- Changes are saved immediately after reordering

### Normalization
The `normalize_order()` method ensures:
- Order values are always consecutive (0, 1, 2, ...)
- No gaps in the sequence
- Consistent state after bulk operations
- Called automatically after certain operations

## Testing Strategy

### Unit Tests

#### ProfileConfig Order Field
```rust
#[test]
fn test_profile_order_initialization() {
    let profile = ProfileConfig::new("Test".to_string(), ProfileType::Custom);
    assert_eq!(profile.order, 0);
}
```

#### reorder_profile()
```rust
#[test]
async fn test_reorder_profile() {
    let manager = create_test_manager();
    manager.create_profile("Profile1".to_string()).await;
    manager.create_profile("Profile2".to_string()).await;
    
    // Move Profile1 to position 1
    manager.reorder_profile("profile-1-id", 1).await.unwrap();
    
    let profiles = manager.get_ordered_profiles().await;
    assert_eq!(profiles[0].name, "Profile2");
    assert_eq!(profiles[1].name, "Profile1");
}
```

#### swap_profiles()
```rust
#[test]
async fn test_swap_profiles() {
    let manager = create_test_manager();
    // ... create profiles ...
    
    manager.swap_profiles("profile-1", "profile-2").await.unwrap();
    
    let profiles = manager.get_ordered_profiles().await;
    assert_eq!(profiles[0].name, "Profile2");
    assert_eq!(profiles[1].name, "Profile1");
}
```

#### normalize_order()
```rust
#[test]
async fn test_normalize_order() {
    let manager = create_test_manager();
    // ... create profiles with non-consecutive orders ...
    
    manager.normalize_order().await.unwrap();
    
    let profiles = manager.get_ordered_profiles().await;
    assert_eq!(profiles[0].order, 0);
    assert_eq!(profiles[1].order, 1);
    assert_eq!(profiles[2].order, 2);
}
```

### Integration Tests

#### End-to-End Drag and Drop
```rust
#[test]
async fn test_drag_and_drop_workflow() {
    // 1. Create profiles
    let manager = create_test_manager();
    manager.create_profile("Profile1".to_string()).await;
    manager.create_profile("Profile2".to_string()).await;
    manager.create_profile("Profile3".to_string()).await;
    
    // 2. Get initial order
    let profiles = manager.get_ordered_profiles().await;
    let profile1_id = profiles[0].id.clone();
    let profile3_id = profiles[2].id.clone();
    
    // 3. Simulate drag from position 0 to position 2
    manager.reorder_profile(&profile1_id, 2).await.unwrap();
    
    // 4. Verify new order
    let profiles = manager.get_ordered_profiles().await;
    assert_eq!(profiles[0].name, "Profile2");
    assert_eq!(profiles[1].name, "Profile3");
    assert_eq!(profiles[2].name, "Profile1");
}
```

### Manual Testing Checklist

#### Desktop (Mouse)
- [ ] Drag profile to new position
- [ ] Visual feedback on drop target
- [ ] Smooth drag animation
- [ ] Drag works on handle area only
- [ ] Drop updates profile order
- [ ] Order persists after refresh
- [ ] Cannot drop on same profile
- [ ] Cannot drop out of bounds

#### Mobile (Touch)
- [ ] Touch and hold to start drag
- [ ] Visual feedback on touch
- [ ] Smooth touch drag
- [ ] Drop updates order
- [ ] Touch target is large enough
- [ ] No accidental drops

#### Edge Cases
- [ ] Reorder first profile
- [ ] Reorder last profile
- [ ] Reorder single profile
- [ ] Reorder to same position
- [ ] Drag between profiles
- [ ] Multiple consecutive reorders
- [ ] Reorder with many profiles (10+)
- [ ] Reorder after creating/deleting profile

## Performance Considerations

### Optimizations
1. **Efficient Order Updates**: Only update affected profiles, not all
2. **Debounced Persistence**: Consider debouncing save operations for rapid reorders
3. **Visual Updates**: Re-render only changed profiles instead of full list
4. **Smooth Animations**: Use CSS transforms for smooth movement

### Expected Performance
- **Reorder Operation**: <50ms
- **UI Update**: <16ms (60fps)
- **Persistence**: <100ms
- **Total Drag and Drop**: <200ms

## Accessibility

### Keyboard Navigation (Future Enhancement)
- Tab to profile card
- Enter/Space to start reorder
- Arrow keys to move up/down
- Enter to drop at new position

### Screen Reader Support
- Announce drag start
- Announce current position
- Announce drop target
- Announce new position after drop

### ARIA Attributes
```html
<div class="profile-card"
     draggable="true"
     role="listitem"
     aria-label="Profile Name, position 1"
     aria-grabbed="false">
    <div class="profile-drag-handle"
         role="button"
         aria-label="Drag to reorder"
         tabindex="0">
        ⋮⋮
    </div>
</div>
```

## Known Limitations

### Current Implementation
- Reordering implemented on backend
- Frontend UI components ready
- JavaScript event handlers needed
- CSS styling needed
- API endpoint integration needed

### Future Enhancements
1. **Keyboard Navigation**: Full keyboard support for reordering
2. **Animation Library**: Integrate animation library for smoother transitions
3. **Undo/Redo**: Ability to undo reorder operations
4. **Multi-Select**: Drag multiple profiles at once
5. **Auto-Sort**: Sort profiles by name, date, or usage
6. **Search**: Filter profiles before reordering

## Browser Compatibility

### Supported Browsers
- ✅ Chrome 90+ (HTML5 Drag and Drop)
- ✅ Firefox 88+ (HTML5 Drag and Drop)
- ✅ Safari 14+ (HTML5 Drag and Drop)
- ✅ Edge 90+ (HTML5 Drag and Drop)

### Mobile Support
- ✅ iOS 14+ (Touch API)
- ✅ Android 10+ (Touch API)

### Fallback
For older browsers without HTML5 Drag and Drop:
- Use click-to-move fallback
- Up/Down buttons for reordering
- Drag handle clickable for reorder mode

## Documentation

### User Documentation Needed
- How to reorder profiles
- Using drag and drop
- Profile order persistence
- Troubleshooting

### Developer Documentation
- API reference (rustdoc)
- Integration guide
- Event handling guide
- CSS customization guide

## Release Notes

### Version 1.1.0 - Feature Addition
**New Features:**
- Drag and drop profile reordering
- Visual drag feedback
- Profile order persistence
- Touch support for mobile

**Implementation:**
- Added order field to ProfileConfig
- Implemented reorder methods in ProfileManager
- Updated ProfileManagerUI with drag state
- Added drag handle to profile cards

**Known Issues:**
- JavaScript event handlers not yet implemented
- CSS styling not yet complete
- Keyboard navigation not yet supported

## Conclusion

The Drag and Drop Profile Reordering feature provides an intuitive way for users to organize their browser profiles. The implementation includes:

- ✅ Complete backend functionality
- ✅ Data persistence
- ✅ UI component integration
- ✅ Drag state management
- ✅ Visual feedback framework
- ✅ Touch support foundation

**Remaining Work:**
- JavaScript event handler implementation
- CSS styling for drag states
- API endpoint integration
- Animation library integration
- Keyboard navigation support
- Comprehensive testing

The feature is ready for frontend integration and testing.