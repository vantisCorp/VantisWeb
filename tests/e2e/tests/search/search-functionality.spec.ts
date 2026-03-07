import { test, expect } from '@playwright/test';
import { BrowserPage } from '../../pages/browser-page';

test.describe('Search Functionality', () => {
  let browserPage: BrowserPage;

  test.beforeEach(async ({ page }) => {
    browserPage = new BrowserPage(page);
    
    test.info().annotations.push({
      type: 'epic',
      description: 'Search Functionality'
    });
    test.info().annotations.push({
      type: 'feature',
      description: 'Address Bar Search'
    });
  });

  test('should perform search from address bar', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify search can be performed directly from address bar'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'critical'
    });

    // Perform search from address bar
    await page.click('#urlbar');
    await page.fill('#urlbar', 'test search query');
    await page.press('#urlbar', 'Enter');
    
    // Verify search results page is loaded
    await expect(page).toHaveURL(/search/);
    const body = await page.textContent('body');
    expect(body).toMatch(/test search query/i);
  });

  test('should show search suggestions', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify search suggestions appear when typing in address bar'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });

    // Start typing in address bar
    await page.click('#urlbar');
    await page.type('#urlbar', 'test search', { delay: 100 });
    
    // Wait for suggestions to appear
    await page.waitForTimeout(500);
    
    // Verify suggestions are shown
    const suggestions = await page.locator('.search-suggestion');
    const suggestionCount = await suggestions.count();
    expect(suggestionCount).toBeGreaterThan(0);
  });

  test('should select search suggestion', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify search suggestions can be selected and executed'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });

    // Type in address bar
    await page.click('#urlbar');
    await page.type('#urlbar', 'test search');
    
    // Wait for suggestions
    await page.waitForTimeout(500);
    
    // Select first suggestion
    await page.click('.search-suggestion:first-child');
    
    // Verify search is performed with suggestion
    await expect(page).toHaveURL(/search/);
  });

  test('should maintain search history', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify search history is maintained and accessible'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });

    // Perform searches
    await page.click('#urlbar');
    await page.fill('#urlbar', 'first search');
    await page.press('#urlbar', 'Enter');
    await page.waitForLoadState('networkidle');
    
    await page.click('#urlbar');
    await page.fill('#urlbar', 'second search');
    await page.press('#urlbar', 'Enter');
    await page.waitForLoadState('networkidle');
    
    // Click address bar again
    await page.click('#urlbar');
    
    // Verify search history appears
    const historyItems = await page.locator('.search-history-item');
    const historyCount = await historyItems.count();
    expect(historyCount).toBeGreaterThan(0);
  });

  test('should search in new tab', async ({ page, context }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify search can be performed in new tab'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });

    const initialPageCount = context.pages().length;
    
    // Open new tab and search
    await page.click('#new-tab-button');
    const newPage = context.pages()[context.pages().length - 1];
    await newPage.bringToFront();
    
    await newPage.click('#urlbar');
    await newPage.fill('#urlbar', 'test search');
    await newPage.press('#urlbar', 'Enter');
    
    // Verify new tab was created and search performed
    expect(context.pages().length).toBe(initialPageCount + 1);
    await expect(newPage).toHaveURL(/search/);
  });

  test('should perform search in incognito mode', async ({ page, context }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify search works in incognito/private mode'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'major'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'privacy'
    });

    // Open incognito window
    const incognitoContext = await browserPage.browser.newContext();
    const incognitoPage = await incognitoContext.newPage();
    
    // Perform search
    await incognitoPage.click('#urlbar');
    await incognitoPage.fill('#urlbar', 'incognito search');
    await incognitoPage.press('#urlbar', 'Enter');
    
    // Verify search works
    await expect(incognitoPage).toHaveURL(/search/);
    const body = await incognitoPage.textContent('body');
    expect(body).toMatch(/incognito search/i);
    
    await incognitoContext.close();
  });

  test('should switch between search engines', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify search engines can be switched'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });

    // Navigate to search settings
    await page.goto('about:preferences#search');
    
    // Select different search engine
    await page.click('#search-engine-dropdown');
    await page.click('option[value="bing"]');
    
    // Perform search
    await page.goto('about:home');
    await page.click('#urlbar');
    await page.fill('#urlbar', 'test search');
    await page.press('#urlbar', 'Enter');
    
    // Verify search uses selected engine (check URL)
    await expect(page).toHaveURL(/bing\.com/);
  });

  test('should add custom search engine', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify custom search engines can be added'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });

    // Navigate to search settings
    await page.goto('about:preferences#search');
    
    // Add custom search engine
    await page.click('#add-search-engine-button');
    await page.fill('#engine-name', 'Custom Engine');
    await page.fill('#engine-url', 'https://custom.com/search?q=%s');
    await page.click('#save-button');
    
    // Verify custom engine appears in list
    const customEngine = await page.locator('.search-engine:has-text("Custom Engine")');
    await expect(customEngine).toBeVisible();
  });

  test('should remove search engine', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify search engines can be removed'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });

    // Navigate to search settings
    await page.goto('about:preferences#search');
    
    // Get initial search engine count
    const initialCount = await page.locator('.search-engine').count();
    
    // Remove a search engine
    await page.locator('.search-engine:nth-child(2)').click('#remove-engine-button');
    
    // Verify engine was removed
    const finalCount = await page.locator('.search-engine').count();
    expect(finalCount).toBe(initialCount - 1);
  });

  test('should use keyboard shortcuts for search', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify keyboard shortcuts work for search'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });

    // Use Ctrl+K or Cmd+K to focus address bar
    if (process.platform === 'darwin') {
      await page.keyboard.press('Meta+K');
    } else {
      await page.keyboard.press('Control+K');
    }
    
    // Verify address bar is focused
    const urlbar = page.locator('#urlbar');
    await expect(urlbar).toBeFocused();
    
    // Type search query
    await page.type('#urlbar', 'keyboard search');
    await page.press('Enter');
    
    // Verify search performed
    await expect(page).toHaveURL(/search/);
  });

  test('should handle special characters in search', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify special characters are handled in search queries'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });

    const specialQuery = 'test + "quoted phrase" -exclude @user #tag';
    
    await page.click('#urlbar');
    await page.fill('#urlbar', specialQuery);
    await page.press('#urlbar', 'Enter');
    
    // Verify search is performed with special characters
    await expect(page).toHaveURL(/search/);
    const body = await page.textContent('body');
    expect(body).toMatch(/test/i);
  });

  test('should clear search history', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify search history can be cleared'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });
    test.info().annotations.push({
      type: 'tag',
      description: 'privacy'
    });

    // Perform searches to build history
    await page.click('#urlbar');
    await page.fill('#urlbar', 'history search 1');
    await page.press('#urlbar', 'Enter');
    await page.waitForLoadState('networkidle');
    
    await page.click('#urlbar');
    await page.fill('#urlbar', 'history search 2');
    await page.press('#urlbar', 'Enter');
    await page.waitForLoadState('networkidle');
    
    // Clear search history (simulated)
    await page.goto('about:preferences#privacy');
    await page.click('#clear-search-history-button');
    await page.click('#confirm-button');
    
    // Verify history is cleared
    await page.click('#urlbar');
    const historyItems = await page.locator('.search-history-item');
    const historyCount = await historyItems.count();
    expect(historyCount).toBe(0);
  });

  test('should navigate to search result', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify search results can be navigated to'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'critical'
    });

    // Perform search
    await page.click('#urlbar');
    await page.fill('#urlbar', 'example.com');
    await page.press('#urlbar', 'Enter');
    
    // Wait for search results
    await page.waitForLoadState('networkidle');
    
    // Click first result
    const firstResult = page.locator('.search-result-link').first();
    await firstResult.click();
    
    // Verify navigation to result
    await expect(page).toHaveURL(/example\.com/);
  });

  test('should open search result in new tab', async ({ page, context }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify search results can be opened in new tab'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });

    const initialPageCount = context.pages().length;
    
    // Perform search
    await page.click('#urlbar');
    await page.fill('#urlbar', 'example.com');
    await page.press('#urlbar', 'Enter');
    
    // Wait for results
    await page.waitForLoadState('networkidle');
    
    // Open first result in new tab (right-click -> open in new tab)
    const firstResult = page.locator('.search-result-link').first();
    await firstResult.click({ button: 'right' });
    await page.click('.context-menu-item:has-text("Open in new tab")');
    
    // Verify new tab was created
    expect(context.pages().length).toBe(initialPageCount + 1);
  });

  test('should handle search with no results', async ({ page }) => {
    test.info().annotations.push({
      type: 'description',
      description: 'Verify appropriate message shown when no search results found'
    });
    test.info().annotations.push({
      type: 'severity',
      description: 'minor'
    });

    // Perform unlikely search query
    await page.click('#urlbar');
    await page.fill('#urlbar', 'xyzabc123noresults');
    await page.press('#urlbar', 'Enter');
    
    // Wait for search results
    await page.waitForLoadState('networkidle');
    
    // Verify no results message
    const noResultsMessage = await page.locator('body').textContent();
    expect(noResultsMessage).toMatch(/no results|not found/i);
  });
});