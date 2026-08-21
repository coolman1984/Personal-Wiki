-- 003 — the personal overlay and durable application state.
-- Nothing in this file may ever be written by a Knowledge Pack apply.

CREATE TABLE favorites (
  item_id    TEXT PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
  created_at TEXT NOT NULL
);

CREATE TABLE pins (
  item_id    TEXT PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
  sort_order INTEGER NOT NULL DEFAULT 100,
  created_at TEXT NOT NULL
);

CREATE TABLE recent_items (
  item_id    TEXT PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
  opened_at  TEXT NOT NULL,
  open_count INTEGER NOT NULL DEFAULT 1
);
CREATE INDEX idx_recent_opened ON recent_items(opened_at DESC);

CREATE TABLE notes (
  id         TEXT PRIMARY KEY,
  item_id    TEXT REFERENCES items(id) ON DELETE CASCADE,
  title      TEXT NOT NULL,
  body       TEXT NOT NULL DEFAULT '',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE INDEX idx_notes_item ON notes(item_id);

CREATE TABLE saved_links (
  id          TEXT PRIMARY KEY,
  title       TEXT NOT NULL,
  url         TEXT NOT NULL,
  description TEXT,
  tool_id     TEXT REFERENCES tools(id) ON DELETE SET NULL,
  category_id TEXT REFERENCES categories(id) ON DELETE SET NULL,
  is_favorite INTEGER NOT NULL DEFAULT 0 CHECK (is_favorite IN (0,1)),
  sort_order  INTEGER NOT NULL DEFAULT 100,
  created_at  TEXT NOT NULL,
  updated_at  TEXT NOT NULL
);

CREATE TABLE app_settings (
  key        TEXT PRIMARY KEY,
  value_json TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE saved_searches (
  id           TEXT PRIMARY KEY,
  name         TEXT NOT NULL UNIQUE,
  query        TEXT NOT NULL,
  filters_json TEXT NOT NULL DEFAULT '{}',
  created_at   TEXT NOT NULL,
  last_run_at  TEXT
);

CREATE TABLE search_history (
  id           TEXT PRIMARY KEY,
  query        TEXT NOT NULL,
  filters_json TEXT NOT NULL DEFAULT '{}',
  result_count INTEGER NOT NULL DEFAULT 0,
  executed_at  TEXT NOT NULL
);
CREATE INDEX idx_search_history_at ON search_history(executed_at DESC);
