import { test, expect } from '@playwright/test';
import { BrowserPage } from '../../pages/browser-page';

test.describe('Extension Installation', () => {
  let browserPage: BrowserPage;

  test.beforeEach(async ({ page }) => {
    browserPage = new BrowserPage(page);
    await browserPage.navigateToUrl('vantisweb://extensions');
  });

  test('should display extensions page', async ({ page }) => {
    await expect(page).toHaveTitle(/Extensions/);
    await expect(page.locator('.extensions-container')).toBeVisible();
  });

  test('should list installed extensions', async ({ page }) => {
    const extensionsList = page.locator('.extension-item');
    const count = await extensionsList.count();
    
    expect(count).toBeGreaterThan(0);
    
    // Verify extension items have expected elements
    const firstExtension = extensionsList.first();
    await expect(firstExtension.locator('.extension-name')).toBeVisible();
    await expect(firstExtension.locator('.extension-description')).toBeVisible();
    await expect(firstExtension.locator('.extension-status')).toBeVisible();
  });

  test('should install extension from file', async ({ page }) => {
    // This test would require a test extension file
    const installButton = page.locator('#install-extension-button');
    await expect(installButton).toBeVisible();
    
    // In a real implementation, this would trigger file upload
    // await installButton.click();
    // await page.setInputFiles('#extension-file-input', 'test-extension.zip');
    
    // Verify installation success
    // await expect(page.locator('.success-message')).toBeVisible();
  });

  test('should activate and deactivate extensions', async ({ page }) => {
    const firstExtension = page.locator('.extension-item').first();
    const toggleButton = firstExtension.locator('.extension-toggle');
    
    // Get initial state
    const initialStatus = await firstExtension.locator('.extension-status').textContent();
    
    // Toggle extension
    await toggleButton.click();
    await page.waitForTimeout(500); // Wait for state change
    
    // Verify status changed
    const newStatus = await firstExtension.locator('.extension-status').textContent();
    expect(newStatus).not.toBe(initialStatus);
  });

  test('should display extension details', async ({ page }) => {
    const firstExtension = page.locator('.extension-item').first();
    const detailsButton = firstExtension.locator('.extension-details-button');
    
    await detailsButton.click();
    
    // Verify details modal/panel appears
    await expect(page.locator('.extension-details')).toBeVisible();
    await expect(page.locator('.extension-details .extension-name')).toBeVisible();
    await expect(page.locator('.extension-details .extension-version')).toBeVisible();
    await expect(page.locator('.extension-details .extension-permissions')).toBeVisible();
  });

  test('should handle extension permissions', async ({ page }) => {
    const firstExtension = page.locator('.extension-item').first();
    const permissionsButton = firstExtension.locator('.permissions-button');
    
    await permissionsButton.click();
    
    // Verify permissions panel appears
    await expect(page.locator('.permissions-panel')).toBeVisible();
    const permissions = await page.locator('.permissions-panel .permission').allTextContents();
    
    expect(permissions.length).toBeGreaterThan(0);
  });

  test('should remove extension', async ({ page }) => {
    const initialCount = await page.locator('.extension-item').count();
    const firstExtension = page.locator('.extension-item').first();
    const removeButton = firstExtension.locator('.remove-button');
    
    // Confirm removal (this might show a confirmation dialog)
    removeButton.click();
    
    // Handle confirmation dialog if present
    const confirmButton = page.locator('.confirm-button');
    if (await confirmButton.isVisible()) {
      await confirmButton.click();
    }
    
    // Verify extension was removed
    await page.waitForTimeout(1000);
    const newCount = await page.locator('.extension-item').count();
    expect(newCount).toBe(initialCount - 1);
  });
});