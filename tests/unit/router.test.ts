import { describe, expect, it } from 'vitest';
import { parse, href } from '../../src/lib/router.svelte';

describe('router.parse', () => {
  it('parses the home route', () => {
    expect(parse('#/').name).toBe('home');
  });

  it('parses a tool route with an optional section', () => {
    const withoutSection = parse('#/tools/claude-code');
    expect(withoutSection.name).toBe('tool');
    expect(withoutSection.params.slug).toBe('claude-code');
    expect(withoutSection.params.section).toBeUndefined();

    const withSection = parse('#/tools/claude-code/agents-manage');
    expect(withSection.params.section).toBe('agents-manage');
  });

  it('parses query parameters for search', () => {
    const route = parse('#/search?q=%D8%AA%D8%AB%D8%A8%D9%8A%D8%AA&tool=git-github');
    expect(route.name).toBe('search');
    expect(route.query.get('q')).toBe('تثبيت');
    expect(route.query.get('tool')).toBe('git-github');
  });

  it('falls back to not-found for an unknown path', () => {
    expect(parse('#/nowhere').name).toBe('not-found');
  });

  it('decodes an encoded slug in an item route', () => {
    const route = parse('#/items/claude-print-mode');
    expect(route.name).toBe('item');
    expect(route.params.slug).toBe('claude-print-mode');
  });
});

describe('href', () => {
  it('omits empty query values', () => {
    expect(href('/search', { q: 'x', tool: undefined })).toBe('#/search?q=x');
  });

  it('produces a bare path with no query object', () => {
    expect(href('/tools')).toBe('#/tools');
  });
});
