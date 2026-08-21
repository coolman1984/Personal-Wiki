/**
 * icon_key → drawing. The database stores a key, never markup or a component
 * reference, so imported data can never inject anything renderable. Unknown keys
 * fall back to a generic glyph instead of breaking the layout.
 *
 * All paths are drawn on a 24×24 grid with round caps and joins.
 */

export type IconSpec = { paths: string[]; circles?: Array<[number, number, number]>; directional?: boolean };

const ICONS: Record<string, IconSpec> = {
  home: { paths: ['M3 10.5 12 3l9 7.5', 'M5 9.5V21h14V9.5', 'M10 21v-6h4v6'] },
  book: { paths: ['M4 4.5A1.5 1.5 0 0 1 5.5 3H19v18H5.5A1.5 1.5 0 0 1 4 19.5z', 'M8 3v18'] },
  terminal: { paths: ['M4 4h16v16H4z', 'm8 9 3 3-3 3', 'M13 15h4'] },
  code: { paths: ['m8 7-5 5 5 5', 'm16 7 5 5-5 5', 'm14 4-4 16'] },
  robot: { paths: ['M6 9h12v9H6z', 'M12 5v4', 'M3 13h3', 'M18 13h3'], circles: [[9.5, 13, 1.2], [14.5, 13, 1.2], [12, 4, 1.4]] },
  brain: { paths: ['M9 4a3 3 0 0 0-3 3 3 3 0 0 0-1 5.8V16a3 3 0 0 0 4 2.8', 'M15 4a3 3 0 0 1 3 3 3 3 0 0 1 1 5.8V16a3 3 0 0 1-4 2.8', 'M12 5v15'] },
  folder: { paths: ['M3 6.5A1.5 1.5 0 0 1 4.5 5H9l2 2.5h8.5A1.5 1.5 0 0 1 21 9v9.5a1.5 1.5 0 0 1-1.5 1.5h-15A1.5 1.5 0 0 1 3 18.5z'] },
  link: { paths: ['M10.5 13.5a4 4 0 0 0 5.7 0l2.3-2.3a4 4 0 0 0-5.7-5.7l-1.2 1.2', 'M13.5 10.5a4 4 0 0 0-5.7 0l-2.3 2.3a4 4 0 0 0 5.7 5.7l1.2-1.2'] },
  shield: { paths: ['M12 3 5 6v6c0 4.4 2.9 7.9 7 9 4.1-1.1 7-4.6 7-9V6z', 'm9 12 2 2 4-4'] },
  key: { paths: ['m14 10 7 7-2 2-2-2-2 2-2-2 2-2z'], circles: [[8, 8, 4.5]] },
  git: { paths: ['M6 3v10.5', 'M6 20.5v-2', 'M18 8.5v3a3 3 0 0 1-3 3H9'], circles: [[6, 17.5, 3], [6, 5.5, 2.5], [18, 5.5, 2.5]] },
  search: { paths: ['m20 20-4.2-4.2'], circles: [[10.5, 10.5, 6.5]] },
  settings: { paths: ['M12 3.5 13.4 6l2.9-.4 1.1 2.7 2.4 1.7-.9 2.8.9 2.8-2.4 1.7-1.1 2.7-2.9-.4L12 20.5 10.6 18l-2.9.4-1.1-2.7L4.2 14l.9-2.8-.9-2.8 2.4-1.7L7.7 4l2.9.4z'], circles: [[12, 12, 3]] },
  star: { paths: ['m12 3.5 2.6 5.4 5.9.8-4.3 4.1 1.1 5.8L12 16.9 6.7 19.6l1.1-5.8-4.3-4.1 5.9-.8z'] },
  note: { paths: ['M6 3h9l4 4v14H6z', 'M15 3v4h4', 'M9 12h6', 'M9 16h4'] },
  bell: { paths: ['M6 9a6 6 0 1 1 12 0c0 4 1.5 5.5 1.5 5.5h-15S6 13 6 9', 'M10.5 18.5a1.8 1.8 0 0 0 3 0'] },
  refresh: { paths: ['M20 5v5h-5', 'M4 19v-5h5', 'M19.5 10a8 8 0 0 0-14-3.2L4 8', 'M4.5 14a8 8 0 0 0 14 3.2L20 16'] },
  clock: { paths: ['M12 7.5V12l3 2'], circles: [[12, 12, 8.5]] },
  target: { paths: [], circles: [[12, 12, 8.5], [12, 12, 4.5], [12, 12, 1.2]] },
  sparkle: { paths: ['m12 3 1.9 5.1L19 10l-5.1 1.9L12 17l-1.9-5.1L5 10l5.1-1.9z', 'M18.5 15.5l.8 2.2 2.2.8-2.2.8-.8 2.2-.8-2.2-2.2-.8 2.2-.8z'] },
  download: { paths: ['M12 3v11', 'm7.5 10 4.5 4 4.5-4', 'M4 19h16'] },
  upload: { paths: ['M12 20V9', 'm7.5 13 4.5-4 4.5 4', 'M4 4h16'] },
  check: { paths: ['m5 12.5 4.5 4.5L19 7.5'] },
  warning: { paths: ['M12 4 3 19.5h18z', 'M12 10v4.5'], circles: [[12, 17, 0.9]] },
  bug: { paths: ['M8 8a4 4 0 0 1 8 0', 'M6.5 11h11v3a5.5 5.5 0 0 1-11 0z', 'M3.5 10.5h3', 'M17.5 10.5h3', 'M4.5 17h2.5', 'M17 17h2.5', 'M9 6 7.5 4', 'm15 6 1.5-2'] },
  play: { paths: ['m9 6 9 6-9 6z'] },
  copy: { paths: ['M8 8h11v11H8z', 'M5 16V5h11'] },
  external: { paths: ['M14 4h6v6', 'M20 4 12 12', 'M18 14v5a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V7a1 1 0 0 1 1-1h5'], directional: true },
  chevron: { paths: ['m10 6 6 6-6 6'], directional: true },
  arrow: { paths: ['M5 12h13', 'm12 6 6 6-6 6'], directional: true },
  pin: { paths: ['M9 4h6l-1 5 3.5 3.5H6.5L10 9z', 'M12 12.5V20'] },
  filter: { paths: ['M4 6h16', 'M7 12h10', 'M10 18h4'] },
  grid: { paths: ['M4 4h6v6H4z', 'M14 4h6v6h-6z', 'M4 14h6v6H4z', 'M14 14h6v6h-6z'] },
  database: { paths: ['M4 6v12c0 1.7 3.6 3 8 3s8-1.3 8-3V6', 'M20 12c0 1.7-3.6 3-8 3s-8-1.3-8-3'], circles: [] },
  cloud: { paths: ['M7 18h10a4 4 0 0 0 .6-8A6 6 0 0 0 6 11.5 3.5 3.5 0 0 0 7 18z'] },
  rocket: { paths: ['M12 3c3 2 5 5.5 5 9l-2.5 2.5h-5L7 12c0-3.5 2-7 5-9z', 'M9.5 17 8 21l3.5-1.5', 'M14.5 17 16 21l-3.5-1.5'], circles: [[12, 10, 1.6]] },
  sliders: { paths: ['M4 8h10', 'M18 8h2', 'M4 16h4', 'M12 16h8'], circles: [[16, 8, 2], [10, 16, 2]] },
  globe: { paths: ['M3.5 12h17', 'M12 3.5c2.5 2.4 2.5 14.6 0 17-2.5-2.4-2.5-14.6 0-17z'], circles: [[12, 12, 8.5]] },
  list: { paths: ['M8 6h12', 'M8 12h12', 'M8 18h12', 'M4 6h.01', 'M4 12h.01', 'M4 18h.01'] },
  trash: { paths: ['M4 7h16', 'M9 7V4h6v3', 'M6 7v13h12V7', 'M10 11v6', 'M14 11v6'] },
  plus: { paths: ['M12 5v14', 'M5 12h14'] },
  close: { paths: ['m6 6 12 12', 'M18 6 6 18'] },
  question: { paths: ['M9.5 9a2.5 2.5 0 1 1 3.4 2.3c-.6.3-.9.9-.9 1.6v.6'], circles: [[12, 12, 8.5], [12, 16.5, 0.9]] },
  user: { paths: ['M4.5 20a7.5 7.5 0 0 1 15 0'], circles: [[12, 8, 4]] },
  eye: { paths: ['M2.5 12S6 6.5 12 6.5 21.5 12 21.5 12 18 17.5 12 17.5 2.5 12 2.5 12z'], circles: [[12, 12, 3]] }
};

export const FALLBACK_ICON = 'book';

export function iconSpec(key: string | null | undefined): IconSpec {
  if (key && ICONS[key]) return ICONS[key]!;
  return ICONS[FALLBACK_ICON]!;
}

export function hasIcon(key: string | null | undefined): boolean {
  return Boolean(key && ICONS[key]);
}

/** Item types get a sensible default glyph when a record does not carry one. */
export function iconForItemType(itemType: string): string {
  switch (itemType) {
    case 'slash_command':
    case 'cli_command':
    case 'command':
      return 'terminal';
    case 'setting':
      return 'settings';
    case 'shortcut':
      return 'sliders';
    case 'workflow':
      return 'git';
    case 'link':
      return 'link';
    case 'extension':
      return 'grid';
    case 'concept':
      return 'brain';
    case 'note':
      return 'note';
    case 'snippet':
      return 'code';
    case 'troubleshooting':
      return 'warning';
    default:
      return FALLBACK_ICON;
  }
}
