# Personal Wiki

A local-first Windows 11 desktop knowledge base for command-line and AI development
tools — Claude Code, Codex CLI, Gemini CLI, OpenCode, Git, VS Code, and anything
added later — organised by **what you want to do**, not by which manual it lives in.

This repository currently holds the **build contract**: the specification a coding
agent implements from. The application source is built against these documents.

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

## How to start a build

1. Open `START_HERE_PROMPT.md` and give its contents to the coding agent.
2. Make sure the agent can read this whole repository, including `references/`.
3. The agent reads `PERSONAL_WIKI_MASTER_GUIDE.md`, then `docs/UI_SPEC.md`, then builds.

## Target stack

Tauri 2 · Svelte 5 · TypeScript (strict) · SQLite + FTS5 · Rust (MSVC) · pnpm ·
Vitest · Playwright. Light theme only. Arabic RTL first, localisation-ready.

## Safety model

External AI agents never write to the trusted database directly. They produce a
validated **Knowledge Pack**, which is staged, diffed, reviewed by a human, and
then applied inside a transaction with a backup taken first. Personal notes and
favourites always survive a source refresh.
