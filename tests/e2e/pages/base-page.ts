import { Page } from '@playwright/test';
import { TestHelpers } from '../utils/test-helpers';

export class BasePage {
  protected page: Page;
  protected helpers: TestHelpers;

  constructor(page: Page) {
    this.page = page;
    this.helpers = new TestHelpers(page);
  }

  async navigate(url: string): Promise<void> {
    await this.helpers.navigateTo(url);
  }

  async reload(): Promise<void> {
    await this.helpers.reload();
  }

  async goBack(): Promise<void> {
    await this.helpers.goBack();
  }

  async goForward(): Promise<void> {
    await this.helpers.goForward();
  }

  async getCurrentUrl(): Promise<string> {
    return this.helpers.getCurrentUrl();
  }

  async getTitle(): Promise<string> {
    return this.helpers.getTitle();
  }

  async isVisible(selector: string): Promise<boolean> {
    return this.helpers.isVisible(selector);
  }

  async takeScreenshot(name: string): Promise<void> {
    await this.helpers.takeScreenshot(name);
  }

  async waitForPageLoad(): Promise<void> {
    await this.page.waitForLoadState('networkidle');
  }

  protected async clickElement(selector: string): Promise<void> {
    await this.helpers.clickElement(selector);
  }

  protected async fillInput(selector: string, value: string): Promise<void> {
    await this.helpers.fillInput(selector, value);
  }

  protected async getText(selector: string): Promise<string> {
    return this.helpers.getText(selector);
  }

  protected async expectVisible(selector: string): Promise<void> {
    await this.helpers.expectElementToBeVisible(selector);
  }

  protected async expectHidden(selector: string): Promise<void> {
    await this.helpers.expectElementToBeHidden(selector);
  }

  protected async expectTextContains(selector: string, text: string): Promise<void> {
    await this.helpers.expectTextToContain(selector, text);
  }
}