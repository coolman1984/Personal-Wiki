-- 002 — knowledge update engine: batches, staged changes, backups, audit, sync history.

CREATE TABLE update_batches (
  id          TEXT PRIMARY KEY,
  source_id   TEXT REFERENCES sources(id) ON DELETE SET NULL,
  created_by  TEXT NOT NULL DEFAULT 'external-agent',
  producer    TEXT,
  summary     TEXT,
  notes       TEXT,
  pack_hash   TEXT,
  status      TEXT NOT NULL DEFAULT 'pending'
                CHECK (status IN ('pending','partially_applied','applied','rejected','failed','reverted')),
  backup_id   TEXT,
  created_at  TEXT NOT NULL,
  applied_at  TEXT,
  reverted_at TEXT
);
CREATE INDEX idx_update_batches_status ON update_batches(status);

CREATE TABLE staged_changes (
  id          TEXT PRIMARY KEY,
  batch_id    TEXT NOT NULL REFERENCES update_batches(id) ON DELETE CASCADE,
  entity_type TEXT NOT NULL CHECK (entity_type IN
                ('tool','task','category','item','alias','example','parameter','tag','source','relationship')),
  entity_key  TEXT NOT NULL,          -- stable external identity within entity_type
  entity_id   TEXT,                   -- resolved local id once known
  change_type TEXT NOT NULL CHECK (change_type IN ('add','update','deprecate','remove_candidate','relink')),
  title       TEXT,
  detail      TEXT,
  before_json TEXT,
  after_json  TEXT,
  source_id   TEXT REFERENCES sources(id) ON DELETE SET NULL,
  confidence  REAL,
  status      TEXT NOT NULL DEFAULT 'pending'
                CHECK (status IN ('pending','approved','rejected','applied','failed')),
  error_text  TEXT,
  detected_at TEXT NOT NULL,
  reviewed_at TEXT,
  applied_at  TEXT,
  UNIQUE (batch_id, entity_type, entity_key)
);
CREATE INDEX idx_staged_batch ON staged_changes(batch_id, status);
CREATE INDEX idx_staged_status ON staged_changes(status);

CREATE TABLE backups (
  id             TEXT PRIMARY KEY,
  path           TEXT NOT NULL,
  reason         TEXT NOT NULL,
  batch_id       TEXT REFERENCES update_batches(id) ON DELETE SET NULL,
  size_bytes     INTEGER NOT NULL DEFAULT 0,
  schema_version INTEGER NOT NULL DEFAULT 0,
  item_count     INTEGER NOT NULL DEFAULT 0,
  created_at     TEXT NOT NULL
);

CREATE TABLE audit_log (
  id          TEXT PRIMARY KEY,
  at          TEXT NOT NULL,
  actor       TEXT NOT NULL,
  action      TEXT NOT NULL,
  entity_type TEXT,
  entity_id   TEXT,
  batch_id    TEXT,
  detail_json TEXT
);
CREATE INDEX idx_audit_at ON audit_log(at DESC);
CREATE INDEX idx_audit_batch ON audit_log(batch_id);

CREATE TABLE sync_runs (
  id            TEXT PRIMARY KEY,
  started_at    TEXT NOT NULL,
  finished_at   TEXT,
  status        TEXT NOT NULL CHECK (status IN ('running','success','partial','failed')),
  checked_count INTEGER NOT NULL DEFAULT 0,
  changed_count INTEGER NOT NULL DEFAULT 0,
  summary       TEXT,
  error_text    TEXT
);
CREATE INDEX idx_sync_runs_started ON sync_runs(started_at DESC);

CREATE TABLE source_checks (
  id           TEXT PRIMARY KEY,
  source_id    TEXT NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
  sync_run_id  TEXT REFERENCES sync_runs(id) ON DELETE SET NULL,
  checked_at   TEXT NOT NULL,
  status       TEXT NOT NULL CHECK (status IN ('ok','unchanged','failed','skipped')),
  content_hash TEXT,
  error_text   TEXT
);
CREATE INDEX idx_source_checks_source ON source_checks(source_id, checked_at DESC);
