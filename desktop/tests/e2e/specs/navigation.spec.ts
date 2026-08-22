import { test, expect } from '../fixture';

test.describe('Shell navigation', () => {
  test('every primary rail destination opens its screen', async ({ open }) => {
    const page = await open('/');
    const routes: Array<[string, string]> = [
      ['nav-tools', '#/tools'],
      ['nav-tasks', '#/tasks'],
      ['nav-links', '#/links'],
      ['nav-favorites', '#/favorites'],
      ['nav-notes', '#/notes'],
      ['nav-updates', '#/updates'],
      ['nav-settings', '#/settings']
    ];
    for (const [testId, hash] of routes) {
      await page.getByTestId(testId).click();
      await expect(page).toHaveURL(new RegExp(hash.replace('/', '\\/') + '$'));
      await expect(page.getByTestId('route-content')).toBeVisible();
    }
  });

  test('an unknown route shows the not-found empty state, not a crash', async ({ open }) => {
    const page = await open('/this-route-does-not-exist');
    await expect(page.getByTestId('empty-state')).toBeVisible();
  });

  test('reloading a deep link restores the same screen', async ({ open }) => {
    const page = await open('/tools/codex-cli');
    await expect(page.getByTestId('tool-stats')).toBeVisible();
    await page.reload();
    await expect(page).toHaveURL(/#\/tools\/codex-cli$/);
    await expect(page.getByTestId('tool-stats')).toBeVisible();
  });
});
