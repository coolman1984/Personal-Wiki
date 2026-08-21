# PERSONAL WIKI — ONE-SHOT BUILD PROMPT

You are the senior engineer responsible for building **Personal Wiki** into a production-quality Windows 11 desktop application.

Your job is not to create a demo, static mockup, or hard-coded catalogue. Build the actual extensible product.

## Files you must read before coding

1. `PERSONAL_WIKI_MASTER_GUIDE.md` — source of truth for product logic and architecture.
2. `docs/UI_SPEC.md` — source of truth for the interface: every screen, region, component, state, and the query behind it.
3. Every image in `references/` — visual/UX references:
   - `01_HOME_DASHBOARD.png`
   - `02_TOOL_FAMILY_CODEX.png`
   - `03_COMMAND_DETAIL.png`
   - `04_GLOBAL_SEARCH.png`
   - `05_UPDATE_CENTER.png`
   - `06_GENERIC_CATEGORY_DETAIL.png`

Treat image layout and UX as reference. **Do not trust command text shown in screenshots as factual syntax.** Verify real commands from official sources before storing them as official data.

`docs/UI_SPEC.md` translates those images into a buildable contract — read it before writing any component, and follow its component inventory instead of inventing page-local variants.

## The product in one sentence

Build a lightweight, beautiful, local-first Personal Wiki where a non-technical Windows user can discover commands and technical knowledge by **what they want to do**, browse by tool, search instantly, save notes/links, and safely update the knowledge base without rebuilding the application.

## Critical architecture rule

The executable is stable. The knowledge is dynamic.

Never hard-code Claude Code, Codex CLI, Gemini CLI, OpenCode, categories, command buttons, or pages into UI logic.

The UI must render from structured SQLite data.

If tomorrow a Knowledge Pack adds:

```text
Tool: PowerShell
Category: Automation
Subcategory: Scheduled Tasks
Items: 40
```

the app must display the tool card, category branch, subcategory, item cards, search results, and detail views after data refresh **without changing or rebuilding application code**.

A rebuild is only for actual executable behavior/schema changes.

## Required stack

Use:

- Tauri 2.x
- Svelte 5
- TypeScript strict mode
- SQLite
- SQLite FTS5
- Rust stable MSVC
- pnpm
- Vitest
- Playwright

Use a current supported Node.js LTS.

Do not add Electron, a permanent local HTTP server, Docker runtime, microservices, or a cloud backend unless a concrete requirement makes it unavoidable. This is a lightweight Windows desktop application.

## Product behavior

The user must have two discovery paths:

### Task-first — primary mental model

Examples:

```text
Login & authentication
Agents & sub-agents
Memory & context
Project setup
Prompting
Search
Debugging
Testing
Git workflow
Automation
Updates
Troubleshooting
```

### Tool-first

Examples:

```text
Claude Code
Codex CLI
OpenCode
Gemini CLI
Antigravity
Git / GitHub
VS Code
OpenRouter
Useful Links
```

One item may belong to several tasks/categories.

## Required screens

Implement the complete navigation and reusable screen system specified in `docs/UI_SPEC.md` §5–§11 and represented by the reference images:

- Home / global dashboard
- Task browser
- Tool browser
- Tool-family page
- Generic category page
- Knowledge item detail
- Global search
- Favorites
- Notes
- Useful links
- Update center
- Settings
- Diagnostics

## Knowledge item detail

A command/concept should be understandable to a non-technical user.

Support:

- title
- tool
- type
- short purpose
- full explanation
- when to use
- syntax
- parameters/options
- example
- expected result
- warnings
- official source
- verification status
- related items
- last verified
- copy
- favorite
- pin/open source

Hide irrelevant sections automatically based on data.

## Search

Use FTS5 for V1.

Search across:

- title
- aliases
- descriptions
- task phrases
- tags
- syntax
- examples
- tool
- category

Rank exact title/alias and syntax matches strongly.

Support filters for tool, task, item type, tag, verification state, favorite, and lifecycle state.

Do not require embeddings or an LLM for V1.

## Safe update architecture

Build a versioned **Knowledge Pack JSON contract** and JSON Schema.

External AI agents should update Personal Wiki through:

```text
research official sources
→ generate Knowledge Pack
→ validate
→ import to staging
→ show diff
→ human approve/reject
→ transactional apply
```

Do not allow an external agent to directly mutate the trusted SQLite database by default.

Support source provenance and update history.

Never invent “official” commands.

## CLI for AI-agent interoperability

Build a `personal-wiki` CLI sharing domain/database logic with the desktop app.

Minimum commands:

```text
personal-wiki status
personal-wiki search "<query>"
personal-wiki tools list
personal-wiki sources list
personal-wiki sources manifest --json
personal-wiki validate <pack.json>
personal-wiki import <pack.json> --stage
personal-wiki changes list
personal-wiki changes diff <batch-id>
personal-wiki backup
personal-wiki export <output.json>
personal-wiki doctor
```

Staging is the safe default.

## Data safety

Use:

- database migrations
- foreign keys
- transactions
- parameterized SQL
- backup before applying update batches
- rollback on failure
- audit/update history

Personal notes and favorites must never be overwritten by vendor/source refreshes.

## Security

Use Tauri least-privilege capabilities.

Do not expose:

- arbitrary shell execution
- unrestricted filesystem access
- automatic execution of copied/imported commands
- arbitrary remote HTML

Imported content is untrusted input until validated.

## Visual requirements

Match the reference images closely in structure and quality, and follow `docs/UI_SPEC.md` for tokens, metrics, RTL rules, and per-screen anatomy.

Light theme only.

Target:

- premium Windows 11 feel
- modern 2026 UI
- Arabic RTL done correctly
- elegant whitespace
- rounded cards
- restrained pastel accents
- clean typography
- subtle shadows
- scalable layout
- no visual clutter

Use reusable components and design tokens.

Do not simply paste the screenshot as an image. Build the actual interactive UI.

## Accuracy policy for seed data

Do not populate fake commands just to make the interface look full.

Either:

- verify current commands from official sources and store provenance, or
- mark sample rows explicitly as demo content.

Accuracy beats quantity.

## How to execute

Work autonomously.

Do not stop after planning.

Do not repeatedly ask for confirmation.

If something is ambiguous but non-blocking, choose the simplest solution compatible with the master guide, document the assumption, and continue.

Before editing, inspect the current repository if one exists.

Keep implementation coherent. Do not create unnecessary architecture.

Use subagents only when work is independent/parallel or benefits from isolated context. Do not spawn subagents for trivial edits or sequential shared-state work.

If temporary scripts/files are created during implementation, remove them when finished.

## Model-specific guidance

If you are **Claude Opus 5 High**:

- take ownership of architecture and hard cross-cutting decisions
- use your verification ability naturally
- avoid unnecessary subagent spawning
- avoid overengineering
- do not repeatedly re-verify settled trivial details

If you are **Claude Sonnet 5 High**:

- execute the architecture faithfully
- use the acceptance tests aggressively
- keep a compact progress checklist
- do not declare completion until the end-to-end flows and release build actually pass

If both models are available, a good workflow is:

```text
Opus 5 High: architecture/review
Sonnet 5 High: implementation/tests
Opus 5 High: final gap audit
Sonnet 5 High: final fixes
```

But the project must still finish if only one model is available.

## Required validation before completion

Prove all of these:

1. Adding a new tool record makes a new tool card appear without code change.
2. Adding a nested category makes navigation update without code change.
3. Importing a new item through a Knowledge Pack creates a staged diff.
4. Approved item appears in browsing and search without rebuild.
5. One task can show commands from several tools.
6. Natural-language aliases work.
7. Offline browse/search/favorites/notes work.
8. Malformed packs cannot corrupt trusted data.
9. Failed apply rolls back.
10. Personal notes survive official-content updates.
11. Major reference screens are visually implemented, not placeholders.
12. Windows release build launches successfully.

## Final deliverables

Do not finish until the repository contains at least:

```text
working source code
database migrations
Knowledge Pack JSON Schema
seed/demo data
tests
reference images
README for non-technical user
developer notes
Windows release build or installer
CLI
```

At the end, give a concise completion report:

- architecture implemented
- major screens
- tests executed and results
- release artifact path
- how to launch
- any remaining limitation

The non-technical user should ultimately be able to install/extract, double-click Personal Wiki, and use it.

## Final principle

**Never confuse growing the wiki with rebuilding the app.**

Knowledge growth changes structured data.
Application feature growth changes code.

Build Personal Wiki accordingly.
