# Personal Wiki

A local-first Windows 11 desktop knowledge engine for command-line and AI
development tools — Claude Code, Codex CLI, Gemini CLI, OpenCode, Git, VS Code,
and anything added later — organised by **what you want to do**, not by which
manual it lives in.

> **The executable is stable. The knowledge is dynamic.** Every tool card,
> section tab, category branch, command card, and detail panel is rendered
> from SQLite at query time. Adding a tool, a task, a category, or a command
> is a data change — a Knowledge Pack import, reviewed and approved by a
> human. It never requires rebuilding the app.

## What's here

This repository contains the finished V1 application, not just the design
contract it was built from:

```text
crates/pw-core/     shared Rust domain core — SQLite, migrations, FTS5 search,
                     Knowledge Pack validation, staging/diff, transactional
                     apply, backup/restore, audit, diagnostics
crates/pw-cli/       the `personal-wiki` CLI — a thin shell over pw-core,
                     built for AI agents to research and propose updates
src-tauri/           the Tauri 2 desktop shell — typed commands, also a thin
                     shell over pw-core
src/                 the Svelte 5 + TypeScript frontend
schemas/             the Knowledge Pack JSON Schema
seed/                starter taxonomy + source-backed content (Claude Code,
                     Codex CLI, Gemini CLI, OpenCode, Git — real commands
                     from official docs, with provenance)
tests/               Playwright E2E suite + Vitest unit tests
crates/*/tests/      Rust integration and CLI end-to-end tests
.github/workflows/   Windows CI: build, test, produce the installer
docs/                the build contract, UI spec, developer notes, user guide
references/          the six original UI reference images
```

## For the non-technical user

See `docs/USER_README.md` — install steps, first run, and how the update
review flow works, in Arabic.

## For a developer continuing this work

See `docs/DEVELOPER_NOTES.md` for the actual architecture as built: why there
is one Rust core behind two shells, how the update pipeline is wired end to
end, how the Playwright suite is driven against a recording of the *real*
core (not a second implementation), and how to add a new tool family with
verified content.

## The original design contract

The documents this application was built from are still here, and remain the
source of truth for product decisions and screen contracts:

| File | What it is |
| --- | --- |
| `START_HERE_PROMPT.md` | The original one-shot implementation prompt. |
| `PERSONAL_WIKI_MASTER_GUIDE.md` | Product logic, architecture, data model, update model, CLI, acceptance tests. |
| `docs/UI_SPEC.md` | The interface contract: shell, tokens, component inventory, every screen region by region. |
| `CLAUDE.md` | Persistent project rules, loaded automatically by Claude Code. |
| `references/` | Six UI reference images — layout and interaction are the design contract; command text inside them is illustrative only. |

## Target stack

Tauri 2 · Svelte 5 · TypeScript (strict) · SQLite + FTS5 · Rust (stable MSVC) ·
pnpm · Vitest · Playwright. Light theme only. Arabic RTL first,
localisation-ready.

## Building it yourself

```bash
pnpm install
cargo build --workspace          # core + CLI + desktop app
pnpm build:only && pnpm check    # frontend build + typecheck
cargo test --workspace --exclude personal-wiki-app
pnpm test                        # Vitest unit tests
pnpm exec playwright test        # E2E (needs `pnpm dev` running)
pnpm tauri dev                   # the desktop app itself (needs a WebView2/webkit2gtk runtime)
pnpm tauri build                 # release installer (NSIS + MSI on Windows)
```

The `personal-wiki` CLI (`target/debug/personal-wiki`) works standalone:

```bash
personal-wiki status
personal-wiki search "تشغيل وكيل في الخلفية"
personal-wiki sources manifest --json      # what a research agent should check
personal-wiki validate pack.json
personal-wiki import pack.json --stage     # staging is always the default
personal-wiki changes diff <batch-id>
personal-wiki changes approve <batch-id>
personal-wiki changes apply <batch-id>
personal-wiki doctor
```

## Safety model

External AI agents never write to the trusted database directly. They produce
a validated **Knowledge Pack** (`schemas/knowledge-pack.schema.json`), which is
staged, diffed, reviewed by a human in the Update Center, and then applied
inside a transaction with a backup taken first. Personal notes, favourites,
and pins always survive a source refresh — that is enforced by the schema
(the personal overlay lives in tables a pack apply never touches) and proven
by an automated test.
