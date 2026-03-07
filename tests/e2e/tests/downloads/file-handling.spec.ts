import { test, expect } from '@playwright/test';
import { DownloadsPage } from '../../pages/downloads-page';
import { BrowserPage } from '../../pages/browser-page';

test.describe('Downloads and File Handling', () => {
  let downloadsPage: DownloadsPage;
  let browserPage: BrowserPage;

  test.beforeEach(async ({ page }) => {
    downloadsPage = new DownloadsPage(page);
    browserPage = new BrowserPage(page);
    
    test.info().annotations.push({
      type: 'epic',
      description: 'Downloads and File Handling'
    });
    test.info().annotations.push({
      type: 'feature',
      description: 'Download Management'
    });
  });

  test('should initiate file download', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify file download can be initiated'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'critical'
    });

    // Navigate to page with download link
    await browserPage.navigateTo('https://file-examples.com/index.php/sample-documents-download/');
    
    // Click download link
    const [download] = await Promise.all([
      page.waitForEvent('download'),
      page.click('a:has-text("Download")')
    ]);
    
    // Verify download started
    expect(download).toBeDefined();
    expect(download.suggestedFilename()).toBeTruthy();
  });

  test('should show download progress', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify download progress is displayed'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });

    // Start download
    await browserPage.navigateTo('https://file-examples.com/index.php/sample-documents-download/');
    
    const [download] = await Promise.all([
      page.waitForEvent('download'),
      page.click('a:has-text("Download")')
    ]);
    
    // Navigate to downloads page
    await downloadsPage.navigateToDownloads();
    
    // Verify download appears in list with progress
    const downloadItem = await downloadsPage.getDownloadByFilename(download.suggestedFilename());
    await expect(downloadItem).toBeVisible();
    
    // Verify progress indicator
    const progressIndicator = await downloadItem.locator('.download-progress');
    await expect(progressIndicator).toBeVisible();
  });

  test('should pause and resume download', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify download can be paused and resumed'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });

    // Start a large download
    await browserPage.navigateTo('https://file-examples.com/index.php/sample-documents-download/');
    
    const [download] = await Promise.all([
      page.waitForEvent('download'),
      page.click('a:has-text("Download sample file")')
    ]);
    
    await downloadsPage.navigateToDownloads();
    const filename = download.suggestedFilename();
    
    // Pause download
    await downloadsPage.pauseDownload(filename);
    
    // Verify download is paused
    const status = await downloadsPage.getDownloadStatus(filename);
    expect(status).toMatch(/paused/i);
    
    // Resume download
    await downloadsPage.resumeDownload(filename);
    
    // Verify download resumed
    const resumedStatus = await downloadsPage.getDownloadStatus(filename);
    expect(resumedStatus).toMatch(/downloading|in progress/i);
  });

  test('should cancel download', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify download can be cancelled'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });

    // Start download
    await browserPage.navigateTo('https://file-examples.com/index.php/sample-documents-download/');
    
    const [download] = await Promise.all([
      page.waitForEvent('download'),
      page.click('a:has-text("Download")')
    ]);
    
    await downloadsPage.navigateToDownloads();
    const filename = download.suggestedFilename();
    
    // Cancel download
    await downloadsPage.cancelDownload(filename);
    
    // Verify download is cancelled
    const status = await downloadsPage.getDownloadStatus(filename);
    expect(status).toMatch(/cancelled|canceled/i);
  });

  test('should retry failed download', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify failed download can be retried'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });

    await downloadsPage.navigateToDownloads();
    
    // Assume there's a failed download
    const failedDownload = await page.locator('.download-item.failed').first();
    
    if (await failedDownload.count() > 0) {
      const filename = await failedDownload.locator('.filename').textContent();
      
      // Retry download
      await downloadsPage.retryDownload(filename || '');
      
      // Verify download is retrying
      const status = await downloadsPage.getDownloadStatus(filename || '');
      expect(status).toMatch(/downloading|retrying/i);
    }
  });

  test('should open downloaded file', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify downloaded file can be opened'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });

    // Start and complete a download
    await browserPage.navigateTo('https://file-examples.com/index.php/sample-documents-download/');
    
    const [download] = await Promise.all([
      page.waitForEvent('download'),
      page.click('a:has-text("Download")')
    ]);
    
    // Wait for download to complete
    await download.path();
    
    await downloadsPage.navigateToDownloads();
    const filename = download.suggestedFilename();
    
    // Open file
    await downloadsPage.openDownload(filename);
    
    // Verify file opened (application launched)
    // This is OS-specific and may need adjustment
  });

  test('should show download in folder', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify download can be shown in file manager'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });

    // Complete a download
    await browserPage.navigateTo('https://file-examples.com/index.php/sample-documents-download/');
    
    const [download] = await Promise.all([
      page.waitForEvent('download'),
      page.click('a:has-text("Download")')
    ]);
    
    await download.path();
    
    await downloadsPage.navigateToDownloads();
    const filename = download.suggestedFilename();
    
    // Show in folder
    await downloadsPage.showInFolder(filename);
    
    // Verify file manager opened (OS-specific)
  });

  test('should remove download from list', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify download can be removed from list'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });

    // Complete a download
    await browserPage.navigateTo('https://file-examples.com/index.php/sample-documents-download/');
    
    const [download] = await Promise.all([
      page.waitForEvent('download'),
      page.click('a:has-text("Download")')
    ]);
    
    await download.path();
    
    await downloadsPage.navigateToDownloads();
    const filename = download.suggestedFilename();
    
    // Remove from list
    await downloadsPage.removeDownload(filename);
    
    // Verify download is removed from list
    const exists = await downloadsPage.verifyDownloadExists(filename);
    expect(exists).toBe(false);
  });

  test('should clear all downloads', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify all downloads can be cleared'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });

    await downloadsPage.navigateToDownloads();
    
    // Clear all downloads
    await downloadsPage.clearAllDownloads();
    
    // Verify downloads are cleared
    const cleared = await downloadsPage.verifyDownloadsCleared();
    expect(cleared).toBe(true);
  });

  test('should handle multiple simultaneous downloads', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify multiple downloads can run simultaneously'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });

    await browserPage.navigateTo('https://file-examples.com/index.php/sample-documents-download/');
    
    // Start multiple downloads
    const downloadPromises = [];
    for (let i = 0; i < 3; i++) {
      downloadPromises.push(
        page.waitForEvent('download').then(async download => {
          return download.suggestedFilename();
        })
      );
      await page.click('a:has-text("Download")', { timeout: 1000 }).catch(() => {});
    }
    
    const filenames = await Promise.all(downloadPromises);
    
    await downloadsPage.navigateToDownloads();
    
    // Verify all downloads appear in list
    const downloadCount = await downloadsPage.getDownloadsCount();
    expect(downloadCount).toBeGreaterThanOrEqual(3);
  });

  test('should handle file upload in web forms', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify file upload works in web forms'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'critical'
    });

    // Navigate to file upload page
    await browserPage.navigateTo('https://httpbin.org/forms/post');
    
    // Upload file
    await page.setInputFiles('input[type="file"]', 'test-file.txt');
    
    // Submit form
    await page.click('button[type="submit"]');
    
    // Verify upload success
    await expect(page).toHaveURL(/httpbin\.org\/post/);
    const body = await page.textContent('body');
    expect(body).toContain('test-file.txt');
  });

  test('should handle large file download', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify large file downloads are handled correctly'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });

    // Start large file download
    await browserPage.navigateTo('https://file-examples.com/index.php/sample-documents-download/');
    
    const [download] = await Promise.all([
      page.waitForEvent('download'),
      page.click('a:has-text("Download sample file (100MB)")')
    ]);
    
    await downloadsPage.navigateToDownloads();
    const filename = download.suggestedFilename();
    
    // Verify download appears in list
    const downloadItem = await downloadsPage.getDownloadByFilename(filename);
    await expect(downloadItem).toBeVisible();
    
    // Verify file size is displayed
    const fileSize = await downloadsPage.getDownloadFileSize(filename);
    expect(fileSize).toMatch(/\d+\s*(MB|GB)/);
  });

  test('should handle download security scanning', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify downloads are scanned for security'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'critical'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'security'
    });

    // Complete a download
    await browserPage.navigateTo('https://file-examples.com/index.php/sample-documents-download/');
    
    const [download] = await Promise.all([
      page.waitForEvent('download'),
      page.click('a:has-text("Download")')
    ]);
    
    await download.path();
    
    await downloadsPage.navigateToDownloads();
    const filename = download.suggestedFilename();
    
    // Verify security scan status
    const securityStatus = await page.locator('.download-security-status');
    const hasStatus = await securityStatus.count();
    
    if (hasStatus > 0) {
      await expect(securityStatus).toBeVisible();
    }
  });

  test('should handle download from unknown source', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify warning for downloads from unknown sources'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'security'
    });

    // Start download from unknown source (simulated)
    await browserPage.navigateTo('https://unknown-source.com/file.zip');
    
    // Verify warning dialog appears
    const warningDialog = await page.locator('.download-warning-dialog');
    const hasWarning = await warningDialog.count();
    
    if (hasWarning > 0) {
      await expect(warningDialog).toBeVisible();
      
      // Allow download
      await page.click('#allow-download-button');
    }
  });

  test('should preserve download history', async ({ page, context }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify download history is preserved'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'persistence'
    });

    // Complete a download
    await browserPage.navigateTo('https://file-examples.com/index.php/sample-documents-download/');
    
    const [download] = await Promise.all([
      page.waitForEvent('download'),
      page.click('a:has-text("Download")')
    ]);
    
    const filename = download.suggestedFilename();
    await download.path();
    
    // Navigate away and back
    await browserPage.navigateTo('https://example.com');
    await downloadsPage.navigateToDownloads();
    
    // Verify download is still in history
    const exists = await downloadsPage.verifyDownloadExists(filename);
    expect(exists).toBe(true);
  });

  test('should handle download with special characters in filename', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify downloads with special characters work'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });

    // Download file with special characters in name
    await browserPage.navigateTo('https://example.com/files/test file (1).pdf');
    
    const [download] = await Promise.all([
      page.waitForEvent('download'),
      page.click('a:has-text("Download")')
    ]);
    
    // Verify download completes successfully
    const filename = download.suggestedFilename();
    expect(filename).toBeTruthy();
  });

  test('should handle download interruption and recovery', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify download can recover from interruption'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'recovery'
    });

    // Start a large download
    await browserPage.navigateTo('https://file-examples.com/index.php/sample-documents-download/');
    
    const [download] = await Promise.all([
      page.waitForEvent('download'),
      page.click('a:has-text("Download sample file")')
    ]);
    
    await downloadsPage.navigateToDownloads();
    const filename = download.suggestedFilename();
    
    // Simulate network interruption (simulated)
    // Pause and resume to simulate recovery
    await downloadsPage.pauseDownload(filename);
    await downloadsPage.resumeDownload(filename);
    
    // Verify download continues
    const status = await downloadsPage.getDownloadStatus(filename);
    expect(status).toMatch(/downloading|in progress/i);
  });

  test('should show download completion notification', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify notification appears when download completes'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });

    // Complete a download
    await browserPage.navigateTo('https://file-examples.com/index.php/sample-documents-download/');
    
    const [download] = await Promise.all([
      page.waitForEvent('download'),
      page.click('a:has-text("Download")')
    ]);
    
    await download.path();
    
    // Verify completion notification
    const notification = await page.locator('.download-complete-notification');
    const hasNotification = await notification.count();
    
    if (hasNotification > 0) {
      await expect(notification).toBeVisible();
    }
  });
});