import { Page, expect } from '@playwright/test';
import { BasePage } from './base-page';

/**
 * SettingsPage - Page object for browser settings and preferences
 * Provides methods for interacting with browser settings
 */
export class SettingsPage extends BasePage {
  constructor(page: Page) {
    super(page);
  }

  /**
   * Navigate to settings page
   */
  async navigateToSettings() {
    await this.page.goto('about:preferences' || 'about:settings');
    await this.waitForPageLoad();
  }

  /**
   * Navigate to specific settings section
   */
  async navigateToSection(section: string) {
    await this.navigateToSettings();
    await this.clickElement(`[data-l10n-id="category-${section}"]`);
    await this.waitForPageLoad();
  }

  /**
   * Get current section
   */
  async getCurrentSection(): Promise<string> {
    const activeSection = await this.page.locator('.category[selected]').getAttribute('data-l10n-id');
    return activeSection || '';
  }

  /**
   * Toggle setting checkbox
   */
  async toggleSetting(settingId: string) {
    const checkbox = this.page.locator(`#${settingId}`);
    const isChecked = await checkbox.isChecked();
    await checkbox.click();
    await this.waitForPageLoad();
  }

  /**
   * Select dropdown option
   */
  async selectDropdownOption(dropdownId: string, optionValue: string) {
    await this.page.click(`#${dropdownId}`);
    await this.page.click(`option[value="${optionValue}"]`);
    await this.waitForPageLoad();
  }

  /**
   * Input text in text field
   */
  async inputTextField(fieldId: string, value: string) {
    await this.page.fill(`#${fieldId}`, value);
    await this.waitForPageLoad();
  }

  /**
   * Click settings button
   */
  async clickSettingsButton(buttonId: string) {
    await this.page.click(`#${buttonId}`);
    await this.waitForPageLoad();
  }

  /**
   * Get setting value
   */
  async getSettingValue(settingId: string): Promise<boolean | string> {
    const element = this.page.locator(`#${settingId}`);
    const type = await element.getAttribute('type');
    
    if (type === 'checkbox') {
      return await element.isChecked();
    } else if (type === 'text' || type === 'url' || type === 'email') {
      return await element.inputValue();
    } else {
      return await element.inputValue();
    }
  }

  /**
   * Reset settings to default
   */
  async resetToDefault() {
    await this.clickSettingsButton('reset-to-default');
    await this.waitForPageLoad();
  }

  /**
   * Verify setting is enabled/disabled
   */
  async verifySettingEnabled(settingId: string, expected: boolean) {
    const element = this.page.locator(`#${settingId}`);
    await expect(element).toBeEnabled({ enabled: expected });
  }

  /**
   * Search in settings
   */
  async searchSettings(query: string) {
    const searchBox = this.page.locator('#search-input');
    await searchBox.fill(query);
    await this.page.keyboard.press('Enter');
    await this.waitForPageLoad();
  }

  /**
   * Get search results count
   */
  async getSearchResultsCount(): Promise<number> {
    const results = this.page.locator('.search-result');
    return await results.count();
  }

  /**
   * Navigate back from settings
   */
  async navigateBack() {
    await this.page.goBack();
    await this.waitForPageLoad();
  }
}