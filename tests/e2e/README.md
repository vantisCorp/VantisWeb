# E2E Testing Framework

This directory contains the End-to-End (E2E) testing framework for VantisWeb browser, implemented using Playwright and following best practices from Mozilla Firefox's testing approach.

## Overview

The E2E testing framework validates real-world user scenarios and ensures the browser functions correctly across different browsers and platforms. This complements the existing unit and integration tests by testing complete user workflows from start to finish.

## Technology Stack

- **Playwright**: Modern E2E testing framework
- **TypeScript**: Type-safe test development
- **Jest**: Test runner and assertion library
- **GitHub Actions**: CI/CD integration

## Project Structure

```
tests/e2e/
├── fixtures/          # Test data and mock data
├── pages/             # Page Object Models
│   ├── base-page.ts   # Base page class
│   └── browser-page.ts # Browser page object
├── tests/             # E2E test files
│   ├── browser/       # Browser functionality tests
│   ├── extensions/    # Extension system tests
│   ├── navigation/    # Navigation tests
│   └── ui/           # UI component tests
├── utils/             # Test utilities and helpers
│   └── test-helpers.ts # Helper functions
└── config/            # Test configuration files
```

## Installation

### Prerequisites

- Node.js 18 or higher
- Rust toolchain (for building VantisWeb)
- Playwright browsers

### Setup

1. Install dependencies:
```bash
npm install
```

2. Install Playwright browsers:
```bash
npm run install:browsers
```

3. Build VantisWeb:
```bash
cargo build --release
```

## Running Tests

### Run all E2E tests:
```bash
npm run test:e2e
```

### Run tests in UI mode (interactive):
```bash
npm run test:e2e:ui
```

### Run tests in debug mode:
```bash
npm run test:e2e:debug
```

### Run tests for specific browser:
```bash
npm run test:e2e:chromium    # Chrome/Edge
npm run test:e2e:firefox     # Firefox
npm run test:e2e:webkit      # Safari
```

### Run tests in headed mode (visible browser):
```bash
npm run test:e2e:headed
```

### Generate test report:
```bash
npm run test:e2e:report
```

## Writing Tests

### Basic Test Structure

```typescript
import { test, expect } from '@playwright/test';
import { BrowserPage } from '../../pages/browser-page';

test.describe('My Test Suite', () => {
  let browserPage: BrowserPage;

  test.beforeEach(async ({ page }) => {
    browserPage = new BrowserPage(page);
    await browserPage.navigateToUrl('https://example.com');
  });

  test('should do something', async ({ page }) => {
    // Arrange
    const expectedText = 'Hello World';

    // Act
    await browserPage.clickElement('#my-button');
    
    // Assert
    await expect(page.locator('#result')).toHaveText(expectedText);
  });
});
```

### Using Page Objects

Page Object Models (POM) help organize tests and make them more maintainable:

```typescript
import { BrowserPage } from '../../pages/browser-page';

test('should navigate to a URL', async ({ page }) => {
  const browserPage = new BrowserPage(page);
  await browserPage.navigateToUrl('https://example.com');
  expect(await browserPage.getCurrentUrl()).toContain('example.com');
});
```

### Test Helpers

Use the built-in test helpers for common operations:

```typescript
import { TestHelpers } from '../../utils/test-helpers';

const helpers = new TestHelpers(page);

// Wait for element
await helpers.waitForElement('#my-element');

// Click element
await helpers.clickElement('#my-button');

// Fill input
await helpers.fillInput('#my-input', 'test value');

// Get text
const text = await helpers.getText('#my-element');
```

## Test Categories

### 1. Browser Tests (`tests/browser/`)
- Basic browser functionality
- Tab management
- Window management
- Settings and preferences

### 2. Extension Tests (`tests/extensions/`)
- Extension installation
- Extension activation
- Extension permissions
- Extension API interaction

### 3. Navigation Tests (`tests/navigation/`)
- URL navigation
- Back/forward navigation
- Bookmark navigation
- History navigation

### 4. UI Tests (`tests/ui/`)
- User interface components
- Settings pages
- Developer tools
- Responsive design

## Configuration

### Playwright Configuration (`playwright.config.ts`)

Key configuration options:
- `testDir`: Directory containing test files
- `timeout`: Default test timeout (30 seconds)
- `retries`: Number of retries on CI (2), 0 locally
- `workers`: Parallel test execution
- `use`: Default browser context settings

### Browser Projects

Tests run on three browsers:
- **Chromium**: Chrome, Edge, and other Chromium-based browsers
- **Firefox**: Mozilla Firefox
- **WebKit**: Safari and other WebKit-based browsers

## CI/CD Integration

The E2E tests run automatically on:
- Push to main/develop branches
- Pull requests
- Daily schedule (midnight UTC)

### GitHub Actions Workflow

The workflow (`.github/workflows/e2e-tests.yml`):
1. Sets up Rust and Node.js environments
2. Installs dependencies and Playwright browsers
3. Builds VantisWeb
4. Runs E2E tests on all browsers
5. Uploads test results, screenshots, and videos
6. Generates summary report

### Test Artifacts

After each test run, the following artifacts are generated:
- **Test Results**: JUnit XML and HTML reports
- **Screenshots**: Captured on test failures
- **Videos**: Recorded test execution (on failure)
- **Traces**: Detailed execution traces (on retry)

## Best Practices

### 1. Test Organization
- Group related tests using `test.describe()`
- Use descriptive test names
- Keep tests focused and independent
- Use `beforeEach`/`afterEach` for setup/teardown

### 2. Page Object Models
- Create reusable page objects
- Encapsulate page-specific logic
- Keep selectors in one place
- Make methods descriptive

### 3. Wait Strategies
- Use `waitForSelector` for element visibility
- Use `waitForLoadState` for page navigation
- Avoid fixed `setTimeout` when possible
- Use Playwright's auto-waiting features

### 4. Assertions
- Use specific assertions (`toHaveText`, `toBeVisible`)
- Assert multiple conditions when needed
- Use custom matchers for complex scenarios
- Provide clear assertion messages

### 5. Test Data
- Use fixtures for test data
- Keep test data minimal and focused
- Avoid hardcoded values when possible
- Use environment variables for configuration

## Troubleshooting

### Common Issues

1. **Tests timeout**
   - Increase timeout in `playwright.config.ts`
   - Check if VantisWeb is running
   - Verify network connectivity

2. **Element not found**
   - Use Playwright Inspector to find selectors
   - Wait for element to be visible
   - Check if element is in iframe

3. **Flaky tests**
   - Add proper wait strategies
   - Use retry mechanisms
   - Check for race conditions
   - Isolate and debug specific tests

### Debug Mode

Run tests in debug mode to step through execution:
```bash
npm run test:e2e:debug
```

### Playwright Inspector

Use Playwright Inspector to inspect and generate selectors:
```bash
npx playwright codegen https://example.com
```

## Contributing

When adding new E2E tests:

1. Follow the existing test structure
2. Use page objects for page interactions
3. Keep tests focused and independent
4. Add descriptive comments
5. Update this README if adding new test categories

## Future Enhancements

- [ ] Visual regression testing
- [ ] Performance testing integration
- [ ] Accessibility testing
- [ ] Mobile browser testing
- [ ] Advanced test reporting dashboards
- [ ] Test result trends and analytics

## References

- [Playwright Documentation](https://playwright.dev/)
- [Playwright Best Practices](https://playwright.dev/docs/best-practices)
- [Mozilla Firefox Testing](https://firefox-source-docs.mozilla.org/testing/)
- [Page Object Model Pattern](https://playwright.dev/docs/pom)

## Support

For issues or questions about E2E testing:
1. Check this README and Playwright documentation
2. Review existing tests for examples
3. Check CI/CD logs for test failures
4. Contact the development team