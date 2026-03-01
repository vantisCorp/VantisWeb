// Popup script for Example Extension

document.addEventListener("DOMContentLoaded", () => {
  const counterElement = document.getElementById("counter");
  const incrementBtn = document.getElementById("incrementBtn");
  const resetBtn = document.getElementById("resetBtn");
  
  // Load counter value
  function loadCounter() {
    chrome.runtime.sendMessage({ type: "getCounter" }, (response) => {
      if (response && response.counter !== undefined) {
        counterElement.textContent = response.counter;
      }
    });
  }
  
  // Increment counter
  function incrementCounter() {
    chrome.runtime.sendMessage({ type: "incrementCounter" }, (response) => {
      if (response && response.counter !== undefined) {
        counterElement.textContent = response.counter;
      }
    });
  }
  
  // Reset counter
  function resetCounter() {
    chrome.runtime.sendMessage({ type: "resetCounter" }, (response) => {
      if (response && response.counter !== undefined) {
        counterElement.textContent = response.counter;
      }
    });
  }
  
  // Event listeners
  incrementBtn.addEventListener("click", incrementCounter);
  resetBtn.addEventListener("click", resetCounter);
  
  // Initial load
  loadCounter();
});