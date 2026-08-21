-- Personal Wiki — 001 initial knowledge schema.
-- Every knowledge entity carries a stable id (ULID) and, when it is managed by an
-- external source, a stable `external_key` used for deterministic refresh matching.

CREATE TABLE tools (
  id                TEXT PRIMARY KEY,
  slug              TEXT NOT NULL UNIQUE,
  name              TEXT NOT NULL,
  short_name        TEXT,
  description       TEXT,
  icon_key          TEXT NOT NULL DEFAULT 'book',
  accent_key        TEXT NOT NULL DEFAULT 'blue',
  official_home_url TEXT,
  sort_order        INTEGER NOT NULL DEFAULT 100,
  is_active         INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0,1)),
  external_key      TEXT UNIQUE,
  managed_by        TEXT NOT NULL DEFAULT 'local' CHECK (managed_by IN ('local','pack')),
  created_at        TEXT NOT NULL,
  updated_at        TEXT NOT NULL
);

CREATE TABLE tasks (
  id           TEXT PRIMARY KEY,
  slug         TEXT NOT NULL UNIQUE,
  name         TEXT NOT NULL,
  description  TEXT,
  icon_key     TEXT NOT NULL DEFAULT 'target',
  sort_order   INTEGER NOT NULL DEFAULT 100,
  is_active    INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0,1)),
  external_key TEXT UNIQUE,
  managed_by   TEXT NOT NULL DEFAULT 'local' CHECK (managed_by IN ('local','pack')),
  created_at   TEXT NOT NULL,
  updated_at   TEXT NOT NULL
);

-- Arbitrary depth tree. parent_id RESTRICT: a parent may not be deleted while it
-- still has children; deactivate instead of deleting.
CREATE TABLE categories (
  id           TEXT PRIMARY KEY,
  slug         TEXT NOT NULL UNIQUE,
  name         TEXT NOT NULL,
  description  TEXT,
  icon_key     TEXT NOT NULL DEFAULT 'folder',
  accent_key   TEXT NOT NULL DEFAULT 'blue',
  parent_id    TEXT REFERENCES categories(id) ON DELETE RESTRICT,
  sort_order   INTEGER NOT NULL DEFAULT 100,
  is_active    INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0,1)),
  external_key TEXT UNIQUE,
  managed_by   TEXT NOT NULL DEFAULT 'local' CHECK (managed_by IN ('local','pack')),
  created_at   TEXT NOT NULL,
  updated_at   TEXT NOT NULL
);
CREATE INDEX idx_categories_parent ON categories(parent_id);

CREATE TABLE items (
  id                  TEXT PRIMARY KEY,
  tool_id             TEXT REFERENCES tools(id) ON DELETE SET NULL,
  item_type           TEXT NOT NULL CHECK (item_type IN
                        ('command','slash_command','cli_command','setting','shortcut',
                         'workflow','link','extension','concept','note','snippet','troubleshooting')),
  slug                TEXT NOT NULL UNIQUE,
  title               TEXT NOT NULL,
  subtitle            TEXT,
  short_description   TEXT,
  full_description    TEXT,
  syntax              TEXT,
  when_to_use         TEXT,
  expected_result     TEXT,
  warning_text        TEXT,
  -- Optional typed diagram payload. Validated against the ResultDiagram contract
  -- (docs/DATA_CONTRACTS.md) on import; never raw HTML or SVG.
  result_diagram      TEXT,
  verification_status TEXT NOT NULL DEFAULT 'unverified'
                        CHECK (verification_status IN ('official','verified','community','unverified','demo')),
  lifecycle_status    TEXT NOT NULL DEFAULT 'active'
                        CHECK (lifecycle_status IN ('active','experimental','deprecated','removed')),
  difficulty          TEXT CHECK (difficulty IN ('beginner','intermediate','advanced')),
  icon_key            TEXT,
  accent_key          TEXT,
  sort_order          INTEGER NOT NULL DEFAULT 100,
  is_active           INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0,1)),
  external_key        TEXT UNIQUE,
  managed_by          TEXT NOT NULL DEFAULT 'local' CHECK (managed_by IN ('local','pack')),
  created_at          TEXT NOT NULL,
  updated_at          TEXT NOT NULL,
  last_verified_at    TEXT
);
CREATE INDEX idx_items_tool ON items(tool_id);
CREATE INDEX idx_items_active ON items(is_active, lifecycle_status);
CREATE INDEX idx_items_type ON items(item_type);

CREATE TABLE item_tasks (
  item_id TEXT NOT NULL REFERENCES items(id) ON DELETE CASCADE,
  task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
  PRIMARY KEY (item_id, task_id)
);
CREATE INDEX idx_item_tasks_task ON item_tasks(task_id);

CREATE TABLE item_categories (
  item_id     TEXT NOT NULL REFERENCES items(id) ON DELETE CASCADE,
  category_id TEXT NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
  PRIMARY KEY (item_id, category_id)
);
CREATE INDEX idx_item_categories_category ON item_categories(category_id);

CREATE TABLE tags (
  id         TEXT PRIMARY KEY,
  slug       TEXT NOT NULL UNIQUE,
  name       TEXT NOT NULL,
  accent_key TEXT NOT NULL DEFAULT 'blue'
);

CREATE TABLE item_tags (
  item_id TEXT NOT NULL REFERENCES items(id) ON DELETE CASCADE,
  tag_id  TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
  PRIMARY KEY (item_id, tag_id)
);
CREATE INDEX idx_item_tags_tag ON item_tags(tag_id);

CREATE TABLE aliases (
  id      TEXT PRIMARY KEY,
  item_id TEXT NOT NULL REFERENCES items(id) ON DELETE CASCADE,
  alias   TEXT NOT NULL,
  UNIQUE (item_id, alias)
);
CREATE INDEX idx_aliases_item ON aliases(item_id);

CREATE TABLE examples (
  id              TEXT PRIMARY KEY,
  item_id         TEXT NOT NULL REFERENCES items(id) ON DELETE CASCADE,
  title           TEXT,
  description     TEXT,
  code            TEXT NOT NULL,
  expected_output TEXT,
  sort_order      INTEGER NOT NULL DEFAULT 100
);
CREATE INDEX idx_examples_item ON examples(item_id);

CREATE TABLE parameters (
  id            TEXT PRIMARY KEY,
  item_id       TEXT NOT NULL REFERENCES items(id) ON DELETE CASCADE,
  name          TEXT NOT NULL,
  description   TEXT,
  is_required   INTEGER NOT NULL DEFAULT 0 CHECK (is_required IN (0,1)),
  default_value TEXT,
  value_hint    TEXT,
  sort_order    INTEGER NOT NULL DEFAULT 100,
  UNIQUE (item_id, name)
);
CREATE INDEX idx_parameters_item ON parameters(item_id);

CREATE TABLE sources (
  id                     TEXT PRIMARY KEY,
  tool_id                TEXT REFERENCES tools(id) ON DELETE SET NULL,
  name                   TEXT NOT NULL,
  source_type            TEXT NOT NULL CHECK (source_type IN
                           ('official_docs','official_repo','official_changelog','official_blog',
                            'community','local_file','manual','agent_managed')),
  url                    TEXT,
  is_official            INTEGER NOT NULL DEFAULT 0 CHECK (is_official IN (0,1)),
  -- How this source is refreshed. V1 never scrapes: 'agent' means an external agent
  -- researches it and produces a Knowledge Pack.
  sync_strategy          TEXT NOT NULL DEFAULT 'agent'
                           CHECK (sync_strategy IN ('agent','manual','local_file')),
  is_active              INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0,1)),
  check_interval_minutes INTEGER NOT NULL DEFAULT 1440,
  last_checked_at        TEXT,
  last_success_at        TEXT,
  last_failure_at        TEXT,
  last_error             TEXT,
  next_check_at          TEXT,
  etag                   TEXT,
  last_modified          TEXT,
  content_hash           TEXT,
  notes                  TEXT,
  external_key           TEXT UNIQUE,
  created_at             TEXT NOT NULL,
  updated_at             TEXT NOT NULL
);
CREATE INDEX idx_sources_tool ON sources(tool_id);

CREATE TABLE item_sources (
  item_id        TEXT NOT NULL REFERENCES items(id) ON DELETE CASCADE,
  source_id      TEXT NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
  source_url     TEXT,
  source_anchor  TEXT,
  source_version TEXT,
  source_hash    TEXT,
  verified_at    TEXT,
  PRIMARY KEY (item_id, source_id)
);
CREATE INDEX idx_item_sources_source ON item_sources(source_id);

CREATE TABLE relationships (
  id                TEXT PRIMARY KEY,
  from_item_id      TEXT NOT NULL REFERENCES items(id) ON DELETE CASCADE,
  to_item_id        TEXT NOT NULL REFERENCES items(id) ON DELETE CASCADE,
  relationship_type TEXT NOT NULL CHECK (relationship_type IN
                      ('related','alternative','prerequisite','replaced_by','similar_to','follows','part_of')),
  UNIQUE (from_item_id, to_item_id, relationship_type),
  CHECK (from_item_id <> to_item_id)
);
CREATE INDEX idx_relationships_to ON relationships(to_item_id);
