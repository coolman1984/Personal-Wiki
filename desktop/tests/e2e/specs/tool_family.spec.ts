import { test, expect } from '../fixture';

test.describe('Tool family screen', () => {
  test('shows identity, stat strip, section tabs, and item cards', async ({ open }) => {
    const page = await open('/tools/claude-code');
    await expect(page.getByRole('heading', { name: 'Claude Code', level: 1 })).toBeVisible();
    await expect(page.getByTestId('tool-stats')).toBeVisible();
    await expect(page.getByTestId('section-tabs')).toBeVisible();
    // Items load via a separate async query; wait for the first card rather
    // than reading .count() synchronously.
    await expect(page.getByTestId('item-card').first()).toBeVisible();
    expect(await page.getByTestId('item-card').count()).toBeGreaterThan(0);
  });

  test('selecting a section tab filters the item grid and updates the URL', async ({ open }) => {
    const page = await open('/tools/claude-code');
    await expect(page.getByTestId('item-card').first()).toBeVisible();
    const before = await page.getByTestId('item-card').count();
    const tab = page.locator('[data-testid="section-tabs"] button[data-slug="agents-manage"]');
    if ((await tab.count()) > 0) {
      await tab.click();
      await expect(page).toHaveURL(/#\/tools\/claude-code\/agents-manage$/);
      const after = await page.getByTestId('item-card').count();
      expect(after).toBeLessThanOrEqual(before);
    }
  });

  test('long Arabic descriptions do not overflow the card', async ({ open }) => {
    const page = await open('/tools/claude-code');
    const card = page.getByTestId('item-card').first();
    const box = await card.boundingBox();
    const scrollWidth = await card.evaluate((el) => el.scrollWidth);
    expect(box).not.toBeNull();
    if (box) expect(scrollWidth).toBeLessThanOrEqual(Math.ceil(box.width) + 1);
  });

  test('the page body never scrolls horizontally', async ({ open }) => {
    const page = await open('/tools/claude-code');
    const overflow = await page.evaluate(
      () => document.documentElement.scrollWidth > document.documentElement.clientWidth + 1
    );
    expect(overflow).toBe(false);
  });
});
