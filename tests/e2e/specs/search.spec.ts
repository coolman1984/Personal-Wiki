import { test, expect } from '../fixture';

test.describe('Global search', () => {
  test('typing a query shows grouped results with relevance and timing', async ({ open }) => {
    const page = await open('/search?q=claude');
    await expect(page.getByTestId('search-hit').first()).toBeVisible();
    await expect(page.getByText(/تم العثور على/)).toBeVisible();
  });

  test('an alias not present in the title still finds the item', async ({ open }) => {
    const page = await open('/search?q=' + encodeURIComponent('تشغيل وكيل في الخلفية'));
    const hits = page.getByTestId('search-hit');
    expect(await hits.count()).toBeGreaterThan(0);
  });

  test('filters narrow the result set and persist in the URL', async ({ open }) => {
    const page = await open('/search?q=stash');
    const before = await page.getByTestId('search-hit').count();
    await page.getByTestId('search-filters').getByText('كل الأدوات').click();
    await page.getByRole('option', { name: /Git/ }).click();
    await page.keyboard.press('Escape');
    await expect(page).toHaveURL(/tool=git-github/);
    const after = await page.getByTestId('search-hit').count();
    expect(after).toBeLessThanOrEqual(before);
  });

  test('no results shows the empty state with guidance', async ({ open }) => {
    const page = await open('/search?q=' + encodeURIComponent('زقموطة لا وجود لها'));
    await expect(page.getByTestId('empty-state')).toBeVisible();
  });

  test('preview opens the detail component in a dialog without navigating', async ({ open }) => {
    const page = await open('/search?q=claude doctor');
    await page.getByTestId('search-hit').first().getByText('معاينة').click();
    await expect(page.getByRole('dialog')).toBeVisible();
    await expect(page).toHaveURL(/#\/search/);
  });
});
