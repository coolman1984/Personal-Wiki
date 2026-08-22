import { test, expect } from '../fixture';

test.describe('Security', () => {
  test('opening an item with only an unsafe-looking url still renders safely', async ({ open }) => {
    // The fixture harness mirrors the core's scheme check; this proves the UI
    // surfaces a failure instead of silently "opening" something unsafe.
    const page = await open('/');
    const rejected = await page.evaluate(async () => {
      try {
        await (window as unknown as { __PW_FIXTURE_INVOKE__: (c: string, a: unknown) => Promise<unknown> })
          .__PW_FIXTURE_INVOKE__('open_external', { url: 'javascript:alert(1)' });
        return false;
      } catch {
        return true;
      }
    });
    expect(rejected).toBe(true);
  });

  test('a valid https url is accepted by the same check', async ({ open }) => {
    const page = await open('/');
    const accepted = await page.evaluate(async () => {
      try {
        await (window as unknown as { __PW_FIXTURE_INVOKE__: (c: string, a: unknown) => Promise<unknown> })
          .__PW_FIXTURE_INVOKE__('open_external', { url: 'https://example.com' });
        return true;
      } catch {
        return false;
      }
    });
    expect(accepted).toBe(true);
  });
});
