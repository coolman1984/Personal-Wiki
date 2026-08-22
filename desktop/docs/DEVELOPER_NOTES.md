# Developer Notes

Internal notes for anyone continuing work on the **desktop** (Tauri/Rust/Svelte)
implementation of Personal Wiki, which lives in this `desktop/` directory. A
separate Next.js/Cloudflare web implementation lives at the repository root —
see `../docs/WEBAPP_README.md` for that one; this file is only about the code
in `desktop/`.

For product/UX rules shared by both implementations, see
`../PERSONAL_WIKI_MASTER_GUIDE.md` and `../docs/UI_SPEC.md`.

## Repository layout (all paths below are relative to `desktop/`)

```text
crates/pw-core/     shared Rust domain core (SQLite, migrations, search,
                     Knowledge Pack validation, staging, apply, backup, audit)
crates/pw-cli/       `personal-wiki` binary — a thin shell over pw-core
src-tauri/           Tauri 2 desktop shell — typed commands, thin shell over pw-core
src/                 Svelte 5 + TypeScript frontend (the SPA the Tauri window loads)
schemas/             Knowledge Pack JSON Schema
seed/                starter taxonomy + source-backed content packs (installed
                     through the same stage→approve→apply pipeline as any pack)
tests/e2e/           Playwright suite, driven against a recorded fixture of the
                     real core (no second implementation of any query)
crates/pw-core/tests/  Rust integration tests (dynamic data, update safety,
                     search, migrations/security)
crates/pw-cli/tests/   CLI end-to-end tests (spawn the real binary)
tests/unit/          Vitest unit tests for pure frontend logic (router, format,
                     icons, types)
../.github/workflows/  Windows CI: build, test, and produce the NSIS/MSI
                     installer for this app (must live at the repo root for
                     GitHub Actions to discover it; its steps `cd desktop`)
```

## The one rule that shaped every decision

**The executable is stable. The knowledge is dynamic.** Every screen queries
`pw-core::repo` at render time. There is no hard-coded tool/category/command
list anywhere in `src/`. Adding a tool, task, category, or item is a Knowledge
Pack import; the frontend never changes.

## Why one Rust core and two shells

`crates/pw-core` owns SQLite, migrations, the read model, search, pack
validation, staging/diff, transactional apply, backup/restore, and diagnostics.
`crates/pw-cli` and `src-tauri` both call into it — neither reimplements a rule.
If you find yourself writing SQL inside `src-tauri/src/lib.rs` or
`crates/pw-cli/src/main.rs`, stop: it belongs in `pw-core::repo` (or `::edit`,
`::personal`, etc.) instead.

The Svelte frontend never touches SQLite. It calls typed functions in
`src/lib/api.ts`, which call `src/lib/ipc.ts`, which invokes a Tauri command.

## Identity and the update pipeline

Every externally managed row (`tools`, `tasks`, `categories`, `items`, `sources`)
carries a nullable `external_key`. A Knowledge Pack item's `externalKey` is the
thing refreshes match on — never the mutable title. See
`crates/pw-core/src/pack.rs` for the full validation contract and
`crates/pw-core/src/staging.rs` / `apply.rs` for how a validated pack becomes a
reviewable diff and then a transactional write.

Pipeline: `parse → validate → stage → diff → review (approve/reject) → backup →
transactional apply → FTS reindex → audit → commit`. Failure rolls the
transaction back and keeps the pre-apply backup. `apply::revert_batch` reverses
an applied batch from its stored `before_json` without touching the personal
overlay (`favorites`, `pins`, `notes`, `saved_links`, `app_settings`,
`saved_searches`, `search_history` — migration `003_personal_and_app_state.sql`).

## Search

`crates/pw-core/src/search.rs`: Arabic-aware `normalize()`, an FTS5 virtual
table (`004_search_fts5.sql`) flattened from relational data, and a scoring
pass on top of `bm25()` implementing the ranking contract in
`docs/UI_SPEC.md` §9. `reindex_item` is called by `apply` for every touched
item inside the same transaction; `rebuild` is the full-recovery path used by
`doctor --repair` and the Settings screen's "rebuild index" action.

Benchmarked at ~10k items in `crates/pw-core/tests/search_benchmark.rs`
(release build): typical queries were in the 0.3–20 ms range on this
development machine, well under the 150 ms target. Re-run with
`cargo test --release -p pw-core --test search_benchmark -- --nocapture`.

## The Playwright / fixture setup

There is no Tauri runtime inside a plain browser, so `tests/e2e` cannot call
real Tauri commands. Instead:

1. `cargo run -p pw-cli --example dev_fixtures -- tests/e2e/fixtures/api.json`
   opens a fresh, seeded database, runs the *actual* `pw-core::repo` /
   `search` / `personal` functions the app calls, and records every response
   keyed by `command|canonicalized-args`.
2. `tests/e2e/harness.browser.js` is injected into the page before app code
   runs. It answers `window.__PW_FIXTURE_INVOKE__` from that recording (falling
   back to the no-argument recording for optional args) and patches a couple
   of fields in place for write commands (`set_favorite`, `copy_text`,
   `open_external`) so the UI's optimistic-update paths are actually exercised.
3. `src/lib/ipc.ts` prefers the real Tauri `invoke` when `__TAURI_INTERNALS__`
   exists, and falls back to `window.__PW_FIXTURE_INVOKE__` otherwise — that
   fallback only ever fires under Playwright/Vite dev without Tauri.

**Regenerate the fixture whenever a query, the seed content, or an API shape
changes**: `cargo run -p pw-cli --example dev_fixtures -- tests/e2e/fixtures/api.json`.
A stale fixture shows up as a Playwright failure with "no recorded response for
\<command\>" — that is the harness telling you to regenerate, not a real bug.

This harness is test scaffolding only. It is never imported by `src/main.ts`
or bundled into the desktop build.

## Seed content and provenance

`seed/starter-pack.json` is the taxonomy only (tools, tasks, categories, tags,
sources) — no commands. `seed/packs/*.json` hold the actual commands, each
sourced from the official docs/repo listed in that pack's `sources[]` (Claude
Code from `code.claude.com`, Codex CLI and OpenCode and Gemini CLI from their
GitHub READMEs, Git from `git-scm.com/docs`). Every seeded item's
`verificationStatus` is either `official` (has a real official source
attached — enforced by pack validation) or `demo` for content that hasn't been
verified yet. `pw_core::seed::ensure_seeded` installs the taxonomy pack first,
then each content pack, through the *same* `stage → approve → apply` pipeline
a human uses — so the seed step exercises the real pipeline, not a shortcut.

**Antigravity, VS Code, and OpenRouter ship with zero items** — the taxonomy
(tool record, sources) is seeded, but their command content was not
researched in this pass. See "Coverage report" below.

## Adding a new tool family with real content

1. Add a `sources` entry (and a `tools` entry, if it's new) to a pack.
2. Research the official docs; never transcribe a screenshot's command text.
3. Write items with a namespaced `externalKey` (`tool-slug:item-slug`),
   `verificationStatus: "official"` + a `sources[]` entry pointing at that
   pack's source `key`, for anything you can actually verify.
4. `cargo run -p pw-cli -- validate seed/packs/your-pack.json`
5. Either add it to `pw_core::seed::STARTER_PACKS` (ships with fresh installs)
   or hand it to a user via `personal-wiki import your-pack.json --stage`
   followed by review in the Update Center.
6. Regenerate the Playwright fixture (see above) if you want E2E coverage of
   the new content.

## Known limitations (see completion report for the full list)

- Linux dev container has no `webkit2gtk` runtime, so the packaged app cannot
  be launched headfully here — verification is Playwright-against-fixture plus
  a real Windows CI build. See the completion report for exactly what that
  does and does not prove.
- Antigravity / VS Code / OpenRouter have no seeded commands yet.
- Code signing is not configured (no certificate available); the NSIS/MSI
  bundles are unsigned. Windows will show an "unknown publisher" prompt.
