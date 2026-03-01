// Background script for Example Extension

console.log("Example Extension background script loaded");

// Listen for extension installation
chrome.runtime.onInstalled.addListener((details) => {
  if (details.reason === "install") {
    console.log("Example Extension installed");
    
    // Set default settings
    chrome.storage.local.set({
      enabled: true,
      counter: 0
    });
  } else if (details.reason === "update") {
    console.log("Example Extension updated");
  }
});

// Listen for messages from content scripts and popup
chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
  console.log("Received message:", request);
  
  if (request.type === "getCounter") {
    chrome.storage.local.get(["counter"], (result) => {
      sendResponse({ counter: result.counter || 0 });
    });
    return true; // Keep message channel open for async response
  }
  
  if (request.type === "incrementCounter") {
    chrome.storage.local.get(["counter"], (result) => {
      const newCounter = (result.counter || 0) + 1;
      chrome.storage.local.set({ counter: newCounter }, () => {
        sendResponse({ counter: newCounter });
      });
    });
    return true;
  }
  
  if (request.type === "resetCounter") {
    chrome.storage.local.set({ counter: 0 }, () => {
      sendResponse({ counter: 0 });
    });
    return true;
  }
});

// Listen for tab updates
chrome.tabs.onUpdated.addListener((tabId, changeInfo, tab) => {
  if (changeInfo.status === "complete" && tab.url) {
    console.log("Tab updated:", tab.url);
  }
});

// Listen for tab creation
chrome.tabs.onCreated.addListener((tab) => {
  console.log("Tab created:", tab.url);
});