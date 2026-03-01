# VantisWeb Extensions

This directory contains browser extensions for VantisWeb.

## Extension Structure

Each extension must have the following structure:

```
extension-name/
├── manifest.json    # Extension manifest (required)
├── background.js    # Background script (optional)
├── content.js       # Content script (optional)
├── popup.html       # Popup UI (optional)
├── popup.js         # Popup script (optional)
└── icons/           # Extension icons (optional)
    ├── icon16.png
    ├── icon48.png
    └── icon128.png
```

## Manifest Format

The `manifest.json` file defines the extension's metadata and configuration:

```json
{
  "name": "Extension Name",
  "version": "1.0.0",
  "description": "Extension description",
  "author": "Author Name",
  "manifest_version": 2,
  "permissions": ["storage", "tabs", "activeTab"],
  "background": {
    "script": "background.js",
    "persistent": false
  },
  "content_scripts": [
    {
      "js": ["content.js"],
      "matches": ["<all_urls>"],
      "run_at": "document_end"
    }
  ],
  "popup": {
    "html": "popup.html",
    "default_title": "Extension Name"
  },
  "icons": {
    "16": "icon16.png",
    "48": "icon48.png",
    "128": "icon128.png"
  }
}
```

## Available Permissions

- `storage` - Local and sync storage
- `tabs` - Tab management
- `activeTab` - Access to the active tab
- `cookies` - Cookie access
- `webRequest` - Network request monitoring
- `webNavigation` - Navigation events
- `notifications` - Browser notifications
- `alarms` - Scheduled tasks
- `bookmarks` - Bookmark access
- `history` - Browsing history
- `downloads` - Download management
- `clipboardRead` - Read clipboard
- `clipboardWrite` - Write clipboard
- `geolocation` - Geolocation access
- `idle` - Idle detection

## Extension APIs

### Browser API
- `chrome.runtime` - Runtime information
- `chrome.tabs` - Tab management
- `chrome.windows` - Window management

### Storage API
- `chrome.storage.local` - Local storage
- `chrome.storage.sync` - Sync storage

### Messaging API
- `chrome.runtime.sendMessage` - Send messages
- `chrome.runtime.onMessage` - Receive messages

### Requests API
- HTTP requests (GET, POST, PUT, DELETE, PATCH)

## Example Extension

See the `example-extension/` directory for a complete example extension that demonstrates:
- Background scripts
- Content scripts
- Popup UI
- Storage API
- Messaging API
- Tab API

## Development

1. Create a new directory for your extension
2. Create a `manifest.json` file
3. Implement your extension logic
4. Test with VantisWeb

## Security

- Extensions run in isolated contexts
- Permissions are required for sensitive operations
- Content Security Policy (CSP) is enforced
- All extensions are validated before loading

## Support

For more information, see:
- [API Reference](../docs/API_REFERENCE.md)
- [Extension Development Guide](../docs/EXTENSIONS.md)
- [GitHub Issues](https://github.com/vantisCorp/VantisWeb/issues)