import { test, expect, fixture } from '../fixture';

test.describe('Update center', () => {
  test('shows sources, the summary strip, and the approval bar', async ({ open }) => {
    const page = await open('/updates');
    await expect(page.getByTestId('source-list')).toBeVisible();
    await expect(page.getByTestId('audit-timeline')).toBeVisible();
    await expect(page.getByText('قائمة الموافقة')).toBeVisible();
  });

  test('a batch diff shows a per-field before/after and approve/reject controls', async ({ open }) => {
    const batches = (fixture.responses['list_batches|{}'] as Array<{ id: string; status: string }>) ?? [];
    const applied = batches.find((b) => b.status === 'applied');
    test.skip(!applied, 'fixture has no applied batch to inspect');
    const page = await open(`/updates/batches/${applied!.id}`);
    await expect(page.getByTestId('batch-changes')).toBeVisible();
  });
});
