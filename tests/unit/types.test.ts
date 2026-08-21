import { describe, expect, it } from 'vitest';
import { emptyFilters } from '../../src/lib/types';

describe('emptyFilters', () => {
  it('produces a filter object with no active filters', () => {
    const filters = emptyFilters();
    expect(filters.toolSlugs).toEqual([]);
    expect(filters.favoritesOnly).toBe(false);
  });

  it('returns a fresh object each call (no shared mutable state)', () => {
    const a = emptyFilters();
    const b = emptyFilters();
    a.toolSlugs.push('x');
    expect(b.toolSlugs).toEqual([]);
  });
});
