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
      matchedStatuses: ['skipped'],
      messageRegex: /.*ignored.*/
    },
    {
      name: 'Infrastructure problems',
      matchedStatuses: ['broken', 'failed'],
      messageRegex: /.*Timeout.*/,
      traceRegex: /.*TimeoutError.*/
    },
    {
      name: 'Product defects',
      matchedStatuses: ['failed']
    },
    {
      name: 'Test defects',
      matchedStatuses: ['broken']
    }
  ]
};