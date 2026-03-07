import { test, expect } from '@playwright/test';
import { BrowserPage } from '../../pages/browser-page';

test.describe('Tab Management', () => {
  let browserPage: BrowserPage;

  test.beforeEach(async ({ page }) => {
    browserPage = new BrowserPage(page);
    await browserPage.navigateToUrl('https://example.com');
  });

  test('should open a new tab', async ({ page, context }) => {
    const initialTabCount = await browserPage.getTabCount();
    
    await browserPage.openNewTab();
    
    const newTabCount = await browserPage.getTabCount();
    expect(newTabCount).toBe(initialTabCount + 1);
  });

  test('should switch between tabs', async ({ page }) => {
    // Open multiple tabs
    await browserPage.openNewTab();
    await browserPage.navigateToUrl('https://example.com/page1');
    
    await browserPage.openNewTab();
    await browserPage.navigateToUrl('https://example.com/page2');

    // Switch between tabs
    await browserPage.switchToTab(0);
    expect(await browserPage.getCurrentUrl()).toContain('example.com');

    await browserPage.switchToTab(1);
    expect(await browserPage.getCurrentUrl()).toContain('page1');

    await browserPage.switchToTab(2);
    expect(await browserPage.getCurrentUrl()).toContain('page2');
  });

  test('should maintain independent page state per tab', async ({ page, context }) => {
    // Open first tab and navigate
    await browserPage.navigateToUrl('https://example.com');
    const firstTabTitle = await browserPage.getTitle();

    // Open second tab and navigate to different page
    await browserPage.openNewTab();
    await browserPage.navigateToUrl('https://example.com/page1');
    const secondTabTitle = await browserPage.getTitle();

    // Switch back to first tab
    await browserPage.switchToTab(0);
    const currentTitle = await browserPage.getTitle();

    expect(firstTabTitle).not.toBe(secondTabTitle);
    expect(currentTitle).toBe(firstTabTitle);
  });

  test('should handle tab closing', async ({ page, context }) => {
    const initialTabCount = await browserPage.getTabCount();
    
    await browserPage.openNewTab();
    await browserPage.navigateToUrl('https://example.com/page1');
    
    const newTabCount = await browserPage.getTabCount();
    expect(newTabCount).toBe(initialTabCount + 1);

    // Close the current tab (implementation-specific)
    // This would typically involve a close button or keyboard shortcut
    await page.keyboard.press('Control+w'); // or Command+w on Mac
    
    const finalTabCount = await browserPage.getTabCount();
    expect(finalTabCount).toBe(initialTabCount);
  });

  test('should handle multiple tabs navigation', async ({ page }) => {
    // Open multiple tabs with different URLs
    const urls = [
      'https://example.com',
      'https://example.com/page1',
      'https://example.com/page2'
    ];

    for (const url of urls) {
      if (await browserPage.getTabCount() > 0) {
        await browserPage.openNewTab();
      }
      await browserPage.navigateToUrl(url);
    }

    // Verify each tab has correct URL
    for (let i = 0; i < urls.length; i++) {
      await browserPage.switchToTab(i);
      const currentUrl = await browserPage.getCurrentUrl();
      expect(currentUrl).toContain(urls[i].replace('https://example.com', ''));
    }
  });
});