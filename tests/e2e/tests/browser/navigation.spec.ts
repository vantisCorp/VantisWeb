import { test, expect } from '@playwright/test';
import { BrowserPage } from '../../pages/browser-page';

test.describe('Browser Navigation', () => {
  let browserPage: BrowserPage;

  test.beforeEach(async ({ page }) => {
    browserPage = new BrowserPage(page);
  });

  test('should navigate to a URL successfully', async ({ page }) => {
    await browserPage.navigateToUrl('https://example.com');
    await expect(page).toHaveTitle(/Example Domain/);
    const url = await browserPage.getCurrentUrl();
    expect(url).toContain('example.com');
  });

  test('should handle back and forward navigation', async ({ page }) => {
    // Navigate to first page
    await browserPage.navigateToUrl('https://example.com');
    expect(await browserPage.getCurrentUrl()).toContain('example.com');

    // Navigate to second page
    await browserPage.navigateToUrl('https://example.com/page1');
    expect(await browserPage.getCurrentUrl()).toContain('page1');

    // Go back
    await browserPage.goBack();
    expect(await browserPage.getCurrentUrl()).toContain('example.com');
    expect(await browserPage.getCurrentUrl()).not.toContain('page1');

    // Go forward
    await browserPage.goForward();
    expect(await browserPage.getCurrentUrl()).toContain('page1');
  });

  test('should refresh the current page', async ({ page }) => {
    await browserPage.navigateToUrl('https://example.com');
    const initialTitle = await browserPage.getTitle();
    
    await browserPage.refresh();
    const refreshedTitle = await browserPage.getTitle();
    
    expect(refreshedTitle).toBe(initialTitle);
  });

  test('should handle invalid URLs gracefully', async ({ page }) => {
    await browserPage.navigateToUrl('not-a-valid-url');
    
    // Should show error page or handle gracefully
    const currentUrl = await browserPage.getCurrentUrl();
    expect(currentUrl).toBeTruthy();
  });

  test('should handle HTTPS vs HTTP', async ({ page }) => {
    // Test with HTTPS
    await browserPage.navigateToUrl('https://example.com');
    const httpsUrl = await browserPage.getCurrentUrl();
    expect(httpsUrl).toContain('https://');

    // Test with HTTP (if allowed)
    await browserPage.navigateToUrl('http://example.com');
    const httpUrl = await browserPage.getCurrentUrl();
    expect(httpUrl).toBeTruthy();
  });
});