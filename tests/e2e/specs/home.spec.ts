import { test, expect } from '../fixture';

test.describe('Home', () => {
  test('renders the hero, tool grid, tasks, and activity panel', async ({ open }) => {
    const page = await open('/');
    await expect(page.getByRole('heading', { name: 'خريطتك الشخصية للأوامر والمعرفة' })).toBeVisible();

    const grid = page.getByTestId('tool-grid');
    await expect(grid).toBeVisible();
    const cards = page.getByTestId('tool-card');
    await expect(cards.first()).toBeVisible();
    expect(await cards.count()).toBeGreaterThanOrEqual(5);

    // Right panel: recent activity / recent items / quick actions.
    await expect(page.getByText('آخر التحديثات')).toBeVisible();
    await expect(page.getByText('إجراءات سريعة')).toBeVisible();
  });

  test('a tool card opens its tool family page', async ({ open }) => {
    const page = await open('/');
    await page.locator('[data-testid="tool-card"][data-slug="claude-code"]').click();
    await expect(page).toHaveURL(/#\/tools\/claude-code$/);
    await expect(page.getByTestId('tool-stats')).toBeVisible();
  });

  test('Ctrl+K focuses the global search field', async ({ open }) => {
    const page = await open('/');
    await page.keyboard.press('Control+k');
    await expect(page.getByTestId('global-search')).toBeFocused();
  });

  test('keyboard focus is visible on a tool card', async ({ open }) => {
    const page = await open('/');
    const firstCard = page.getByTestId('tool-card').first();
    await firstCard.focus();
    await expect(firstCard).toBeFocused();
  });
});
