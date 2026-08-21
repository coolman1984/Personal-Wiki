import { defineConfig, devices } from '@playwright/test';

// The suite drives the real Svelte UI in Chromium against recorded core
// responses. Chromium is pre-installed in this environment; on Windows CI the
// same suite runs after `playwright install chromium`.
export default defineConfig({
  testDir: './tests/e2e',
  timeout: 30_000,
  expect: { timeout: 7_000 },
  fullyParallel: false,
  workers: 1,
  reporter: [['list'], ['html', { open: 'never', outputFolder: 'playwright-report' }]],
  outputDir: 'test-results',
  use: {
    baseURL: 'http://localhost:5183',
    locale: 'ar',
    viewport: { width: 1440, height: 900 },
    screenshot: 'only-on-failure',
    trace: 'retain-on-failure'
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'], launchOptions: { executablePath: '/opt/pw-browsers/chromium' } }
    }
  ],
  webServer: {
    command: 'pnpm dev',
    port: 5183,
    reuseExistingServer: true,
    timeout: 60_000
  }
});
