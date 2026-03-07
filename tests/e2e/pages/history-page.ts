import { Page, expect } from '@playwright/test';
import { BasePage } from './base-page';

/**
 * HistoryPage - Page object for browsing history management
 * Provides methods for interacting with history functionality
 */
export class HistoryPage extends BasePage {
  constructor(page: Page) {
    super(page);
  }

  /**
   * Navigate to history page
   */
  async navigateToHistory() {
    await this.page.goto('about:history');
    await this.waitForPageLoad();
  }

  /**
   * Get history items count
   */
  async getHistoryCount(): Promise<number> {
    const historyItems = this.page.locator('.history-item');
    return await historyItems.count();
  }

  /**
   * Get history by URL
   */
  async getHistoryByUrl(url: string) {
    return this.page.locator(`.history-item:has-text("${url}")`);
  }

  /**
   * Get history by title
   */
  async getHistoryByTitle(title: string) {
    return this.page.locator(`.history-item:has-text("${title}")`);
  }

  /**
   * Clear all history
   */
  async clearAllHistory() {
    await this.page.click('#clear-history-button');
    await this.page.waitForSelector('#confirm-clear-dialog');
    await this.page.click('#confirm-clear-button');
    await this.waitForPageLoad();
  }

  /**
   * Clear history for specific time range
   */
  async clearHistoryForRange(range: 'hour' | 'day' | 'week' | 'month' | 'all') {
    await this.page.click('#clear-history-button');
    await this.page.waitForSelector('#confirm-clear-dialog');
    
    const rangeMap = {
      hour: '#clear-last-hour',
      day: '#clear-last-day',
      week: '#clear-last-week',
      month: '#clear-last-month',
      all: '#clear-all-time'
    };
    
    await this.page.click(rangeMap[range]);
    await this.page.click('#confirm-clear-button');
    await this.waitForPageLoad();
  }

  /**
   * Delete specific history item
   */
  async deleteHistoryItem(url: string) {
    const historyItem = await this.getHistoryByUrl(url);
    await historyItem.locator('.delete-button').click();
    await this.waitForPageLoad();
  }

  /**
   * Search history
   */
  async searchHistory(query: string) {
    const searchBox = this.page.locator('#history-search');
    await searchBox.fill(query);
    await this.page.keyboard.press('Enter');
    await this.waitForPageLoad();
  }

  /**
   * Get search results count
   */
  async getSearchResultsCount(): Promise<number> {
    const results = this.page.locator('.history-item:visible');
    return await results.count();
  }

  /**
   * Click history item
   */
  async clickHistoryItem(url: string) {
    const historyItem = await this.getHistoryByUrl(url);
    await historyItem.locator('.history-link').click();
    await this.waitForPageLoad();
  }

  /**
   * Open history item in new tab
   */
  async openInNewTab(url: string) {
    const historyItem = await this.getHistoryByUrl(url);
    await historyItem.locator('.open-in-new-tab-button').click();
    await this.waitForPageLoad();
  }

  /**
   * Verify history item exists
   */
  async verifyHistoryExists(url: string): Promise<boolean> {
    const count = await this.getHistoryByUrl(url).count();
    return count > 0;
  }

  /**
   * Verify history cleared
   */
  async verifyHistoryCleared(): Promise<boolean> {
    const count = await this.getHistoryCount();
    return count === 0;
  }

  /**
   * Get history item timestamp
   */
  async getHistoryItemTimestamp(url: string): Promise<string> {
    const historyItem = await this.getHistoryByUrl(url);
    const timestamp = await historyItem.locator('.history-timestamp').textContent();
    return timestamp?.trim() || '';
  }

  /**
   * Get history item title
   */
  async getHistoryItemTitle(url: string): Promise<string> {
    const historyItem = await this.getHistoryByUrl(url);
    const title = await historyItem.locator('.history-title').textContent();
    return title?.trim() || '';
  }

  /**
   * Get history item URL
   */
  async getHistoryItemUrl(title: string): Promise<string> {
    const historyItem = await this.getHistoryByTitle(title);
    const url = await historyItem.locator('.history-url').textContent();
    return url?.trim() || '';
  }

  /**
   * Filter history by time range
   */
  async filterByTimeRange(range: 'today' | 'yesterday' | 'week' | 'month' | 'older') {
    const filterMap = {
      today: '#filter-today',
      yesterday: '#filter-yesterday',
      week: '#filter-week',
      month: '#filter-month',
      older: '#filter-older'
    };
    
    await this.page.click(filterMap[range]);
    await this.waitForPageLoad();
  }

  /**
   * Get history count for time range
   */
  async getHistoryCountForRange(range: 'today' | 'yesterday' | 'week' | 'month' | 'older'): Promise<number> {
    await this.filterByTimeRange(range);
    const count = await this.getHistoryCount();
    return count;
  }

  /**
   * Verify no history in private browsing
   */
  async verifyNoHistoryInPrivateBrowsing(): Promise<boolean> {
    await this.navigateToHistory();
    const count = await this.getHistoryCount();
    return count === 0;
  }

  /**
   * Copy history URL
   */
  async copyHistoryUrl(url: string) {
    const historyItem = await this.getHistoryByUrl(url);
    await historyItem.locator('.copy-url-button').click();
  }

  /**
   * Open history item in incognito
   */
  async openInIncognito(url: string) {
    const historyItem = await this.getHistoryByUrl(url);
    await historyItem.locator('.open-incognito-button').click();
    await this.waitForPageLoad();
  }

  /**
   * Get history groupings count
   */
  async getHistoryGroupingsCount(): Promise<number> {
    const groups = this.page.locator('.history-group');
    return await groups.count();
  }

  /**
   * Expand/collapse history group
   */
  async toggleHistoryGroup(groupName: string) {
    const group = this.page.locator(`.history-group:has-text("${groupName}")`);
    await group.locator('.group-toggle').click();
    await this.waitForPageLoad();
  }
}