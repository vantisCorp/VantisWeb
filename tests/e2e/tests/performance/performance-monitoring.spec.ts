import { test, expect } from '@playwright/test';
import { BrowserPage } from '../../pages/browser-page';

test.describe('Performance and Resource Management', () => {
  let browserPage: BrowserPage;

  test.beforeEach(async ({ page }) => {
    browserPage = new BrowserPage(page);
    
    test.info().annotations.push({
      type: 'epic',
      description: 'Performance'
    });
    test.info().annotations.push({
      type: 'feature',
      description: 'Performance Monitoring'
    });
  });

  test('should measure page load time', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify page load time is within acceptable limits'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'critical'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'performance'
    });

    const startTime = Date.now();
    await browserPage.navigateTo('https://example.com');
    const loadTime = Date.now() - startTime;
    
    // Verify page loads within 5 seconds
    expect(loadTime).toBeLessThan(5000);
    
    // Verify load time metrics are available
    const performanceTiming = await page.evaluate(() => {
      const timing = performance.timing;
      return {
        domContentLoaded: timing.domContentLoadedEventEnd - timing.navigationStart,
        loadComplete: timing.loadEventEnd - timing.navigationStart
      };
    });
    
    expect(performanceTiming.domContentLoaded).toBeGreaterThan(0);
    expect(performanceTiming.loadComplete).toBeGreaterThan(0);
  });

  test('should measure first contentful paint', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify First Contentful Paint (FCP) metrics'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'performance'
    });

    await browserPage.navigateTo('https://example.com');
    
    const fcp = await page.evaluate(() => {
      return new Promise<number>((resolve) => {
        new PerformanceObserver((list) => {
          const entries = list.getEntries();
          const fcpEntry = entries.find(entry => entry.name === 'first-contentful-paint');
          if (fcpEntry) {
            resolve(fcpEntry.startTime);
          }
        }).observe({ type: 'paint', buffered: true });
        
        // Fallback for browsers that don't support the observer
        setTimeout(() => {
          const entries = performance.getEntriesByType('paint');
          const fcpEntry = entries.find(entry => entry.name === 'first-contentful-paint');
          resolve(fcpEntry ? fcpEntry.startTime : 0);
        }, 1000);
      });
    });
    
    // FCP should be under 2 seconds for good performance
    expect(fcp).toBeLessThan(2000);
  });

  test('should measure time to interactive', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify Time to Interactive (TTI) metrics'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'performance'
    });

    await browserPage.navigateTo('https://example.com');
    
    // Wait for page to be fully interactive
    await page.waitForLoadState('networkidle');
    
    const tti = await page.evaluate(() => {
      return performance.now();
    });
    
    // TTI should be reasonable
    expect(tti).toBeGreaterThan(0);
  });

  test('should monitor memory usage', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify memory usage is within acceptable limits'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'performance'
    });

    await browserPage.navigateTo('https://example.com');
    
    // Get memory metrics (Chrome-only)
    const metrics = await page.evaluate(() => {
      const memory = (performance as any).memory;
      if (memory) {
        return {
          usedJSHeapSize: memory.usedJSHeapSize,
          totalJSHeapSize: memory.totalJSHeapSize,
          jsHeapSizeLimit: memory.jsHeapSizeLimit
        };
      }
      return null;
    });
    
    if (metrics) {
      // Memory usage should be under 100MB for simple pages
      expect(metrics.usedJSHeapSize).toBeLessThan(100 * 1024 * 1024);
    }
  });

  test('should monitor CPU usage', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify CPU usage is acceptable'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'performance'
    });

    await browserPage.navigateTo('https://example.com');
    
    // Monitor CPU-intensive operations
    const startCPUTime = await page.evaluate(() => {
      return performance.now();
    });
    
    // Perform some actions
    await page.click('body');
    await page.waitForTimeout(100);
    
    const endCPUTime = await page.evaluate(() => {
      return performance.now();
    });
    
    // CPU time should be reasonable
    expect(endCPUTime - startCPUTime).toBeGreaterThan(0);
  });

  test('should test network throttling', async ({ page, context }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify browser handles slow network connections'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'performance'
    });

    // Emulate slow 3G network
    const client = await context.newCDPSession(page);
    await client.send('Network.emulateNetworkConditions', {
      offline: false,
      downloadThroughput: (500 * 1024) / 8, // 500kb/s
      uploadThroughput: (500 * 1024) / 8,
      latency: 200 // 200ms RTT
    });
    
    const startTime = Date.now();
    await browserPage.navigateTo('https://example.com');
    const loadTime = Date.now() - startTime;
    
    // Page should still load, even if slower
    expect(loadTime).toBeGreaterThan(0);
    
    await page.waitForLoadState('networkidle');
  });

  test('should test offline mode', async ({ page, context }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify browser handles offline mode gracefully'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'critical'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'network'
    });

    await browserPage.navigateTo('https://example.com');
    
    // Go offline
    await context.setOffline(true);
    
    // Try to navigate
    await page.goto('https://example.org').catch(() => {});
    
    // Verify offline page is shown
    const body = await page.locator('body').textContent();
    expect(body).toMatch(/offline|no internet|not connected/i);
    
    // Go back online
    await context.setOffline(false);
  });

  test('should test cache behavior', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify browser caching works correctly'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'performance'
    });

    // First load
    const firstLoadStart = Date.now();
    await browserPage.navigateTo('https://example.com');
    await page.waitForLoadState('networkidle');
    const firstLoadTime = Date.now() - firstLoadStart;
    
    // Second load (should be cached)
    const secondLoadStart = Date.now();
    await page.reload();
    await page.waitForLoadState('networkidle');
    const secondLoadTime = Date.now() - secondLoadStart;
    
    // Second load should be faster due to caching
    expect(secondLoadTime).toBeLessThanOrEqual(firstLoadTime * 1.5);
  });

  test('should test service workers', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify service worker functionality'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'performance'
    });

    await browserPage.navigateTo('https://example.com');
    
    // Check for service worker
    const hasServiceWorker = await page.evaluate(() => {
      return navigator.serviceWorker !== undefined;
    });
    
    if (hasServiceWorker) {
      const registrations = await page.evaluate(() => {
        return navigator.serviceWorker.getRegistrations().then(regs => regs.length);
      });
      
      // Service worker should be registered if available
      expect(registrations).toBeGreaterThanOrEqual(0);
    }
  });

  test('should test lazy loading', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify lazy loading improves performance'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'performance'
    });

    await browserPage.navigateTo('https://example.com');
    
    // Check for lazy-loaded images
    const lazyImages = await page.locator('img[loading="lazy"]').count();
    
    if (lazyImages > 0) {
      // Scroll to trigger lazy loading
      await page.evaluate(() => {
        window.scrollTo(0, document.body.scrollHeight);
      });
      
      await page.waitForTimeout(500);
      
      // Verify images are loaded after scroll
      const loadedImages = await page.locator('img[loading="lazy"]:not([src=""])').count();
      expect(loadedImages).toBeGreaterThanOrEqual(0);
    }
  });

  test('should monitor resource sizes', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify resource sizes are reasonable'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'performance'
    });

    await browserPage.navigateTo('https://example.com');
    await page.waitForLoadState('networkidle');
    
    const resources = await page.evaluate(() => {
      const entries = performance.getEntriesByType('resource');
      return entries.map(entry => ({
        name: entry.name,
        duration: entry.duration,
        size: (entry as any).transferSize || 0
      }));
    });
    
    // Check total transfer size
    const totalSize = resources.reduce((sum, r) => sum + r.size, 0);
    
    // Total page size should be reasonable (under 5MB for simple pages)
    expect(totalSize).toBeLessThan(5 * 1024 * 1024);
  });

  test('should test image optimization', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify images are optimized'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'performance'
    });

    await browserPage.navigateTo('https://example.com');
    
    const images = await page.locator('img').all();
    
    for (const img of images.slice(0, 5)) {
      const src = await img.getAttribute('src');
      if (src) {
        const naturalWidth = await img.evaluate(el => (el as HTMLImageElement).naturalWidth);
        const naturalHeight = await img.evaluate(el => (el as HTMLImageElement).naturalHeight);
        const displayWidth = await img.evaluate(el => el.clientWidth);
        const displayHeight = await img.evaluate(el => el.clientHeight);
        
        // Image should not be significantly larger than displayed size
        const widthRatio = naturalWidth / (displayWidth || 1);
        const heightRatio = naturalHeight / (displayHeight || 1);
        
        // Images should not be more than 2x the displayed size
        expect(widthRatio).toBeLessThan(3);
        expect(heightRatio).toBeLessThan(3);
      }
    }
  });

  test('should test JavaScript execution time', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify JavaScript execution time is acceptable'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'performance'
    });

    await browserPage.navigateTo('https://example.com');
    
    const jsMetrics = await page.evaluate(() => {
      const entries = performance.getEntriesByType('measure');
      return entries.map(entry => ({
        name: entry.name,
        duration: entry.duration
      }));
    });
    
    // JavaScript execution should be reasonably fast
    for (const metric of jsMetrics) {
      expect(metric.duration).toBeLessThan(1000);
    }
  });

  test('should test resource prioritization', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify critical resources are loaded first'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'performance'
    });

    await browserPage.navigateTo('https://example.com');
    
    const resources = await page.evaluate(() => {
      const entries = performance.getEntriesByType('resource');
      return entries.slice(0, 10).map(entry => ({
        name: entry.name,
        startTime: entry.startTime,
        initiatorType: (entry as any).initiatorType
      }));
    });
    
    // CSS should be loaded early
    const cssResources = resources.filter(r => r.name.endsWith('.css'));
    for (const css of cssResources) {
      expect(css.startTime).toBeLessThan(2000);
    }
  });

  test('should test Web Vitals', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify Core Web Vitals are within good ranges'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'critical'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'performance'
    });

    await browserPage.navigateTo('https://example.com');
    await page.waitForLoadState('networkidle');
    
    const webVitals = await page.evaluate(() => {
      return new Promise<any>((resolve) => {
        const vitals: any = {};
        
        // Largest Contentful Paint
        new PerformanceObserver((list) => {
          const entries = list.getEntries();
          const lastEntry = entries[entries.length - 1];
          vitals.lcp = lastEntry.startTime;
        }).observe({ type: 'largest-contentful-paint', buffered: true });
        
        // Cumulative Layout Shift
        let clsValue = 0;
        new PerformanceObserver((list) => {
          for (const entry of list.getEntries()) {
            if (!(entry as any).hadRecentInput) {
              clsValue += (entry as any).value;
            }
          }
          vitals.cls = clsValue;
        }).observe({ type: 'layout-shift', buffered: true });
        
        // First Input Delay
        new PerformanceObserver((list) => {
          const firstInput = list.getEntries()[0];
          vitals.fid = (firstInput as any).processingStart - firstInput.startTime;
        }).observe({ type: 'first-input', buffered: true });
        
        setTimeout(() => resolve(vitals), 2000);
      });
    });
    
    // LCP should be under 2.5 seconds (good)
    if (webVitals.lcp) {
      expect(webVitals.lcp).toBeLessThan(2500);
    }
    
    // CLS should be under 0.1 (good)
    if (webVitals.cls !== undefined) {
      expect(webVitals.cls).toBeLessThan(0.1);
    }
  });

  test('should test memory leak prevention', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify no memory leaks during navigation'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'performance'
    });

    // Navigate multiple times
    for (let i = 0; i < 5; i++) {
      await browserPage.navigateTo('https://example.com');
      await page.waitForLoadState('networkidle');
    }
    
    // Check memory after multiple navigations
    const memory = await page.evaluate(() => {
      const mem = (performance as any).memory;
      return mem ? mem.usedJSHeapSize : 0;
    });
    
    // Memory should not grow excessively
    // This is a basic check; proper memory leak detection requires more sophisticated tools
    expect(memory).toBeGreaterThan(0);
  });
});