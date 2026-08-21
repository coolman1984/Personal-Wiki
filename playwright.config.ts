import { defineConfig, devices } from '@playwright/test';
import { existsSync } from 'node:fs';

// The suite drives the real Svelte UI in Chromium against recorded core
// responses. Some sandboxes pre-install a browser outside Playwright's normal
// cache path (see PLAYWRIGHT_CHROMIUM_PATH); everywhere else — including
// Windows CI, after `playwright install chromium` — Playwright resolves its
// own managed browser and this override is skipped entirely.
const pinnedChromium = process.env.PLAYWRIGHT_CHROMIUM_PATH;
const executablePath = pinnedChromium && existsSync(pinnedChromium) ? pinnedChromium : undefined;

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
      use: {
        ...devices['Desktop Chrome'],
        // devices['Desktop Chrome'] carries its own 1280x720 viewport; restate
        // the project's real viewport after the spread so it wins.
        viewport: { width: 1440, height: 900 },
        ...(executablePath ? { launchOptions: { executablePath } } : {})
      }
    }
  ],
  webServer: {
    command: 'pnpm dev',
    port: 5183,
    reuseExistingServer: true,
    timeout: 60_000
  }
});
