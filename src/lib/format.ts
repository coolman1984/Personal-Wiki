/** Arabic-facing formatting helpers. Numbers stay Western digits (LTR-isolated). */

const RTF = new Intl.RelativeTimeFormat('ar', { numeric: 'auto' });

export function relativeTime(iso: string | null | undefined): string {
  if (!iso) return '—';
  const then = Date.parse(iso);
  if (Number.isNaN(then)) return '—';
  const diffSeconds = Math.round((then - Date.now()) / 1000);
  const abs = Math.abs(diffSeconds);
  if (abs < 60) return 'الآن';
  const units: Array<[Intl.RelativeTimeFormatUnit, number]> = [
    ['minute', 60],
    ['hour', 3600],
    ['day', 86400],
    ['month', 2592000],
    ['year', 31536000]
  ];
  let chosen: [Intl.RelativeTimeFormatUnit, number] = ['minute', 60];
  for (const unit of units) {
    if (abs >= unit[1]) chosen = unit;
  }
  return RTF.format(Math.round(diffSeconds / chosen[1]), chosen[0]);
}

export function absoluteDate(iso: string | null | undefined): string {
  if (!iso) return '—';
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return '—';
  return new Intl.DateTimeFormat('ar', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  }).format(date);
}

export function count(n: number): string {
  return new Intl.NumberFormat('ar-EG-u-nu-latn').format(n);
}

export function bytes(n: number): string {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  return `${(n / 1024 / 1024).toFixed(1)} MB`;
}

/** "3 أوامر" style pluralisation for the few counters the UI shows. */
export function plural(n: number, one: string, two: string, few: string, many: string): string {
  if (n === 1) return one;
  if (n === 2) return two;
  if (n % 100 >= 3 && n % 100 <= 10) return `${count(n)} ${few}`;
  return `${count(n)} ${many}`;
}
