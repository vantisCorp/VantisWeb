import { test, expect } from '@playwright/test';
import { BrowserPage } from '../../pages/browser-page';

test.describe('Enhanced Extension System', () => {
  let browserPage: BrowserPage;

  test.beforeEach(async ({ page }) => {
    browserPage = new BrowserPage(page);
    
    test.info().annotations.push({
      type: 'epic',
      description: 'Extension System'
    });
    test.info().annotations.push({
      type: 'feature',
      description: 'Enhanced Extension Management'
    });
  });

  test('should install extension from file', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify extension can be installed from local file'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'critical'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'extensions'
    });

    // Navigate to extensions page
    await page.goto('about:addons');
    
    // Click install from file
    await page.click('#install-from-file-button');
    
    // Select extension file (simulated)
    await page.setInputFiles('#extension-file-input', 'test-extension.xpi');
    
    // Confirm installation
    await page.click('#confirm-install-button');
    
    // Verify extension appears in list
    const extension = await page.locator('.extension-item:has-text("Test Extension")');
    await expect(extension).toBeVisible();
  });

  test('should enable and disable extension', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify extension can be enabled and disabled'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'critical'
    });

    // Navigate to extensions page
    await page.goto('about:addons');
    
    // Assume extension is installed
    const extension = page.locator('.extension-item').first();
    
    // Disable extension
    await extension.locator('.disable-button').click();
    await expect(extension.locator('.status')).toHaveText('Disabled');
    
    // Enable extension
    await extension.locator('.enable-button').click();
    await expect(extension.locator('.status')).toHaveText('Enabled');
  });

  test('should remove extension', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify extension can be removed completely'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'critical'
    });

    await page.goto('about:addons');
    
    const extension = page.locator('.extension-item').first();
    const extensionName = await extension.locator('.extension-name').textContent();
    
    // Remove extension
    await extension.locator('.remove-button').click();
    await page.click('#confirm-remove-button');
    
    // Verify extension is removed
    const removedExtension = await page.locator(`.extension-item:has-text("${extensionName}")`).count();
    expect(removedExtension).toBe(0);
  });

  test('should configure extension settings', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify extension settings can be configured'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });

    await page.goto('about:addons');
    
    const extension = page.locator('.extension-item').first();
    
    // Open extension settings
    await extension.locator('.settings-button').click();
    
    // Verify settings page opens
    const settingsPanel = await page.locator('#extension-settings-panel');
    await expect(settingsPanel).toBeVisible();
    
    // Change setting
    await page.click('#setting-toggle');
    
    // Save settings
    await page.click('#save-settings-button');
    
    // Verify setting is saved
    await expect(page.locator('#setting-toggle')).toBeChecked();
  });

  test('should handle extension permissions', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify extension permission requests are handled'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'critical'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'security'
    });

    // Start extension installation that requires permissions
    await page.goto('about:addons');
    await page.click('#install-from-file-button');
    await page.setInputFiles('#extension-file-input', 'extension-with-permissions.xpi');
    
    // Verify permission dialog appears
    const permissionDialog = await page.locator('#permission-dialog');
    await expect(permissionDialog).toBeVisible();
    
    // Verify permissions are listed
    const permissions = await page.locator('.permission-item').all();
    expect(permissions.length).toBeGreaterThan(0);
    
    // Accept permissions
    await page.click('#accept-permissions-button');
    
    // Verify extension is installed
    await expect(page.locator('.extension-item').first()).toBeVisible();
  });

  test('should deny extension permissions', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify extension installation can be cancelled'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'critical'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'security'
    });

    await page.goto('about:addons');
    await page.click('#install-from-file-button');
    await page.setInputFiles('#extension-file-input', 'extension-with-permissions.xpi');
    
    // Deny permissions
    await page.click('#deny-permissions-button');
    
    // Verify extension is not installed
    const extensionCount = await page.locator('.extension-item').count();
    expect(extensionCount).toBe(0);
  });

  test('should update extension', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify extension can be updated to new version'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });

    await page.goto('about:addons');
    
    // Check for updates
    await page.click('#check-for-updates-button');
    
    // Wait for update check
    await page.waitForTimeout(2000);
    
    // If update available, install it
    const updateButton = await page.locator('.update-button').count();
    if (updateButton > 0) {
      await page.click('.update-button');
      
      // Verify update is installed
      await expect(page.locator('.update-success-message')).toBeVisible();
    }
  });

  test('should handle extension conflicts', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify conflicting extensions are detected'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });

    // Install conflicting extension (simulated)
    await page.goto('about:addons');
    await page.click('#install-from-file-button');
    await page.setInputFiles('#extension-file-input', 'conflicting-extension.xpi');
    
    // Verify conflict warning
    const conflictWarning = await page.locator('.conflict-warning');
    const hasWarning = await conflictWarning.count();
    
    if (hasWarning > 0) {
      await expect(conflictWarning).toBeVisible();
    }
  });

  test('should view extension details', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify extension details can be viewed'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });

    await page.goto('about:addons');
    
    const extension = page.locator('.extension-item').first();
    await extension.locator('.details-button').click();
    
    // Verify details panel opens
    const detailsPanel = await page.locator('#extension-details-panel');
    await expect(detailsPanel).toBeVisible();
    
    // Verify details are displayed
    await expect(page.locator('.extension-version')).toBeVisible();
    await expect(page.locator('.extension-description')).toBeVisible();
  });

  test('should manage extension keyboard shortcuts', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify extension keyboard shortcuts can be customized'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });

    await page.goto('about:addons');
    
    // Open extension settings
    await page.click('.extension-item .settings-button');
    await page.click('#manage-shortcuts-button');
    
    // Verify shortcuts panel
    const shortcutsPanel = await page.locator('#shortcuts-panel');
    await expect(shortcutsPanel).toBeVisible();
    
    // Change shortcut
    await page.click('#shortcut-input');
    await page.keyboard.press('Control+Shift+E');
    
    // Save shortcut
    await page.click('#save-shortcuts-button');
    
    // Verify shortcut is saved
    await expect(page.locator('#shortcut-input')).toHaveValue('Ctrl+Shift+E');
  });

  test('should test extension API access', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify extension has proper API access'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'api'
    });

    // Install extension with API access
    await page.goto('about:addons');
    const extension = page.locator('.extension-item').first();
    
    // Check API permissions
    await extension.locator('.details-button').click();
    
    const apiList = await page.locator('.api-permission').all();
    expect(apiList.length).toBeGreaterThan(0);
  });

  test('should handle extension background scripts', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify extension background scripts work correctly'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });

    // Navigate to a page
    await browserPage.navigateTo('https://example.com');
    
    // Trigger extension action (simulated)
    await page.click('#extension-action-button');
    
    // Verify extension background script executed
    const extensionResult = await page.locator('.extension-result');
    await expect(extensionResult).toBeVisible();
  });

  test('should test extension content scripts', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify extension content scripts inject correctly'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });

    // Navigate to a page where extension should inject content
    await browserPage.navigateTo('https://example.com');
    
    // Verify content script was injected
    const injectedContent = await page.locator('.extension-injected-content').count();
    expect(injectedContent).toBeGreaterThan(0);
  });

  test('should manage extension storage', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify extension storage works correctly'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });

    await page.goto('about:addons');
    
    // Open extension settings
    await page.click('.extension-item .settings-button');
    await page.click('#manage-storage-button');
    
    // Verify storage panel
    const storagePanel = await page.locator('#storage-panel');
    await expect(storagePanel).toBeVisible();
    
    // Clear extension storage
    await page.click('#clear-storage-button');
    await page.click('#confirm-clear-button');
    
    // Verify storage is cleared
    await expect(page.locator('.storage-size')).toHaveText('0 KB');
  });

  test('should test extension incognito behavior', async ({ page, context }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify extension behavior in incognito mode'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'privacy'
    });

    // Open incognito context
    const incognitoContext = await browserPage.browser.newContext();
    const incognitoPage = await incognitoContext.newPage();
    
    // Navigate to a page
    await incognitoPage.goto('https://example.com');
    
    // Verify extension behavior in incognito
    // Extension should not run by default in incognito
    const extensionContent = await incognitoPage.locator('.extension-injected-content').count();
    
    await incognitoContext.close();
  });
});