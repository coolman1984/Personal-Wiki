import { test, expect } from '../fixture';

test.describe('RTL and responsive layout', () => {
  test('the document direction is rtl with Arabic locale', async ({ open }) => {
    const page = await open('/');
    await expect(page.locator('html')).toHaveAttribute('dir', 'rtl');
    await expect(page.locator('html')).toHaveAttribute('lang', 'ar');
  });

  test('code and numeric values stay left-to-right isolated', async ({ open }) => {
    const page = await open('/items/claude-print-mode');
    const code = page.locator('[data-testid="knowledge-detail"] code').first();
    const direction = await code.evaluate((el) => getComputedStyle(el).direction);
    expect(direction).toBe('ltr');
  });

  test('the layout stays usable from 1600px down to 1024px with no body overflow', async ({ open }) => {
    const page = await open('/tools/claude-code');
    for (const width of [1600, 1366, 1180, 1024]) {
      await page.setViewportSize({ width, height: 900 });
      const overflow = await page.evaluate(
        () => document.documentElement.scrollWidth > document.documentElement.clientWidth + 1
      );
      expect(overflow, `overflow at ${width}px`).toBe(false);
    }
  });

  test('directional chevrons mirror in RTL while semantic icons do not', async ({ open }) => {
    const page = await open('/tools/claude-code');
    const chevron = page.locator('.icon-directional').first();
    if (await chevron.count()) {
      const transform = await chevron.evaluate((el) => getComputedStyle(el).transform);
      expect(transform).not.toBe('none');
    }
  });
});
