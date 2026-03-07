import { Page, expect } from '@playwright/test';
import { BasePage } from './base-page';

/**
 * DownloadsPage - Page object for downloads management
 * Provides methods for interacting with download functionality
 */
export class DownloadsPage extends BasePage {
  constructor(page: Page) {
    super(page);
  }

  /**
   * Navigate to downloads page
   */
  async navigateToDownloads() {
    await this.page.goto('about:downloads');
    await this.waitForPageLoad();
  }

  /**
   * Get download items count
   */
  async getDownloadsCount(): Promise<number> {
    const downloads = this.page.locator('.download-item');
    return await downloads.count();
  }

  /**
   * Get download by filename
   */
  async getDownloadByFilename(filename: string) {
    return this.page.locator(`.download-item:has-text("${filename}")`);
  }

  /**
   * Get download status
   */
  async getDownloadStatus(filename: string): Promise<string> {
    const download = await this.getDownloadByFilename(filename);
    const status = await download.locator('.download-status').textContent();
    return status?.trim() || '';
  }

  /**
   * Pause download
   */
  async pauseDownload(filename: string) {
    const download = await this.getDownloadByFilename(filename);
    await download.locator('.pause-button').click();
  }

  /**
   * Resume download
   */
  async resumeDownload(filename: string) {
    const download = await this.getDownloadByFilename(filename);
    await download.locator('.resume-button').click();
  }

  /**
   * Cancel download
   */
  async cancelDownload(filename: string) {
    const download = await this.getDownloadByFilename(filename);
    await download.locator('.cancel-button').click();
  }

  /**
   * Retry failed download
   */
  async retryDownload(filename: string) {
    const download = await this.getDownloadByFilename(filename);
    await download.locator('.retry-button').click();
  }

  /**
   * Open download
   */
  async openDownload(filename: string) {
    const download = await this.getDownloadByFilename(filename);
    await download.locator('.open-button').click();
  }

  /**
   * Show download in folder
   */
  async showInFolder(filename: string) {
    const download = await this.getDownloadByFilename(filename);
    await download.locator('.show-in-folder-button').click();
  }

  /**
   * Remove download from list
   */
  async removeDownload(filename: string) {
    const download = await this.getDownloadByFilename(filename);
    await download.locator('.remove-button').click();
  }

  /**
   * Clear all downloads
   */
  async clearAllDownloads() {
    await this.page.click('#clear-downloads-button');
    await this.waitForPageLoad();
  }

  /**
   * Verify download exists
   */
  async verifyDownloadExists(filename: string): Promise<boolean> {
    const count = await this.getDownloadByFilename(filename).count();
    return count > 0;
  }

  /**
   * Verify download completed successfully
   */
  async verifyDownloadComplete(filename: string): Promise<boolean> {
    const status = await this.getDownloadStatus(filename);
    return status === 'Completed' || status === 'Done';
  }

  /**
   * Verify download failed
   */
  async verifyDownloadFailed(filename: string): Promise<boolean> {
    const status = await this.getDownloadStatus(filename);
    return status === 'Failed' || status === 'Error';
  }

  /**
   * Verify download in progress
   */
  async verifyDownloadInProgress(filename: string): Promise<boolean> {
    const status = await this.getDownloadStatus(filename);
    return status === 'Downloading' || status === 'In progress';
  }

  /**
   * Get download file size
   */
  async getDownloadFileSize(filename: string): Promise<string> {
    const download = await this.getDownloadByFilename(filename);
    const size = await download.locator('.file-size').textContent();
    return size?.trim() || '';
  }

  /**
   * Get download source URL
   */
  async getDownloadSource(filename: string): Promise<string> {
    const download = await this.getDownloadByFilename(filename);
    const source = await download.locator('.download-source').textContent();
    return source?.trim() || '';
  }

  /**
   * Get download time
   */
  async getDownloadTime(filename: string): Promise<string> {
    const download = await this.getDownloadByFilename(filename);
    const time = await download.locator('.download-time').textContent();
    return time?.trim() || '';
  }

  /**
   * Wait for download to complete
   */
  async waitForDownloadComplete(filename: string, timeout: number = 60000) {
    await this.page.waitForFunction(
      (filename: string) => {
        const status = document.querySelector(`.download-item:has-text("${filename}") .download-status`)?.textContent;
        return status === 'Completed' || status === 'Done';
      },
      filename,
      { timeout }
    );
  }

  /**
   * Verify downloads cleared
   */
  async verifyDownloadsCleared(): Promise<boolean> {
    const count = await this.getDownloadsCount();
    return count === 0;
  }
}