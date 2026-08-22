# Personal Wiki

A shared Arabic-first knowledge application for desktop and mobile web. This
repository now contains both the original product contract and the working web
application, so ChatGPT and Claude work on exactly the same source.

The current web build is functional but still contains demo content and navigation
shortcuts that must be repaired. The authoritative implementation plan is
`docs/LUNA_MAX_PERSONAL_WIKI_REPAIR_PLAN.md`.

## The one rule everything follows

> The executable is stable. The knowledge is dynamic.

Adding a tool, a category, a command, an example, or a link changes **data**.
It never requires rebuilding the app. Only changes to application behaviour or
database schema produce a new build.

## Contents

| File | What it is |
| --- | --- |
| `START_HERE_PROMPT.md` | The one-shot implementation prompt. Send this to the coding agent. |
| `PERSONAL_WIKI_MASTER_GUIDE.md` | Product logic, architecture, data model, update model, CLI, acceptance tests. |
| `docs/UI_SPEC.md` | The interface contract: shell, tokens, component inventory, every screen region by region, states, and the query behind each element. |
| `CLAUDE.md` | Persistent project rules, loaded automatically by Claude Code. |
| `references/` | Six UI reference images (see below). |
| `docs/BUILD_PACK_README.md` | Original build-pack notes. |
| `docs/LUNA_MAX_PERSONAL_WIKI_REPAIR_PLAN.md` | Current audited repair plan and completion gates. |
| `docs/WEBAPP_README.md` | Web application setup, build, and hosting notes. |
| `app/`, `server/`, `db/` | Current web interface, server logic, and data model. |
| `desktop/` | A separate Tauri + Rust + Svelte desktop implementation of the same product contract — its own README/build/CI, not part of this web build. |

## Reference images

| Image | Screen |
| --- | --- |
| `01_HOME_DASHBOARD.png` | Home — the outermost face of the app: hero, tool grid, activity, quick actions. |
| `02_TOOL_FAMILY_CODEX.png` | Inside one tool: identity header, stat strip, section tabs, command cards. |
| `03_COMMAND_DETAIL.png` | One command in full: purpose, when to use, parameters, example, expected result, source, related. |
| `04_GLOBAL_SEARCH.png` | Cross-tool search by intent, with filters, ranking, suggestions, saved searches. |
| `05_UPDATE_CENTER.png` | Sources, detected changes, diff review, human approval before anything is applied. |
| `06_GENERIC_CATEGORY_DETAIL.png` | The tool-independent task/category view with a pinned detail panel. |

Layout and interaction in these images are the design contract. Command text
inside them is illustrative — verify every command against an official source
before storing it as official.

## How an agent starts work

1. Read `CLAUDE.md`.
2. Read `docs/LUNA_MAX_PERSONAL_WIKI_REPAIR_PLAN.md` completely.
3. Inspect the current web app and run its existing checks before changing code.
4. Implement the repair phases in order and do not deploy without a separate user request.

## Current web stack

Vinext/React with TypeScript, D1 data storage, R2 backup storage, and ChatGPT
authentication.

## Desktop implementation

A working Tauri + Rust + Svelte desktop build now exists in `desktop/` — see
`desktop/README.md`. It shares this repository's product contract
(`PERSONAL_WIKI_MASTER_GUIDE.md`, `docs/UI_SPEC.md`) and Knowledge Pack model,
but is a fully separate codebase, dependency tree, and CI pipeline from the
web app above; changing one does not require changing the other.

## Safety model

External AI agents never write to the trusted database directly. They produce a
validated **Knowledge Pack**, which is staged, diffed, reviewed by a human, and
then applied inside a transaction with a backup taken first. Personal notes and
favourites always survive a source refresh.
