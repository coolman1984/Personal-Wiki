import { describe, expect, it } from 'vitest';
import { FALLBACK_ICON, hasIcon, iconForItemType, iconSpec } from '../../src/lib/icons';

describe('icons', () => {
  it('falls back to a generic glyph for an unknown key', () => {
    expect(hasIcon('totally-made-up-key')).toBe(false);
    expect(iconSpec('totally-made-up-key')).toEqual(iconSpec(FALLBACK_ICON));
  });

  it('resolves a known key', () => {
    expect(hasIcon('terminal')).toBe(true);
    expect(iconSpec('terminal').paths.length).toBeGreaterThan(0);
  });

  it('maps every item type to some icon key', () => {
    const types = [
      'command', 'slash_command', 'cli_command', 'setting', 'shortcut', 'workflow',
      'link', 'extension', 'concept', 'note', 'snippet', 'troubleshooting', 'unknown_type'
    ];
    for (const type of types) {
      const key = iconForItemType(type);
      expect(typeof key).toBe('string');
      expect(key.length).toBeGreaterThan(0);
    }
  });
});
