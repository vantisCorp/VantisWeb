import { test, expect } from '@playwright/test';
import { SettingsPage } from '../../pages/settings-page';
import { BrowserPage } from '../../pages/browser-page';

test.describe('User Preferences and Settings', () => {
  let settingsPage: SettingsPage;
  let browserPage: BrowserPage;

  test.beforeEach(async ({ page }) => {
    settingsPage = new SettingsPage(page);
    browserPage = new BrowserPage(page);
    
    test.info().annotations.push({
      type: 'epic',
      description: 'User Preferences'
    });
    test.info().annotations.push({
      type: 'feature',
      description: 'Settings Management'
    });
  });

  test('should navigate to settings page', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify settings page can be accessed'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'critical'
    });

    await settingsPage.navigateToSettings();
    
    // Verify settings page is displayed
    await expect(page).toHaveURL(/about:preferences|about:settings/);
    await expect(page.locator('#settings-container')).toBeVisible();
  });

  test('should navigate to specific settings section', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify navigation to specific settings sections'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });

    await settingsPage.navigateToSettings();
    
    // Navigate to Privacy section
    await settingsPage.navigateToSection('privacy');
    await expect(page.locator('#privacy-settings')).toBeVisible();
    
    // Navigate to Security section
    await settingsPage.navigateToSection('security');
    await expect(page.locator('#security-settings')).toBeVisible();
  });

  test('should toggle boolean settings', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify boolean settings can be toggled'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'critical'
    });

    await settingsPage.navigateToSection('general');
    
    // Get initial state
    const initialValue = await settingsPage.getSettingValue('auto-update');
    
    // Toggle setting
    await settingsPage.toggleSetting('auto-update');
    
    // Verify setting changed
    const newValue = await settingsPage.getSettingValue('auto-update');
    expect(newValue).toBe(!initialValue);
  });

  test('should select dropdown options', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify dropdown settings can be changed'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });

    await settingsPage.navigateToSection('general');
    
    // Select different option
    await settingsPage.selectDropdownOption('default-search-engine', 'duckduckgo');
    
    // Verify selection is saved
    const selectedValue = await page.locator('#default-search-engine').inputValue();
    expect(selectedValue).toBe('duckduckgo');
  });

  test('should input text in settings fields', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify text input settings work correctly'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });

    await settingsPage.navigateToSection('general');
    
    // Input home page URL
    await settingsPage.inputTextField('home-page-url', 'https://example.com');
    
    // Verify input is saved
    const value = await settingsPage.getSettingValue('home-page-url');
    expect(value).toBe('https://example.com');
  });

  test('should reset settings to default', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify settings can be reset to default values'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });

    await settingsPage.navigateToSection('general');
    
    // Change a setting
    await settingsPage.toggleSetting('auto-update');
    
    // Reset to default
    await settingsPage.resetToDefault();
    
    // Verify setting is reset
    const defaultValue = await settingsPage.getSettingValue('auto-update');
    expect(defaultValue).toBe(true); // Assuming default is true
  });

  test('should search in settings', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify settings search functionality works'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });

    await settingsPage.navigateToSettings();
    
    // Search for a setting
    await settingsPage.searchSettings('privacy');
    
    // Verify search results
    const resultsCount = await settingsPage.getSearchResultsCount();
    expect(resultsCount).toBeGreaterThan(0);
  });

  test('should manage privacy settings', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify privacy settings can be configured'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'critical'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'privacy'
    });

    await settingsPage.navigateToSection('privacy');
    
    // Toggle tracking protection
    await settingsPage.toggleSetting('tracking-protection');
    
    // Toggle do not track
    await settingsPage.toggleSetting('do-not-track');
    
    // Verify settings are applied
    const trackingProtection = await settingsPage.getSettingValue('tracking-protection');
    const doNotTrack = await settingsPage.getSettingValue('do-not-track');
    
    expect(trackingProtection).toBe(true);
    expect(doNotTrack).toBe(true);
  });

  test('should manage security settings', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify security settings can be configured'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'critical'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'security'
    });

    await settingsPage.navigateToSection('security');
    
    // Toggle safe browsing
    await settingsPage.toggleSetting('safe-browsing');
    
    // Toggle https-only mode
    await settingsPage.toggleSetting('https-only-mode');
    
    // Verify settings are applied
    const safeBrowsing = await settingsPage.getSettingValue('safe-browsing');
    const httpsOnly = await settingsPage.getSettingValue('https-only-mode');
    
    expect(safeBrowsing).toBe(true);
    expect(httpsOnly).toBe(true);
  });

  test('should manage appearance settings', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify appearance settings can be configured'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });

    await settingsPage.navigateToSection('appearance');
    
    // Select theme
    await settingsPage.selectDropdownOption('theme', 'dark');
    
    // Toggle animations
    await settingsPage.toggleSetting('animations');
    
    // Verify settings are applied
    const theme = await page.locator('#theme').inputValue();
    expect(theme).toBe('dark');
  });

  test('should manage search engine settings', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify search engine settings can be configured'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });

    await settingsPage.navigateToSection('search');
    
    // Set default search engine
    await settingsPage.selectDropdownOption('default-search-engine', 'google');
    
    // Toggle search suggestions
    await settingsPage.toggleSetting('search-suggestions');
    
    // Verify settings
    const searchEngine = await page.locator('#default-search-engine').inputValue();
    expect(searchEngine).toBe('google');
  });

  test('should manage download settings', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify download settings can be configured'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });

    await settingsPage.navigateToSection('downloads');
    
    // Set download folder
    await settingsPage.inputTextField('download-folder', '/home/user/Downloads');
    
    // Toggle always ask for download location
    await settingsPage.toggleSetting('always-ask-download');
    
    // Verify settings
    const downloadFolder = await settingsPage.getSettingValue('download-folder');
    expect(downloadFolder).toBe('/home/user/Downloads');
  });

  test('should manage language and localization settings', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify language settings can be configured'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });

    await settingsPage.navigateToSection('language');
    
    // Set language
    await settingsPage.selectDropdownOption('preferred-language', 'en-US');
    
    // Add alternate language
    await page.click('#add-language-button');
    await page.click('option[value="pl-PL"]');
    
    // Verify languages
    const preferredLanguage = await page.locator('#preferred-language').inputValue();
    expect(preferredLanguage).toBe('en-US');
  });

  test('should manage accessibility settings', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify accessibility settings can be configured'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'accessibility'
    });

    await settingsPage.navigateToSection('accessibility');
    
    // Toggle text size
    await settingsPage.selectDropdownOption('text-size', 'large');
    
    // Toggle high contrast mode
    await settingsPage.toggleSetting('high-contrast');
    
    // Toggle reduced motion
    await settingsPage.toggleSetting('reduced-motion');
    
    // Verify settings
    const textSize = await page.locator('#text-size').inputValue();
    expect(textSize).toBe('large');
  });

  test('should manage keyboard shortcuts', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify keyboard shortcuts can be customized'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });

    await settingsPage.navigateToSection('shortcuts');
    
    // Click on shortcut to modify
    await page.click('#shortcut-new-tab');
    await page.keyboard.press('Control+T');
    
    // Verify shortcut is set
    const shortcut = await page.locator('#shortcut-new-tab').inputValue();
    expect(shortcut).toBe('Ctrl+T');
  });

  test('should manage sync settings', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify sync settings can be configured'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });

    await settingsPage.navigateToSection('sync');
    
    // Toggle sync for bookmarks
    await settingsPage.toggleSetting('sync-bookmarks');
    
    // Toggle sync for history
    await settingsPage.toggleSetting('sync-history');
    
    // Toggle sync for passwords
    await settingsPage.toggleSetting('sync-passwords');
    
    // Verify settings
    const syncBookmarks = await settingsPage.getSettingValue('sync-bookmarks');
    expect(syncBookmarks).toBe(true);
  });

  test('should verify setting persistence after browser restart', async ({ page, context }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify settings persist after browser restart'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'persistence'
    });

    // Change a setting
    await settingsPage.navigateToSection('general');
    await settingsPage.toggleSetting('auto-update');
    const settingValue = await settingsPage.getSettingValue('auto-update');
    
    // Simulate browser restart
    await context.close();
    const newContext = await browserPage.browser.newContext();
    const newPage = await newContext.newPage();
    
    // Verify setting persisted
    const newSettingsPage = new SettingsPage(newPage);
    await newSettingsPage.navigateToSection('general');
    const persistedValue = await newSettingsPage.getSettingValue('auto-update');
    
    expect(persistedValue).toBe(settingValue);
    
    await newContext.close();
  });

  test('should handle invalid settings values', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify invalid settings values are handled gracefully'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });

    await settingsPage.navigateToSection('general');
    
    // Try to input invalid URL
    await settingsPage.inputTextField('home-page-url', 'not-a-valid-url');
    
    // Verify error message
    const errorMessage = await page.locator('.error-message');
    await expect(errorMessage).toBeVisible();
  });

  test('should export and import settings', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify settings can be exported and imported'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });

    await settingsPage.navigateToSettings();
    
    // Export settings
    await page.click('#export-settings-button');
    
    // Verify export file is created
    const exportNotification = await page.locator('.export-success-notification');
    await expect(exportNotification).toBeVisible();
    
    // Import settings
    await page.click('#import-settings-button');
    await page.setInputFiles('#import-file-input', 'settings.json');
    
    // Verify import success
    const importNotification = await page.locator('.import-success-notification');
    await expect(importNotification).toBeVisible();
  });
});