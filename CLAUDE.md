# Personal Wiki Project Instructions

Read `PERSONAL_WIKI_MASTER_GUIDE.md` before making architectural or product decisions.

Read `docs/UI_SPEC.md` before building or changing any screen or component.

Inspect all files in `references/` when implementing or reviewing UI.

Critical rules:

- The UI is data-driven. Never hard-code tool/category/command lists.
- Knowledge updates must not require an application rebuild.
- SQLite is the local source of truth; FTS5 powers V1 search.
- External agents update through validated Knowledge Packs and staging, not direct DB mutation by default.
- Preserve personal notes/favorites across source refreshes.
- Light theme only for V1.
- Build for a non-technical Windows 11 user.
- Verify official commands from official sources; screenshots are visual references, not factual command documentation.
- Prefer the simplest coherent architecture. Avoid unnecessary files, services, abstractions, and subagents.
- Reuse the component inventory in `docs/UI_SPEC.md` §3; do not fork a component per screen or per tool.
- Every list, tab, badge, and counter comes from a query. See the checklist in `docs/UI_SPEC.md` §15.
- Real RTL via logical properties; code, syntax, and numbers stay LTR-isolated.
- Run tests and inspect the real UI before declaring completion.
