# VantisWeb Extensions System

## Overview

The VantisWeb Extensions System allows developers to extend browser functionality through plugins/add-ons. Extensions can modify web pages, add UI elements, interact with browser APIs, and communicate with each other.

## Architecture

### Core Components

1. **Extension Loader** - Loads extensions from the filesystem
2. **Extension Registry** - Manages loaded extensions
3. **Extension Manager** - Coordinates extension lifecycle
4. **Extension API** - Provides APIs for extensions to interact with the browser

### Extension Types

- **Content Script** - Runs in web page context
- **Background Script** - Runs in background, persistent
- **Popup** - UI extension with popup window
- **Theme** - Customizes browser appearance

## Extension Manifest

Every extension must have a `manifest.json` file:

```json
{
  "name": "Extension Name",
  "version": "1.0.0",
  "description": "Extension description",
  "author": "Author Name",
  "manifest_version": 2,
  "permissions": ["storage", "tabs"],
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

### Manifest Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Extension name |
| `version` | string | Yes | Extension version (semver) |
| `description` | string | Yes | Extension description |
| `author` | string | Yes | Extension author |
| `manifest_version` | number | Yes | Manifest version (must be 2) |
| `permissions` | array | No | Required permissions |
| `background` | object | No | Background script config |
| `content_scripts` | array | No | Content script configs |
| `popup` | object | No | Popup UI config |
| `icons` | object | No | Extension icons |

## Extension APIs

### Browser API

Access browser runtime information and control browser behavior.

```javascript
// Get runtime info
chrome.runtime.getManifest();

// Open a new tab
chrome.tabs.create({ url: "https://example.com" });

// Get active tab
chrome.tabs.query({ active: true, currentWindow: true }, (tabs) => {
  console.log(tabs[0]);
});

// Show notification
chrome.notifications.create({
  type: "basic",
  iconUrl: "icon.png",
  title: "Notification",
  message: "Hello from extension!"
});
```

### Storage API

Store data locally or sync across devices.

```javascript
// Local storage
chrome.storage.local.set({ key: "value" }, () => {
  console.log("Saved");
});

chrome.storage.local.get(["key"], (result) => {
  console.log(result.key);
});

// Sync storage
chrome.storage.sync.set({ key: "value" });
chrome.storage.sync.get(["key"], (result) => {
  console.log(result.key);
});
```

### Messaging API

Communicate between extension components.

```javascript
// Send message from content script to background
chrome.runtime.sendMessage({ type: "greet", name: "World" }, (response) => {
  console.log(response);
});

// Listen for messages in background script
chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
  if (request.type === "greet") {
    sendResponse({ message: `Hello, ${request.name}!` });
  }
});
```

### Tabs API

Interact with browser tabs.

```javascript
// Get all tabs
chrome.tabs.query({}, (tabs) => {
  console.log(tabs);
});

// Create new tab
chrome.tabs.create({ url: "https://example.com" });

// Update tab
chrome.tabs.update(tabId, { url: "https://newurl.com" });

// Execute script in tab
chrome.tabs.executeScript(tabId, { code: "document.body.style.backgroundColor = 'red';" });

// Send message to tab
chrome.tabs.sendMessage(tabId, { type: "update", data: "value" });
```

### Requests API

Make HTTP requests from extensions.

```javascript
// GET request
fetch("https://api.example.com/data")
  .then(response => response.json())
  .then(data => console.log(data));

// POST request
fetch("https://api.example.com/data", {
  method: "POST",
  headers: {
    "Content-Type": "application/json"
  },
  body: JSON.stringify({ key: "value" })
})
  .then(response => response.json())
  .then(data => console.log(data));
```

## Extension Lifecycle

### Loading

1. Extension directory is scanned
2. `manifest.json` is parsed and validated
3. Extension files are loaded
4. `on_load()` is called
5. Extension is registered

### Enabling

1. `on_enable()` is called
2. Background script is started
3. Content scripts are injected
4. Extension state is set to `Enabled`

### Disabling

1. `on_disable()` is called
2. Background script is stopped
3. Content scripts are removed
4. Extension state is set to `Disabled`

### Unloading

1. `on_unload()` is called
2. Extension is unregistered
3. Resources are cleaned up

## Security

### Permissions

Extensions must declare permissions in the manifest:

```json
{
  "permissions": [
    "storage",
    "tabs",
    "activeTab",
    "webRequest"
  ]
}
```

### Content Security Policy

Extensions are subject to Content Security Policy (CSP):

- No inline scripts (except in extension pages)
- No eval()
- Restricted external resource loading

### Sandboxing

- Content scripts run in isolated world
- Background scripts run in separate process
- No direct access to browser internals

## Development Guide

### Creating a New Extension

1. **Create extension directory**
   ```bash
   mkdir extensions/my-extension
   cd extensions/my-extension
   ```

2. **Create manifest.json**
   ```json
   {
     "name": "My Extension",
     "version": "1.0.0",
     "description": "My first extension",
     "author": "Your Name",
     "manifest_version": 2,
     "permissions": ["storage"]
   }
   ```

3. **Create background.js** (optional)
   ```javascript
   console.log("Background script loaded");
   ```

4. **Create content.js** (optional)
   ```javascript
   console.log("Content script loaded");
   ```

5. **Create popup.html** (optional)
   ```html
   <!DOCTYPE html>
   <html>
   <head>
     <title>My Extension</title>
   </head>
   <body>
     <h1>My Extension</h1>
     <script src="popup.js"></script>
   </body>
   </html>
   ```

6. **Create popup.js** (optional)
   ```javascript
   console.log("Popup loaded");
   ```

### Testing Extensions

1. Load extension in VantisWeb
2. Enable extension
3. Test functionality
4. Check console for errors

### Debugging

- Use browser DevTools for content scripts
- Check extension background page console
- Use `console.log()` for debugging
- Monitor network requests in DevTools

## Best Practices

### Performance

- Minimize DOM manipulations
- Use event delegation
- Debounce/throttle expensive operations
- Clean up event listeners

### Security

- Validate all user input
- Use HTTPS for network requests
- Don't store sensitive data in localStorage
- Implement proper error handling

### User Experience

- Provide clear UI feedback
- Handle errors gracefully
- Respect user preferences
- Follow browser design guidelines

### Code Quality

- Use consistent code style
- Add comments for complex logic
- Write unit tests
- Document public APIs

## Example Extensions

### Simple Counter Extension

See `extensions/example-extension/` for a complete example that demonstrates:
- Background scripts
- Content scripts
- Popup UI
- Storage API
- Messaging API

### Content Modifier Extension

```javascript
// content.js
document.querySelectorAll("a").forEach(link => {
  link.addEventListener("click", (e) => {
    console.log("Link clicked:", link.href);
  });
});
```

### Tab Manager Extension

```javascript
// background.js
chrome.tabs.onCreated.addListener((tab) => {
  console.log("New tab created:", tab.url);
});

chrome.tabs.onRemoved.addListener((tabId) => {
  console.log("Tab closed:", tabId);
});
```

## Troubleshooting

### Extension Not Loading

- Check `manifest.json` syntax
- Verify all required files exist
- Check console for errors
- Ensure manifest version is 2

### Permissions Errors

- Verify permissions in manifest
- Check if permission is supported
- Ensure proper API usage

### Messaging Issues

- Verify message format
- Check if listener is registered
- Ensure proper response handling

### Storage Issues

- Check storage quota
- Verify data format
- Handle storage errors

## API Reference

For complete API documentation, see [API Reference](API_REFERENCE.md).

## Support

- GitHub: https://github.com/vantisCorp/VantisWeb
- Issues: https://github.com/vantisCorp/VantisWeb/issues
- Documentation: https://github.com/vantisCorp/VantisWeb/wiki

## License

MIT License - See LICENSE file for details.