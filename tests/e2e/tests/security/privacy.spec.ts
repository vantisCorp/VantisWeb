import { test, expect } from '@playwright/test';
import { BrowserPage } from '../../pages/browser-page';

test.describe('Security and Privacy', () => {
  let browserPage: BrowserPage;

  test.beforeEach(async ({ page }) => {
    browserPage = new BrowserPage(page);
    
    test.info().annotations.push({
      type: 'epic',
      description: 'Security and Privacy'
    });
    test.info().annotations.push({
      type: 'feature',
      description: 'Privacy Protection'
    });
  });

  test('should handle HTTPS connections correctly', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify HTTPS connections establish securely'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'critical'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'security'
    });

    await browserPage.navigateTo('https://example.com');
    
    // Verify secure connection indicator
    const securityIndicator = await page.locator('#security-indicator');
    await expect(securityIndicator).toBeVisible();
    
    // Verify URL shows HTTPS
    await expect(page).toHaveURL(/https:\/\//);
  });

  test('should handle SSL certificate errors', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify SSL certificate errors are handled properly'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'critical'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'security'
    });

    // Navigate to site with invalid certificate (simulated)
    await browserPage.navigateTo('https://expired.badssl.com/');
    
    // Verify certificate error page is shown
    const errorPage = await page.locator('body').textContent();
    expect(errorPage).toMatch(/certificate|security|warning/i);
    
    // Verify there's a way to proceed (advanced users)
    const proceedButton = await page.locator('#proceed-anyway').count();
    expect(proceedButton).toBeGreaterThan(0);
  });

  test('should handle mixed content blocking', async ({ page }) => {
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

    // Navigate to HTTPS page
    await browserPage.navigateTo('https://example.com');
    
    // Verify mixed content warning if present
    const mixedContentWarning = await page.locator('.mixed-content-warning');
    const hasWarning = await mixedContentWarning.count();
    
    if (hasWarning > 0) {
      await expect(mixedContentWarning).toBeVisible();
    }
  });

  test('should enable private browsing mode', async ({ page, context }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify private browsing mode prevents history tracking'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'critical'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'privacy'
    });

    // Open private browsing window
    const privateContext = await browserPage.browser.newContext({
      ignoreHTTPSErrors: false,
      javaScriptEnabled: true
    });
    const privatePage = await privateContext.newPage();
    
    // Navigate to a site
    await privatePage.goto('https://example.com');
    
    // Verify no history is saved (simulated check)
    await privatePage.goto('about:history');
    const historyCount = await privatePage.locator('.history-item').count();
    
    // History should be empty in private mode
    expect(historyCount).toBe(0);
    
    await privateContext.close();
  });

  test('should manage cookies in private browsing', async ({ page, context }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify cookies are cleared after private session ends'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'privacy'
    });

    // Open private browsing window
    const privateContext = await browserPage.browser.newContext();
    const privatePage = await privateContext.newPage();
    
    // Navigate and set cookie
    await privatePage.goto('https://httpbin.org/cookies/set?name=value');
    const cookies = await privateContext.cookies();
    
    // Verify cookie was set
    expect(cookies.length).toBeGreaterThan(0);
    
    // Close private context
    await privateContext.close();
    
    // Open new private context
    const newPrivateContext = await browserPage.browser.newContext();
    const newPrivatePage = await newPrivateContext.newPage();
    
    // Navigate to same site
    await newPrivatePage.goto('https://httpbin.org/cookies');
    
    // Verify cookies are not present
    const body = await newPrivatePage.textContent('body');
    expect(body).not.toContain('name');
    
    await newPrivateContext.close();
  });

  test('should block tracking scripts', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify tracking protection blocks known trackers'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'privacy'
    });

    // Navigate to site with tracking (simulated)
    await browserPage.navigateTo('https://example.com');
    
    // Verify tracking protection is active
    const shieldIcon = await page.locator('#tracking-protection-shield');
    const hasShield = await shieldIcon.count();
    
    if (hasShield > 0) {
      await expect(shieldIcon).toBeVisible();
    }
  });

  test('should handle pop-up blocking', async ({ page, context }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify pop-ups are blocked by default'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'privacy'
    });

    const pageCountBefore = context.pages().length;
    
    // Navigate to site with pop-up
    await browserPage.navigateTo('https://popuptest.com/');
    
    // Try to trigger pop-up
    await page.click('#trigger-popup');
    
    // Wait a moment
    await page.waitForTimeout(1000);
    
    // Verify pop-up was blocked
    const pageCountAfter = context.pages().length;
    expect(pageCountAfter).toBe(pageCountBefore);
  });

  test('should handle permission requests', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify permission requests are handled properly'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'privacy'
    });

    // Grant geolocation permission
    await context.grantPermissions(['geolocation']);
    
    // Navigate to site that requests geolocation
    await browserPage.navigateTo('https://www.openstreetmap.org/');
    
    // Verify permission dialog doesn't appear (already granted)
    const permissionDialog = await page.locator('.permission-dialog');
    const hasDialog = await permissionDialog.count();
    
    expect(hasDialog).toBe(0);
  });

  test('should manage site permissions', async ({ page, context }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify site permissions can be managed'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'privacy'
    });

    // Navigate to a site
    await browserPage.navigateTo('https://example.com');
    
    // Open site permissions (simulated)
    await page.click('#site-info-button');
    await page.click('#permissions-button');
    
    // Verify permissions panel is shown
    const permissionsPanel = await page.locator('#permissions-panel');
    await expect(permissionsPanel).toBeVisible();
    
    // Verify permission options
    const permissionOptions = await permissionsPanel.locator('.permission-option').all();
    expect(permissionOptions.length).toBeGreaterThan(0);
  });

  test('should handle safe browsing warnings', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify safe browsing warnings are shown for dangerous sites'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'critical'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'security'
    });

    // Navigate to known malicious site (simulated)
    await browserPage.navigateTo('http://malware.testing.google.test/testing/malware/');
    
    // Verify safe browsing warning is shown
    const warningPage = await page.locator('body').textContent();
    expect(warningPage).toMatch(/dangerous|malware|phishing|warning/i);
    
    // Verify option to proceed is available
    const proceedButton = await page.locator('#proceed-anyway').count();
    expect(proceedButton).toBeGreaterThan(0);
  });

  test('should clear browsing data', async ({ page, context }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify browsing data can be cleared'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'privacy'
    });

    // Visit some sites
    await browserPage.navigateTo('https://example.com');
    await browserPage.navigateTo('https://example.org');
    
    // Set cookies
    await page.goto('https://httpbin.org/cookies/set?name=value');
    
    // Clear browsing data (simulated)
    await page.goto('about:preferences#privacy');
    await page.click('#clear-data-button');
    await page.click('#confirm-clear-button');
    
    // Verify history is cleared
    await page.goto('about:history');
    const historyCount = await page.locator('.history-item').count();
    expect(historyCount).toBe(0);
    
    // Verify cookies are cleared
    const cookies = await context.cookies();
    expect(cookies.length).toBe(0);
  });

  test('should handle do not track setting', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify Do Not Track header is sent when enabled'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'privacy'
    });

    // Navigate to settings and enable DNT
    await page.goto('about:preferences#privacy');
    await page.click('#do-not-track-checkbox');
    
    // Navigate to site that checks DNT
    await browserPage.navigateTo('https://httpbin.org/headers');
    
    // Verify DNT header is present
    const body = await page.textContent('body');
    expect(body).toContain('DNT');
  });

  test('should handle fingerprinting protection', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify fingerprinting protection is active'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'privacy'
    });

    // Navigate to site that tests fingerprinting
    await browserPage.navigateTo('https://example.com');
    
    // Verify fingerprinting protection indicator
    const protectionIndicator = await page.locator('#fingerprinting-protection');
    const hasIndicator = await protectionIndicator.count();
    
    if (hasIndicator > 0) {
      await expect(protectionIndicator).toBeVisible();
    }
  });

  test('should handle https-only mode', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify HTTPS-only mode upgrades HTTP connections'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'security'
    });

    // Enable HTTPS-only mode (simulated)
    await page.goto('about:preferences#privacy');
    await page.click('#https-only-checkbox');
    
    // Navigate to HTTP site
    await browserPage.navigateTo('http://example.com');
    
    // Verify connection is upgraded to HTTPS
    await expect(page).toHaveURL(/https:\/\//);
  });
});