import { describe, expect, it } from 'vitest';
import { absoluteDate, bytes, count, relativeTime } from '../../src/lib/format';

describe('format.count', () => {
  it('renders Western digits regardless of locale folding', () => {
    expect(count(1234)).toBe('1,234');
    expect(count(0)).toBe('0');
  });
});

describe('format.bytes', () => {
  it('scales to KB and MB', () => {
    expect(bytes(500)).toBe('500 B');
    expect(bytes(2048)).toBe('2.0 KB');
    expect(bytes(5 * 1024 * 1024)).toBe('5.0 MB');
  });
});

describe('format.relativeTime', () => {
  it('handles missing and invalid input without throwing', () => {
    expect(relativeTime(null)).toBe('—');
    expect(relativeTime('not-a-date')).toBe('—');
  });

  it('reports a value just now as "الآن"', () => {
    expect(relativeTime(new Date().toISOString())).toBe('الآن');
  });
});

describe('format.absoluteDate', () => {
  it('handles missing input without throwing', () => {
    expect(absoluteDate(null)).toBe('—');
  });

  it('formats a real date', () => {
    const formatted = absoluteDate('2026-08-21T12:00:00Z');
    expect(formatted).not.toBe('—');
    expect(formatted.length).toBeGreaterThan(0);
  });
});
