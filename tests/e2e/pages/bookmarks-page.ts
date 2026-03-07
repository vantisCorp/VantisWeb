import { Page, expect } from '@playwright/test';
import { BasePage } from './base-page';

/**
 * BookmarksPage - Page object for bookmarks management
 * Provides methods for interacting with bookmark functionality
 */
export class BookmarksPage extends BasePage {
  constructor(page: Page) {
    super(page);
  }

  /**
   * Navigate to bookmarks page
   */
  async navigateToBookmarks() {
    await this.page.goto('about:bookmarks');
    await this.waitForPageLoad();
  }

  /**
   * Get bookmark items count
   */
  async getBookmarksCount(): Promise<number> {
    const bookmarks = this.page.locator('.bookmark-item');
    return await bookmarks.count();
  }

  /**
   * Get bookmark by name
   */
  async getBookmarkByName(name: string) {
    return this.page.locator(`.bookmark-item:has-text("${name}")`);
  }

  /**
   * Add bookmark for current page
   */
  async addBookmark(title: string, url?: string) {
    // Click bookmark star in URL bar
    await this.page.click('#star-button');
    
    // Wait for bookmark dialog
    await this.page.waitForSelector('#bookmark-dialog');
    
    // Set bookmark title
    await this.page.fill('#bookmark-title', title);
    
    // Set bookmark URL if provided
    if (url) {
      await this.page.fill('#bookmark-url', url);
    }
    
    // Save bookmark
    await this.page.click('#bookmark-save-button');
    await this.waitForPageLoad();
  }

  /**
   * Add bookmark to folder
   */
  async addBookmarkToFolder(title: string, folderName: string, url?: string) {
    await this.page.click('#star-button');
    await this.page.waitForSelector('#bookmark-dialog');
    await this.page.fill('#bookmark-title', title);
    
    if (url) {
      await this.page.fill('#bookmark-url', url);
    }
    
    // Select folder
    await this.page.click('#bookmark-folder-dropdown');
    await this.page.click(`option:has-text("${folderName}")`);
    
    await this.page.click('#bookmark-save-button');
    await this.waitForPageLoad();
  }

  /**
   * Edit bookmark
   */
  async editBookmark(name: string, newTitle?: string, newUrl?: string, newFolder?: string) {
    const bookmark = await this.getBookmarkByName(name);
    await bookmark.locator('.edit-button').click();
    await this.page.waitForSelector('#bookmark-dialog');
    
    if (newTitle) {
      await this.page.fill('#bookmark-title', newTitle);
    }
    
    if (newUrl) {
      await this.page.fill('#bookmark-url', newUrl);
    }
    
    if (newFolder) {
      await this.page.click('#bookmark-folder-dropdown');
      await this.page.click(`option:has-text("${newFolder}")`);
    }
    
    await this.page.click('#bookmark-save-button');
    await this.waitForPageLoad();
  }

  /**
   * Delete bookmark
   */
  async deleteBookmark(name: string) {
    const bookmark = await this.getBookmarkByName(name);
    await bookmark.locator('.delete-button').click();
    await this.waitForPageLoad();
  }

  /**
   * Create new folder
   */
  async createFolder(folderName: string) {
    await this.page.click('#create-folder-button');
    await this.page.waitForSelector('#folder-dialog');
    await this.page.fill('#folder-name', folderName);
    await this.page.click('#folder-save-button');
    await this.waitForPageLoad();
  }

  /**
   * Delete folder
   */
  async deleteFolder(folderName: string) {
    const folder = this.page.locator(`.folder-item:has-text("${folderName}")`);
    await folder.locator('.delete-button').click();
    await this.waitForPageLoad();
  }

  /**
   * Click bookmark
   */
  async clickBookmark(name: string) {
    const bookmark = await this.getBookmarkByName(name);
    await bookmark.locator('.bookmark-link').click();
    await this.waitForPageLoad();
  }

  /**
   * Verify bookmark exists
   */
  async verifyBookmarkExists(name: string): Promise<boolean> {
    const count = await this.getBookmarkByName(name).count();
    return count > 0;
  }

  /**
   * Verify bookmark URL
   */
  async verifyBookmarkUrl(name: string, expectedUrl: string): Promise<boolean> {
    const bookmark = await this.getBookmarkByName(name);
    const url = await bookmark.locator('.bookmark-url').textContent();
    return url?.trim() === expectedUrl;
  }

  /**
   * Verify folder exists
   */
  async verifyFolderExists(folderName: string): Promise<boolean> {
    const count = this.page.locator(`.folder-item:has-text("${folderName}")`).count();
    return (await count) > 0;
  }

  /**
   * Get bookmark count in folder
   */
  async getBookmarkCountInFolder(folderName: string): Promise<number> {
    const folder = this.page.locator(`.folder-item:has-text("${folderName}")`);
    const bookmarks = await folder.locator('.bookmark-item').count();
    return bookmarks;
  }

  /**
   * Search bookmarks
   */
  async searchBookmarks(query: string) {
    const searchBox = this.page.locator('#bookmark-search');
    await searchBox.fill(query);
    await this.page.keyboard.press('Enter');
    await this.waitForPageLoad();
  }

  /**
   * Get search results count
   */
  async getSearchResultsCount(): Promise<number> {
    const results = this.page.locator('.bookmark-item:visible');
    return await results.count();
  }

  /**
   * Import bookmarks
   */
  async importBookmarks(filePath: string) {
    await this.page.click('#import-bookmarks-button');
    await this.page.waitForSelector('#import-dialog');
    await this.page.fill('#import-file-path', filePath);
    await this.page.click('#import-button');
    await this.waitForPageLoad();
  }

  /**
   * Export bookmarks
   */
  async exportBookmarks(filePath: string) {
    await this.page.click('#export-bookmarks-button');
    await this.page.waitForSelector('#export-dialog');
    await this.page.fill('#export-file-path', filePath);
    await this.page.click('#export-button');
    await this.waitForPageLoad();
  }

  /**
   * Verify bookmark in folder
   */
  async verifyBookmarkInFolder(bookmarkName: string, folderName: string): Promise<boolean> {
    const folder = this.page.locator(`.folder-item:has-text("${folderName}")`);
    const bookmark = await folder.locator(`.bookmark-item:has-text("${bookmarkName}")`).count();
    return bookmark > 0;
  }

  /**
   * Get bookmark folders count
   */
  async getFoldersCount(): Promise<number> {
    const folders = this.page.locator('.folder-item');
    return await folders.count();
  }
}