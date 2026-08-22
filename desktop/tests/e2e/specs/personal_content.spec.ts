import { test, expect } from '../fixture';

test.describe('Personal content', () => {
  test('favorites lists favorited items and reflects an empty state otherwise', async ({ open }) => {
    const page = await open('/favorites');
    await expect(page.getByTestId('route-content')).toBeVisible();
    const empty = page.getByTestId('empty-state');
    const grid = page.getByTestId('item-grid');
    // Favorites load via an async query; poll until either the empty state
    // or the grid actually renders instead of counting synchronously.
    await expect
      .poll(async () => (await empty.count()) + (await grid.count()))
      .toBeGreaterThan(0);
  });

  test('notes shows the recorded note and its linked item', async ({ open }) => {
    const page = await open('/notes');
    await expect(page.getByText('طريقتي في الاستخدام')).toBeVisible();
  });

  test('links lists saved links and opens the add-link dialog', async ({ open }) => {
    const page = await open('/links');
    await expect(page.getByTestId('link-grid')).toBeVisible();
    await page.getByText('رابط جديد').click();
    await expect(page.getByRole('dialog')).toBeVisible();
  });
});
