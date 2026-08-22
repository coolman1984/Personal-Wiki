/**
 * Hash routing. Packaged Tauri serves the SPA from the filesystem, so hash
 * routes are the option that works identically in dev, in the packaged app, and
 * under Playwright — no server rewrite rules involved.
 *
 * Every screen's state (tool, section, selected item, query, filters) lives in
 * the URL so deep links and reloads restore exactly what the user was looking at.
 */

export interface Route {
  name:
    | 'home' | 'tools' | 'tool' | 'tasks' | 'task' | 'category' | 'item'
    | 'search' | 'favorites' | 'notes' | 'links' | 'updates' | 'batch'
    | 'settings' | 'diagnostics' | 'not-found';
  params: Record<string, string>;
  query: URLSearchParams;
  path: string;
}

const PATTERNS: Array<[RegExp, Route['name'], string[]]> = [
  [/^\/$/, 'home', []],
  [/^\/tools$/, 'tools', []],
  [/^\/tools\/([^/]+)$/, 'tool', ['slug']],
  [/^\/tools\/([^/]+)\/([^/]+)$/, 'tool', ['slug', 'section']],
  [/^\/tasks$/, 'tasks', []],
  [/^\/tasks\/([^/]+)$/, 'task', ['slug']],
  [/^\/c\/([^/]+)$/, 'category', ['slug']],
  [/^\/items\/([^/]+)$/, 'item', ['slug']],
  [/^\/search$/, 'search', []],
  [/^\/favorites$/, 'favorites', []],
  [/^\/notes$/, 'notes', []],
  [/^\/links$/, 'links', []],
  [/^\/updates$/, 'updates', []],
  [/^\/updates\/batches\/([^/]+)$/, 'batch', ['id']],
  [/^\/settings$/, 'settings', []],
  [/^\/diagnostics$/, 'diagnostics', []]
];

export function parse(hash: string): Route {
  const raw = hash.replace(/^#/, '') || '/';
  const [pathPart = '/', queryPart = ''] = raw.split('?');
  const path = decodeURI(pathPart);
  const query = new URLSearchParams(queryPart);

  for (const [pattern, name, keys] of PATTERNS) {
    const match = pattern.exec(path);
    if (!match) continue;
    const params: Record<string, string> = {};
    keys.forEach((key, index) => {
      params[key] = decodeURIComponent(match[index + 1] ?? '');
    });
    return { name, params, query, path };
  }
  return { name: 'not-found', params: {}, query, path };
}

export function href(path: string, query?: Record<string, string | undefined>): string {
  const params = new URLSearchParams();
  for (const [key, value] of Object.entries(query ?? {})) {
    if (value !== undefined && value !== '') params.set(key, value);
  }
  const suffix = params.toString();
  return `#${path}${suffix ? `?${suffix}` : ''}`;
}

export function navigate(path: string, query?: Record<string, string | undefined>): void {
  window.location.hash = href(path, query).slice(1);
}

export function replace(path: string, query?: Record<string, string | undefined>): void {
  const next = href(path, query);
  window.history.replaceState(null, '', next);
  window.dispatchEvent(new HashChangeEvent('hashchange'));
}

/** Reactive current route (Svelte 5 runes). */
export function createRouteState() {
  let current = $state(parse(window.location.hash));
  const onChange = () => {
    current = parse(window.location.hash);
  };
  window.addEventListener('hashchange', onChange);
  return {
    get route() {
      return current;
    },
    dispose() {
      window.removeEventListener('hashchange', onChange);
    }
  };
}
