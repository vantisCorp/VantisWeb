# Allure Test Reporting Guide

## Overview

Allure Framework is a flexible lightweight multi-language test report tool that provides comprehensive test analytics and visualization. This guide explains how to use Allure reporting with the VantisWeb E2E testing framework.

## Features

### 📊 Test Analytics
- **Test Execution Timeline**: Visual timeline of test execution
- **Pass/Fail Statistics**: Clear overview of test results
- **Performance Metrics**: Test execution time tracking
- **Coverage Analysis**: Integration with code coverage tools

### 🔍 Failure Analysis
- **Detailed Error Information**: Stack traces and error messages
- **Screenshot Attachments**: Automatic screenshots on failures
- **Video Recordings**: Test execution videos
- **Categorization**: Failure categorization for easier analysis

### 📈 Trend Analysis
- **Historical Data**: Track test results over time
- **Performance Trends**: Monitor test execution time trends
- **Flaky Test Detection**: Identify unstable tests
- **Regression Detection**: Spot performance and quality regressions

## Installation

The Allure dependencies are already included in `package.json`:

```json
{
  "devDependencies": {
    "allure-commandline": "^2.24.0",
    "allure-playwright": "^2.6.0"
  }
}
```

Install dependencies:
```bash
npm install
```

## Usage

### Running Tests with Allure

Run E2E tests normally - Allure will automatically collect results:
```bash
npm run test:e2e
```

### Generating Reports

Generate the Allure report:
```bash
npm run test:allure:generate
```

This creates an `allure-report` directory with the HTML report.

### Viewing Reports

#### Open Report Locally
```bash
npm run test:allure:open
```
This opens the report in your default browser.

#### Serve Report
```bash
npm run test:allure:serve
```
This starts a local web server for the report.

#### Generate with History
```bash
npm run test:allure:history
```
Generate report with historical trend data.

## Allure Annotations

### Test Descriptions
Add descriptions to tests using annotations:

```typescript
test('should navigate to URL @smoke @critical', async ({ page }) => {
  test.info().annotations.push({
    type: 'description',
    description: 'Verify basic URL navigation functionality'
  });
  
  // Test implementation
});
```

### Severity Levels
Categorize tests by severity:

```typescript
test.info().annotations.push({
  type: 'severity',
  description: 'critical'  // blocker, critical, normal, minor, trivial
});
```

### Test Organization
Organize tests with features and stories:

```typescript
test.info().annotations.push({
  type: 'feature',
  description: 'Browser Navigation'
});

test.info().annotations.push({
  type: 'story',
  description: 'URL Navigation'
});
```

### Custom Annotations
Add any custom annotations:

```typescript
test.info().annotations.push({
  type: 'performance',
  description: `Page load time: ${loadTime}ms`
});

test.info().annotations.push({
  type: 'testStep',
  description: 'Completed user authentication flow'
});
```

### Test Tags
Use tags for filtering and organization:

```typescript
test('should login @smoke @auth @critical', async ({ page }) => {
  // Test implementation
});

test('should navigate @regression @navigation', async ({ page }) => {
  // Test implementation
});
```

Common tags:
- `@smoke` - Critical functionality tests
- `@regression` - Bug prevention tests
- `@critical` - Critical path tests
- `@performance` - Performance tests
- `@security` - Security tests
- `@auth` - Authentication tests
- `@navigation` - Navigation tests

## Configuration

### Allure Configuration File
The `allure.config.js` file controls Allure behavior:

```javascript
module.exports = {
  name: 'VantisWeb E2E Tests',
  outputDir: 'allure-results',
  environmentInfo: {
    node: process.version,
    platform: process.platform,
    arch: process.arch,
    browsers: 'Chromium, Firefox, WebKit'
  },
  categories: [
    {
      name: 'Ignored tests',
      matchedStatuses: ['skipped']
    },
    {
      name: 'Infrastructure problems',
      matchedStatuses: ['broken', 'failed'],
      messageRegex: /.*Timeout.*/
    },
    {
      name: 'Product defects',
      matchedStatuses: ['failed']
    }
  ]
};
```

### Playwright Integration
Allure is integrated in `playwright.config.ts`:

```typescript
export default defineConfig({
  reporter: [
    ['html'],
    ['junit', { outputFile: 'test-results/junit.xml' }],
    ['list'],
    ['allure-playwright']  // Allure integration
  ]
});
```

## Report Features

### Dashboard Overview
The main dashboard shows:
- Overall test statistics
- Pass/fail rate
- Test execution time
- Duration trends
- Environment information

### Test Suites
Organized by test suites with:
- Individual test results
- Execution time per test
- Failure details
- Screenshots and attachments

### Timeline View
Visual timeline showing:
- Parallel test execution
- Browser-specific results
- Test duration
- Dependencies

### Categories
Tests are categorized by:
- Ignored tests
- Infrastructure problems
- Product defects
- Test defects

### History
Historical data showing:
- Test result trends
- Performance changes
- Stability improvements

## CI/CD Integration

### GitHub Actions
Allure reports are automatically generated in CI/CD:

1. **Test Execution**: Tests run with Allure collection
2. **Result Upload**: Allure results uploaded as artifacts
3. **Report Generation**: Allure report generated
4. **Report Upload**: HTML report uploaded as artifact
5. **PR Comments**: Report links added to PR comments

### Accessing CI Reports
1. Go to the GitHub Actions workflow run
2. Download the `allure-report` artifact
3. Extract and open `index.html` in a browser

## Best Practices

### 1. Test Organization
- Use descriptive test names
- Organize tests with features and stories
- Use appropriate severity levels
- Tag tests for filtering

### 2. Annotations
- Add meaningful descriptions
- Use severity levels appropriately
- Include performance metrics
- Document test steps

### 3. Failure Analysis
- Investigate failures systematically
- Use categorization to identify patterns
- Monitor flaky tests
- Track regressions

### 4. Performance Monitoring
- Track execution time trends
- Identify slow tests
- Monitor performance regressions
- Optimize test execution

### 5. Historical Tracking
- Generate reports with history
- Compare results over time
- Identify quality trends
- Monitor improvement progress

## Troubleshooting

### Report Not Generated
```bash
# Clear previous results and regenerate
rm -rf allure-results allure-report
npm run test:e2e
npm run test:allure:generate
```

### Missing Screenshots
Ensure tests use proper wait strategies:
```typescript
await expect(page.locator('#element')).toBeVisible();
```

### Historical Data Missing
Generate report with history:
```bash
npm run test:allure:history
```

### Report Not Opening
Check if port is available or use a different port:
```bash
allure open allure-report --port 8081
```

## Advanced Features

### Custom Dashboard
Customize the Allure dashboard by modifying:
- Categories in `allure.config.js`
- Report templates
- Custom widgets
- Environment information

### Integration with Other Tools
Allure integrates with:
- **Jira**: Link tests to issues
- **TestRail**: Sync test results
- **Slack**: Send notifications
- **Email**: Email reports

### Performance Tracking
Add performance metrics to tests:
```typescript
const startTime = Date.now();
// Test execution
const duration = Date.now() - startTime;
test.info().annotations.push({
  type: 'performance',
  description: `Execution time: ${duration}ms`
});
```

## Examples

### Complete Test with Allure
```typescript
import { test, expect } from '@playwright/test';

test('should complete user login @smoke @auth @critical', async ({ page }) => {
  // Add annotations
  test.info().annotations.push({
    type: 'description',
    description: 'Verify user login functionality with valid credentials'
  });
  
  test.info().annotations.push({
    type: 'severity',
    description: 'critical'
  });
  
  test.info().annotations.push({
    type: 'feature',
    description: 'Authentication'
  });

  // Performance tracking
  const startTime = Date.now();
  
  // Test implementation
  await page.goto('/login');
  await page.fill('#username', 'testuser');
  await page.fill('#password', 'testpass');
  await page.click('#login-button');
  
  // Verify login
  await expect(page).toHaveURL('/dashboard');
  
  // Performance annotation
  const duration = Date.now() - startTime;
  test.info().annotations.push({
    type: 'performance',
    description: `Login completed in ${duration}ms`
  });
});
```

## Conclusion

Allure reporting provides powerful test analytics and visualization that complements the E2E testing framework. Use it to:
- Monitor test quality
- Identify trends and regressions
- Improve test stability
- Make data-driven decisions

For more information, visit [Allure Framework Documentation](https://docs.qameta.io/allure/).