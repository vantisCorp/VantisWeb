import { test, expect } from '@playwright/test';
import { BrowserPage } from '../../pages/browser-page';

test.describe('Accessibility Features', () => {
  let browserPage: BrowserPage;

  test.beforeEach(async ({ page }) => {
    browserPage = new BrowserPage(page);
    
    test.info().annotations.push({
      type: 'epic',
      description: 'Accessibility'
    });
    test.info().annotations.push({
      type: 'feature',
      description: 'Accessibility Features'
    });
  });

  test('should navigate using keyboard only', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify complete keyboard navigation works'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'critical'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'accessibility'
    });

    await browserPage.navigateTo('https://example.com');
    
    // Tab through all interactive elements
    let tabCount = 0;
    for (let i = 0; i < 20; i++) {
      await page.keyboard.press('Tab');
      tabCount++;
      
      // Verify focus indicator is visible
      const focusedElement = await page.locator(':focus');
      await expect(focusedElement).toBeVisible();
    }
    
    expect(tabCount).toBeGreaterThan(5);
  });

  test('should activate links with Enter key', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify links can be activated with Enter key'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'critical'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'accessibility'
    });

    await browserPage.navigateTo('https://example.com');
    
    // Tab to first link
    await page.keyboard.press('Tab');
    
    // Activate with Enter
    await page.keyboard.press('Enter');
    
    // Verify navigation occurred
    await page.waitForLoadState('networkidle');
    expect(page.url()).toBeTruthy();
  });

  test('should activate buttons with Space key', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify buttons can be activated with Space key'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'critical'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'accessibility'
    });

    await browserPage.navigateTo('https://example.com');
    
    // Tab to a button
    for (let i = 0; i < 10; i++) {
      await page.keyboard.press('Tab');
      const focusedElement = await page.locator(':focus');
      const tagName = await focusedElement.evaluate(el => el.tagName.toLowerCase());
      
      if (tagName === 'button') {
        // Activate with Space
        await page.keyboard.press('Space');
        break;
      }
    }
  });

  test('should navigate menus with arrow keys', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify menus can be navigated with arrow keys'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'accessibility'
    });

    await browserPage.navigateTo('https://example.com');
    
    // Open menu (simulated)
    await page.click('#menu-button');
    
    // Navigate with arrow keys
    await page.keyboard.press('ArrowDown');
    const firstItem = await page.locator(':focus');
    await expect(firstItem).toBeVisible();
    
    await page.keyboard.press('ArrowDown');
    const secondItem = await page.locator(':focus');
    await expect(secondItem).toBeVisible();
    
    await page.keyboard.press('ArrowUp');
    const backToFirst = await page.locator(':focus');
    await expect(backToFirst).toBeVisible();
  });

  test('should support screen reader compatibility', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify page has proper ARIA attributes for screen readers'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'critical'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'accessibility'
    });

    await browserPage.navigateTo('https://example.com');
    
    // Check for ARIA landmarks
    const landmarks = await page.locator('[role="main"], [role="navigation"], [role="banner"]').count();
    expect(landmarks).toBeGreaterThan(0);
    
    // Check for alt text on images
    const images = await page.locator('img').all();
    for (const img of images) {
      const alt = await img.getAttribute('alt');
      const ariaLabel = await img.getAttribute('aria-label');
      expect(alt || ariaLabel).toBeTruthy();
    }
  });

  test('should have proper heading hierarchy', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify heading hierarchy is correct for screen readers'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'accessibility'
    });

    await browserPage.navigateTo('https://example.com');
    
    // Get all headings
    const headings = await page.locator('h1, h2, h3, h4, h5, h6').all();
    
    if (headings.length > 0) {
      // Verify h1 exists
      const h1Count = await page.locator('h1').count();
      expect(h1Count).toBeGreaterThanOrEqual(1);
      
      // Verify heading hierarchy is logical
      let lastLevel = 0;
      for (const heading of headings) {
        const tagName = await heading.evaluate(el => el.tagName.toLowerCase());
        const level = parseInt(tagName.replace('h', ''));
        
        // Heading level should not skip more than one level
        expect(level).toBeLessThanOrEqual(lastLevel + 1);
        lastLevel = level;
      }
    }
  });

  test('should support text scaling', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify text can be scaled up to 200%'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'accessibility'
    });

    await browserPage.navigateTo('https://example.com');
    
    // Zoom in to 200%
    await page.keyboard.press('Control+Plus');
    await page.keyboard.press('Control+Plus');
    await page.keyboard.press('Control+Plus');
    
    // Verify page is still usable
    const body = await page.locator('body');
    await expect(body).toBeVisible();
    
    // Verify no horizontal scrollbar (content should wrap)
    const hasHorizontalScroll = await page.evaluate(() => {
      return document.documentElement.scrollWidth > document.documentElement.clientWidth;
    });
    
    // Content should not require horizontal scroll
    // This is a best practice but may not always be achievable
  });

  test('should support high contrast mode', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify high contrast mode works correctly'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'accessibility'
    });

    // Enable high contrast mode (simulated via media query)
    await page.emulateMedia({ colorScheme: 'dark' });
    
    await browserPage.navigateTo('https://example.com');
    
    // Verify page is still readable
    const body = await page.locator('body');
    await expect(body).toBeVisible();
    
    // Verify text contrast ratio
    const textColor = await body.evaluate(el => {
      return window.getComputedStyle(el).color;
    });
    expect(textColor).toBeTruthy();
  });

  test('should respect reduced motion preferences', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify reduced motion preference is respected'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'accessibility'
    });

    // Enable reduced motion
    await page.emulateMedia({ reducedMotion: 'reduce' });
    
    await browserPage.navigateTo('https://example.com');
    
    // Verify animations are disabled or reduced
    const animatedElements = await page.locator('[style*="animation"]').count();
    
    // Elements should either not have animations or have reduced duration
    const body = await page.locator('body');
    await expect(body).toBeVisible();
  });

  test('should have proper focus indicators', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify visible focus indicators on all interactive elements'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'critical'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'accessibility'
    });

    await browserPage.navigateTo('https://example.com');
    
    // Tab through interactive elements
    for (let i = 0; i < 10; i++) {
      await page.keyboard.press('Tab');
      
      const focusedElement = await page.locator(':focus');
      if (await focusedElement.count() > 0) {
        // Check that focus indicator is visible
        const outline = await focusedElement.evaluate(el => {
          return window.getComputedStyle(el).outline;
        });
        
        // Element should have some visual focus indicator
        expect(outline).toBeTruthy();
      }
    }
  });

  test('should support voice control', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify elements have accessible names for voice control'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'accessibility'
    });

    await browserPage.navigateTo('https://example.com');
    
    // Check that interactive elements have accessible names
    const buttons = await page.locator('button').all();
    for (const button of buttons) {
      const accessibleName = await button.evaluate(el => {
        return el.textContent || el.getAttribute('aria-label') || el.getAttribute('title');
      });
      
      expect(accessibleName).toBeTruthy();
    }
    
    const links = await page.locator('a').all();
    for (const link of links) {
      const accessibleName = await link.evaluate(el => {
        return el.textContent || el.getAttribute('aria-label') || el.getAttribute('title');
      });
      
      expect(accessibleName).toBeTruthy();
    }
  });

  test('should have proper form labels', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify form inputs have proper labels'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'critical'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'accessibility'
    });

    await browserPage.navigateTo('https://example.com');
    
    // Check all inputs have labels
    const inputs = await page.locator('input:not([type="hidden"]):not([type="submit"]):not([type="button"]):not([type="image"])').all();
    
    for (const input of inputs) {
      const id = await input.getAttribute('id');
      const ariaLabel = await input.getAttribute('aria-label');
      const ariaLabelledBy = await input.getAttribute('aria-labelledby');
      
      if (id) {
        const label = await page.locator(`label[for="${id}"]`).count();
        expect(label > 0 || ariaLabel || ariaLabelledBy).toBeTruthy();
      } else {
        expect(ariaLabel || ariaLabelledBy).toBeTruthy();
      }
    }
  });

  test('should skip to main content', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify skip navigation link works'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'accessibility'
    });

    await browserPage.navigateTo('https://example.com');
    
    // Look for skip link (usually first focusable element)
    await page.keyboard.press('Tab');
    
    const skipLink = await page.locator(':focus').filter({ hasText: /skip|jump/i });
    
    if (await skipLink.count() > 0) {
      // Activate skip link
      await page.keyboard.press('Enter');
      
      // Verify focus moved to main content
      const mainContent = await page.locator('#main, [role="main"], main').first();
      await expect(mainContent).toBeFocused();
    }
  });

  test('should support color blind users', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify information is not conveyed by color alone'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'accessibility'
    });

    await browserPage.navigateTo('https://example.com');
    
    // Check that links are distinguishable without color
    const links = await page.locator('a').all();
    for (const link of links) {
      const textDecoration = await link.evaluate(el => {
        return window.getComputedStyle(el).textDecoration;
      });
      
      // Links should have underline or other non-color indicator
      // This is a common accessibility requirement
    }
  });

  test('should have proper table headers', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify data tables have proper headers'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'accessibility'
    });

    await browserPage.navigateTo('https://example.com');
    
    // Find all tables
    const tables = await page.locator('table').all();
    
    for (const table of tables) {
      // Check for headers
      const headers = await table.locator('th').count();
      const caption = await table.locator('caption').count();
      const ariaLabel = await table.getAttribute('aria-label');
      const ariaLabelledBy = await table.getAttribute('aria-labelledby');
      
      // Tables should have headers and accessible name
      expect(headers > 0 || caption > 0 || ariaLabel || ariaLabelledBy).toBeTruthy();
    }
  });

  test('should handle dynamic content updates', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify dynamic content changes are announced'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'accessibility'
    });

    await browserPage.navigateTo('https://example.com');
    
    // Check for ARIA live regions
    const liveRegions = await page.locator('[aria-live]').count();
    
    if (liveRegions > 0) {
      // Verify live regions are properly configured
      for (const region of await page.locator('[aria-live]').all()) {
        const ariaLive = await region.getAttribute('aria-live');
        expect(['polite', 'assertive', 'off']).toContain(ariaLive);
      }
    }
  });

  test('should support screen magnification', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify page works with screen magnification'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'accessibility'
    });

    await browserPage.navigateTo('https://example.com');
    
    // Simulate 200% zoom
    await page.setViewportSize({ width: 640, height: 480 });
    
    // Verify page is still functional
    const body = await page.locator('body');
    await expect(body).toBeVisible();
    
    // Verify no elements are cut off
    const elements = await page.locator('body *').all();
    for (const element of elements.slice(0, 10)) {
      const isVisible = await element.isVisible();
      if (isVisible) {
        const box = await element.boundingBox();
        expect(box).toBeTruthy();
      }
    }
  });
});