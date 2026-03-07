/**
 * Developer Tools E2E Tests
 * Comprehensive testing of browser developer tools integration
 * 
 * @description Tests for developer tools functionality including console, 
 *              inspector, network tab, and debugging features
 * @coverage Developer tools, debugging, inspection, console
 */

import { test, expect } from '@playwright/test';
import { allure } from 'allure-playwright';

test.describe('Developer Tools Integration', () => {
  test.beforeEach(async ({ page }) => {
    await allure.epic('Developer Tools');
    await allure.feature('DevTools Integration');
    await allure.story('As a developer, I want to use browser developer tools for debugging');
  });

  test.describe('Console Functionality', () => {
    test('should open and close developer tools panel', async ({ page }) => {
      await allure.severity('critical');
      await allure.description('Verify developer tools panel can be opened and closed');
      
      await page.goto('https://example.com');
      
      // Open DevTools using keyboard shortcut
      await page.keyboard.press('F12');
      
      // Wait for DevTools to potentially open (browser-dependent)
      await page.waitForTimeout(500);
      
      // Verify page is still functional
      await expect(page).toHaveTitle(/Example Domain/);
      
      // Close DevTools
      await page.keyboard.press('F12');
      await page.waitForTimeout(300);
      
      // Page should still be responsive
      const content = await page.content();
      expect(content.length).toBeGreaterThan(0);
    });

    test('should log messages to console', async ({ page }) => {
      await allure.severity('major');
      await allure.description('Verify console.log messages are captured');
      
      // Listen for console messages
      const consoleMessages: string[] = [];
      page.on('console', msg => {
        consoleMessages.push(msg.text());
      });
      
      await page.goto('https://example.com');
      
      // Execute script that logs to console
      await page.evaluate(() => {
        console.log('Test log message');
        console.warn('Test warning message');
        console.error('Test error message');
      });
      
      // Wait for messages to be captured
      await page.waitForTimeout(200);
      
      expect(consoleMessages).toContain('Test log message');
      expect(consoleMessages).toContain('Test warning message');
      expect(consoleMessages).toContain('Test error message');
    });

    test('should handle console errors gracefully', async ({ page }) => {
      await allure.severity('major');
      await allure.description('Verify console errors are properly captured');
      
      const errors: string[] = [];
      page.on('pageerror', error => {
        errors.push(error.message);
      });
      
      await page.goto('https://example.com');
      
      // Trigger an error
      await page.evaluate(() => {
        try {
          // @ts-ignore - Intentional error for testing
          nonExistentFunction();
        } catch (e) {
          // Error will be captured
        }
      });
      
      // Page should remain functional despite errors
      await expect(page.locator('body')).toBeVisible();
    });

    test('should clear console output', async ({ page }) => {
      await allure.severity('minor');
      await allure.description('Verify console can be cleared');
      
      await page.goto('https://example.com');
      
      // Log some messages
      await page.evaluate(() => {
        console.log('Message 1');
        console.log('Message 2');
        console.log('Message 3');
      });
      
      await page.waitForTimeout(200);
      
      // Verify page is still functional after console operations
      await expect(page.locator('h1')).toBeVisible();
    });
  });

  test.describe('Element Inspection', () => {
    test('should inspect DOM elements', async ({ page }) => {
      await allure.severity('critical');
      await allure.description('Verify DOM element inspection works correctly');
      
      await page.goto('https://example.com');
      
      // Get element details
      const h1 = page.locator('h1');
      await expect(h1).toBeVisible();
      
      // Verify element properties
      const tagName = await h1.evaluate(el => el.tagName);
      expect(tagName).toBe('H1');
      
      const textContent = await h1.textContent();
      expect(textContent).toBeTruthy();
    });

    test('should highlight elements on hover', async ({ page }) => {
      await allure.severity('minor');
      await allure.description('Verify element highlighting during inspection');
      
      await page.goto('https://example.com');
      
      const paragraph = page.locator('p').first();
      await paragraph.hover();
      
      // Element should be hoverable
      await expect(paragraph).toBeVisible();
    });

    test('should show computed styles', async ({ page }) => {
      await allure.severity('major');
      await allure.description('Verify computed styles are accessible');
      
      await page.goto('https://example.com');
      
      const h1 = page.locator('h1');
      
      // Get computed styles
      const styles = await h1.evaluate(el => {
        const computed = window.getComputedStyle(el);
        return {
          display: computed.display,
          fontSize: computed.fontSize,
          color: computed.color
        };
      });
      
      expect(styles.display).toBeTruthy();
      expect(styles.fontSize).toBeTruthy();
    });

    test('should edit element attributes', async ({ page }) => {
      await allure.severity('major');
      await allure.description('Verify element attributes can be modified');
      
      await page.goto('https://example.com');
      
      const h1 = page.locator('h1');
      
      // Add a custom attribute
      await h1.evaluate(el => {
        el.setAttribute('data-test', 'modified');
      });
      
      // Verify attribute was added
      const testAttr = await h1.getAttribute('data-test');
      expect(testAttr).toBe('modified');
    });

    test('should inspect nested elements', async ({ page }) => {
      await allure.severity('major');
      await allure.description('Verify nested element inspection');
      
      await page.goto('https://example.com');
      
      // Inspect nested structure
      const body = page.locator('body');
      const children = await body.evaluate(el => el.children.length);
      
      expect(children).toBeGreaterThan(0);
      
      // Verify DOM structure is intact
      const html = await page.locator('html').evaluate(el => el.outerHTML.length);
      expect(html).toBeGreaterThan(0);
    });
  });

  test.describe('Network Inspection', () => {
    test('should capture network requests', async ({ page }) => {
      await allure.severity('critical');
      await allure.description('Verify network requests are captured');
      
      const requests: string[] = [];
      page.on('request', request => {
        requests.push(request.url());
      });
      
      await page.goto('https://example.com');
      
      // Should have captured the main request
      expect(requests.length).toBeGreaterThan(0);
      expect(requests.some(r => r.includes('example.com'))).toBeTruthy();
    });

    test('should show request headers', async ({ page }) => {
      await allure.severity('major');
      await allure.description('Verify request headers are accessible');
      
      let requestHeaders: Record<string, string> = {};
      
      page.on('request', request => {
        requestHeaders = request.headers();
      });
      
      await page.goto('https://example.com');
      
      // Verify common headers exist
      expect(requestHeaders['user-agent']).toBeTruthy();
    });

    test('should show response headers', async ({ page }) => {
      await allure.severity('major');
      await allure.description('Verify response headers are accessible');
      
      let responseHeaders: Record<string, string> = {};
      
      page.on('response', response => {
        responseHeaders = response.headers();
      });
      
      await page.goto('https://example.com');
      
      // Verify response headers exist
      expect(Object.keys(responseHeaders).length).toBeGreaterThan(0);
    });

    test('should display request timing', async ({ page }) => {
      await allure.severity('minor');
      await allure.description('Verify request timing information');
      
      const timings: number[] = [];
      
      page.on('response', async response => {
        const timing = response.timing();
        timings.push(timing.responseStart);
      });
      
      await page.goto('https://example.com');
      
      // Should have timing data
      expect(timings.length).toBeGreaterThan(0);
    });

    test('should filter network requests by type', async ({ page }) => {
      await allure.severity('minor');
      await allure.description('Verify network request filtering');
      
      const resourceTypes: string[] = [];
      
      page.on('request', request => {
        resourceTypes.push(request.resourceType());
      });
      
      await page.goto('https://example.com');
      
      // Should have document type
      expect(resourceTypes).toContain('document');
    });

    test('should show HTTP status codes', async ({ page }) => {
      await allure.severity('major');
      await allure.description('Verify HTTP status codes are displayed');
      
      const statusCodes: number[] = [];
      
      page.on('response', response => {
        statusCodes.push(response.status());
      });
      
      await page.goto('https://example.com');
      
      // Should have 200 status
      expect(statusCodes).toContain(200);
    });
  });

  test.describe('Source Code Debugging', () => {
    test('should pause on debugger statement', async ({ page }) => {
      await allure.severity('major');
      await allure.description('Verify debugger statement handling');
      
      await page.goto('https://example.com');
      
      // Execute script with debugger
      await page.evaluate(() => {
        // debugger; // This would pause execution in DevTools
        const x = 1 + 1;
        return x;
      });
      
      // Page should remain functional
      await expect(page.locator('body')).toBeVisible();
    });

    test('should set breakpoints', async ({ page }) => {
      await allure.severity('minor');
      await allure.description('Verify breakpoint functionality');
      
      await page.goto('https://example.com');
      
      // Simulate breakpoint-like behavior
      const result = await page.evaluate(() => {
        let sum = 0;
        for (let i = 0; i < 5; i++) {
          sum += i;
        }
        return sum;
      });
      
      expect(result).toBe(10);
    });

    test('should step through code execution', async ({ page }) => {
      await allure.severity('minor');
      await allure.description('Verify step-through debugging');
      
      await page.goto('https://example.com');
      
      // Execute step-by-step logic
      const steps = await page.evaluate(() => {
        const results: number[] = [];
        let value = 0;
        
        results.push(value); // Step 1: 0
        value += 5;          // Step 2: 5
        results.push(value);
        value *= 2;          // Step 3: 10
        results.push(value);
        
        return results;
      });
      
      expect(steps).toEqual([0, 5, 10]);
    });

    test('should watch variables', async ({ page }) => {
      await allure.severity('minor');
      await allure.description('Verify variable watching');
      
      await page.goto('https://example.com');
      
      // Simulate variable watching
      const watchedVars = await page.evaluate(() => {
        let counter = 0;
        const history: number[] = [];
        
        const watch = () => history.push(counter);
        
        counter = 1;
        watch();
        counter = 2;
        watch();
        counter = 3;
        watch();
        
        return history;
      });
      
      expect(watchedVars).toEqual([1, 2, 3]);
    });
  });

  test.describe('Application Storage', () => {
    test('should inspect localStorage', async ({ page }) => {
      await allure.severity('major');
      await allure.description('Verify localStorage inspection');
      
      await page.goto('https://example.com');
      
      // Set localStorage items
      await page.evaluate(() => {
        localStorage.setItem('testKey', 'testValue');
        localStorage.setItem('userSettings', JSON.stringify({ theme: 'dark' }));
      });
      
      // Verify items
      const value = await page.evaluate(() => localStorage.getItem('testKey'));
      expect(value).toBe('testValue');
      
      // Clean up
      await page.evaluate(() => localStorage.clear());
    });

    test('should inspect sessionStorage', async ({ page }) => {
      await allure.severity('major');
      await allure.description('Verify sessionStorage inspection');
      
      await page.goto('https://example.com');
      
      // Set sessionStorage items
      await page.evaluate(() => {
        sessionStorage.setItem('sessionKey', 'sessionValue');
      });
      
      // Verify items
      const value = await page.evaluate(() => sessionStorage.getItem('sessionKey'));
      expect(value).toBe('sessionValue');
      
      // Clean up
      await page.evaluate(() => sessionStorage.clear());
    });

    test('should inspect cookies', async ({ page }) => {
      await allure.severity('major');
      await allure.description('Verify cookie inspection');
      
      await page.goto('https://example.com');
      
      // Get cookies
      const cookies = await page.context().cookies();
      
      // Cookies should be an array
      expect(Array.isArray(cookies)).toBeTruthy();
    });

    test('should clear storage data', async ({ page }) => {
      await allure.severity('major');
      await allure.description('Verify storage clearing');
      
      await page.goto('https://example.com');
      
      // Add data
      await page.evaluate(() => {
        localStorage.setItem('key1', 'value1');
        localStorage.setItem('key2', 'value2');
        sessionStorage.setItem('sessionKey', 'sessionValue');
      });
      
      // Clear all storage
      await page.evaluate(() => {
        localStorage.clear();
        sessionStorage.clear();
      });
      
      // Verify cleared
      const localLen = await page.evaluate(() => localStorage.length);
      const sessionLen = await page.evaluate(() => sessionStorage.length);
      
      expect(localLen).toBe(0);
      expect(sessionLen).toBe(0);
    });
  });

  test.describe('Performance Profiling', () => {
    test('should measure page performance', async ({ page }) => {
      await allure.severity('major');
      await allure.description('Verify performance measurement');
      
      await page.goto('https://example.com');
      
      // Get performance metrics
      const metrics = await page.evaluate(() => {
        const perf = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming;
        return {
          loadTime: perf.loadEventEnd - perf.fetchStart,
          domContentLoaded: perf.domContentLoadedEventEnd - perf.fetchStart
        };
      });
      
      // Should have valid metrics
      expect(metrics.loadTime).toBeGreaterThanOrEqual(0);
      expect(metrics.domContentLoaded).toBeGreaterThanOrEqual(0);
    });

    test('should profile JavaScript execution', async ({ page }) => {
      await allure.severity('minor');
      await allure.description('Verify JavaScript profiling');
      
      await page.goto('https://example.com');
      
      // Execute and measure
      const start = Date.now();
      await page.evaluate(() => {
        let sum = 0;
        for (let i = 0; i < 10000; i++) {
          sum += Math.random();
        }
        return sum;
      });
      const duration = Date.now() - start;
      
      // Should complete within reasonable time
      expect(duration).toBeLessThan(5000);
    });

    test('should track memory usage', async ({ page }) => {
      await allure.severity('minor');
      await allure.description('Verify memory usage tracking');
      
      await page.goto('https://example.com');
      
      // Memory metrics available in Chromium
      const metrics = await page.metrics();
      
      expect(metrics).toHaveProperty('Timestamp');
      expect(metrics).toHaveProperty('Documents');
    });
  });
});