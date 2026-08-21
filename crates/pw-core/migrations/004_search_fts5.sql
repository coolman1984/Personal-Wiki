-- 004 — FTS5 search index.
--
-- The index is a flattened projection of relational data (aliases, tags, examples,
-- task names/phrases, tool and category names) so one MATCH covers every field the
-- UI spec requires. It is maintained explicitly by pw-core::search after any write
-- that touches an item, not by triggers: the flattening needs joins that triggers
-- cannot express cheaply, and explicit maintenance keeps apply transactional.
--
-- All indexed text is passed through pw-core::search::normalize first (Arabic
-- folding, digit folding, case folding), and queries are normalized the same way.

CREATE VIRTUAL TABLE items_fts USING fts5(
  item_id UNINDEXED,
  title,
  aliases,
  short_description,
  full_description,
  task_phrases,
  tags,
  syntax,
  examples,
  tool,
  category,
  tokenize = 'unicode61 remove_diacritics 2'
);
