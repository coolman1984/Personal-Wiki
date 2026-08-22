import { test as base, expect, type Page } from '@playwright/test';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

export interface Fixture {
  generatedBy: string;
  itemSlugs: string[];
  responses: Record<string, unknown>;
}

const fixture = JSON.parse(
  readFileSync(fileURLToPath(new URL('./fixtures/api.json', import.meta.url)), 'utf8')
) as Fixture;

const harnessSource = readFileSync(
  fileURLToPath(new URL('./harness.browser.js', import.meta.url)),
  'utf8'
);

/**
 * `open(path)` navigates to a hash route with the recorded core transport
 * already installed, and waits for the route to render.
 */
export const test = base.extend<{ open: (path?: string) => Promise<Page> }>({
  open: async ({ page }, use) => {
    await page.addInitScript(
      ({ source, data }: { source: string; data: Fixture }) => {
        new Function('fixture', source)(data);
      },
      { source: harnessSource, data: fixture }
    );
    await use(async (path = '/') => {
      await page.goto(`/#${path}`);
      await page.waitForSelector('[data-testid="route-content"]');
      return page;
    });
  }
});

export { expect, fixture };
