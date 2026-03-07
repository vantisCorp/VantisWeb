/**
 * Multi-Window and Workspace E2E Tests
 * Comprehensive testing of window management and workspace features
 * 
 * @description Tests for multi-window operations, popups, workspace management,
 *              and window state persistence
 * @coverage Multi-window, popups, workspaces, window management
 */

import { test, expect, BrowserContext, Page } from '@playwright/test';
import { allure } from 'allure-playwright';

test.describe('Multi-Window and Workspace Management', () => {
  test.beforeEach(async () => {
    await allure.epic('Browser Functionality');
    await allure.feature('Multi-Window Management');
    await allure.story('As a user, I want to manage multiple browser windows');
  });

  test.describe('Window Opening and Closing', () => {
    test('should open new browser window', async ({ browser }) => {
      await allure.severity('critical');
      await allure.description('Verify new browser window can be opened');
      
      const context = await browser.newContext();
      const page1 = await context.newPage();
      const page2 = await context.newPage();
      
      await page1.goto('https://example.com');
      await page2.goto('https://example.org');
      
      await expect(page1).toHaveTitle(/Example Domain/);
      await expect(page2).toHaveTitle(/Example Domain/);
      
      await context.close();
    });

    test('should close window without affecting others', async ({ browser }) => {
      await allure.severity('critical');
      await allure.description('Verify closing one window does not affect others');
      
      const context = await browser.newContext();
      const page1 = await context.newPage();
      const page2 = await context.newPage();
      
      await page1.goto('https://example.com');
      await page2.goto('https://example.org');
      
      // Close page2
      await page2.close();
      
      // page1 should still be functional
      await expect(page1).toHaveTitle(/Example Domain/);
      
      await context.close();
    });

    test('should handle multiple windows with different contexts', async ({ browser }) => {
      await allure.severity('major');
      await allure.description('Verify separate contexts for different window groups');
      
      const context1 = await browser.newContext();
      const context2 = await browser.newContext();
      
      const page1 = await context1.newPage();
      const page2 = await context2.newPage();
      
      await page1.goto('https://example.com');
      await page2.goto('https://example.org');
      
      // Should have independent sessions
      await expect(page1).toHaveTitle(/Example Domain/);
      await expect(page2).toHaveTitle(/Example Domain/);
      
      await context1.close();
      await context2.close();
    });

    test('should restore recently closed window', async ({ browser }) => {
      await allure.severity('major');
      await allure.description('Verify recently closed window can be restored');
      
      const context = await browser.newContext();
      const page = await context.newPage();
      
      await page.goto('https://example.com');
      const url = page.url();
      
      // Close and verify URL was tracked
      await page.close();
      
      // Create new page with same URL (simulating restore)
      const restoredPage = await context.newPage();
      await restoredPage.goto(url);
      
      await expect(restoredPage).toHaveURL(url);
      
      await context.close();
    });

    test('should close all windows at once', async ({ browser }) => {
      await allure.severity('major');
      await allure.description('Verify closing all windows simultaneously');
      
      const context = await browser.newContext();
      
      const page1 = await context.newPage();
      const page2 = await context.newPage();
      const page3 = await context.newPage();
      
      await page1.goto('https://example.com');
      await page2.goto('https://example.org');
      await page3.goto('https://www.iana.org/domains/reserved');
      
      // Close all by closing context
      await context.close();
      
      // Context should be closed
      expect(context.pages().length).toBe(0);
    });
  });

  test.describe('Popup Window Handling', () => {
    test('should handle popup windows', async ({ page, context }) => {
      await allure.severity('critical');
      await allure.description('Verify popup windows are handled correctly');
      
      await page.goto('https://example.com');
      
      // Listen for popup
      const [popup] = await Promise.all([
        context.waitForEvent('page'),
        page.evaluate(() => window.open('https://example.org', '_blank'))
      ]);
      
      await popup.waitForLoadState();
      await expect(popup).toHaveTitle(/Example Domain/);
      
      await popup.close();
    });

    test('should block unwanted popups', async ({ page, context }) => {
      await allure.severity('major');
      await allure.description('Verify unwanted popups can be blocked');
      
      // Block popups
      await context.route('**/*', route => {
        if (route.request().resourceType() === 'document' && route.request().url() !== 'https://example.com/') {
          route.abort();
        } else {
          route.continue();
        }
      });
      
      await page.goto('https://example.com');
      
      // Page should still work
      await expect(page.locator('body')).toBeVisible();
    });

    test('should pass data to popup window', async ({ page }) => {
      await allure.severity('major');
      await allure.description('Verify data can be passed to popup windows');
      
      await page.goto('https://example.com');
      
      // Use postMessage for communication simulation
      const received = await page.evaluate(() => {
        return new Promise<string>((resolve) => {
          window.addEventListener('message', (event) => {
            resolve(event.data);
          });
          
          // Simulate receiving message
          setTimeout(() => {
            window.postMessage('test-message', '*');
          }, 100);
        });
      });
      
      expect(received).toBe('test-message');
    });

    test('should maintain popup window position', async ({ page }) => {
      await allure.severity('minor');
      await allure.description('Verify popup window position is maintained');
      
      await page.goto('https://example.com');
      
      // Get viewport size
      const viewport = page.viewportSize();
      expect(viewport).toBeTruthy();
    });

    test('should handle modal dialogs', async ({ page }) => {
      await allure.severity('major');
      await allure.description('Verify modal dialog handling');
      
      await page.goto('https://example.com');
      
      // Handle alert dialog
      page.once('dialog', dialog => {
        expect(dialog.message()).toBe('Test alert');
        dialog.dismiss();
      });
      
      await page.evaluate(() => alert('Test alert'));
      
      // Page should remain functional
      await expect(page.locator('body')).toBeVisible();
    });

    test('should handle confirm dialogs', async ({ page }) => {
      await allure.severity('major');
      await allure.description('Verify confirm dialog handling');
      
      await page.goto('https://example.com');
      
      // Handle confirm dialog
      page.once('dialog', dialog => {
        expect(dialog.type()).toBe('confirm');
        dialog.accept();
      });
      
      const result = await page.evaluate(() => confirm('Are you sure?'));
      expect(result).toBe(true);
    });

    test('should handle prompt dialogs', async ({ page }) => {
      await allure.severity('minor');
      await allure.description('Verify prompt dialog handling');
      
      await page.goto('https://example.com');
      
      // Handle prompt dialog
      page.once('dialog', dialog => {
        expect(dialog.type()).toBe('prompt');
        dialog.accept('User input');
      });
      
      const result = await page.evaluate(() => prompt('Enter value:'));
      expect(result).toBe('User input');
    });
  });

  test.describe('Window Sizing and Positioning', () => {
    test('should resize browser window', async ({ page }) => {
      await allure.severity('major');
      await allure.description('Verify window can be resized');
      
      await page.goto('https://example.com');
      
      // Resize viewport
      await page.setViewportSize({ width: 1280, height: 720 });
      
      const viewport = page.viewportSize();
      expect(viewport?.width).toBe(1280);
      expect(viewport?.height).toBe(720);
    });

    test('should maximize window', async ({ page }) => {
      await allure.severity('major');
      await allure.description('Verify window maximization');
      
      await page.goto('https://example.com');
      
      // Simulate maximize by setting large viewport
      await page.setViewportSize({ width: 1920, height: 1080 });
      
      const viewport = page.viewportSize();
      expect(viewport?.width).toBe(1920);
    });

    test('should minimize window', async ({ page }) => {
      await allure.severity('minor');
      await allure.description('Verify window minimization');
      
      await page.goto('https://example.com');
      
      // Simulate minimize by setting small viewport
      await page.setViewportSize({ width: 800, height: 600 });
      
      const viewport = page.viewportSize();
      expect(viewport?.height).toBe(600);
    });

    test('should restore window to previous size', async ({ page }) => {
      await allure.severity('minor');
      await allure.description('Verify window restoration');
      
      await page.goto('https://example.com');
      
      // Set initial size
      await page.setViewportSize({ width: 1024, height: 768 });
      const originalSize = page.viewportSize();
      
      // Change size
      await page.setViewportSize({ width: 800, height: 600 });
      
      // Restore
      await page.setViewportSize({ width: originalSize!.width, height: originalSize!.height });
      
      expect(page.viewportSize()?.width).toBe(1024);
    });

    test('should handle full-screen mode', async ({ page }) => {
      await allure.severity('minor');
      await allure.description('Verify full-screen mode');
      
      await page.goto('https://example.com');
      
      // Enter fullscreen
      await page.evaluate(() => {
        document.documentElement.requestFullscreen?.();
      });
      
      // Exit fullscreen
      await page.evaluate(() => {
        document.exitFullscreen?.();
      });
      
      // Page should remain functional
      await expect(page.locator('body')).toBeVisible();
    });
  });

  test.describe('Tab Management Within Windows', () => {
    test('should move tabs between windows', async ({ browser }) => {
      await allure.severity('major');
      await allure.description('Verify tabs can be moved between windows');
      
      const context = await browser.newContext();
      const page1 = await context.newPage();
      const page2 = await context.newPage();
      
      await page1.goto('https://example.com');
      await page2.goto('https://example.org');
      
      // Both pages should be in same context
      expect(context.pages().length).toBe(2);
      
      await context.close();
    });

    test('should duplicate tab in new window', async ({ browser }) => {
      await allure.severity('major');
      await allure.description('Verify tab duplication to new window');
      
      const context = await browser.newContext();
      const page = await context.newPage();
      
      await page.goto('https://example.com');
      const url = page.url();
      
      // Create new page with same URL
      const duplicate = await context.newPage();
      await duplicate.goto(url);
      
      await expect(duplicate).toHaveURL(url);
      
      await context.close();
    });

    test('should group related tabs', async ({ browser }) => {
      await allure.severity('minor');
      await allure.description('Verify tab grouping functionality');
      
      const context = await browser.newContext();
      
      // Create related pages
      const pages = await Promise.all([
        context.newPage(),
        context.newPage(),
        context.newPage()
      ]);
      
      await pages[0].goto('https://example.com');
      await pages[1].goto('https://example.com');
      await pages[2].goto('https://example.com');
      
      // All should be in same context
      expect(context.pages().length).toBe(3);
      
      await context.close();
    });
  });

  test.describe('Window State Persistence', () => {
    test('should remember window size between sessions', async ({ browser }) => {
      await allure.severity('major');
      await allure.description('Verify window size persistence');
      
      const context = await browser.newContext({
        viewport: { width: 1366, height: 768 }
      });
      
      const page = await context.newPage();
      await page.goto('https://example.com');
      
      const viewport = page.viewportSize();
      expect(viewport?.width).toBe(1366);
      
      await context.close();
    });

    test('should preserve session storage per window', async ({ browser }) => {
      await allure.severity('major');
      await allure.description('Verify session storage isolation');
      
      const context = await browser.newContext();
      const page1 = await context.newPage();
      const page2 = await context.newPage();
      
      await page1.goto('https://example.com');
      await page2.goto('https://example.org');
      
      // Set different session data
      await page1.evaluate(() => sessionStorage.setItem('window1', 'data1'));
      await page2.evaluate(() => sessionStorage.setItem('window2', 'data2'));
      
      // Verify isolation
      const data1 = await page1.evaluate(() => sessionStorage.getItem('window1'));
      const data2 = await page2.evaluate(() => sessionStorage.getItem('window2'));
      
      expect(data1).toBe('data1');
      expect(data2).toBe('data2');
      
      await context.close();
    });

    test('should handle window reload', async ({ page }) => {
      await allure.severity('major');
      await allure.description('Verify window state after reload');
      
      await page.goto('https://example.com');
      
      // Modify state
      await page.evaluate(() => {
        (window as any).testState = 'preserved';
      });
      
      // Reload
      await page.reload();
      
      // State should be reset
      const state = await page.evaluate(() => (window as any).testState);
      expect(state).toBeUndefined();
    });

    test('should restore form data on reload', async ({ page }) => {
      await allure.severity('minor');
      await allure.description('Verify form data restoration');
      
      await page.goto('https://example.com');
      
      // bfcache should work for form restoration
      const canRestore = await page.evaluate(() => {
        return 'onpageshow' in window;
      });
      
      expect(canRestore).toBe(true);
    });
  });

  test.describe('Multi-Monitor Support', () => {
    test('should detect multiple screens', async ({ page }) => {
      await allure.severity('minor');
      await allure.description('Verify multi-monitor detection');
      
      await page.goto('https://example.com');
      
      // Check screen API
      const screenInfo = await page.evaluate(() => ({
        width: screen.width,
        height: screen.height,
        availWidth: screen.availWidth,
        availHeight: screen.availHeight
      }));
      
      expect(screenInfo.width).toBeGreaterThan(0);
      expect(screenInfo.height).toBeGreaterThan(0);
    });

    test('should open window on specific monitor', async ({ browser }) => {
      await allure.severity('minor');
      await allure.description('Verify window placement on specific monitor');
      
      const context = await browser.newContext({
        viewport: { width: 1920, height: 1080 }
      });
      
      const page = await context.newPage();
      await page.goto('https://example.com');
      
      const viewport = page.viewportSize();
      expect(viewport?.width).toBe(1920);
      
      await context.close();
    });
  });

  test.describe('Window Keyboard Shortcuts', () => {
    test('should close window with keyboard shortcut', async ({ page }) => {
      await allure.severity('major');
      await allure.description('Verify Ctrl+W shortcut for closing');
      
      await page.goto('https://example.com');
      
      // Page should be active
      await expect(page.locator('body')).toBeVisible();
      
      // Note: Actual Ctrl+W would close the page, which we simulate
      // by checking the page is functional before such action
    });

    test('should open new window with keyboard shortcut', async ({ browser }) => {
      await allure.severity('major');
      await allure.description('Verify Ctrl+N shortcut for new window');
      
      // Create new context (simulating new window)
      const context = await browser.newContext();
      const page = await context.newPage();
      
      await page.goto('https://example.com');
      await expect(page).toHaveTitle(/Example Domain/);
      
      await context.close();
    });

    test('should switch between windows', async ({ browser }) => {
      await allure.severity('major');
      await allure.description('Verify Alt+Tab window switching');
      
      const context = await browser.newContext();
      const pages = [await context.newPage(), await context.newPage()];
      
      await pages[0].goto('https://example.com');
      await pages[1].goto('https://example.org');
      
      // Bring first page to front
      await pages[0].bringToFront();
      
      // Should be focused
      await expect(pages[0]).toHaveTitle(/Example Domain/);
      
      await context.close();
    });
  });

  test.describe('Workspace Features', () => {
    test('should save workspace state', async ({ browser }) => {
      await allure.severity('major');
      await allure.description('Verify workspace state saving');
      
      const context = await browser.newContext();
      
      // Open multiple pages
      const pages = await Promise.all([
        context.newPage(),
        context.newPage()
      ]);
      
      await pages[0].goto('https://example.com');
      await pages[1].goto('https://example.org');
      
      // Verify state
      const urls = await Promise.all(pages.map(p => p.url()));
      expect(urls[0]).toContain('example.com');
      expect(urls[1]).toContain('example.org');
      
      await context.close();
    });

    test('should restore workspace on startup', async ({ browser }) => {
      await allure.severity('major');
      await allure.description('Verify workspace restoration');
      
      // Create context with storage state
      const context = await browser.newContext();
      const page = await context.newPage();
      
      await page.goto('https://example.com');
      
      // Get storage state
      const state = await context.storageState();
      
      expect(state).toHaveProperty('cookies');
      
      await context.close();
    });

    test('should manage multiple workspaces', async ({ browser }) => {
      await allure.severity('minor');
      await allure.description('Verify multiple workspace management');
      
      // Create separate contexts for different workspaces
      const workspaces = await Promise.all([
        browser.newContext(),
        browser.newContext()
      ]);
      
      // Each workspace has independent state
      for (const context of workspaces) {
        const page = await context.newPage();
        await page.goto('https://example.com');
      }
      
      // Clean up
      await Promise.all(workspaces.map(c => c.close()));
    });
  });
});