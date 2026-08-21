# PERSONAL WIKI — UI IMPLEMENTATION SPEC

**Companion to:** `PERSONAL_WIKI_MASTER_GUIDE.md` (product + architecture source of truth)
**Scope of this file:** the visual and interaction contract — every screen, region, component, state, and the data query behind it.
**Reference images:** `references/01…06`.

---

## 0. HOW TO USE THIS DOCUMENT

The master guide answers *what the product is and how the data is shaped*.
This document answers *what the coder must actually build on screen*.

Rules that override any screenshot detail:

1. Every list, card, tab, badge, branch, and counter on these screens is **rendered from a query**, never from a literal array in code.
2. A screenshot showing four tool cards means "render N tool cards"; it does not mean "hard-code Claude Code, Codex CLI, OpenCode, Gemini CLI".
3. Arabic strings shown in the images are **UI chrome copy** (localized in the app), while command text inside cards is **data** (from SQLite).
4. Command syntax visible in the images is illustrative. Verify from official sources before storing it as `verification_status = official`.

Each screen section below is written in the same order:
region map → components → data binding → states → interactions → acceptance.

---

## 1. GLOBAL SHELL

Every screen except modals renders inside one persistent shell. Build it once
(`components/shell/`) and never duplicate it per route.

```text
┌──────────────────────────────────────────────────────────────────────────┐
│ TITLE BAR  (custom, draggable)                    ─  □  ✕                │
│  ▸ brand (logo + "Personal Wiki")                                        │
│  ▸ global search field (centred, Ctrl K)                                 │
│  ▸ action cluster: Updates · Sync · Settings · avatar                    │
├────────────┬─────────────────────────────────────────┬───────────────────┤
│ LEFT NAV   │ CONTENT (route outlet)                  │ RIGHT PANEL       │
│ (rail)     │                                         │ (contextual,      │
│            │                                         │  optional)        │
│            │                                         │                   │
│ …          │                                         │                   │
│            │                                         │                   │
│ STATUS     │                                         │                   │
│ CARD       │                                         │                   │
│ v1.0.0     │                                         │                   │
└────────────┴─────────────────────────────────────────┴───────────────────┘
```

Because the app is RTL, "left nav" means **the navigation rail sits on the right
edge in Arabic**. Implement with logical properties (`inline-start` / `inline-end`)
so a future LTR locale mirrors automatically. Never hard-flip with `left:`/`right:`.

### 1.1 Title bar

| Element | Spec |
| --- | --- |
| Height | 64 px, surface `--surface`, bottom hairline `--border` |
| Drag region | whole bar except interactive controls (`data-tauri-drag-region`) |
| Window controls | minimize / maximize / close, inline-end edge, Windows 11 metrics (46×32 hit area), hover fill, close hovers `--danger` |
| Brand | 32 px rounded icon + wordmark, click → Home |
| Search field | max-width 640 px, height 48, radius `--radius-lg`, muted surface, leading search glyph, trailing `Ctrl K` key-cap. Placeholder is intent-shaped: `ماذا تريد أن تفعل؟` |
| Action cluster | three icon+label buttons — Updates (`تحديث`), Sync (`مزامنة`), Settings (`الإعدادات`) — plus user avatar |
| Badges | Updates shows a blue dot when `staged_changes.status='pending' > 0`; Sync shows a green check when last sync succeeded, amber dot while running, red dot on failure |

Two visual variants appear in the references (icon-over-label in `01–05`, pill
buttons in `06`). **Ship the icon-over-label variant** from `01`; it is the
denser, more Windows-11-native form. Keep the pill styling as the pattern for
in-content primary actions.

### 1.2 Navigation rail

Width 232 px, surface `--surface`, no border, items 44 px tall, radius
`--radius-md`, 4 px gap. Active item: tinted background `--accent-blue-soft`,
icon in `--accent-blue`, 3 px inline-start indicator bar, medium weight.

Items are **data + route driven**, in this order:

```text
لوحة التحكم      Home        /
الأدوات          Tools       /tools
الأوامر          Tasks       /tasks        (task-first browser)
الروابط          Links       /links
المفضلة          Favorites   /favorites
الملاحظات        Notes       /notes
التحديثات        Updates     /updates
الإعدادات        Settings    /settings
```

Reference `06` shows the rail in **tree mode**: a nav item can expand into its
category children (chevron rotates, children indented 16 px, 36 px rows). Build
one `NavTree` component that recurses on `categories.parent_id`; depth is not
limited to two. The flat rail in `01–05` is the same component with all nodes
collapsed.

Rail footer holds two blocks:

- **Sync status card** — `--surface-muted`, radius `--radius-md`: state dot +
  label (`متزامنة` / `جارٍ المزامنة` / `تعذّرت المزامنة`) and `آخر مزامنة: …`.
- **Version row** — `v1.0.0` + chevron → Diagnostics.

### 1.3 Right panel

Optional, 340–420 px, appears only when the route declares contextual content.
It is a slot, not a screen: Home puts activity + recents in it, tool-family pages
put a "how to read this page" explainer, item detail puts nothing (the detail
already occupies the pane). Collapses below 1280 px viewport width.

### 1.4 Content column

Max content width 1180 px, centred, page padding `--space-6` (24 px) inline and
`--space-5` block. Every route begins with either a breadcrumb row or a page
header — never with a bare card.

---

## 2. DESIGN TOKENS AND METRICS

Light theme only. Declare once in `src/design/tokens.css`; components consume
variables only. No raw hex inside a component.

```css
/* surfaces */
--background:      #F6F8FC;   /* app canvas, faint blue-grey */
--surface:         #FFFFFF;
--surface-muted:   #F3F5FA;
--surface-sunken:  #EEF2F9;
--border:          #E4E9F2;
--border-strong:   #D3DAE8;

/* text */
--text-primary:    #10182B;
--text-secondary:  #5A6782;
--text-tertiary:   #8A96AD;
--text-inverse:    #FFFFFF;

/* accents (semantic keys map here — see guide §7 accent system) */
--accent-blue:     #2F6BFF;  --accent-blue-soft:   #E8EFFF;
--accent-green:    #17A46A;  --accent-green-soft:  #E3F7EE;
--accent-purple:   #7A5AF8;  --accent-purple-soft: #EDE9FE;
--accent-orange:   #F2762E;  --accent-orange-soft: #FFEDE1;
--accent-teal:     #0FA3B1;  --accent-teal-soft:   #E1F6F8;

/* status */
--success: #17A46A;  --warning: #E0A32E;  --danger: #E0483E;  --info: #2F6BFF;

/* radius */
--radius-sm: 8px;  --radius-md: 12px;  --radius-lg: 16px;  --radius-xl: 20px;

/* elevation — soft and low, never dramatic */
--shadow-sm: 0 1px 2px rgba(16,24,43,.04), 0 1px 3px rgba(16,24,43,.06);
--shadow-md: 0 4px 12px rgba(16,24,43,.06), 0 2px 4px rgba(16,24,43,.04);
--shadow-lg: 0 12px 32px rgba(16,24,43,.10);

/* spacing scale (4px base) */
--space-1:4px; --space-2:8px;  --space-3:12px; --space-4:16px;
--space-5:20px; --space-6:24px; --space-7:32px; --space-8:40px;
```

**Type scale** (Arabic-first stack: `"Segoe UI", "IBM Plex Sans Arabic", "Noto Sans Arabic", system-ui`):

| Role | Size / line / weight |
| --- | --- |
| Hero title | 34 / 44 / 700 |
| Page title | 24 / 32 / 700 |
| Section title | 17 / 24 / 600 |
| Card title | 15 / 22 / 600 |
| Body | 14 / 22 / 400 |
| Secondary | 13 / 20 / 400 |
| Meta / chip | 12 / 16 / 500 |
| Code | 13 / 22 / 500, `"Cascadia Code", Consolas, monospace`, always `dir="ltr"` |

**Motion:** 120 ms hover, 180 ms panel/route transitions, `cubic-bezier(.2,.8,.2,1)`.
Respect `prefers-reduced-motion`.

**Focus:** 2 px `--accent-blue` ring at 2 px offset on every interactive element.
Keyboard focus must be visible on cards, chips, tabs, and list rows — not only inputs.

---

## 3. COMPONENT INVENTORY

Build these once. If a screen needs something not on this list, extend a
component rather than writing a page-local variant.

| Component | Used by | Key props (all from data) |
| --- | --- | --- |
| `AppShell` | all | — |
| `NavTree` | shell | `nodes[]` (recursive), `activeId` |
| `GlobalSearchField` | shell | `value`, `onSubmit`, `suggestions[]` |
| `Breadcrumbs` | B, C, F | `crumbs[] {label, href}` |
| `PageHeader` | all content routes | `icon_key`, `accent_key`, `title`, `subtitle`, `badges[]`, `actions[]` |
| `StatTile` | B, F | `icon_key`, `value`, `label`, `sublabel`, `accent_key` |
| `ToolCard` | A | `tool` + counts `{items, examples, updated_at}` |
| `TaskCard` | A, tasks | `task`, `item_count`, `tool_count` |
| `SectionTabs` | B | `sections[]` (from categories), `activeSlug`, overflow-scroll |
| `ItemCard` | B, F, search | `item`, `tags[]`, `quickActions[]` |
| `ItemListRow` | C (master pane) | `item`, `selected` |
| `KnowledgeDetail` | C, F, search preview | `item` + related records; renders only sections whose data exists |
| `CodeBlock` | detail, examples | `code`, `lang`, `copyable`, forced LTR |
| `ParamTable` | detail | `parameters[]` |
| `RelatedChips` | detail | `relationships[]` |
| `SourceLine` | detail, update centre | `source`, `verified_at`, opens externally |
| `TagChip` | everywhere | `tag`, `accent_key` |
| `StatusPill` | update centre, detail | `kind: official/verified/community/unverified/deprecated` |
| `TimelineList` | E | `events[]` |
| `DiffViewer` | E | `before_json`, `after_json` |
| `ActivityList` | A, F | `entries[]` |
| `QuickActionCard` | A | `icon_key`, `title`, `subtitle`, `action` |
| `EmptyState` | all | `icon_key`, `title`, `body`, `cta` |
| `ErrorState` | all | `title`, `body`, `retry`, `diagnosticsLink` |
| `SkeletonCard` / `SkeletonRow` | all | count |
| `VirtualList` | any list > 100 rows | `items`, `rowHeight` |

**Card baseline:** `--surface`, 1 px `--border`, `--radius-lg`, `--shadow-sm`,
padding `--space-5`; hover raises to `--shadow-md` and `--border-strong`,
transform `translateY(-1px)`. Selected card: 1.5 px `--accent-blue` border +
`--accent-blue-soft` wash.

**Icon system:** every entity carries `icon_key` (string) and `accent_key`.
The renderer maps key → component through one lookup table with a generic
fallback. Never store markup or component references in the database.

---

## 4. ROUTE MAP

```text
/                          Home dashboard              → ref 01
/tools                     Tool browser (grid of all tools)
/tools/:toolSlug           Tool family page            → ref 02
/tools/:toolSlug/:section  Tool family, section active → ref 02
/tasks                     Task browser (task-first grid)
/tasks/:taskSlug           Generic task/category page  → ref 06
/c/:categoryPath*          Generic category page (arbitrary depth) → ref 06
/items/:itemSlug           Item detail (two-pane)      → ref 03
/search?q=&filters=        Global search               → ref 04
/favorites                 Favorites
/notes                     Notes
/links                     Useful links
/updates                   Update centre               → ref 05
/updates/batches/:id       Batch diff / review
/settings                  Settings
/diagnostics               Diagnostics
```

Deep links must be restorable: the same URL reopens the same tool, section,
selected item, query, and filter set.

---

## 5. SCREEN A — HOME DASHBOARD

**Reference:** `references/01_HOME_DASHBOARD.png`
**Purpose:** the outermost face of the app. Answers "what is in here, what
changed, and where do I continue" in one glance.

### Region map

```text
┌───────────────── content column ─────────────────┬──── right panel ────┐
│ HERO BANNER                                      │ RECENT UPDATES card │
│  gradient wash + illustration + title/subtitle   │  (activity list)    │
├──────────────────────────────────────────────────┤                     │
│ TOOL GRID   (N cards, 4 per row @1440, 3 @1200,  │ RECENTLY USED card  │
│              2 @980, 1 @720)                     │  (item list)        │
│  each: icon tile, name, description,             │                     │
│        3 metric chips, updated-ago               │ QUICK ACTIONS card  │
├──────────────────────────────────────────────────┤  (3 tiles)          │
│ "عرض جميع الأقسام"  full-width ghost button      │                     │
└──────────────────────────────────────────────────┴─────────────────────┘
```

### Hero banner

- Height ~200 px, `--radius-xl`, background: white → `--accent-blue-soft`
  diagonal wash, decorative node/document illustration on the inline-end side
  (inline SVG, never a raster screenshot).
- Title `خريطتك الشخصية للأوامر والمعرفة`, subtitle one line of body text.
- Sparkle glyph in `--accent-purple` at the inline-start of the title.
- The banner is chrome, not data — but if a "featured item" exists it renders a
  CTA button inside the banner.

### Tool card (the heart of this screen)

```text
┌──────────────────────────────┐
│ ▢  56×56 icon tile, accent   │   ← accent_key drives tile + soft background
│                              │
│ Claude Code                  │   ← tools.name
│ أفضل أوامر Claude Code …      │   ← tools.description, 2 lines, ellipsis
│                              │
│ [128 أمر] [42 مثال] [منذ يوم]│   ← metric chips, tinted --surface-muted
└──────────────────────────────┘
```

Metric chips are counts, computed — not stored strings:

```sql
-- cards
SELECT t.*,
  (SELECT COUNT(*) FROM items i
     WHERE i.tool_id = t.id AND i.is_active = 1)                    AS item_count,
  (SELECT COUNT(*) FROM examples e JOIN items i2 ON i2.id = e.item_id
     WHERE i2.tool_id = t.id)                                       AS example_count,
  (SELECT MAX(i3.updated_at) FROM items i3 WHERE i3.tool_id = t.id) AS last_updated_at
FROM tools t
WHERE t.is_active = 1
ORDER BY t.sort_order, t.name;
```

The grid must stay elegant at 5 tools and at 50 — no fixed row count, no
"featured four". Cards with a missing description collapse that line rather
than reserving empty space. The trailing `عرض جميع الأقسام` button routes to
`/tools`.

### Right panel cards

- **آخر التحديثات** — latest `update_batches` / applied `staged_changes`,
  each row: state dot (`add` green, `update` blue, `remove` red), title,
  relative time, tool icon at the inline-end. Header carries a `عرض الكل` link
  to `/updates`.
- **العناصر المستخدمة مؤخرًا** — `recent_items` joined to `items`, chevron per
  row, relative time, tool icon.
- **إجراءات سريعة** — three `QuickActionCard`s: Update centre, Import commands
  (from file/source), Add new source. These are app capabilities, so they may be
  declared in code — but each must be permission-checked and hidden when the
  capability is disabled.

### States

| State | Rendering |
| --- | --- |
| Loading | hero renders immediately; 8 `SkeletonCard`s in the grid; panel skeleton rows |
| No tools | `EmptyState`: "أضف أول أداة أو استورد حزمة معرفة" + two buttons (Add tool, Import pack) |
| No recent activity | panel card shows a one-line muted note, card is not hidden |
| DB unreadable | full-page `ErrorState` with a Diagnostics button |

### Interactions

- Tool card: click → `/tools/:slug`; `Enter`/`Space` when focused; middle-click ignored.
- Activity row → the affected item, or the batch diff if the item no longer exists.
- `Ctrl+K` from anywhere focuses global search and opens the suggestions popover.

### Acceptance

1. Inserting a `tools` row and refreshing shows a new card with correct counts and no code change.
2. Deactivating a tool (`is_active = 0`) removes the card without deleting knowledge.
3. Resizing 1440 → 900 px reflows the grid without horizontal scroll or clipped Arabic text.

---

## 6. SCREEN B — TOOL FAMILY PAGE

**Reference:** `references/02_TOOL_FAMILY_CODEX.png`
**Purpose:** everything belonging to one tool, organised by its own branches.

### Region map

```text
BREADCRUMBS  الأدوات › Codex CLI › الوكلاء والوكيل الفرعية
┌───────────────── content ─────────────────┬──── right panel ────┐
│ IDENTITY HEADER                           │ "كيف تنظّم هذه       │
│  icon tile · name · description · tags    │   الصفحة؟" explainer │
├───────────────────────────────────────────┤  numbered 1-4 list   │
│ STAT STRIP (5 tiles)                      │  with links          │
│  أوامر · أمثلة · ملاحظات · مصادر · آخر تحديث│                     │
├───────────────────────────────────────────┤ HELP card            │
│ SECTION TABS (horizontal, scrollable)     │                     │
├───────────────────────────────────────────┤                     │
│ SECTION HEADER: title · count · sort menu │                     │
│ ITEM GRID (2 columns, dense cards)        │                     │
└───────────────────────────────────────────┴─────────────────────┘
```

### Identity header

Icon tile 56 px in the tool's accent, `tools.name` as page title, description
below, then tag chips (`CLI`, `AI`, `تطوير`, `أتمتة`) from `item_tags`
aggregated over the tool, or from tool-level tags where modelled. Inline-end of
the header holds secondary actions: open official home URL, favourite the tool,
export this tool.

### Stat strip

Five `StatTile`s in one row (wrap to 3+2 under 1100 px). Each: soft-tinted icon
tile, big number, label, sublabel.

```text
إجمالي الأوامر 96 · أمر متاح
الأمثلة        31 · مثال عملي
الملاحظات      19 · ملاحظة
المصادر        12 · مصدر موثوق
آخر تحديث   2 أيام · <date>
```

All five are queries. If a tool has no notes, the tile shows `0` — it is not removed.

### Section tabs

```sql
SELECT c.* FROM categories c
WHERE c.is_active = 1
  AND EXISTS (SELECT 1 FROM item_categories ic JOIN items i ON i.id = ic.item_id
              WHERE ic.category_id = c.id AND i.tool_id = :toolId AND i.is_active = 1)
ORDER BY c.sort_order, c.name;
```

- Rendered as pills with icon + label; active pill = `--accent-blue` border,
  `--accent-blue-soft` fill, blue text.
- The strip scrolls horizontally with fade masks on overflow. **Never truncate
  the list to a fixed count.**
- Selecting a tab pushes `/tools/:toolSlug/:sectionSlug`.
- A tool with nested categories renders the parent row of tabs plus a
  second-level chip row beneath it; deeper levels appear in the rail tree.

### Item card (compact, 2-up)

```text
┌───────────────────────────────────────────┐
│ ▢ icon   codex agent list        🔖 pin   │
│ عرض جميع الوكلاء المتاحين في مشروعك الحالي│
│ [الاستخدام] [codex] [agent] [list]        │
│ ───────────────────────────────────────── │
│ [نسخ] [مثال] [مصدر] [رابط]                │
└───────────────────────────────────────────┘
```

- Title in monospace when `item_type` is a command family; body sans.
- Tag row from `item_tags` (max 4 visible, `+N` chip for the rest).
- Quick actions are conditional: `نسخ` only when `syntax` exists, `مثال` only
  when the item has `examples`, `مصدر` only when `item_sources` exists.
- Bookmark toggles `favorites` optimistically and rolls back on failure.

### Section header

`الوكلاء والوكيل الفرعية` + `8 أوامر` muted count at the inline-end, and a sort
control (`الأحدث` / `الأبجدي` / `الأكثر استخدامًا`). Sorting is client-visible
state persisted per tool in settings.

### Right panel explainer

A numbered 1→4 walkthrough of the page anatomy ("page title and path",
"section navigation", "command cards", "detail panel"). This is static help copy
keyed by screen id, stored in the locale file, not in the knowledge DB.

### States

- Tool with zero items → `EmptyState` inside the section area, tabs hidden,
  stat strip still rendered with zeros.
- Section with zero items (possible after filtering) → inline empty note plus a
  "clear filter" action.
- Long Arabic tool descriptions wrap to a maximum of 3 lines then ellipsis with
  full text in a tooltip.

### Acceptance

1. Adding a category and linking one item to it makes a new tab appear — no code change.
2. Adding a nested child category shows it as a second-level chip and in the rail tree.
3. A tool whose items all become `is_active = 0` renders the empty state, not a broken grid.

---

## 7. SCREEN C — ITEM DETAIL (TWO-PANE)

**Reference:** `references/03_COMMAND_DETAIL.png`
**Purpose:** understand one command well enough to use it without prior knowledge.

This is the most important screen in the product. It is **one reusable
`KnowledgeDetail` component** that adapts to whatever the record contains.

### Region map

```text
BREADCRUMBS  الرئيسية › الأدوات › Codex CLI › العملاء والوكلاء الفرعيون
┌─── MASTER (380px) ───┬────────── DETAIL (fluid, 2 columns) ──────────┐
│ scoped search        │ HEADER: icon · title · type chip · ★           │
│ + filter button      │ subtitle line                                  │
│ "12 أمرًا متاحًا"      │ ACTION BAR: تثبيت · مفضلة · فتح المصدر · نسخ   │
│ ┌──────────────────┐ │ ┌───── column 1 ─────┬───── column 2 ─────┐   │
│ │ item row         │ │ │ الهدف              │ مثال على الاستخدام │   │
│ │ item row (sel.)  │ │ │ متى يُستخدم         │ (CodeBlock + copy) │   │
│ │ item row         │ │ │ المعاملات (table)  │ النتيجة المتوقعة    │   │
│ │ …                │ │ │ ملاحظات وتنبيهات   │ المصدر الرسمي      │   │
│ └──────────────────┘ │ │                    │ أوامر ذات صلة      │   │
│ "عرض المزيد"          │ └────────────────────┴────────────────────┘   │
│                      │ FOOTER: آخر تحديث: <date>                     │
└──────────────────────┴───────────────────────────────────────────────┘
```

### Master pane

- Scoped search input (`البحث في أوامر Codex CLI…`) filters within the current
  tool/category only; it does not navigate to global search.
- Filter button opens a popover with type / tag / verification / favourite filters.
- Count line reflects the filtered result, not the total.
- Row: 44 px icon tile, title, one-line description, up to 2 tag chips, chevron.
  Selected row: blue border + soft fill + filled star if favourited.
- Rows virtualise past 100 entries. `عرض المزيد` pages by 50 when virtualisation
  is off (small lists).
- Keyboard: `↑/↓` moves selection, `Enter` focuses the detail pane, `/` focuses
  the scoped search.

### Detail sections — every one is conditional

| Section | Arabic label | Renders when |
| --- | --- | --- |
| Purpose | الهدف | `short_description` or `full_description` |
| When to use | متى يُستخدم | `when_to_use` (bulleted if the text contains list markers) |
| Syntax | الصيغة | `syntax` |
| Parameters | المعاملات (الخيارات) | ≥1 `parameters` row |
| Example | مثال على الاستخدام | ≥1 `examples` row (tabs when >1) |
| Expected result | النتيجة المتوقعة | `expected_result` |
| Warnings | ملاحظات وتنبيهات | `warning_text` |
| Source | المصدر الرسمي | ≥1 `item_sources` row |
| Related | أوامر ذات صلة | ≥1 `relationships` row |
| Personal notes | ملاحظاتي | ≥1 `notes` row for this item |
| Last verified | آخر تحديث | `last_verified_at` or `updated_at` |

**A link item shows purpose + source only. A concept shows purpose + when-to-use.
A command shows everything.** No placeholder text, no "N/A", no empty card.

### Section styling

Each section is a card with a leading accent glyph (target, clock, sliders,
warning triangle, globe, chain) matching the reference. Section titles are
17/24/600 with the glyph in the section's accent colour.

**Parameter table** columns (RTL order: rightmost first):
`الخيار` (mono, LTR) · `الوصف` · `مطلوب` (نعم/لا, `--danger`/muted) · `الافتراضي`.
Zebra rows on `--surface-muted`, header row 600 weight, horizontal scroll
container on narrow widths so the page body never scrolls sideways.

**Code block:** `#0F1729` surface, `--radius-md`, 16 px padding, mono 13/22,
`dir="ltr"` with `text-align:left` even inside the RTL page, floating
`نسخ الأمر` button pinned to the block's top-inline-end corner. Comments may
contain Arabic — the block must therefore support mixed bidi runs; wrap Arabic
comment spans in `<span dir="rtl">`.

**Expected result:** prose plus, when the record carries a `result_diagram`
payload, a small inline SVG flow (as in the reference: main agent → two
sub-agents → check marks). Where no diagram data exists, render prose only —
never fabricate a diagram.

**Related items:** chips that navigate within the master list when the target
shares the current scope, otherwise route to the target's own detail page.

### Action bar

`تثبيت` (pin to rail quick list) · `إضافة إلى المفضلة` · `فتح المصدر`
(external, safe-URL allow-list) · `نسخ الأمر`. Copy writes `syntax`, shows a
2 s inline confirmation, and **never executes anything**. If `syntax` is null,
the copy button is not rendered.

### Acceptance

1. An item with only title + description renders a clean short page — no empty sections.
2. An item with every field populated renders all sections in the specified order.
3. Copy places exactly `items.syntax` on the clipboard.
4. `فتح المصدر` opens the OS browser through Tauri's opener with a validated `https:` URL.
5. Adding a personal note leaves official content untouched, and survives a source refresh.

---

## 8. SCREEN D — GLOBAL SEARCH

**Reference:** `references/04_GLOBAL_SEARCH.png`
**Purpose:** the fastest path from human intent to the right command.

### Region map

```text
TOP BAR search field is active and carries the query
┌────────────── results column ──────────────┬─── suggestions panel ───┐
│ FILTER BAR: الأدوات ▾ · نوع المحتوى ▾ ·     │ اقتراحات بحث            │
│             الوسوم ▾ · المصدر ▾ · المفضلة   │  (5 alias/phrase rows)  │
│ RESULT META: تم العثور على 28 نتيجة خلال    │ عمليات البحث الحديثة    │
│              0.32 ثانية      | ترتيب ▾      │  (with clear-all)       │
│ ── GROUP: Claude Code ──────────────────    │ عمليات البحث المحفوظة   │
│   result row (score chip, actions)          │  (with overflow menu)   │
│ ── GROUP: Codex CLI ───────────────────     │ + حفظ البحث الحالي      │
│   result row …                              │                         │
│ "عرض المزيد من النتائج (8)"                 │                         │
└─────────────────────────────────────────────┴─────────────────────────┘
```

### Query pipeline (V1, deterministic — no embeddings)

```text
normalize (trim, fold Arabic diacritics/tatweel, unify ا/أ/إ, ه/ة, ي/ى,
           map Arabic-Indic digits, lowercase Latin)
→ FTS5 MATCH over items_fts(title, aliases, short_description, full_description,
                            task_phrases, tags, syntax, examples, tool, category)
→ score = bm25 base
        + 40 exact title       + 30 exact alias
        + 18 syntax prefix     + 12 task match      + 8 tool match
        + 3 favourite          + 2 recently opened
        − 25 deprecated        − 60 removed
→ group by tool, order groups by best member score
→ paginate 20
```

Target: results visible under 150 ms for a 10k-item database. Debounce input at
120 ms; cancel in-flight queries on new input.

### Result row

Tool group header (icon + tool name) then rows:
lifecycle dot · title · one-line description · tag chips · relevance chip
(`98% صلة`, tinted green ≥90, blue ≥70, muted below) · actions
`فتح` / `نسخ` / `معاينة`. `معاينة` opens the `KnowledgeDetail` in a side sheet
without leaving the results.

### Filters

Each filter is a dropdown fed by a query (tools, item types, tags, sources,
lifecycle) plus a `المفضلة فقط` toggle. Active filters render as removable chips
under the bar and are encoded in the URL. A trailing icon clears all.

### Suggestions panel

- **اقتراحات بحث** — alias and task-phrase completions from `aliases` +
  `tasks.name`, arrow glyph, click fills and runs the query.
- **عمليات البحث الحديثة** — local history with timestamps and a `مسح الكل`.
- **عمليات البحث المحفوظة** — named query + filter snapshots with a `⋮` menu
  (rename, run, delete) and a `+ حفظ البحث الحالي` action.

### States

- Empty query → panel only, results area shows top tasks and recent items.
- No results → `EmptyState`: "لا توجد نتيجة. جرّب عبارة مهمة أو اسم أداة أو اسمًا بديلًا للأمر"
  plus the three nearest alias suggestions.
- FTS index missing/corrupt → error state with a "rebuild index" action that
  runs the repair routine and reports the outcome.

### Acceptance

1. Searching an alias absent from title and syntax still ranks its item in the top 3.
2. A deprecated item ranks below an active equivalent for the same query.
3. Filters survive reload via the URL.
4. Search returns correctly with the network disconnected.

---

## 9. SCREEN E — UPDATE CENTRE

**Reference:** `references/05_UPDATE_CENTER.png`
**Purpose:** review what changed in the outside world and decide what enters the
trusted database. This screen is the human gate in front of every AI-produced pack.

### Region map

```text
PAGE HEADER  مركز التحديثات والمصادر  + illustration
┌───────────────┬─────────────────────────────┬──────────────────────┐
│ SOURCES card  │ SUMMARY STRIP (4 tiles)     │ SYNC TIMELINE        │
│  table:       │  آخر مزامنة · المزامنة       │  vertical, icon +    │
│  source ·     │  التالية · التحديثات        │  title + detail +    │
│  type ·       │  المعلقة · الحالة العامة     │  time                │
│  last check · ├─────────────────────────────┤                      │
│  state pill · │ CHANGE TABS                 │  عرض السجل الكامل     │
│  ⋮ menu       │  الكل | إضافية | محدثة |     │                      │
│               │  محذوفة   (with counts)     │                      │
│  + إضافة مصدر  │ CHANGE ROWS                 │                      │
│  إدارة المصادر │  badge · title · detail ·   │                      │
│               │  time                       │                      │
└───────────────┴─────────────────────────────┴──────────────────────┘
┌──────────────────────── APPROVAL BAR ────────────────────────────────┐
│ counts (18 عنصر · 9 جديدة · 6 محدثة · 3 محذوفة)                       │
│ [مراجعة التغييرات] [الموافقة على الكل] [رفض الكل] [استيراد ملف]        │
│ note: جميع التغييرات قابلة للتراجع — نسخة احتياطية قبل التطبيق         │
└──────────────────────────────────────────────────────────────────────┘
```

### Sources card

One row per `sources` record: favicon/icon, name, host, `source_type` label,
`last_checked_at` relative, state pill (`متصل` green / `تعذّر` red /
`لم يُفحص` muted), overflow menu (check now, edit, disable, delete). Counter
line "5 / 5 مصادر نشطة". Rows are data — adding a source row adds a row here.

### Summary tiles

`آخر مزامنة` (status + time, with `عرض السجل الكامل`), `المزامنة التالية`
(schedule), `التحديثات المعلقة` (pending count, links to review),
`الحالة العامة` (health roll-up). Each tile carries an icon in a soft tint.

### Change list

Tabs are counts over `staged_changes.change_type` for the open batches. Each row:
type badge (`جديد` blue, `محدث` amber, `محذوف` red), item title (mono for
commands), one-line detail, timestamp. Clicking a row opens the **diff drawer**:
before/after side by side, field-level highlighting, source link, confidence,
and per-row `موافقة` / `رفض`.

### Apply flow — the safety contract

```text
Approve selection
→ create backup snapshot (record path + batch id)
→ BEGIN TRANSACTION
→ apply approved staged_changes in dependency order
→ rebuild affected FTS rows
→ write audit record
→ COMMIT              (on any error: ROLLBACK, keep backup, surface ErrorState)
→ refresh affected views
```

The UI must show: a progress state during apply, an explicit success summary
("تم تطبيق 15 تغييرًا"), and an **undo** affordance for the last applied batch.
`رفض الكل` marks rows rejected without touching trusted data.

**Never** let this screen apply changes silently on load, and never let an
external agent bypass staging: the CLI's default is `--stage`.

### Two update types — must be visually separate

The screen shows **knowledge status** (sources, packs, staged changes). The
application version/update (Tauri updater) belongs to a distinct card labelled
`تحديث التطبيق` with its own version string and check button. Do not merge the
two into one "update" concept.

### Acceptance

1. A malformed pack is rejected at validation; nothing reaches staging; the user sees the failing field and line.
2. A pack that fails mid-apply rolls back; `PRAGMA integrity_check` still returns `ok`; the backup remains.
3. Approving an `update` change preserves that item's personal notes and favourite state.
4. Undo restores the previous state and records the reversal in history.

---

## 10. SCREEN F — GENERIC CATEGORY / TASK PAGE

**Reference:** `references/06_GENERIC_CATEGORY_DETAIL.png`
**Purpose:** the tool-independent view — "everything about logging in", regardless
of which product it belongs to. This is the **task-first** face of the product and
uses exactly the same components as Screen B and C.

### Region map

```text
RAIL in tree mode (category expanded)
BREADCRUMBS  الرئيسية › الدخول والحسابات
┌───────────────── content ─────────────────┬──── detail panel (fixed) ───┐
│ CENTRED PAGE HEADER                        │ ← الرجوع إلى القائمة   ⋮   │
│  title · subtitle · large accent icon      │ ITEM HEADER (icon, title,   │
│ STAT STRIP (4 tiles: أوامر/أمثلة/روابط/    │   subtitle, tag chips)      │
│              ملاحظات)                      │ الوصف                       │
│ SECTION: الأوامر الشائعة   [عرض الكل]      │ متى يُستخدم؟                │
│  3-column ITEM CARD grid                   │ مثال الأمر (CodeBlock)      │
│  (icon at inline-end of each card,         │ ملاحظات (bullets)           │
│   tags + نسخ / مثال / المصدر actions)      │ آخر تحديث                   │
│ SECTION: روابط مهمة        [عرض الكل]      │ المصدر (external link)      │
│  4-column LINK CARD row                    │ ─────────────────────────── │
│                                            │ [نسخ الأمر] [فتح المصدر]    │
└────────────────────────────────────────────┴─────────────────────────────┘
LEFT SIDE (inline-start): آخر التحديثات activity card under the rail
```

### Differences from Screen B (and why)

- **Header is centred** with the icon beside the title: a task/category is a
  concept, not a product, so it gets a calmer, more editorial header.
- **Item cards are 3-up and place the icon at the inline-end** of the card head.
- **Sections are content groupings** (`الأوامر الشائعة`, `روابط مهمة`,
  `ملاحظاتي`), each with a `عرض الكل` link, rather than a tab strip.
- **The detail panel is pinned** to the inline-end with its own sticky footer of
  primary actions, instead of the two-column detail of Screen C.

Everything else — cards, chips, code block, source line — is the same component
set. Do **not** fork `ItemCard` or `KnowledgeDetail` for this screen; pass a
layout variant prop.

### Data binding

```sql
-- items in this category (and, when the category has children, its subtree)
WITH RECURSIVE subtree(id) AS (
  SELECT id FROM categories WHERE slug = :slug
  UNION ALL
  SELECT c.id FROM categories c JOIN subtree s ON c.parent_id = s.id
)
SELECT i.* FROM items i
JOIN item_categories ic ON ic.item_id = i.id
WHERE ic.category_id IN (SELECT id FROM subtree)
  AND i.is_active = 1
ORDER BY i.sort_order, i.title;
```

A task page uses `item_tasks` with the same shape. **One item may appear on
several task and category pages without being duplicated in storage.**

### Acceptance

1. A task page shows items from four different tools, visually grouped or badged by tool.
2. Importing a three-level category (`PowerShell → Automation → Scheduled Tasks`)
   renders rail tree, breadcrumbs, and page without code changes.
3. Selecting a card fills the pinned detail panel; the URL updates; reload restores the selection.

---

## 11. REMAINING SCREENS (same components, no new patterns)

| Screen | Composition |
| --- | --- |
| **Tool browser** `/tools` | `PageHeader` + the Home tool grid at full width + a search/sort row |
| **Task browser** `/tasks` | `PageHeader` + `TaskCard` grid (icon, task name, item count, contributing tool icons) |
| **Favorites** | filter chips (tool / type) + `ItemCard` grid, empty state pointing at the star action |
| **Notes** | two-pane: notes list + editor (title, body, linked item chip); autosave with a saved indicator |
| **Links** | `saved_links` grid identical to the link cards on Screen F, with add/edit dialog and favicon fallback |
| **Settings** | grouped rows: language/direction, startup screen, sync schedule, source management link, backup now / restore, data folder, app update card, reset |
| **Diagnostics** | read-only report: app version, schema version, DB path, `integrity_check`, item/source counts, FTS status, last batch, recent errors, plus **copy report** and **open logs folder** |

---

## 12. INTERACTION AND KEYBOARD MODEL

| Key | Action |
| --- | --- |
| `Ctrl+K` / `Ctrl+F` | focus global search |
| `Esc` | close popover/sheet/dialog; second press clears search |
| `↑ ↓` | move within any list or result set |
| `Enter` | open focused row / submit search |
| `Ctrl+C` on a focused item card | copy its syntax |
| `Ctrl+D` | toggle favourite on the open item |
| `Alt+←` / `Alt+→` | route history back/forward (mirrored in RTL) |
| `F5` | re-run the current query / refresh the current view |
| `Tab` | strict DOM-order traversal; focus trapped inside dialogs |

Every action that mutates data (favourite, note, approve) gives feedback inside
150 ms — optimistic UI with rollback on error, plus a toast for failures only.

---

## 13. RTL IMPLEMENTATION RULES

1. `<html dir="rtl" lang="ar">`; direction comes from the locale, never from a component.
2. Use logical properties everywhere: `margin-inline-start`, `padding-inline-end`,
   `inset-inline-start`, `border-start-start-radius`. Physical `left/right` is a bug.
3. Icons that encode direction (chevrons, back arrows, breadcrumb separators)
   mirror with `[dir="rtl"] .icon-directional { transform: scaleX(-1) }`.
   Icons that do not (search, star, clock, code) never mirror.
4. Numbers, code, URLs, file paths, and command syntax stay LTR: wrap in
   `dir="ltr"` with `unicode-bidi: isolate`.
5. Mixed Arabic + Latin strings (`أوامر Claude Code`) must be isolated so the
   Latin run does not jump: `<bdi>`.
6. Charts, timelines, and progress bars flow inline-start → inline-end, i.e.
   right → left in Arabic.
7. Test every screen at the longest realistic Arabic string; Arabic runs ~20–30 %
   longer than the English equivalent. No fixed-width labels.

---

## 14. EMPTY, LOADING, AND ERROR STATES

Every list, panel, and detail section defines all three. Minimum copy:

```text
لا توجد أدوات        → أضف أول أداة أو استورد حزمة معرفة.
لا توجد نتيجة        → جرّب عبارة مهمة أو اسم أداة أو اسمًا بديلًا للأمر.
لا توجد مفضلة        → اضغط النجمة في أي أمر لإضافته هنا.
لا توجد ملاحظات      → اكتب أول ملاحظة لتحفظ طريقتك في استخدام الأداة.
تعذّر فحص المصدر     → المعرفة المحفوظة محليًا لا تزال متاحة.
مشكلة في قاعدة البيانات → تعذّرت قراءة قاعدة المعرفة. افتح التشخيص.
```

Loading uses skeletons that match the final layout's shape — never a centred
spinner on a full page. Errors always offer a next action (retry, diagnostics,
restore backup); never a bare message.

---

## 15. THE DATA-DRIVEN CHECKLIST (VISUAL LAYER)

Before declaring the UI done, confirm that **none** of the following exists in
the frontend source:

```text
□ a literal array of tool names, slugs, or icons
□ a literal array of categories, sections, or tabs
□ a per-tool component, route, or stylesheet
□ a `switch (toolSlug)` or `if (tool === …)` branch in rendering
□ a hard-coded item count, badge number, or "last updated" string
□ a colour hex inside a component file
□ an icon imported for one specific database record
□ a screenshot used as a page background instead of real markup
```

And confirm that each of the following works with **zero code changes**:

```text
□ insert a tool row            → card appears on Home and in /tools
□ insert a nested category     → tab, rail tree node, and breadcrumb appear
□ import a Knowledge Pack item → stages, diffs, approves, appears in search + detail
□ link one item to two tasks   → it appears on both task pages, stored once
□ add an alias                 → search finds the item by that alias
□ deactivate a record          → it disappears from browsing but is not deleted
```

---

## 16. VISUAL ACCEPTANCE PER SCREEN

For Home, Tool family, Item detail, Search, Update centre, Generic category,
Favorites, Notes/Links, and Settings, verify:

- structure matches the corresponding reference image (regions, order, density)
- spacing follows the 4 px scale; no ad-hoc margins
- RTL is real (logical properties), not mirrored by hand
- typographic hierarchy is visible without reading the text
- long Arabic titles wrap or ellipsis cleanly; no overflow, no body-level horizontal scroll
- empty, loading, error, hover, selected, and keyboard-focus states all exist
- 100 % / 125 % / 150 % Windows scaling remains usable
- window resize from 1600 px down to 1024 px reflows without breakage

Capture a Playwright screenshot per screen and review it. Do not assume the CSS
is right because the component compiled.
