import { test, expect } from '@playwright/test';
import { BrowserPage } from '../../pages/browser-page';

test.describe('Browser Navigation with Allure Reporting', () => {
  let browserPage: BrowserPage;

  test.beforeEach(async ({ page }) => {
    browserPage = new BrowserPage(page);
  });

  test('should navigate to a URL successfully @smoke @critical @navigation', async ({ page }) => {
    // Add Allure annotations
    test.info().annotations.push({
      type: 'description',
      description: 'Verify basic URL navigation functionality and page load handling'
    });
    
    test.info().annotations.push({
      type: 'severity',
      description: 'critical'
    });
    
    test.info().annotations.push({
      type: 'feature',
      description: 'Browser Navigation'
    });
    
    test.info().annotations.push({
      type: 'story',
      description: 'URL Navigation'
    });

    // Test execution
    await browserPage.navigateToUrl('https://example.com');
    
    // Verify page loaded successfully
    await expect(page).toHaveTitle(/Example Domain/);
    const currentUrl = await browserPage.getCurrentUrl();
    expect(currentUrl).toContain('example.com');
    
    // Add test step evidence
    test.info().annotations.push({
      type: 'testStep',
      description: 'Navigated to example.com and verified page title'
    });
  });

  test('should handle back and forward navigation @regression @navigation', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify browser history navigation (back and forward buttons)'
    });
    
    test.info().annotations.push({
      type: 'severity',
      description: 'normal'
    });
    
    test.info().annotations.push({
      type: 'feature',
      description: 'Browser Navigation'
    });
    
    test.info().annotations.push({
      type: 'story',
      description: 'History Navigation'
    });

    // Navigate to first page
    await browserPage.navigateToUrl('https://example.com');
    const firstUrl = await browserPage.getCurrentUrl();
    expect(firstUrl).toContain('example.com');

    // Navigate to second page
    await browserPage.navigateToUrl('https://example.com/page1');
    const secondUrl = await browserPage.getCurrentUrl();
    expect(secondUrl).toContain('page1');

    // Test back navigation
    await browserPage.goBack();
    const backUrl = await browserPage.getCurrentUrl();
    expect(backUrl).toContain('example.com');
    expect(backUrl).not.toContain('page1');

    // Test forward navigation
    await browserPage.goForward();
    const forwardUrl = await browserPage.getCurrentUrl();
    expect(forwardUrl).toContain('page1');
    
    test.info().annotations.push({
      type: 'testStep',
      description: 'Successfully tested back and forward navigation with URL verification'
    });
  });

  test('should refresh the current page @smoke @navigation', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify page refresh functionality maintains page state'
    });
    
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });
    
    test.info().annotations.push({
      type: 'feature',
      description: 'Browser Navigation'
    });

    await browserPage.navigateToUrl('https://example.com');
    const initialTitle = await browserPage.getTitle();
    const initialUrl = await browserPage.getCurrentUrl();
    
    // Refresh the page
    await browserPage.refresh();
    
    // Verify state is maintained
    const refreshedTitle = await browserPage.getTitle();
    const refreshedUrl = await browserPage.getCurrentUrl();
    
    expect(refreshedTitle).toBe(initialTitle);
    expect(refreshedUrl).toBe(initialUrl);
    
    test.info().annotations.push({
      type: 'testStep',
      description: 'Page refresh maintained title and URL correctly'
    });
  });

  test('should handle invalid URLs gracefully @regression @navigation', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify browser handles invalid URLs with appropriate error handling'
    });
    
    test.info().annotations.push({
      type: 'severity',
      description: 'normal'
    });
    
    test.info().annotations.push({
      type: 'feature',
      description: 'Browser Navigation'
    });
    
    test.info().annotations.push({
      type: 'story',
      description: 'Error Handling'
    });

    await browserPage.navigateToUrl('not-a-valid-url');
    
    // Should show error page or handle gracefully
    const currentUrl = await browserPage.getCurrentUrl();
    expect(currentUrl).toBeTruthy();
    
    // Verify we're still on a valid page
    const pageTitle = await browserPage.getTitle();
    expect(pageTitle).toBeTruthy();
    
    test.info().annotations.push({
      type: 'testStep',
      description: 'Invalid URL handled gracefully without crashing'
    });
  });

  test('should measure page load performance @performance @navigation', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Measure and verify page load performance metrics'
    });
    
    test.info().annotations.push({
      type: 'severity',
      description: 'normal'
    });
    
    test.info().annotations.push({
      type: 'feature',
      description: 'Performance Monitoring'
    });

    const startTime = Date.now();
    
    await browserPage.navigateToUrl('https://example.com');
    
    const loadTime = Date.now() - startTime;
    
    // Log performance metric
    test.info().annotations.push({
      type: 'performance',
      description: `Page load time: ${loadTime}ms`
    });
    
    // Verify page loaded
    await expect(page).toHaveTitle(/Example Domain/);
    
    // Performance assertion - should load within 5 seconds
    expect(loadTime).toBeLessThan(5000);
    
    test.info().annotations.push({
      type: 'testStep',
      description: `Performance test completed: ${loadTime}ms load time`
    });
  });
});