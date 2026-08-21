# PERSONAL WIKI — MASTER BUILD GUIDE

**Version:** 1.0  
**Target:** Windows 11 desktop application  
**Primary user:** Non-technical power user  
**Visual direction:** Premium light theme only  
**Core architecture rule:** **The application is stable; the knowledge is dynamic.**

---

## 0. READ THIS FIRST

This document is the source of truth for the Personal Wiki project.

Personal Wiki is **not** a static collection of hard-coded pages, buttons, or commands. It is a small, fast desktop knowledge engine that renders its interface from structured local data. New tools, task groups, categories, subcategories, commands, explanations, examples, links, tags, and relationships must appear automatically after data changes, without rebuilding the application.

The attached images define the intended UX language. Treat their **layout, hierarchy, density, interaction patterns, light-theme styling, and information architecture** as design references. Any command text visible inside an image is illustrative until verified against an official source.

### Reference images

- `references/01_HOME_DASHBOARD.png` — first screen / global home.
- `references/02_TOOL_FAMILY_CODEX.png` — inside one tool family.
- `references/03_COMMAND_DETAIL.png` — one knowledge item in full detail.
- `references/04_GLOBAL_SEARCH.png` — cross-tool smart search.
- `references/05_UPDATE_CENTER.png` — source sync, detected changes, approval.
- `references/06_GENERIC_CATEGORY_DETAIL.png` — generic category/card/detail pattern.

---

# 1. PRODUCT VISION

Personal Wiki is a **local-first personal operating memory for technical tools**.

The user should not need to remember slash commands, CLI syntax, flags, configuration names, useful links, extension names, setup steps, or which product contains a capability.

The user thinks in outcomes:

- “Create a sub-agent.”
- “Log in.”
- “Resume a session.”
- “Run work in the background.”
- “Search this repository.”
- “Save this useful link.”
- “Compare how Claude Code, Codex CLI, Gemini CLI, and OpenCode do the same task.”

The application maps that human intent to trusted knowledge.

The deep product principle is:

> **Organize primarily by what the user wants to achieve, while also allowing browsing by tool.**

Therefore Personal Wiki has two simultaneous navigation dimensions.

### Task-first

Examples:

- install and setup
- login and authentication
- sessions
- agents and sub-agents
- memory and context
- project instructions
- prompting
- code generation
- refactoring
- debugging
- testing
- search
- Git workflow
- model selection
- permissions
- automation
- background work
- extensions
- updates
- troubleshooting

### Tool-first

Examples:

- Claude Code
- Codex CLI
- OpenCode
- Gemini CLI
- Antigravity
- Git / GitHub
- VS Code
- OpenRouter
- PowerShell
- Python
- Useful Links

A single knowledge item can appear in several task groups and categories without duplicating the underlying record.

---

# 2. NON-NEGOTIABLE PRODUCT RULES

## 2.1 Dynamic UI, never hard-coded content

Never create one component per tool or one hard-coded button per category.

Wrong:

```ts
if (tool === "Codex") showCodexButton();
if (tool === "Claude") showClaudeButton();
```

Correct:

```ts
const tools = await repository.listActiveTools();
tools.forEach(tool => renderToolCard(tool));
```

The same rule applies to:

- tools
- tasks
- categories
- subcategories
- commands
- examples
- links
- notes
- tags
- related items
- update badges
- source records
- favorites
- recent items

Adding a row to the data layer must be enough for the UI to render the new element.

## 2.2 Content updates must not require rebuilding the app

No rebuild for:

- adding a tool
- adding a task
- adding a category or nested category
- adding commands
- changing explanations
- adding examples
- changing icons or accent keys
- adding links
- changing relationships
- adding tags
- importing new documentation
- deprecating an item

A new executable build is needed only when changing application behavior, native capabilities, code, or database schema through a migration.

## 2.3 Local-first

The core app works without a server.

Offline, the user must still be able to:

- launch the app
- browse existing knowledge
- search
- view examples
- copy commands
- add/edit personal notes and links
- use favorites and history
- import a local knowledge pack

Internet is optional and used for source synchronization or opening web sources.

## 2.4 No mandatory AI/API dependency

The core product must not require an LLM key.

External AI agents may research and propose updates by producing a validated Knowledge Pack. Personal Wiki remains the deterministic storage, search, review, provenance, and presentation layer.

Future AI “Ask my Wiki” features may be added later as optional modules.

## 2.5 Never invent tool commands

A command is not trusted simply because an AI model generated it.

Externally sourced knowledge should store:

- source URL
- source title
- source type
- verification status
- last verified date
- source hash/version when available
- lifecycle state

Possible verification states:

- official
- verified
- community
- unverified

Possible lifecycle states:

- active
- experimental
- deprecated
- removed

If a command cannot be verified, never label it official.

## 2.6 Explain for a non-technical human

Each command/concept should answer:

- What is it?
- What does it do?
- When should I use it?
- How do I use it?
- What is a simple example?
- What result should I expect?
- Is there any warning?
- Where did this information come from?

---

# 3. RECOMMENDED TECHNOLOGY STACK

## Desktop shell — Tauri 2.x

Use Tauri as the lightweight Windows desktop shell.

Why:

- small native desktop footprint
- Windows 11 friendly
- WebView2 rendering already present on modern Windows
- Rust backend where native access is needed
- explicit security/capability model
- official SQL and updater plugins
- no requirement to run a separate local web server

Important mental model:

> **Tauri is the shell. The knowledge is data.**

Tauri does not need to be rebuilt whenever the knowledge changes.

## Frontend — Svelte 5 + TypeScript

Use Svelte 5 with TypeScript.

Why:

- fast, responsive desktop UI
- compact runtime
- excellent for data-driven rendering
- easy reusable component model
- TypeScript catches data-shape mistakes

Avoid a massive UI framework unless it materially improves speed of delivery. Prefer a small reusable design system implemented with components and CSS tokens.

## Styling — CSS variables + scoped styles

Use one intentional visual direction:

> **Light theme only. Premium, calm, elegant, modern 2026.**

Use CSS design tokens instead of random colors embedded throughout components.

## Database — SQLite

SQLite is the durable local source of truth.

Use:

- relational tables
- foreign keys
- migrations
- indexes
- JSON only where flexibility genuinely helps
- FTS5 for search

The knowledge base should remain portable and local.

## Search — SQLite FTS5

Index at least:

- titles
- aliases
- short descriptions
- full explanations
- task phrases
- tags
- syntax
- examples
- tool names
- category names

Exact title/alias matches rank above broad body matches.

## Development runtime

Use a current supported Node.js LTS. For the initial build in August 2026, Node 24 LTS is a sensible baseline unless the generated project pins another compatible LTS.

Use `pnpm` and commit the lockfile.

Use current stable Rust MSVC compatible with the selected Tauri 2 release.

## Tests

Use:

- Vitest — unit tests
- Playwright — UI / end-to-end tests
- Rust tests — native logic if custom Rust commands exist
- SQLite migration/integrity tests
- visual/screenshot checks for major screens when practical

---

# 4. HIGH-LEVEL ARCHITECTURE

```text
┌──────────────────────────────────────────────────────────────┐
│                       PERSONAL WIKI                          │
├──────────────────────────────────────────────────────────────┤
│ Svelte 5 UI                                                  │
│ Home | Tasks | Tools | Search | Item Detail                  │
│ Favorites | Links | Notes | Updates | Settings               │
├──────────────────────────────────────────────────────────────┤
│ Application Services                                         │
│ Navigation | Knowledge | Search | Import/Export               │
│ Update Review | Sources | Favorites | History                 │
├──────────────────────────────────────────────────────────────┤
│ SQLite                                                       │
│ tools / tasks / categories / items / examples / sources      │
│ tags / relationships / favorites / history / staged changes  │
│ FTS5                                                         │
├──────────────────────────────────────────────────────────────┤
│ Tauri 2 / Rust                                               │
│ App paths | Files | Clipboard | Safe URL open | HTTP sync     │
│ Updater | Logging                                            │
└──────────────────────────────────────────────────────────────┘

External AI research/coding agent
        │
        │ produces validated Knowledge Pack JSON
        ▼
Validate → Stage → Diff → Human Review → Approve → SQLite
```

AI agents may research and propose knowledge. They must not silently overwrite trusted data.

---

# 5. INFORMATION ARCHITECTURE

## 5.1 Tool families

A tool family is a record, not a page coded by hand.

Example:

```text
Claude Code
Codex CLI
OpenCode
Gemini CLI
Antigravity
Git / GitHub
VS Code
OpenRouter
PowerShell
Python
Useful Links
```

Adding another tool in the future must require no UI code.

## 5.2 Tasks

Tasks represent the user's goal.

Example:

```text
Agents & sub-agents
Authentication
Memory & context
Project setup
Debugging
Testing
Git workflow
Automation
Troubleshooting
```

A task connects to items from many tools.

## 5.3 Categories and subcategories

Categories form an arbitrary tree using a nullable `parent_id`.

Do not restrict the system to exactly two levels.

Example:

```text
Agents
├─ Create agents
├─ Manage agents
├─ Background agents
└─ Parallel execution
```

## 5.4 Knowledge item

A generic item may be:

- command
- slash command
- CLI command
- setting
- shortcut
- workflow
- concept
- link
- extension
- note
- snippet
- troubleshooting fix

Do not create a separate storage model for every tiny content type unless behavior is genuinely different.

---

# 6. CORE DATA MODEL

Use stable UUIDs or ULIDs.

## `tools`

```text
id
slug
name
short_name
description
icon_key
accent_key
official_home_url
sort_order
is_active
created_at
updated_at
```

## `tasks`

```text
id
slug
name
description
icon_key
sort_order
is_active
```

## `categories`

```text
id
slug
name
description
icon_key
parent_id nullable
sort_order
is_active
created_at
updated_at
```

## `items`

```text
id
tool_id nullable
item_type
slug
title
subtitle
short_description
full_description
syntax nullable
when_to_use nullable
expected_result nullable
warning_text nullable
verification_status
lifecycle_status
difficulty nullable
icon_key nullable
sort_order
is_active
created_at
updated_at
last_verified_at nullable
```

Suggested `item_type` values:

```text
command
slash_command
cli_command
setting
shortcut
workflow
link
extension
concept
note
snippet
troubleshooting
```

## `item_tasks`

```text
item_id
task_id
```

## `item_categories`

```text
item_id
category_id
```

## `tags`

```text
id
slug
name
```

## `item_tags`

```text
item_id
tag_id
```

## `aliases`

```text
id
item_id
alias
```

Aliases are important for natural-language discovery.

## `examples`

```text
id
item_id
title
description
code
expected_output nullable
sort_order
```

## `parameters`

```text
id
item_id
name
description
is_required
default_value nullable
value_hint nullable
sort_order
```

## `sources`

```text
id
tool_id nullable
name
source_type
url
is_official
sync_strategy
is_active
last_checked_at nullable
last_success_at nullable
etag nullable
last_modified nullable
content_hash nullable
notes nullable
```

Suggested `source_type`:

```text
official_docs
official_repo
official_changelog
official_blog
community
local_file
manual
```

## `item_sources`

```text
item_id
source_id
source_url
source_anchor nullable
source_version nullable
source_hash nullable
verified_at
```

## `relationships`

```text
id
from_item_id
to_item_id
relationship_type
```

Suggested relationship types:

```text
related
alternative
prerequisite
replaced_by
similar_to
follows
part_of
```

## `favorites`

```text
item_id
created_at
```

## `recent_items`

```text
item_id
opened_at
```

## `notes`

```text
id
item_id nullable
title
body
created_at
updated_at
```

## `saved_links`

```text
id
title
url
description
tool_id nullable
category_id nullable
is_favorite
created_at
updated_at
```

## `update_batches`

```text
id
source_id nullable
created_by
summary
created_at
applied_at nullable
status
```

## `staged_changes`

```text
id
batch_id
entity_type
entity_key
change_type
before_json nullable
after_json nullable
source_id nullable
confidence nullable
status
detected_at
reviewed_at nullable
```

Possible `change_type`:

```text
add
update
deprecate
remove
relink
```

Possible `status`:

```text
pending
approved
rejected
applied
failed
```

---

# 7. DYNAMIC UI ENGINE

The UI is a renderer of data.

## Home

Query active tools:

```sql
SELECT *
FROM tools
WHERE is_active = 1
ORDER BY sort_order, name;
```

Render a reusable `ToolCard` for each tool.

If an agent imports a new `PowerShell` tool, the next refresh automatically shows it.

## Tool page

Given `tool_id`, query linked task/category branches and items.

Never write:

```ts
const codexTabs = [...]
```

Write generic logic such as:

```ts
const sections = await knowledge.getSectionsForTool(toolId);
```

## Item detail

Use one reusable `KnowledgeDetail` component.

Sections appear only when relevant data exists.

A link may have no syntax. A command may have parameters. A concept may have no code block. The component adapts from the record.

## Tree navigation

Build the category tree from `parent_id` at runtime.

## Icon system

Store `icon_key`, not executable component code.

Safe examples:

```text
terminal
robot
brain
folder
link
shield
key
git
code
search
book
settings
```

Unknown keys fall back to a generic icon.

## Accent system

Store semantic accent keys:

```text
blue
green
purple
orange
teal
```

Map them to actual light-theme design tokens.

---

# 8. SCREEN MAP

## Screen A — Global Home

Reference: `references/01_HOME_DASHBOARD.png`

Purpose: first screen after launch.

Include:

- global search
- task-first shortcuts
- tool family cards
- command/example counts
- recent updates
- recently used items
- quick actions
- sync status
- favorites

The layout must remain elegant whether there are 5 tools or 50.

## Screen B — Tool Family

Reference: `references/02_TOOL_FAMILY_CODEX.png`

Purpose: all knowledge belonging to one tool.

Include:

- breadcrumbs
- tool identity
- summary statistics
- task/category navigation
- item count
- item cards/list
- sort/filter
- source freshness

No tool-specific hard-coded screen.

## Screen C — Item Detail

Reference: `references/03_COMMAND_DETAIL.png`

Include:

- title
- tool badge
- type
- purpose
- when to use
- syntax
- parameters/options
- example
- expected result
- warnings
- official source
- related items
- last verified
- copy
- favorite
- pin
- open source

## Screen D — Global Search

Reference: `references/04_GLOBAL_SEARCH.png`

Search across all tools by human intent.

Example queries:

```text
create sub agent
login
resume session
check changes
run in background
find command for testing
```

Filters:

- tool
- type
- task
- tag
- source/verification
- favorite
- lifecycle state

## Screen E — Update Center

Reference: `references/05_UPDATE_CENTER.png`

Include:

- connected sources
- source health
- last checked
- detected changes
- diff preview
- approval queue
- approve/reject
- apply
- sync history
- rollback information

## Screen F — Generic Task/Category

Reference: `references/06_GENERIC_CATEGORY_DETAIL.png`

Use the same reusable cards/details when browsing a task independent of one specific tool.

---

# 9. GLOBAL SEARCH

The search bar is the most important interaction.

V1 should be deterministic and fast:

1. normalize query
2. run FTS5
3. boost exact title/alias
4. boost syntax match
5. boost task match
6. boost tool match
7. use favorite/recent as a light tie-breaker
8. penalize deprecated/removed
9. return quickly

Do not require embeddings in V1.

Optional future hybrid semantic search may be added without replacing FTS5.

Example aliases:

```text
Item: Create sub-agent

Aliases:
delegate task
create worker
spawn agent
subagent
parallel worker
```

Aliases live in data, not code.

---

# 10. KNOWLEDGE UPDATE MODEL

The user wants any capable desktop coding/research agent to update Personal Wiki.

The safest architecture is a **Knowledge Pack import contract**.

Workflow:

```text
Agent reads source manifest
        ↓
Agent researches official sources
        ↓
Agent extracts additions/changes/removals
        ↓
Agent creates Knowledge Pack JSON
        ↓
JSON Schema validation
        ↓
Import into staging
        ↓
Diff against trusted DB
        ↓
Human review
        ↓
Approved changes applied transactionally
```

This avoids giving an AI agent unrestricted direct database mutation.

## Example Knowledge Pack

```json
{
  "schema_version": "1.0",
  "batch": {
    "producer": "external-agent",
    "generated_at": "2026-08-21T12:00:00Z",
    "notes": "Official Codex CLI documentation refresh"
  },
  "tools": [
    {
      "slug": "codex-cli",
      "name": "Codex CLI",
      "description": "..."
    }
  ],
  "items": [
    {
      "external_key": "codex-cli:verified-command",
      "tool_slug": "codex-cli",
      "item_type": "cli_command",
      "title": "Verified command",
      "syntax": "verified syntax",
      "short_description": "...",
      "full_description": "...",
      "when_to_use": "...",
      "verification_status": "official",
      "tasks": ["agents"],
      "categories": ["agents/sub-agents"],
      "tags": ["agent"],
      "sources": [
        {
          "url": "https://official.example/docs/...",
          "title": "Official documentation",
          "is_official": true,
          "verified_at": "2026-08-21T12:00:00Z"
        }
      ]
    }
  ]
}
```

The repository must contain a formal JSON Schema.

---

# 11. PERSONAL WIKI CLI

Provide a safe CLI so external agents can interact with the knowledge system without editing SQLite by hand.

Recommended interface:

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

Controlled mutation commands may exist:

```text
personal-wiki changes approve <batch-id>
personal-wiki changes reject <batch-id>
personal-wiki changes apply <batch-id>
```

Default external-agent behavior is **stage**, not silent apply.

Prefer a Rust CLI binary sharing domain/database code with the Tauri app so business logic is not duplicated.

---

# 12. SOURCE STRATEGY

Supported sources may include:

- official documentation
- official GitHub repositories
- official changelogs
- official release notes
- official vendor blogs
- local Markdown
- manually saved URLs
- imported JSON packs

Do not create a brittle universal scraper in V1.

Preferred source strategy:

1. structured official feed/API when available
2. official repository files
3. stable official documentation
4. external AI agent research producing a pack
5. manual import

Each source stores its own `sync_strategy`.

---

# 13. CHANGE DETECTION AND APPROVAL

Every externally managed item needs a stable external identity.

Example:

```text
external_key = tool_slug + ":" + canonical_item_slug
```

Compare normalized incoming data to trusted data.

Produce:

- add
- update
- deprecate
- remove

Never hard-delete something merely because it disappeared from one crawl.

Removal policy:

```text
candidate removed
→ source evidence
→ human approval
→ preserve history
```

Prefer deprecation over destructive deletion.

Apply changes transactionally:

```text
BEGIN TRANSACTION
create backup metadata
apply approved changes
rebuild affected FTS rows
write audit record
COMMIT
```

On failure:

```text
ROLLBACK
```

---

# 14. BACKUP, HISTORY, AND RECOVERY

Before applying an update batch:

- create a backup/snapshot
- record the batch ID
- retain before/after data
- preserve provenance

Support:

- undo last applied knowledge batch
- restore previous snapshot
- export full knowledge base
- backup SQLite

Personal notes/favorites must survive vendor-content refreshes.

Treat the visible knowledge as:

```text
trusted imported content
+
personal overlay
=
user experience
```

---

# 15. PERSONAL CONTENT

The app is more than vendor documentation.

The user can save:

- notes
- useful links
- snippets
- preferred workflows
- bookmarks
- personal examples
- custom categories
- favorite commands
- “how I use this” notes

Imported source updates must never overwrite personal annotations.

---

# 16. EDITING EXPERIENCE

V1 needs simple structured editing, not a giant Notion clone.

Allow:

- add/edit tool
- add/edit task
- add/edit category
- add/edit item
- add link
- add note
- add example
- add alias
- add personal annotation
- reorder when useful
- archive/deactivate

Use typed forms and validation.

---

# 17. DESIGN SYSTEM

## Theme

**Light theme only.**

No dark theme in V1.

## Visual feeling

- Windows 11 compatible
- elegant
- modern 2026
- calm
- premium
- soft surfaces
- subtle shadows
- generous whitespace
- rounded corners
- crisp typography
- restrained pastel accents
- information-rich without clutter

## Persistent shell

```text
Top bar
  Global search
  Update
  Sync
  Settings

Left navigation
  Home
  Tasks
  Tools
  Commands
  Links
  Favorites
  Notes
  Updates
  Settings

Main content
  Dynamic route content

Optional right panel
  Contextual details / recent / help / update information
```

## Tokens

Create design variables:

```text
background
surface
surface-muted
border
text-primary
text-secondary
accent-blue
accent-green
accent-purple
accent-orange
success
warning
danger
radius-sm
radius-md
radius-lg
shadow-sm
shadow-md
space-1 ... space-8
```

Never scatter arbitrary hex colors through components.

## RTL

Arabic is the initial visual reference.

Support real RTL using:

- `dir="rtl"`
- CSS logical properties
- layout primitives that work in both directions

Do not “fake RTL” by manually flipping components.

Keep localization architecture ready for English later.

---

# 18. PERFORMANCE

Target a native-feeling desktop experience.

Requirements:

- quick launch
- fast home rendering
- near-instant normal local search
- smooth behavior with thousands of items
- no loading the entire database into memory

Use:

- indexes
- pagination
- virtualization for large lists
- debounced search
- lazy-loaded detail bodies
- cached small reference tables
- parameterized SQL

Do not add a heavy local backend server.

---

# 19. SECURITY

Use Tauri least privilege.

Rules:

- no unrestricted filesystem exposure to frontend
- no arbitrary shell execution from UI
- safe URL opening only
- validate all imports
- parameterized SQL
- never execute imported command text
- command examples are display/copy content only
- sanitize rich content
- do not render arbitrary remote HTML
- do not store external secrets in wiki records
- imported packs are untrusted input
- keep Tauri capabilities explicit and minimal

Personal Wiki is a knowledge application, not a remote command execution console.

---

# 20. TWO DIFFERENT UPDATE TYPES

This distinction is critical.

## Knowledge update

Changes data.

Examples:

- new Codex command
- updated Claude Code option
- new Gemini link
- revised explanation

**No app rebuild.**

## Application update

Changes executable behavior.

Examples:

- new editor
- new native integration
- new database migration
- changed search engine logic

This requires a new build and can use the Tauri updater.

UI should show separately:

```text
Knowledge status
Application version status
```

---

# 21. DATABASE MIGRATIONS

Use numbered migrations.

Example:

```text
001_initial.sql
002_add_aliases.sql
003_add_update_batches.sql
```

Rules:

- never modify an already released migration
- migrate forward
- test clean install
- test upgrade from previous DB
- back up before destructive migration
- fail safely with a useful message

---

# 22. SEED DATA

Ship enough realistic seed content to demonstrate the product.

Do not fill the database with invented “official” commands.

Use:

- verified current commands with sources
- or clearly marked demo content

Accuracy matters more than impressive counts.

---

# 23. IMPORT / EXPORT

Support:

- export full knowledge base to JSON
- export one tool
- export a task/category
- import validated Knowledge Pack
- export source manifest
- backup SQLite
- restore backup

A portable export must not require the user to understand SQLite.

---

# 24. LOGGING AND DIAGNOSTICS

Create local structured logs for application errors.

Never log secrets.

Provide a Diagnostics view or export containing:

- app version
- DB schema version
- DB path
- DB integrity
- item count
- source count
- FTS status
- last update batch
- recent errors

CLI:

```text
personal-wiki doctor
```

should check common problems and return human-readable fixes.

---

# 25. PROJECT STRUCTURE

Suggested structure:

```text
personal-wiki/
├─ src/
│  ├─ app/
│  ├─ components/
│  │  ├─ shell/
│  │  ├─ cards/
│  │  ├─ search/
│  │  ├─ detail/
│  │  ├─ updates/
│  │  └─ forms/
│  ├─ routes/
│  ├─ stores/
│  ├─ services/
│  ├─ domain/
│  ├─ db/
│  ├─ design/
│  └─ utils/
├─ src-tauri/
│  ├─ src/
│  │  ├─ lib.rs
│  │  ├─ commands/
│  │  ├─ services/
│  │  └─ bin/
│  ├─ capabilities/
│  ├─ migrations/
│  └─ tauri.conf.json
├─ schemas/
│  └─ knowledge-pack.schema.json
├─ seed/
├─ tests/
│  ├─ unit/
│  ├─ integration/
│  └─ e2e/
├─ docs/
├─ references/
├─ CLAUDE.md
├─ PERSONAL_WIKI_MASTER_GUIDE.md
├─ package.json
└─ README.md
```

The exact tree may adapt to framework conventions, but preserve separation of concerns.

---

# 26. IMPLEMENTATION MILESTONES

The coding agent should execute continuously, but internally follow these milestones.

### Foundation

- scaffold Tauri + Svelte + TypeScript
- light theme
- SQLite
- migrations
- domain types
- repository/services
- app shell
- seed loader

### Knowledge core

- tools
- tasks
- category tree
- items
- sources
- examples
- tags
- relationships

### Dynamic views

- home
- task browser
- tool browser
- category view
- detail view

### Search

- FTS5
- aliases
- filters
- ranking
- recent/saved searches

### Personal layer

- favorites
- notes
- links
- history

### Update engine

- Knowledge Pack schema
- validation
- staging
- diff
- approval
- transactional apply
- backup

### CLI

- status
- search
- validate
- import
- changes
- export
- backup
- doctor

### Quality

- unit tests
- DB tests
- E2E
- keyboard navigation
- empty/error states
- visual review
- performance pass

### Packaging

- Windows build
- installer
- non-technical README

---

# 27. REQUIRED ACCEPTANCE TESTS

## New tool without rebuild

Add a tool record:

```text
PowerShell
```

Expected:

- appears on Home
- appears in Tools
- page opens
- no source code change

## New nested category without rebuild

Import:

```text
PowerShell
  Automation
    Scheduled Tasks
```

Expected: hierarchy appears automatically.

## New item through Knowledge Pack

Expected:

- appears in staging
- can review diff
- approval applies it
- search finds it
- detail page renders it
- no app rebuild

## Cross-tool task

Create:

```text
Agents & sub-agents
```

Link items from four tool families.

Expected: one task page groups the linked items by tool.

## Natural-language alias search

Search using an alias not present in syntax.

Expected: correct knowledge ranks near top.

## Offline mode

Disconnect internet.

Expected:

- launch works
- browse works
- search works
- notes work
- favorites work

## Malformed pack

Expected:

- validation rejects it
- trusted DB unchanged
- user sees useful error

## Failed apply

Expected:

- transaction rolls back
- DB remains valid

## Personal overlay survives update

Add personal note to an official item, then refresh official item.

Expected: personal note remains.

---

# 28. CODING QUALITY RULES

- TypeScript strict mode.
- Avoid `any`.
- No giant components.
- No duplicated business logic between UI and CLI.
- No hard-coded tool lists.
- No hard-coded category lists.
- No magic database IDs.
- No magic filesystem paths.
- Parameterized SQL.
- Useful error states.
- Reusable domain types.
- Deterministic migrations.
- Tests for critical logic.
- Clean temporary agent files at end.
- Keep abstractions proportional.
- No microservices.
- No Docker runtime dependency.
- No local HTTP server without a proven need.
- No cloud sync in V1 unless explicitly requested.
- No authentication for single-user local V1 unless required.
- Do not “future-proof” by adding complexity nobody asked for.

---

# 29. CLAUDE OPUS 5 HIGH VS CLAUDE SONNET 5 HIGH

This section guides the coding workflow when Claude 5 models are available.

## Claude Opus 5 — High effort

Best use:

- architecture
- difficult cross-cutting decisions
- codebase-wide changes
- root-cause debugging
- final technical review
- UX/architecture reconciliation
- detecting missing requirements
- hard ambiguous tasks

Relevant strengths:

- strong long-horizon agentic coding
- strong codebase understanding
- careful verification and iteration
- strong architectural judgment
- strong frontend/visual artifact work
- good root-cause reasoning

Watch-outs:

- higher cost than Sonnet
- may use more time/tokens than routine tasks need
- may delegate to subagents aggressively
- repeated “verify again” prompting can produce over-verification

Guidance:

> Use Opus 5 High as architect, difficult-problem solver, and final reviewer. Give strong goals and constraints rather than micromanaging its reasoning.

## Claude Sonnet 5 — High effort

Best use:

- main implementation
- UI components
- migrations
- repeated coding
- testing
- deterministic feature completion
- medium-to-complex debugging
- iterative fixes

Relevant strengths:

- highly agentic
- sustained coding and tool use
- strong follow-through
- good debugging
- cost efficient for long implementation work

Watch-outs:

- for the most ambiguous architecture conflicts, prefer Opus if available
- explicit acceptance criteria help prevent premature “done”
- final architecture consistency review is valuable

## Recommended two-model workflow

```text
Opus 5 High
Architecture + constraints + design reconciliation
        ↓
Sonnet 5 High
Implementation + tests + iteration
        ↓
Opus 5 High
Final audit + gap detection
        ↓
Sonnet 5 High
Fix remaining gaps
```

If only one model exists, it must still complete the project. Do not block waiting for another model.

## Subagent rule

Use subagents when work:

- is independent
- can run in parallel
- benefits from isolated context
- is a separate research stream

Do not use subagents for:

- tiny edits
- sequential shared-state work
- single-file changes
- tasks where orchestration costs more than execution

This matters particularly with Opus 5 because recent Claude models can delegate aggressively.

---

# 30. HOW THE CODING AGENT MUST WORK

The coding agent must:

1. Read this guide fully.
2. Inspect all reference images.
3. Inspect the repository before changing architecture.
4. Create a concise internal implementation checklist.
5. Execute without repeatedly asking for confirmation.
6. Keep a small progress record if work spans many stages.
7. Run tests after meaningful milestones.
8. Inspect the actual rendered UI instead of assuming CSS is correct.
9. Fix issues found.
10. Produce a Windows release build.
11. Finish with:
   - what was built
   - tests run
   - known limitations
   - output locations
   - exact non-technical launch instructions

If a requirement is ambiguous but not blocking, choose the simplest approach consistent with this guide and record the assumption.

Do not stop halfway to ask whether to continue.

---

# 31. VISUAL VALIDATION

For every major screen verify:

- reference-image structure
- spacing
- RTL
- typography hierarchy
- card alignment
- long Arabic titles
- overflow
- empty state
- selected state
- keyboard focus
- Windows scaling where practical
- responsive resizing

Required screens:

- Home
- Task browser
- Tool family
- Generic category
- Item detail
- Global search
- Update center
- Favorites
- Links/Notes
- Settings

---

# 32. EMPTY AND ERROR STATES

Examples:

**No tools**
> Add your first tool or import a Knowledge Pack.

**No search results**
> No match found. Try a task phrase, tool name, or command alias.

**Source unavailable**
> The source could not be checked. Existing local knowledge remains available.

**Database problem**
> Personal Wiki could not read the local knowledge database. Run Diagnostics.

---

# 33. FUTURE EXTENSIONS — DO NOT BUILD NOW

Do not block future additions such as:

- optional local embeddings
- cloud sync
- profiles
- browser extension
- clipboard capture
- AI “Ask my Wiki”
- MCP server
- mobile companion
- shared team packs

But V1 must remain small and coherent.

---

# 34. DEFINITION OF DONE

Personal Wiki V1 is done only when:

- it is a real Windows 11 desktop app
- light theme closely follows the reference visual language
- all knowledge is data-driven
- new tools/categories/items appear without rebuild
- SQLite is the local source of truth
- FTS5 search works
- task-first and tool-first discovery both work
- detail pages explain enough for a non-technical user
- links/notes/favorites work
- Knowledge Pack import exists
- changes stage before apply
- provenance is stored
- backup/rollback protection exists
- CLI supports agent-safe update workflow
- major tests pass
- Windows release build works
- non-technical launch instructions exist

---

# 35. OFFICIAL TECHNICAL REFERENCES CHECKED FOR THIS GUIDE

The coding agent should re-check current docs when internet is available.

## Anthropic / Claude

- Claude Opus 5  
  https://www.anthropic.com/news/claude-opus-5

- Claude Sonnet 5  
  https://www.anthropic.com/news/claude-sonnet-5

- Prompting best practices  
  https://platform.claude.com/docs/en/build-with-claude/prompt-engineering/claude-prompting-best-practices

- Agent Skills  
  https://platform.claude.com/docs/en/agents-and-tools/agent-skills/overview

## Tauri

- Windows prerequisites  
  https://v2.tauri.app/start/prerequisites/

- SQL plugin  
  https://v2.tauri.app/plugin/sql/

- Updater  
  https://v2.tauri.app/plugin/updater/

- Permission / ACL model  
  https://v2.tauri.app/reference/acl/permission/

## SQLite

- FTS5  
  https://www.sqlite.org/fts5.html

- JSON functions  
  https://www.sqlite.org/json1.html

---

# 36. THE ONE SENTENCE THAT MUST SURVIVE EVERY REFACTOR

> **Personal Wiki is a stable desktop knowledge engine whose pages, buttons, branches, cards, and explanations are generated from structured local data. Content growth changes the data, not the executable.**
