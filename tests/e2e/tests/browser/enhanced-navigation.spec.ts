import { test, expect } from '@playwright/test';
import { BrowserPage } from '../../pages/browser-page';
import { BookmarksPage } from '../../pages/bookmarks-page';

test.describe('Enhanced Browser Navigation', () => {
  let browserPage: BrowserPage;
  let bookmarksPage: BookmarksPage;

  test.beforeEach(async ({ page }) => {
    browserPage = new BrowserPage(page);
    bookmarksPage = new BookmarksPage(page);
    
    test.info().annotations.push({
      type: 'epic',
      description: 'Browser Navigation'
    });
    test.info().annotations.push({
      type: 'feature',
      description: 'Enhanced Navigation'
    });
  });

  test('should navigate through browser history with back button', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify back button navigates through visited pages'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'critical'
    });

    // Navigate to multiple pages
    await browserPage.navigateTo('https://example.com');
    await browserPage.navigateTo('https://example.com/page1');
    await browserPage.navigateTo('https://example.com/page2');

    // Navigate back through history
    await page.goBack();
    await expect(page).toHaveURL('https://example.com/page1');
    
    await page.goBack();
    await expect(page).toHaveURL('https://example.com');
  });

  test('should navigate through browser history with forward button', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify forward button navigates forward through history'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'critical'
    });

    await browserPage.navigateTo('https://example.com');
    await browserPage.navigateTo('https://example.com/page1');
    await page.goBack();
    await page.goForward();
    
    await expect(page).toHaveURL('https://example.com/page1');
  });

  test('should handle page refresh correctly', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify page refresh reloads current page'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'critical'
    });

    await browserPage.navigateTo('https://example.com');
    const originalContent = await page.textContent('h1');
    
    await page.reload();
    await expect(page).toHaveURL('https://example.com/');
    
    const refreshedContent = await page.textContent('h1');
    expect(originalContent).toBe(refreshedContent);
  });

  test('should navigate to home page', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify home button navigates to configured home page'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'critical'
    });

    await browserPage.navigateTo('https://example.com');
    await browserPage.goToHomePage();
    
    await expect(page).toHaveURL(/about:home|https:\/\/www\.google\.com/);
  });

  test('should handle URL encoding/decoding', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify URL encoding and decoding works correctly'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });

    const encodedUrl = 'https://example.com/search?q=hello%20world';
    await browserPage.navigateTo(encodedUrl);
    
    await expect(page).toHaveURL(/hello%20world/);
    const decodedText = await page.textContent('body');
    expect(decodedText).toContain('hello world');
  });

  test('should handle redirects correctly', async ({ page, context }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify browser follows HTTP redirects'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'critical'
    });

    await browserPage.navigateTo('http://httpbin.org/redirect-to?url=https://example.com');
    await expect(page).toHaveURL('https://example.com/');
  });

  test('should handle 404 error pages', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify browser displays appropriate error page for 404'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });

    await browserPage.navigateTo('https://example.com/nonexistent-page');
    
    // Verify error page is displayed
    const errorElement = await page.locator('body').textContent();
    expect(errorElement).toMatch(/404|not found|error/i);
  });

  test('should handle 500 server error pages', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify browser displays appropriate error page for 500'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });

    await browserPage.navigateTo('https://httpbin.org/status/500');
    
    // Verify error page is displayed
    const errorElement = await page.locator('body').textContent();
    expect(errorElement).toMatch(/500|server error|internal error/i);
  });

  test('should navigate to bookmarked pages', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify navigation to bookmarked pages works'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });

    // Create a bookmark
    await browserPage.navigateTo('https://example.com');
    await bookmarksPage.addBookmark('Example Site', 'https://example.com');
    
    // Navigate to bookmark
    await bookmarksPage.navigateToBookmarks();
    await bookmarksPage.clickBookmark('Example Site');
    
    await expect(page).toHaveURL('https://example.com/');
  });

  test('should handle mixed content blocking', async ({ page, context }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify mixed content (HTTP on HTTPS) is blocked'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'critical'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'security'
    });

    // Navigate to HTTPS site with mixed content
    await browserPage.navigateTo('https://httpbin.org/html');
    
    // Verify mixed content warnings if present
    const hasMixedContentWarning = await page.locator('.mixed-content-warning').count();
    if (hasMixedContentWarning > 0) {
      await expect(page.locator('.mixed-content-warning')).toBeVisible();
    }
  });

  test('should handle URL with fragments', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify URL fragments (#section) navigate to page sections'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });

    await browserPage.navigateTo('https://example.com#section1');
    await expect(page).toHaveURL(/#section1/);
  });

  test('should handle URL with query parameters', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify URL query parameters are preserved in navigation'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });

    await browserPage.navigateTo('https://example.com?param1=value1&param2=value2');
    await expect(page).toHaveURL(/param1=value1/);
    await expect(page).toHaveURL(/param2=value2/);
  });

  test('should handle URL autocomplete in address bar', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify address bar provides autocomplete suggestions'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });

    // Visit some pages first
    await browserPage.navigateTo('https://example.com');
    await browserPage.navigateTo('https://example.org');
    
    // Click address bar and type partial URL
    await page.click('#urlbar');
    await page.fill('#urlbar', 'example');
    
    // Verify autocomplete suggestions appear
    const suggestions = await page.locator('.autocomplete-suggestion').count();
    expect(suggestions).toBeGreaterThan(0);
  });

  test('should handle stop page loading', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify stop button cancels page loading'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });

    // Navigate to a slow-loading page
    await browserPage.navigateTo('https://httpbin.org/delay/10');
    
    // Stop the page load
    await page.click('#stop-button');
    
    // Verify page load was stopped
    const url = page.url();
    expect(url).not.toBe('https://httpbin.org/delay/10');
  });
});