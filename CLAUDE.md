# Personal Wiki Project Instructions

The active deliverable is the web application in this repository. It is the shared
codebase for ChatGPT, Claude, and every later coding agent.

Read these files in this order before changing code:

1. `docs/LUNA_MAX_PERSONAL_WIKI_REPAIR_PLAN.md` — current repair scope and acceptance gates.
2. `PERSONAL_WIKI_MASTER_GUIDE.md` — product and data principles.
3. `docs/UI_SPEC.md` — visual reference only where it does not conflict with the repair plan.
4. `docs/WEBAPP_README.md` — current web stack, commands, and hosting notes.

The repair plan supersedes older UI guidance. In particular, knowledge items must
open as real routed pages, not as temporary side drawers.

Inspect all files in `references/` when reviewing UI, but never treat screenshot
text or counts as factual content.

Critical rules:

- The UI is data-driven. Never hard-code tool/category/command lists.
- Knowledge updates must not require an application rebuild.
- D1 is the web source of truth; the domain model must remain portable to SQLite for the future desktop shell.
- External agents update through validated Knowledge Packs and staging, not direct DB mutation by default.
- Preserve personal notes/favorites across source refreshes.
- Light theme only for V1.
- Build for a non-technical user on desktop and mobile web.
- Verify official commands from official sources; screenshots are visual references, not factual command documentation.
- Prefer the simplest coherent architecture. Avoid unnecessary files, services, abstractions, and subagents.
- Reuse the component inventory in `docs/UI_SPEC.md` §3; do not fork a component per screen or per tool.
- Every list, tab, badge, and counter comes from a query. See the checklist in `docs/UI_SPEC.md` §15.
- Real RTL via logical properties; code, syntax, and numbers stay LTR-isolated.
- Run tests and inspect the real UI before declaring completion.
- Never publish or deploy unless the user asks separately after the acceptance gates pass.
