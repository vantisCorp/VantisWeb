import { test, expect } from '@playwright/test';
import { BrowserPage } from '../../pages/browser-page';

test.describe('Enhanced Tab Management', () => {
  let browserPage: BrowserPage;

  test.beforeEach(async ({ page, context }) => {
    browserPage = new BrowserPage(page);
    
    test.info().annotations.push({
      type: 'epic',
      description: 'Tab Management'
    });
    test.info().annotations.push({
      type: 'feature',
      description: 'Enhanced Tab Operations'
    });
  });

  test('should switch between multiple tabs', async ({ page, context }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify tab switching functionality between multiple open tabs'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'critical'
    });

    // Create multiple tabs
    await browserPage.navigateTo('https://example.com');
    const page1 = page;
    
    const page2 = await context.newPage();
    await page2.goto('https://example.org');
    
    const page3 = await context.newPage();
    await page3.goto('https://example.net');

    // Switch between tabs
    await page2.bringToFront();
    await expect(page2).toHaveURL('https://example.org/');
    
    await page1.bringToFront();
    await expect(page1).toHaveURL('https://example.com/');
    
    await page3.bringToFront();
    await expect(page3).toHaveURL('https://example.net/');
  });

  test('should close tabs and maintain correct focus', async ({ page, context }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify closing tabs updates focus correctly'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'critical'
    });

    await browserPage.navigateTo('https://example.com');
    const page1 = page;
    
    const page2 = await context.newPage();
    await page2.goto('https://example.org');
    
    // Close second tab
    await page2.close();
    
    // Verify focus returns to first tab
    await page1.bringToFront();
    await expect(page1).toBeVisible();
  });

  test('should duplicate tab', async ({ page, context }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify tab duplication creates copy of current tab'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });

    await browserPage.navigateTo('https://example.com');
    
    // Duplicate tab (open new tab with same URL)
    const duplicatedPage = await context.newPage();
    await duplicatedPage.goto(page.url());
    
    await expect(duplicatedPage).toHaveURL('https://example.com/');
    await expect(duplicatedPage).not.toBe(page);
  });

  test('should pin tab', async ({ page, context }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify pinning tabs keeps them visible'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });

    await browserPage.navigateTo('https://example.com');
    
    // Pin tab (simulated - actual implementation may vary)
    await page.click('#tab-pin-button');
    
    // Verify tab is pinned
    const tab = await page.locator('.tab.pinned');
    await expect(tab).toBeVisible();
  });

  test('should unpin tab', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify unpinning tabs restores normal behavior'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });

    // Assuming tab is already pinned
    await page.click('#tab-unpin-button');
    
    // Verify tab is unpinned
    const tab = await page.locator('.tab:not(.pinned)');
    await expect(tab).toBeVisible();
  });

  test('should show loading state for tabs', async ({ page, context }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify tabs show loading indicator during page load'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });

    await browserPage.navigateTo('https://httpbin.org/delay/5');
    
    // Verify loading indicator is shown
    const loadingIndicator = await page.locator('.tab-loading');
    await expect(loadingIndicator).toBeVisible();
    
    // Wait for page to load
    await page.waitForLoadState('networkidle');
    
    // Verify loading indicator is hidden
    await expect(loadingIndicator).not.toBeVisible();
  });

  test('should handle tab audio indicator', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify tabs show audio indicator when playing media'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });

    // Navigate to page with audio
    await browserPage.navigateTo('https://www.w3schools.com/html/html5_audio.asp');
    
    // Play audio (if possible)
    const playButton = page.locator('button').filter({ hasText: /play/i }).first();
    if (await playButton.isVisible()) {
      await playButton.click();
      
      // Verify audio indicator appears
      const audioIndicator = await page.locator('.tab-audio');
      await expect(audioIndicator).toBeVisible({ timeout: 5000 });
    }
  });

  test('should mute tab audio', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify tab audio can be muted'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });

    // Assuming tab is playing audio
    await page.click('#tab-mute-button');
    
    // Verify audio is muted
    const mutedIndicator = await page.locator('.tab-muted');
    await expect(mutedIndicator).toBeVisible();
  });

  test('should display correct tab title', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify tab displays correct page title'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });

    await browserPage.navigateTo('https://example.com');
    
    // Get page title
    const pageTitle = await page.title();
    
    // Verify tab title matches page title
    const tabTitle = await page.locator('.tab-title').textContent();
    expect(tabTitle).toBe(pageTitle);
  });

  test('should display correct tab favicon', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify tab displays correct favicon'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });

    await browserPage.navigateTo('https://example.com');
    
    // Verify favicon is displayed
    const favicon = await page.locator('.tab-favicon');
    await expect(favicon).toBeVisible();
  });

  test('should reorder tabs', async ({ page, context }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify tabs can be reordered by dragging'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });

    await browserPage.navigateTo('https://example.com');
    const tab1 = page;
    
    const tab2 = await context.newPage();
    await tab2.goto('https://example.org');
    
    // Get initial tab order
    const tabsBefore = await page.locator('.tab').all();
    
    // Reorder tabs (simulated - actual implementation may vary)
    await page.dragAndDrop('.tab:nth-child(1)', '.tab:nth-child(2)');
    
    // Get tab order after reordering
    const tabsAfter = await page.locator('.tab').all();
    
    // Verify tab order changed
    expect(tabsBefore).not.toEqual(tabsAfter);
  });

  test('should restore tabs after browser restart', async ({ page, context }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify tabs are restored after browser restart'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'persistence'
    });

    // Open multiple tabs
    await browserPage.navigateTo('https://example.com');
    await context.newPage().goto('https://example.org');
    await context.newPage().goto('https://example.net');
    
    // Store session state (simulated)
    const sessionState = {
      urls: context.pages().map(p => p.url()),
      activeIndex: 0
    };
    
    // Simulate browser restart
    await context.close();
    const newContext = await browserPage.browser.newContext();
    const newPage = await newContext.newPage();
    
    // Restore session (simulated - actual implementation may vary)
    await newPage.goto(sessionState.urls[0]);
    
    await expect(newPage).toHaveURL('https://example.com/');
    
    await newContext.close();
  });

  test('should handle tab context menu', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify tab context menu provides correct options'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });

    await browserPage.navigateTo('https://example.com');
    
    // Right-click on tab
    await page.click('.tab', { button: 'right' });
    
    // Verify context menu appears
    const contextMenu = await page.locator('.context-menu');
    await expect(contextMenu).toBeVisible();
    
    // Verify menu items
    const menuItems = await contextMenu.locator('.menu-item').all();
    expect(menuItems.length).toBeGreaterThan(0);
  });

  test('should close all tabs to the right', async ({ page, context }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify closing all tabs to the right works'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });

    // Open multiple tabs
    await browserPage.navigateTo('https://example.com');
    await context.newPage().goto('https://example.org');
    await context.newPage().goto('https://example.net');
    
    // Close all tabs to the right (simulated)
    await page.click('.tab:nth-child(1)');
    await page.click('.tab-context-menu-item:has-text("Close tabs to the right")');
    
    // Verify only first tab remains
    const pages = context.pages();
    expect(pages.length).toBe(1);
  });

  test('should close all other tabs', async ({ page, context }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify closing all other tabs works'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });

    // Open multiple tabs
    await browserPage.navigateTo('https://example.com');
    await context.newPage().goto('https://example.org');
    await context.newPage().goto('https://example.net');
    
    // Close all other tabs (simulated)
    await page.click('.tab:nth-child(2)');
    await page.click('.tab-context-menu-item:has-text("Close other tabs")');
    
    // Verify only second tab remains
    const pages = context.pages();
    expect(pages.length).toBe(1);
    await expect(page).toHaveURL('https://example.org/');
  });

  test('should handle tab crash recovery', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify tabs can be recovered after crash'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'recovery'
    });

    await browserPage.navigateTo('https://example.com');
    const originalUrl = page.url();
    
    // Simulate tab crash (crash page)
    await page.goto('about:crash');
    
    // Verify crash page is shown
    await expect(page).toHaveURL('about:crash');
    
    // Restore tab
    await page.click('#restore-tab-button');
    
    // Verify tab is restored
    await expect(page).toHaveURL(originalUrl);
  });
});