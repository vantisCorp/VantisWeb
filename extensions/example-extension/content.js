// Content script for Example Extension

console.log("Example Extension content script loaded");

// Inject a simple indicator into the page
function injectIndicator() {
  const indicator = document.createElement("div");
  indicator.id = "example-extension-indicator";
  indicator.style.cssText = `
    position: fixed;
    bottom: 20px;
    right: 20px;
    background: #4CAF50;
    color: white;
    padding: 10px 15px;
    border-radius: 5px;
    font-family: Arial, sans-serif;
    font-size: 14px;
    box-shadow: 0 2px 5px rgba(0,0,0,0.2);
    z-index: 999999;
    cursor: pointer;
  `;
  indicator.textContent = "Example Extension Active";
  
  indicator.addEventListener("click", () => {
    // Send message to background script
    chrome.runtime.sendMessage({ type: "incrementCounter" }, (response) => {
      console.log("Counter:", response.counter);
      indicator.textContent = `Example Extension (Count: ${response.counter})`;
    });
  });
  
  document.body.appendChild(indicator);
}

// Wait for page to load
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", injectIndicator);
} else {
  injectIndicator();
}

// Listen for messages from background script
chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
  console.log("Content script received message:", request);
  
  if (request.type === "updateIndicator") {
    const indicator = document.getElementById("example-extension-indicator");
    if (indicator) {
      indicator.textContent = `Example Extension (Count: ${request.count})`;
    }
  }
});

// Observe DOM changes
const observer = new MutationObserver((mutations) => {
  // React to DOM changes if needed
});

observer.observe(document.body, {
  childList: true,
  subtree: true
});