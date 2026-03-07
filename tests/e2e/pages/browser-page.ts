import { Page } from '@playwright/test';
import { BasePage } from './base-page';

export class BrowserPage extends BasePage {
  // Selectors
  private readonly addressBar = '#address-bar';
  private readonly navigationButtons = '.navigation-buttons';
  private readonly backButton = '#back-button';
  private readonly forwardButton = '#forward-button';
  private readonly refreshButton = '#refresh-button';
  private readonly bookmarksBar = '.bookmarks-bar';
  private readonly tabsContainer = '.tabs-container';
  private readonly newTabButton = '#new-tab-button';

  constructor(page: Page) {
    super(page);
  }

  async navigateToUrl(url: string): Promise<void> {
    await this.navigate(url);
  }

  async getAddressBarText(): Promise<string> {
    await this.expectVisible(this.addressBar);
    return await this.getText(this.addressBar);
  }

  async setAddressBarText(text: string): Promise<void> {
    await this.fillInput(this.addressBar, text);
  }

  async goBack(): Promise<void> {
    await this.clickElement(this.backButton);
    await this.waitForPageLoad();
  }

  async goForward(): Promise<void> {
    await this.clickElement(this.forwardButton);
    await this.waitForPageLoad();
  }

  async refresh(): Promise<void> {
    await this.clickElement(this.refreshButton);
    await this.waitForPageLoad();
  }

  async openNewTab(): Promise<void> {
    await this.clickElement(this.newTabButton);
  }

  async getTabCount(): Promise<number> {
    const tabs = await this.page.$$(this.tabsContainer + ' .tab');
    return tabs.length;
  }

  async switchToTab(index: number): Promise<void> {
    const tabs = await this.page.$$(this.tabsContainer + ' .tab');
    if (index >= 0 && index < tabs.length) {
      await tabs[index].click();
    }
  }

  async isBookmarkPresent(title: string): Promise<boolean> {
    const bookmarks = await this.page.$$(this.bookmarksBar + ' .bookmark');
    for (const bookmark of bookmarks) {
      const text = await bookmark.innerText();
      if (text.includes(title)) {
        return true;
      }
    }
    return false;
  }

  async clickBookmark(title: string): Promise<void> {
    const bookmarks = await this.page.$$(this.bookmarksBar + ' .bookmark');
    for (const bookmark of bookmarks) {
      const text = await bookmark.innerText();
      if (text.includes(title)) {
        await bookmark.click();
        await this.waitForPageLoad();
        break;
      }
    }
  }

  async isNavigationButtonsVisible(): Promise<boolean> {
    return await this.isVisible(this.navigationButtons);
  }

  async isBackButtonEnabled(): Promise<boolean> {
    const button = await this.page.$(this.backButton);
    if (!button) return false;
    return !(await button.isDisabled());
  }

  async isForwardButtonEnabled(): Promise<boolean> {
    const button = await this.page.$(this.forwardButton);
    if (!button) return false;
    return !(await button.isDisabled());
  }
}