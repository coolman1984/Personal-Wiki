import { test, expect } from '../fixture';

test.describe('Item detail (KnowledgeDetail)', () => {
  test('a fully populated command shows every section in order', async ({ open }) => {
    const page = await open('/items/claude-print-mode');
    const detail = page.getByTestId('knowledge-detail');
    await expect(detail).toBeVisible();

    const sections = ['purpose', 'when', 'syntax', 'parameters', 'example', 'sources'];
    const order: number[] = [];
    for (const name of sections) {
      const el = detail.locator(`[data-section="${name}"]`);
      await expect(el).toBeVisible();
      const box = await el.boundingBox();
      order.push(box?.y ?? 0);
    }
    for (let i = 1; i < order.length; i++) {
      expect(order[i]!).toBeGreaterThan(order[i - 1]!);
    }
  });

  test('a bare link-like item renders only the sections its data has', async ({ open }) => {
    const page = await open('/items/gemini-context-file');
    const detail = page.getByTestId('knowledge-detail');
    await expect(detail).toBeVisible();
    await expect(detail.locator('[data-section="syntax"]')).toHaveCount(0);
    await expect(detail.locator('[data-section="parameters"]')).toHaveCount(0);
  });

  test('copy places the syntax on the clipboard and never executes anything', async ({ open }) => {
    const page = await open('/items/claude-print-mode');
    await page.getByTestId('detail-copy').click();
    const clipboard = await page.evaluate(() => (window as unknown as { __PW_CLIPBOARD__?: string[] }).__PW_CLIPBOARD__);
    expect(clipboard?.[clipboard.length - 1]).toContain('claude -p');
  });

  test('favoriting is optimistic and reflected immediately', async ({ open }) => {
    const page = await open('/items/claude-doctor');
    const button = page.getByTestId('detail-favorite');
    const initial = await button.getAttribute('aria-pressed');
    await button.click();
    await expect(button).not.toHaveAttribute('aria-pressed', initial ?? '');
  });

  test('the master list supports arrow-key navigation between siblings', async ({ open }) => {
    const page = await open('/items/claude-print-mode');
    const list = page.getByRole('listbox');
    await list.focus();
    await page.keyboard.press('ArrowDown');
    await expect(page).not.toHaveURL(/claude-print-mode$/);
  });

  test('code blocks render left-to-right even in the RTL page', async ({ open }) => {
    const page = await open('/items/claude-print-mode');
    const code = page.locator('[data-testid="knowledge-detail"] pre').first();
    await expect(code).toHaveAttribute('dir', 'ltr');
  });
});
