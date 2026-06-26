---
document_type: ux-spec
surface: S2
status: PROPOSED/DRAFT
do_not_execute: true
produced_by: ux-designer
timestamp: "2026-06-26"
traces_to:
  - matured-vision-day2-requirements.md §11.3, §11.3.1, §11.3.2
  - research/ui-requirements-2026-06-25.md §1–§5
  - matured-vision-day2-requirements.md §3.6 (partial-result semantics)
  - matured-vision-day2-requirements.md §12.1 (entity/observable FIND pivot)
  - matured-vision-day2-requirements.md §3.3 (demand-driven cache)
  - matured-vision-day2-requirements.md §13 (static vs dynamic connectors)
  - matured-vision-day2-requirements.md §14 (detection-as-query)
provenance: >
  2026-06-25 side-analysis addendum — day-2 UI design. Produced by ux-designer.
  do_not_execute. Separate from the live factory pipeline.
  Pending PO + architect adjudication at day-2 morph time.
  All content is PROPOSED/DRAFT.
note: >
  This spec covers the S2 full browser investigations console (E-UI-CONSOLE-001).
  The U1 admin/ops console is in S2-investigations-console.md's companion doc,
  U1-admin-console-inventory.md. The S3 server-hosted agent runtime (E-UI-EMBEDDED-AI-001)
  is architecturally sketched in §11.3.2 of the matured-vision doc; its UI manifestations
  (chat sidecar, inline actions) are designed here where they surface in S2 screens.
---

# S2 — Full Browser Investigations Console
## UX Specification (PROPOSED/DRAFT)

> All content in this document is PROPOSED/DRAFT, pending PO + architect adjudication
> at day-2 morph time. Decision references are to matured-vision-day2-requirements.md
> sections and ui-requirements-2026-06-25.md decision IDs.

---

## 0. Cross-Screen Architecture

### 0.1 Information Architecture Map

```
╔══════════════════════════════════════════════════════════════════════╗
║  GLOBAL SHELL (persistent across all S2 screens)                    ║
║  ┌──────────────────────────────────────────────────────────────┐   ║
║  │  [≡] Prism  | Investigations | Detections | Dashboards | ... ║   ║
║  │             | Query   Sources | Admin (RBAC-gated)           ║   ║
║  │  [org: Acme MSSP ▾]  [search/command ⌘K]  [AI ✦]  [🔔] [👤]  ║   ║
║  └──────────────────────────────────────────────────────────────┘   ║
║                                                                      ║
║  MAIN CONTENT AREA (screen-specific)                                 ║
║                                                                      ║
║  AI CHAT SIDECAR (S3, §11.3.2) — collapsible right panel            ║
╚══════════════════════════════════════════════════════════════════════╝
```

**Top-level nav sections:**
| Section | Screens |
|---------|---------|
| Investigations | Triage Queue → Case Detail → Entity 360 |
| Query | Investigation Workspace → Results Explorer → Saved Queries |
| Detections | Rule Editor + Library → Findings/Alerts Queue |
| Dashboards | Posture Dashboards / Summary Insights → Reporting |
| Data | Cache/Retention Browser → Sources/Connectors → Satellite/Topology |
| Admin | (U1; RBAC-gated — see U1-admin-console-inventory.md) |

### 0.2 Global Layout Shell

**Persistent elements across all screens:**

1. **Top navigation bar**
   - Org context switcher (multi-tenant; only shows orgs the analyst has access to)
   - Global command palette trigger (`⌘K` / `Ctrl+K`) — keyboard-driven nav to any screen, entity, or query
   - AI chat sidecar toggle button (S3 — labeled "AI" with robot/spark icon; always machine-labeled, never ambiguous)
   - Notification bell (findings/alerts; badge count)
   - User menu (profile, settings, sign out)

2. **Left navigation** (collapsible to icon-only on narrow viewports)
   - Section groupings per IA map above
   - Active screen highlighted
   - Keyboard: `⌘[1-9]` for top-level nav, `?` for keyboard shortcut help

3. **AI Chat Sidecar** (S3 integration — §11.3.2; UI-D2 trust-first AI UX)
   - Collapsible right panel, ~320px wide
   - Always labeled "Prism AI (Beta)" — never pretends to be a human
   - Every AI response: shows the exact PrismQL query it generated (editable inline), links to the source evidence in the Results Explorer, and records the action in the case wall if a case is open
   - Human approval required before any impactful action (add note to case, run suggested query as new investigation, change alert disposition)
   - Inline suggestions also appear in-context on screens (e.g., "AI suggests: check Entity 360 for this IP") — always dismissible

4. **Per-source coverage banner** (§3.6; UI-D3) — global component rendered by the Results Explorer and wherever federated results appear. Color-blind-safe: green check + "Answered" / amber triangle + "Degraded" / red X + "Timed-out" — icon + text label, never color alone.

5. **Responsive breakpoints**
   - XL (≥1440px): full three-column layout where applicable
   - LG (1200–1439px): two-column; sidecar collapses over content
   - MD (768–1199px): single-column; left nav auto-collapses to icon rail
   - SM (<768px): mobile-optimized; nav in bottom sheet; read-only mode (no query editing on mobile)

### 0.3 Tier-1 vs Tier-3 Default Modes (UI-D1, §3)

The console ships two default modes switchable per-user in settings:
- **Guided/Triage mode** (tier-1 default): pre-configured dashboards, AI summaries prominently surfaced, PrismQL editor hidden behind "Advanced Query" toggle, entity cards emphasize risk score and recommended actions
- **Analyst/Hunting mode** (tier-3 default): PrismQL editor front-and-center, raw results, detection rule editor accessible, cache/retention browser visible in nav

These are presentation defaults, not permission boundaries. Role-based permission is separate (U1 RBAC).

---

## Screen 1 — Triage Queue

**Traces to:** ui-requirements §1.1; E-UI-CONSOLE-001; UI-D1, UI-D3, UI-D6

### Purpose & Persona

The entry point for an on-shift analyst starting their day. Tier-1 analysts use this as their primary work surface. Tier-3 analysts pass through to deeper investigation screens.

**Persona:** SOC Tier-1/2 analyst; SOC manager (summary view); Incident Responder (picking up a case).

### Layout

```
┌─────────────────────────────────────────────────────────────────┐
│  Triage Queue          [+ New Case]    [Filters ▾]  [View ▾]   │
│  ─────────────────────────────────────────────────────────────  │
│  COVERAGE STATUS BAR (global per-source coverage — §3.6)        │
│  ✓ CrowdStrike · ✓ Cyberint · △ Armis (degraded) · ✓ Claroty   │
│  ─────────────────────────────────────────────────────────────  │
│  [Severity ▾][Status ▾][Owner ▾][Source ▾][Time ▾][Saved View▾] │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ ● CRIT  CVE-2026-1234 Exploit — acme-dc-01    unowned    │   │
│  │         3 sources · 2h ago · Risk 94           [Assign▾] │   │
│  ├──────────────────────────────────────────────────────────┤   │
│  │ ▲ HIGH  Lateral movement — user alice@…       jsmith     │   │
│  │         2 sources · 45m ago · Risk 78          [View]    │   │
│  ├──────────────────────────────────────────────────────────┤   │
│  │ ▲ HIGH  Brute force — 10.0.5.22               unowned    │   │
│  │         1 source (Armis degraded) · 1h ago               │   │
│  │         △ Partial coverage — Armis data missing           │   │
│  └──────────────────────────────────────────────────────────┘   │
│  [Load more]                                                     │
└─────────────────────────────────────────────────────────────────┘
```

### Key Components

- **Table/list rendering:** TanStack Virtual (virtualized, handles 10k+ rows without pagination overhead)
- **Saved views:** named filter sets (per-user + shared tenant-wide); Quick Access pinned at top
- **View modes:** Card (default for tier-1; summary-first), Compact (density for tier-3), Table (sortable columns — AG Grid)
- **Severity indicators:** icon + text label + color (not color-only; icons: ⬤ critical, ▲ high, ■ medium, ◆ low; WCAG color-blind-safe palette) — UI-D3
- **Source coverage inline:** when a case has partial source coverage, an amber △ badge appears inline on the card with tooltip "Armis degraded — results may be incomplete" (§3.6)

### States

| State | Rendering |
|-------|-----------|
| Loading | Skeleton rows (10 shimmer cards); no spinner blocking the whole screen |
| Empty (no items match filter) | "No cases match your current filters. [Clear filters]" |
| Empty (fresh tenant) | Onboarding prompt: "No alerts yet. [Configure connectors] or [Run your first query]" |
| Streaming (new alert arrives) | New row slides in at top; badge "+N new" if user has scrolled down; click to jump |
| Partial coverage | △ badge on affected cases; global coverage banner shows degraded sources |
| No permission | "You don't have permission to view this organization's cases. Contact your admin." |
| Error | "Could not load cases. [Retry]" with error detail expandable |

### Interactions

- Click row → navigate to **Case Detail** (Screen 2), preserving scroll position (browser history)
- Assign owner → inline dropdown; saves immediately; audit-logged
- Quick filter bar: type to filter by entity, IP, hostname, rule name — debounced 300ms
- Keyboard: `j/k` to move between rows, `Enter` to open, `/` to focus search, `a` to assign
- Bulk select (checkbox) → bulk status change, bulk assign, bulk export
- Right-click row → context menu: "View Case", "Find related alerts", "Search entity in PrismQL", "Add to existing case"
- One-click pivot: click an entity name (IP/hostname/user) in a row → opens **Entity 360** in a slide-over panel without leaving the queue

### Trust-First AI Hooks (UI-D2)

- AI-generated "Attack Narrative" card visible if S3 is enabled: collapsed by default, expand to see summary. Header shows "AI" label. Footer shows "Generated from: [view PrismQL query] · [view source events]"
- AI-suggested priority reordering: "AI suggests 3 unowned CRIT cases may be related — [view suggestion]" — presented as suggestion, not auto-applied

### Federation/OCSF Elements (UI-D6)

- Coverage bar at top of queue: one indicator per configured source
- Cases sourced from multiple sources show a multi-source badge ("3 sources")
- Source-degraded cases clearly marked — analyst never assumes complete picture

### Accessibility (UI-D3)

- Full keyboard navigation; focus ring visible on all interactive elements
- Screen reader: list roles (`role="list"`) with meaningful aria-labels per row
- Color-blind-safe: severity uses icons + text, not color alone
- High contrast mode: no information lost when system high-contrast is active

---

## Screen 2 — Case / Incident Detail

**Traces to:** ui-requirements §1.2; E-UI-CONSOLE-001; UI-D1, UI-D2, UI-D3, UI-D6

### Purpose & Persona

The primary unit of work. An analyst spends most of their investigation time here. Breadth-first summary for tier-1; deep-dive tabs for tier-2/3.

**Persona:** All analyst tiers; SOC manager (read-only oversight).

### Layout

```
┌──────────────────────────────────────────────────────────────────┐
│ ← Back to Queue                                                  │
│                                                                  │
│ CASE HEADER                                                      │
│ [● CRIT] [OPEN] CVE-2026-1234 Exploit — acme-dc-01              │
│ Risk: 94  |  Created 2h ago  |  Owner: jsmith  |  [▾ Actions]   │
│ Tags: [lateral-movement] [cve-exploit] [+ Add tag]              │
│                                                                  │
│ COVERAGE STATUS (§3.6):                                          │
│ ✓ CrowdStrike  ✓ Cyberint  △ Armis (degraded since 1h)  ✓ Claroty│
│                                                                  │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ Overview │ Evidence │ Entities │ Timeline/Case Wall │        │ │
│ │ Actions  │ AI Insights                                       │ │
│ └─────────────────────────────────────────────────────────────┘ │
│                                                                  │
│ [TAB CONTENT AREA — see sub-specs below]                         │
└──────────────────────────────────────────────────────────────────┘
```

### Key Components

- **Case header:** always visible regardless of active tab; sticky on scroll
- **Tab bar:** AG Grid / standard React tabs; keyboard accessible (arrow keys, Tab)
- **Coverage bar:** persistent below header; communicates data completeness (§3.6)

### Tab: Overview

```
┌─────────────────────────────────────────────────────────────────┐
│ AI ATTACK NARRATIVE (UI-D2 — clearly labeled "AI Summary")      │
│ "acme-dc-01 was accessed by a process matching CVE-2026-1234    │
│  exploit signatures at 14:22 UTC. Lateral movement to..."       │
│ [View PrismQL query] [View source events] [Edit/Override]        │
│                                                                  │
│ KEY FACTS                         MITRE ATT&CK MAP               │
│ First seen: 14:20 UTC             [TA0001][TA0002][TA0008]       │
│ Last seen:  16:18 UTC             Highlighted: T1059, T1021      │
│ Affected entities: 3              (ECharts heatmap, interactive) │
│ Alert count: 12                                                  │
│                                                                  │
│ RISK SCORE BREAKDOWN                                             │
│ [horizontal bar per contributing factor]                         │
│                                                                  │
│ RECOMMENDED ACTIONS (AI — labeled; requires human approval)      │
│ ☐ Isolate acme-dc-01 [Review & Approve]                         │
│ ☐ Search for related activity on 10.0.1.5 [Run in Query]        │
└─────────────────────────────────────────────────────────────────┘
```

- **MITRE ATT&CK heatmap:** ECharts heatmap; click a technique → filters Evidence tab to events matching that technique
- **AI Summary:** labeled, evidence-linked, editable. "Edit/Override" lets analyst replace the AI narrative with their own text (recorded in case wall as "Analyst replaced AI summary at 16:45 UTC by jsmith")
- **Recommended actions:** shown as checklist, NOT auto-executed. Each has [Review & Approve] button that opens a confirmation dialog before anything runs

### Tab: Evidence

```
┌─────────────────────────────────────────────────────────────────┐
│ [Type ▾: All / Alert / Event / Artifact]  [Source ▾]  [Time ▾]  │
│                                                                  │
│ AG Grid (virtualized — TanStack Virtual for 10k+ rows)           │
│ time | source | type | entity | summary | severity | actions     │
│ ─────────────────────────────────────────────────────────────── │
│ 14:22 CrowdStrike  Alert  acme-dc-01  ExploitBlocked  CRIT  [▸] │
│ 14:23 CrowdStrike  Event  acme-dc-01  ProcessSpawn    HIGH  [▸] │
│ 14:25 Claroty     Event  PLC-Zone-A  Comm anomaly     MED   [▸] │
└─────────────────────────────────────────────────────────────────┘
│ ▸ expand row → full OCSF normalized record (JSON tree viewer)    │
│ right-click entity → FIND pivot / Entity 360 / Add to evidence   │
│ [+ Link external artifact]  [Export CSV]                         │
```

- **AG Grid** for the evidence table; grouped by source, expandable
- **Row drill-in:** expands to full OCSF JSON; optional "raw record" toggle to see original pre-normalization record
- **Entity pivot:** right-click any entity value → "Search in PrismQL (FIND ip '...')" → opens Investigation Workspace pre-populated (§12.1)
- **Partial coverage note** inline: rows from degraded sources show △ badge; row from timed-out source shows placeholder "Armis data unavailable for this time window"

### Tab: Entities

```
┌─────────────────────────────────────────────────────────────────┐
│ Entities in this case (4)  [Filter by type ▾]                   │
│                                                                  │
│ ┌─────────────────┐  ┌─────────────────┐  ┌──────────────────┐ │
│ │ HOST            │  │ USER            │  │ IP               │ │
│ │ acme-dc-01      │  │ alice@example   │  │ 10.0.1.5         │ │
│ │ Risk: 94        │  │ Risk: 72        │  │ Risk: 45         │ │
│ │ 3 sources       │  │ 2 sources       │  │ 1 source         │ │
│ │ [Open 360]      │  │ [Open 360]      │  │ [Open 360]       │ │
│ └─────────────────┘  └─────────────────┘  └──────────────────┘ │
│                                                                  │
│ ENTITY RELATIONSHIP GRAPH (Cytoscape.js)                         │
│ [graph canvas: nodes = entities, edges = observed interactions]  │
│ [zoom/pan, click node → Entity 360 slide-over]                  │
│                                                                  │
│ [+ Add entity to case]                                           │
└─────────────────────────────────────────────────────────────────┘
```

- **Cytoscape.js** for the entity graph; force-directed layout; cross-source edges shown with different edge styles (OCSF source = solid, inferred = dashed)
- Entity cards show aggregate risk + source count
- [Open 360] opens Entity 360 (Screen 3) as a slide-over without leaving Case Detail

### Tab: Timeline / Case Wall

```
┌─────────────────────────────────────────────────────────────────┐
│ [Toggle: Events | Analyst Actions | All]  [Time range ▾]        │
│                                                                  │
│ ── 2026-06-25 14:20 UTC ─────────────────────────────────────── │
│ 🔴 14:20  CrowdStrike ALERT: ExploitBlocked on acme-dc-01        │
│                                                                  │
│ ── 2026-06-25 14:30 UTC ─────────────────────────────────────── │
│ 👤 14:30  jsmith opened case and set severity = CRITICAL          │
│ 🤖 14:32  AI Insight: "Possible lateral movement pattern found"  │
│           [View PrismQL] [View evidence]  (machine-labeled)      │
│                                                                  │
│ ── 2026-06-25 15:00 UTC ─────────────────────────────────────── │
│ 🔵 15:00  jsmith ran query: SELECT * FROM federated WHERE…       │
│           [View query] [View results]                            │
│ 📎 15:05  jsmith added note: "Verified: acme-dc-01 patch missing"│
│                                                                  │
│ [+ Add note]  [+ Link alert]                                     │
└─────────────────────────────────────────────────────────────────┘
```

- Chronological log of both events AND analyst actions (and AI actions, always labeled with 🤖)
- AI actions are distinguishable at a glance — icon, label, never just text
- Every PrismQL run by the analyst is logged here with a link to replay the query
- "Add note" → free-text markdown editor; note saved with timestamp + analyst identity
- Immutable log (audit discipline); notes can be appended but not edited/deleted after save

### Tab: Actions

- Lists available response actions (send to SOAR, open ticket in ServiceNow/Jira, send notification)
- Each action shows: what it will do, who approves, what will be logged
- Impactful actions (isolate host, disable account) have a two-step confirmation dialog: "This will [X]. Type the hostname to confirm: ___" — requires re-entry, not just click
- All actions logged in Timeline/Case Wall
- Feature-flag gated: write/response actions only available when the feature flag is enabled (project memory: writes gated)

### Tab: AI Insights

- Dedicated view of all AI-generated content for this case
- Every item: query that generated it, link to source events, timestamp, model used (provider info shown if configurable)
- Analyst can dismiss an insight, mark it incorrect (feeds feedback loop), or promote it to a note on the case wall
- "Generate new analysis" button: asks the S3 agent to re-analyze (may be rate-limited per tenant)

### States (Case Detail)

| State | Rendering |
|-------|-----------|
| Loading | Tab content skeleton; header loads first (title + severity always visible quickly) |
| Streaming updates | New evidence rows stream in; case wall live-updates; badge on tab |
| Partial source coverage | Persistent banner under case header; affected tabs show △ |
| No permission | Read-only view; edit controls hidden; clear "View only" badge |
| Case closed | Read-only; "Reopen" button if role permits |
| Error loading tab | Tab-local error message + retry; other tabs continue to work |

### Trust-First AI Hooks (UI-D2)

- All AI content labeled "AI" + model name (never anonymous)
- "Explain this" button on any AI claim opens a side panel showing: the PrismQL query, the matched OCSF records, the reasoning (if model supports chain-of-thought)
- Human approval gates on all impactful actions the AI might suggest (see Actions tab)
- Case wall records EVERY AI action with attribution

### Accessibility (UI-D3)

- Tab navigation via keyboard arrow keys; focus returns to tab bar on activation
- Timeline: semantic `<article>` elements per entry with time as `<time datetime="..."/>`
- Evidence grid: AG Grid accessibility mode; screen reader announces sort changes
- Relationship graph: keyboard-navigable node list as alternative to mouse-driven canvas

---

## Screen 3 — Entity 360 / Observable Profile

**Traces to:** ui-requirements §1.3; §12.1 (FIND pivot); E-UI-CONSOLE-001; UI-D1, UI-D6

### Purpose & Persona

A unified profile for any observable (IP, user, host, domain, file hash, CVE). Aggregates cross-source data via OCSF normalization. Sentinel's three-pane model adapted for prism's multi-source federated structure.

**Persona:** Tier-2/3 analyst doing entity-centric investigation; threat hunter; IR analyst.

### Layout

```
┌──────────────────────────────────────────────────────────────────┐
│ Entity: 10.0.1.5 (IP Address)      Risk: 78  [FIND in PrismQL]  │
│ Last seen: 16:18 UTC  |  Sources: CrowdStrike (✓), Claroty (✓), │
│                                    Armis (△ degraded)            │
├──────────────────────┬───────────────────────┬───────────────────┤
│  IDENTITY / PROFILE  │  EVENT TIMELINE       │  BEHAVIORAL       │
│  (left ~280px)       │  (center, flex)       │  INSIGHTS         │
│                      │                       │  (right ~240px)   │
│  Type: IP address    │ ◄──── time ──────►    │                   │
│  CIDR block: /24     │                       │  Risk score: 78   │
│  Owner: IT (CMDB)    │ 14:20 ALERT(CS)       │  Trend: ↑ +12     │
│  Asset: acme-dc-01   │   ExploitBlocked       │                   │
│  Hostname: dc-01     │ 14:25 EVENT(Claroty)  │  MITRE: T1059     │
│  Org: Acme Corp      │   CommAnomaly          │                   │
│                      │ 14:30 (Armis missing) │  Related entities │
│  First seen: 2026-01 │   △ degraded           │  - acme-dc-01     │
│  Cases: 3 open       │                       │  - alice@acme     │
│  Findings: 12 (7d)   │ 14:45 EVENT(CS)        │                   │
│                      │   ProcessSpawn         │  AI assessment:   │
│  [FIND all events]   │                       │  "High risk —     │
│  [Open related case] │ [Load more events]    │  matches IOC..."  │
│  [Add to case]       │                       │  [View evidence]  │
└──────────────────────┴───────────────────────┴───────────────────┘
│ RELATED ENTITIES (bottom, full width)                             │
│ Cytoscape.js graph: this entity + 1-hop neighbors, cross-source  │
│ [Expand to 2-hop]  [Filter by source]  [Export as evidence]      │
└──────────────────────────────────────────────────────────────────┘
```

### Key Components

- **Three-pane layout:** Identity panel (left, fixed width), Event Timeline (center, virtualized), Behavioral Insights (right, fixed width)
- **Event timeline:** TanStack Virtual (handles 10k+ events); grouped by source, color-blind-safe source indicators (icon per source + source name label)
- **Entity relationship graph:** Cytoscape.js; 1-hop default, expandable to 2-hop; cross-source edges
- **FIND pivot button** (§12.1): "FIND in PrismQL" → navigates to Investigation Workspace with pre-populated `FIND ip '10.0.1.5' SINCE 24h`; opens in a new browser tab so Entity 360 isn't lost
- **Source coverage** per event on timeline: events from degraded sources show △; time gaps where a source was down shown as empty spans with a label "Armis data unavailable 14:00–15:00 UTC"

### States

| State | Rendering |
|-------|-----------|
| Loading | Three-pane skeleton; identity panel loads first (fast, from cache) |
| Streaming events | Timeline auto-appends new rows at top; scroll position preserved |
| Partial source coverage | △ badges on affected events; coverage indicator per source in header |
| No events in time range | "No events found in this time range. [Expand range]" in timeline center |
| Entity not found | "No profile found for this observable. [Search in PrismQL]" |
| Error from specific source | Inline error note in timeline at the point the source data was missing |

### Interactions

- Time range picker in timeline: draggable brush selection (ECharts-style sparkline above timeline + drag-to-zoom)
- Click event in timeline → drill into full OCSF record (slide-over drawer)
- Click related entity in graph → slide-over Entity 360 for that entity (breadcrumb trail maintained)
- "Add to case" → entity is linked to the current open case (or prompts to create new case)
- Entity type toggle: if an entity appears under multiple types (e.g. same IP is both a source endpoint and a device IP), a type switcher shows

### Trust-First AI Hooks (UI-D2)

- "AI assessment" in right pane: brief AI-generated risk narrative with "View evidence" link
- Threat-intel enrichment: if a TI source labels this entity as malicious, shows TI source name + link + confidence; NOT presented as "AI says this is bad" but as "TI source X reports this IP as C2 infrastructure with confidence 94%"

### Federation/OCSF Elements (UI-D6)

- All event data is OCSF-normalized at query time; field labels use OCSF names (not source-native field names)
- Cross-source edges in the entity graph are the core differentiator: an IP appearing in CrowdStrike events AND Claroty OT events AND Armis device inventory shows edges to all three, building a richer picture than any single source

### Accessibility (UI-D3)

- Three-pane layout has proper landmark roles (`<aside>`, `<main>`, `<aside>`)
- Timeline list: `role="feed"` with live region updates when new events stream in
- Entity graph: keyboard-navigable node list in a companion `<ul>` (canvas is supplementary)

---

## Screen 4 — Investigation Workspace

**Traces to:** ui-requirements §1.4, §1.5; §11.3.1 (Investigation workspace, Results explorer); §12.1 (FIND pivot); E-UI-CONSOLE-001; UI-D1, UI-D2, UI-D3

### Purpose & Persona

The primary query surface. PrismQL editor with NL toggle, time picker, source selector, and run/cancel. The "workbench" for active investigation, threat hunting, and ad-hoc analysis.

**Persona:** Tier-2/3 analyst (direct query); Tier-1 analyst using NL→PrismQL (AI-assisted mode); Detection engineer (authoring query to become a detection rule).

### Layout

```
┌──────────────────────────────────────────────────────────────────┐
│  Investigation Workspace          [Save Query ▾]  [→ Rule]        │
│  ─────────────────────────────────────────────────────────────── │
│  MODE: [PrismQL ●] [Natural Language ○]    [Tutorial] [History]  │
│  ─────────────────────────────────────────────────────────────── │
│                                                                   │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │  Monaco Editor (PrismQL)                                    │ │
│  │                                                             │ │
│  │  SELECT *                                                   │ │
│  │  FROM federated                                             │ │
│  │  WHERE entity('ip', '10.0.1.5')                             │ │
│  │    AND severity >= 'high'                                   │ │
│  │  SINCE 24h                                                  │ │
│  │                                                             │ │
│  │  ── Line 6, Col 1 ── OCSF v1.3 ── CrowdStrike, Claroty ──  │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                                                                   │
│  ┌────────────────────┬─────────────────────┬──────────────────┐ │
│  │  TIME RANGE        │  SOURCES            │  OPTIONS         │ │
│  │  [SINCE 24h ▾]     │  ☑ CrowdStrike      │  [Row limit ▾]   │ │
│  │  Custom: from–to   │  ☑ Cyberint         │  [Format ▾]      │ │
│  │                    │  ☑ Claroty          │  [EXPLAIN]       │ │
│  │                    │  △ Armis (degraded) │                  │ │
│  │                    │  [+ Add source]     │                  │ │
│  └────────────────────┴─────────────────────┴──────────────────┘ │
│                                                                   │
│  [▶ Run Query]  [■ Cancel]                    Keyboard: ⌘↵ / F5  │
└──────────────────────────────────────────────────────────────────┘
```

### Key Components

- **Monaco Editor** with PrismQL language server:
  - Syntax highlighting (PrismQL keywords, OCSF field names, string literals, comments)
  - OCSF schema-aware autocomplete (field names, entity types, operator keywords, `SINCE` durations)
  - Inline error markers (red squiggles) with hover tooltip showing error code + suggested fix
  - `EXPLAIN` mode: inline annotation showing which predicates push down to which sources
  - Keyboard: `⌘↵` or `F5` to run; `⌘/` to comment/uncomment; `⌘Shift+F` to format
- **Natural Language mode** (S3 AI, §11.3.2): replaces Monaco with a text input; S3 agent generates PrismQL and shows it in a "Generated query" panel (always visible, always editable before running); analyst can switch back to PrismQL mode to edit the generated query — UI-D2 trust-first
- **Time picker:** preset durations (SINCE 15m, 1h, 6h, 24h, 7d) + custom date/time range; timezone displayed; UTC default with per-user override
- **Source selector:** shows all configured sources with health status; degraded sources have △ badge; unchecked sources are excluded from the query
- **[→ Rule] button:** converts the current query to a detection rule stub, navigating to Rule Editor (Screen 6) with the query pre-populated

### States

| State | Rendering |
|-------|-----------|
| Loading (schema) | Monaco editor shows "Loading schema..." in status bar; autocomplete unavailable until ready |
| Query running | Run button becomes Cancel; progress indicator in status bar; streaming results appear in Results Explorer (Screen 5) below/adjacent |
| Query cancelled | Results cleared; "Query cancelled" toast; editor restored to active |
| Query error (syntax) | Inline error in Monaco; error message in results area |
| Query error (execution) | Structured error message with E-QUERY-NNN code; "View details" expands the full error |
| NL mode — generating | Spinner + "Generating PrismQL…" while S3 agent is working; Cancel available |
| Source degraded | △ badge on source in selector; warning: "Results from Armis may be incomplete or missing" |
| Join guard triggered (§12.2) | Error banner: "Cross-source join requires a selective key predicate. Add a join key or [learn more]." |
| Mandatory time bound missing | Warning (not error for analyst override): "No time predicate detected — a default window of SINCE 24h has been applied." |

### Interactions

- Run → results stream into Results Explorer (Screen 5) rendered below the editor in a split pane (resizable)
- Save Query → saves to personal query library; optionally share with tenant; name + description prompt
- History → right panel shows last 20 queries for this session; click to restore any
- EXPLAIN toggle → re-renders the editor output showing pushdown annotations without running
- Keyboard shortcut sheet accessible via `?`

### Trust-First AI Hooks (UI-D2)

- NL mode: always shows the generated PrismQL query before running; "Edit before running" is the default workflow
- S3 agent suggestions ("Try adding a time filter for better performance") appear as non-blocking inline hints in the editor gutter; never auto-applied
- If the AI modifies a query (e.g., injects a mandatory time bound), the modification is annotated: "Time bound added by Prism AI — [remove]"

### Accessibility (UI-D3)

- Monaco has built-in keyboard accessibility; screen reader mode selectable
- Time picker: date/time inputs as `<input type="datetime-local">` with ARIA labels; no calendar-only selection
- Source selector: checkboxes with labels; group labeled by region/type if applicable
- Status bar reads key state changes to screen readers via aria-live

---

## Screen 5 — Results Explorer

**Traces to:** ui-requirements §1.5; §3.6 (partial-result semantics); §12.1 (FIND/observable pivot); E-UI-CONSOLE-001; UI-D1, UI-D3, UI-D6

### Purpose & Persona

The results view for any executed query. Shows OCSF-normalized federated results with per-source coverage, event/entity mode toggle, row drill-in, entity pivots, and export.

**Persona:** All analyst tiers during active investigation; detection engineer reviewing a candidate rule's output.

### Layout

```
┌──────────────────────────────────────────────────────────────────┐
│  Results Explorer                    286 rows · 3 sources        │
│  ─────────────────────────────────────────────────────────────── │
│  ┌──────────────────────────────────────────────────────────────┐│
│  │  SOURCE COVERAGE BANNER (§3.6)  — always visible             ││
│  │  ✓ CrowdStrike (186 rows)  ✓ Claroty (100 rows)             ││
│  │  △ Armis — DEGRADED: returned 0 rows (timeout after 5s)     ││
│  │  [View Armis error]  [Re-run with Armis only]                ││
│  └──────────────────────────────────────────────────────────────┘│
│                                                                   │
│  MODE: [Event mode ●]  [Entity mode ○]   [▾ Columns]  [Export ▾] │
│                                                                   │
│  AG Grid (virtualized; TanStack Virtual for 10k+ rows)           │
│  ─────────────────────────────────────────────────────────────── │
│  time       source     event_type      entity         severity   │
│  14:20 UTC  CrowdStr.  ExploitBlock…  10.0.1.5        CRITICAL   │
│  14:21 UTC  CrowdStr.  ProcessSpawn   acme-dc-01      HIGH       │
│  14:25 UTC  Claroty    CommAnomaly    PLC-Zone-A      MEDIUM     │
│  …                                                               │
│                                                                   │
│  [Load more rows]            Status: Streaming complete (286/286) │
└──────────────────────────────────────────────────────────────────┘
```

### Key Components

- **Per-source coverage banner** (§3.6; UI-D3; UI-D6): always at the top of results; shows per-source row counts + status (answered/degraded/timed-out); color-blind-safe (icon + text, never color alone); "DEGRADED" source shows error detail on expand
- **AG Grid** for Event mode: column sorting, reordering, pinning; row virtualization via TanStack Virtual; 10k+ rows without performance degradation
- **Entity mode:** aggregates results by entity type; shows entity + occurrence count + severity distribution + sources — uses TanStack Virtual cards
- **Column picker** (§11.3.1): show/hide OCSF fields; OCSF field names displayed with friendly labels (e.g., "Source IP (src_endpoint.ip)")
- **Row drill-in:** click any row → expands a details drawer showing the full OCSF-normalized record as a structured tree; toggle "Show raw record" to compare pre-normalization original
- **Observable pivot** (§12.1): right-click on any IP, user, host, domain, hash, or CVE value → "FIND [value] in PrismQL" → opens Investigation Workspace with `FIND ip '...' SINCE 24h`; middle-click opens in new tab

### States

| State | Rendering |
|-------|-----------|
| Loading / streaming | Rows appear as they arrive from sources; progress per source shown in coverage banner ("CrowdStrike: streaming…"); skeleton rows for unreceived sources |
| Partial coverage (degraded source) | Coverage banner shows △ for degraded source; results below are marked "Note: Armis data absent" |
| Empty results | "No results. The query ran successfully but found no matching events." + coverage status to confirm all sources answered |
| Empty due to all sources degraded | Coverage banner shows all sources as △/✗; "Could not retrieve results — all sources unavailable" |
| Streaming complete | Status bar: "Streaming complete (286/286 rows)" |
| Large result set (>100k) | Warning: "Large result set detected (>100,000 rows). Showing first 10,000 — [Download full results as CSV]" |
| Error in row rendering | Row-level error marker; rest of grid continues to work |

### Interactions

- Sort: click column header; secondary sort with `Shift+click`
- Filter inline: type in filter row below column header (AG Grid built-in)
- Row expand: click row OR press `Enter` on focused row
- Entity pivot (§12.1): right-click entity cell value → context menu with FIND pivot option
- Export: CSV (all visible columns), JSON (OCSF-normalized full records), Excel; capped at 50k rows for browser export; [Request background export] for larger sets
- "Re-run with [source] only": re-executes the query scoped to a single source — useful for debugging which source contributed which data
- Add selected rows to case evidence: multi-select rows → [+ Add to Case evidence]

### Trust-First AI Hooks (UI-D2)

- If S3 agent is open and a query was AI-generated, "AI ran: [view original NL input] → [view generated PrismQL]" shown above the coverage banner
- No AI automatic interpretation of results — the results are shown as-is; AI commentary is in the sidecar, never embedded in the grid cells

### Federation/OCSF Elements (UI-D6)

- Source column identifies the origin source for every row; cross-source rows in Entity mode show all contributing sources
- OCSF field names are the canonical display names; hover on a field name shows the full OCSF path (e.g., `src_endpoint.ip`) as a tooltip
- Coverage banner is the primary federation UX differentiator — analyst always knows the completeness of what they are seeing

### Accessibility (UI-D3)

- AG Grid accessibility mode: `role="grid"`, keyboard cell navigation (arrow keys, Tab, Enter)
- Coverage banner: uses `role="status"` for live announcements when streaming completes or a source degrades
- Expandable rows: keyboard-accessible (Enter to expand/collapse)
- Export dialog: standard form controls with proper labels

---

## Screen 6 — Detection Rule Editor + Library

**Traces to:** ui-requirements §1.6; §14 (detection-as-query); §14.4 (rule editor surfaces); §14.1, §14.2; E-DETECT-EDITOR-001; UI-D1, UI-D2, UI-D3

### Purpose & Persona

Author, test, manage, and version detection rules. Detection-as-query: a rule IS a scheduled PrismQL query with YAML metadata. The rule library is searchable, filterable by MITRE tactic/technique.

**Persona:** Detection engineer (primary); Tier-3 analyst with detection authoring permissions.

### Layout (Library View)

```
┌──────────────────────────────────────────────────────────────────┐
│  Detection Rules         [+ New Rule]  [Import Sigma]            │
│  ─────────────────────────────────────────────────────────────── │
│  [Search rules…]  [Tactic ▾]  [Technique ▾]  [Status ▾]         │
│  [Source ▾]        [Severity ▾]   [Quality ▾]                    │
│                                                                   │
│  MITRE ATT&CK COVERAGE (ECharts heatmap — tiles by technique)    │
│  [Tactics across top; technique rows below; active rules shown]  │
│                                                                   │
│  ─────────────────────────────────────────────────────────────── │
│  ● credential_theft   Production  CRIT  T1003 · T1059  jsmith   │
│    "Mimikatz + LSASS access + kdbx write sequence"  [Edit][Test] │
│                                                                   │
│  ▲ brute_then_success HIGH  T1110  jdoe  shadow  [Edit][Test]    │
│    "Failed auth burst followed by success — same user+IP"        │
│                                                                   │
│  ◆ port_scan          MED   T1046  system production  [Edit]     │
│    "High-rate port scan from internal host"                       │
└──────────────────────────────────────────────────────────────────┘
```

### Layout (Rule Editor View)

```
┌──────────────────────────────────────────────────────────────────┐
│ ← Rules Library          credential_theft (v1.2.0 · Production)  │
│ [Edit Metadata]  [Test]  [Backtest]  [Status: Production ▾]      │
│ ─────────────────────────────────────────────────────────────── │
│ SPLIT PANE:                                                       │
│ ┌────────────────────────────────┬────────────────────────────┐  │
│ │  RULE METADATA (left)          │  PRISMQL LOGIC (right)     │  │
│ │  id: credential_theft          │  Monaco Editor (PrismQL)   │  │
│ │  name: Credential theft…       │                             │  │
│ │  severity: CRITICAL            │  DETECT credential_theft   │  │
│ │  status: production            │    SEQUENCE BY user.name   │  │
│ │  mitre:                        │    WITHIN 30m              │  │
│ │    - TA0006/T1003              │    STEP a: process.name    │  │
│ │    - TA0002/T1059              │      = 'mimikatz.exe'      │  │
│ │  schedule: "*/5 * * * *"       │    THEN b: access.type…    │  │
│ │  window: "30m"                 │    THEN c: file.path…      │  │
│ │  group_by: [user.name]         │    EMIT user.name,…        │  │
│ │  false_positives:              │                             │  │
│ │    - "Pen test"                │  [EXPLAIN pushdown]        │  │
│ │  version: 1.2.0                │  [Desugar to MATCH_RECOG.] │  │
│ └────────────────────────────────┴────────────────────────────┘  │
│                                                                   │
│ BACKTEST PANEL (collapsible)                                      │
│ Run against cache: [Time range ▾]  [Sources ▾]  [▶ Run backtest] │
│ Results: X matches in Y events over Z period                     │
│ TP/FP/Unknown breakdown (if annotated events available)          │
│                                                                   │
│ LIFECYCLE CONTROLS                                                │
│ Status: draft → review → testing → shadow → canary → production  │
│ Current: production  [Demote to canary] [Archive]                │
│ Auto-rollback: if FP rate > 15% in canary, rollback to shadow    │
└──────────────────────────────────────────────────────────────────┘
```

### Key Components

- **Monaco Editor** (PrismQL/DETECT mode): OCSF-aware autocomplete; `SEQUENCE…THEN…WITHIN` sugar supported (§12.4); "Desugar" button shows the equivalent `MATCH_RECOGNIZE` SQL for education
- **MITRE ATT&CK coverage heatmap:** ECharts; click a technique tile → filter library to rules covering that technique; highlight gaps (uncovered techniques)
- **Rule lifecycle state machine:** visual stepper; transitions require appropriate role; promoted/demoted states logged
- **Staged rollout:** shadow (runs but doesn't alert) → canary (alerts on 5% of traffic) → production; auto-rollback on FP spike is configurable
- **Backtest panel:** re-queries historical data (via cache/Iceberg cold tier per §14.3); shows match count, example matches, estimated FP rate if labeled data available
- **Sigma import** (§14.7): drag-and-drop .yml; Sigma rule translated to PrismQL stub + metadata; analyst reviews before saving
- **[→ Rule] from Investigation Workspace** (§12.1): when arriving from Screen 4, the rule is pre-populated with the query; analyst adds metadata

### States

| State | Rendering |
|-------|-----------|
| New rule | Empty Monaco editor with commented template + autocomplete ready |
| Editing existing rule | Monaco pre-filled; "unsaved changes" indicator in tab |
| Test run active | Results stream in below editor; [Cancel test] available |
| Backtest running | Progress bar + coverage banner (which time range, which sources) |
| Validation error (syntax) | Inline Monaco error + "Fix before saving" |
| Validation error (missing required metadata) | Inline field-level error on metadata form |
| Rule in production | "This rule is in production. Changes will require re-promotion. [Save as draft version]" |
| Rule auto-rolled back | Amber banner: "Rule was automatically rolled back from canary due to FP spike at [time]. [View details]" |

### Interactions

- "Convert investigative query → rule" (from Investigation Workspace): query is pre-populated in Monaco; analyst fills metadata
- "Propose rule from AI" (S3, UI-D2): AI suggests a rule based on current case/alert patterns; shows generated DETECT block + metadata; always editable; always requires analyst to explicitly save
- Sigma import: maps to PrismQL stub; MITRE tags preserved; sources mapped to available connectors (unmapped field warning)
- Keyboard: same as Investigation Workspace for Monaco interactions

### Trust-First AI Hooks (UI-D2)

- "AI auto-tune suggestions" (exception suppression, threshold recommendations) shown as non-blocking suggestions; never auto-applied; each suggestion shows rationale + evidence link
- If rule was AI-generated (via S3), that is noted in metadata: `generated_by: prism-ai` with timestamp; analyst can clear this attribution

### Accessibility (UI-D3)

- MITRE heatmap: keyboard-navigable tile grid; each tile has aria-label with tactic + technique name + rule count
- Lifecycle stepper: uses `role="progressbar"` / `role="listbox"` semantics
- Form fields in metadata panel: proper labels, error association via aria-describedby

---

## Screen 7 — Findings / Alerts Queue

**Traces to:** ui-requirements §1.7; §14.5 (alert model, source-coverage record, replay link); §10.3 ADOPT-4; E-UI-CONSOLE-001; UI-D6

### Purpose & Persona

The output queue of the detection engine. Findings are the results of scheduled detection rules. Closely related to the Triage Queue (Screen 1) but focused on raw detection output before case escalation.

**Persona:** Detection engineer (reviewing rule output quality); Tier-1/2 analyst (triaging new findings).

### Layout

```
┌──────────────────────────────────────────────────────────────────┐
│  Findings / Alerts Queue                 [▾ Filters]  [View ▾]  │
│  ─────────────────────────────────────────────────────────────── │
│  [Rule ▾][Severity ▾][Status ▾][Source coverage ▾][Time ▾]       │
│                                                                   │
│  ┌──────────────────────────────────────────────────────────────┐│
│  │ ● CRIT  credential_theft (v1.2.0)             unacknowledged  ││
│  │   14:22 UTC · matched user: alice@acme · 3 steps matched     ││
│  │   Coverage: ✓ CrowdStrike · ✓ Claroty · △ Armis (degraded)  ││
│  │   [View finding]  [Replay window ↩]  [→ Case]  [Dismiss ▾]  ││
│  ├──────────────────────────────────────────────────────────────┤│
│  │ ▲ HIGH   brute_then_success (v1.0.1)          acknowledged    ││
│  │   13:55 UTC · user: bob@acme · failures: 24 → success       ││
│  │   Coverage: ✓ CrowdStrike (all sources answered)             ││
│  │   [View finding]  [Replay window ↩]  [→ Case]               ││
│  └──────────────────────────────────────────────────────────────┘│
└──────────────────────────────────────────────────────────────────┘
```

### Key Components

- **TanStack Virtual** for the findings list
- **Source coverage record** per finding (§14.5; §10.3 ADOPT-4): which sources answered/degraded when the rule ran
- **Replay link** (§10.3 ADOPT-4): "[Replay window ↩]" re-runs the exact detection window (same time range, same sources, same rule version) in Investigation Workspace; shows what the rule saw at firing time
- **Finding detail drawer:** slides in on click; shows matched events, the PrismQL DETECT query that fired, OCSF fields that matched, source coverage, enrichment
- **Disposition:** Acknowledge, Close, False Positive, Escalate to Case; all logged with analyst identity + timestamp

### States

Same pattern as Triage Queue (Screen 1). Additional states:
- **Finding has degraded coverage:** Amber banner in finding detail: "Note: Armis was degraded when this rule fired. The rule may have missed events from that source. [Re-run now]"
- **Rule no longer active:** "This finding was generated by rule v1.0.0. The rule is now at v1.2.0 (production). [View current rule]"

### Interactions

- Replay window: opens Investigation Workspace with the exact frozen query + time bounds; result is a "read-only replay" (not the current live state)
- Escalate to Case: links finding to a new or existing case; opens Case Detail (Screen 2) with finding pre-linked
- Right-click entity in finding → observable pivot (FIND, Screen 3)

### Trust-First AI Hooks (UI-D2)

- AI triage suggestion: if S3 is enabled, a collapsed "AI triage" card per finding shows "AI assessment: [summary] — [View evidence]"; analyst decides whether to act on it

---

## Screen 8 — Posture Dashboards / Summary Insights

**Traces to:** ui-requirements §1.7 (Posture dashboards); §11.3.1 (Summary Insights); E-UI-CONSOLE-001; UI-D1, UI-D6

### Purpose & Persona

Federated metrics over OCSF-normalized data. MITRE coverage heatmap. Time-scoped summary tiles. For management reporting and continuous monitoring.

**Persona:** SOC manager; CISO; Detection engineer (coverage gap analysis).

### Layout

```
┌──────────────────────────────────────────────────────────────────┐
│  Summary Insights — Acme Corp    [Time: Last 7d ▾]  [Export ▾]  │
│                                                                   │
│  COVERAGE STATUS: ✓ CrowdStrike · △ Armis (2h degraded) · ✓ 2   │
│  ─────────────────────────────────────────────────────────────── │
│                                                                   │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────────┐ │
│  │ Open     │  │ New (7d) │  │ MTTD     │  │ Source coverage  │ │
│  │ Cases: 8 │  │ Alerts:  │  │ avg: 4.2h│  │ ✓ 3/4 sources   │ │
│  │ (▲+2)    │  │ 247      │  │          │  │ △ Armis (2h)    │ │
│  └──────────┘  └──────────┘  └──────────┘  └──────────────────┘ │
│                                                                   │
│  ALERT TREND (ECharts line)      RISK DISTRIBUTION (ECharts pie) │
│  [7-day trend; area chart;        [CRIT/HIGH/MED/LOW breakdown;   │
│   per-source breakdown toggle]    color-blind-safe palette]       │
│                                                                   │
│  MITRE ATT&CK COVERAGE HEATMAP (ECharts — full width)            │
│  [tactics across top; techniques as rows; cells show rule count  │
│   + alert count; gray = no coverage; click → filter detections]  │
│                                                                   │
│  TOP ENTITIES AT RISK              RECENT CASES                  │
│  [rank list of IPs/users/hosts     [last 5 open cases; click →   │
│   with risk score; click → 360]    Case Detail]                  │
└──────────────────────────────────────────────────────────────────┘
```

### Key Components

- **ECharts** for all charts (line trend, pie/donut risk distribution, MITRE heatmap)
- **MITRE ATT&CK heatmap**: interactive — click a technique tile → opens Detection Rules library filtered to that technique; shading by coverage: no rules = gray, has rules = green-shade by density
- **Source coverage summary tile**: always visible, shows federated data completeness; degraded sources call-out
- **Dashboard customization** (later): drag-and-drop tile arrangement; add/remove tiles; save custom dashboard layouts (day-2 enhancement, not day-1 requirement)

### States

| State | Rendering |
|-------|-----------|
| Loading | Skeleton tiles; charts load progressively |
| Some sources degraded | Coverage tile shows amber; degraded sources noted in each chart legend where their data is absent |
| All sources healthy | Green coverage indicator |
| No data in time range | Individual tiles show "No data for selected period" |
| Streaming (real-time) | Alert trend chart animates as new alerts arrive |

### Trust-First AI Hooks (UI-D2)

- "AI insight" card: weekly summary narrative (e.g., "This week saw a 40% increase in credential-related alerts. Key contributors: [query link]. [View detailed analysis]"); labeled "AI"; always evidence-linked

### Accessibility (UI-D3)

- All charts have accessible equivalents: data tables reachable via a "Show data table" toggle below each chart
- Color-blind-safe palettes (visx token set: uses shape + texture + label in addition to color)
- MITRE heatmap: full keyboard navigation; each cell has aria-label "Tactic X, Technique Y, N detection rules, M alerts"

---

## Screen 9 — Reporting

**Traces to:** ui-requirements §1.8; E-UI-CONSOLE-001

### Purpose & Persona

Generate PDF reports from cases, dashboards, or custom templates. For management, compliance, and client reporting in MSSP context.

**Persona:** SOC manager; compliance officer; MSSP client account manager.

### Layout

```
┌──────────────────────────────────────────────────────────────────┐
│  Reports                                 [+ New Report]          │
│  ─────────────────────────────────────────────────────────────── │
│  [Type ▾: Case / Dashboard / Custom]  [Template ▾]  [Time ▾]    │
│                                                                   │
│  Report Builder:                                                  │
│  ┌──────────────────────────────────────────────────────────────┐│
│  │  Report title: _________________________________             ││
│  │  Include sections:                                           ││
│  │  ☑ Executive summary    ☑ Case summary table               ││
│  │  ☑ MITRE coverage map   ☑ Alert trend charts               ││
│  │  ☑ Top entities at risk ☑ Source coverage notes            ││
│  │  ☐ Raw findings list    ☐ Query details (technical)        ││
│  │                                                              ││
│  │  [Preview]  [Export PDF]  [Schedule recurring ▾]           ││
│  └──────────────────────────────────────────────────────────────┘│
│                                                                   │
│  RECENT REPORTS                                                   │
│  Weekly SOC Report — 2026-06-23.pdf  [Download] [Resend]         │
│  Incident Report — Case-042.pdf     [Download] [Resend]         │
└──────────────────────────────────────────────────────────────────┘
```

### Key Components

- **PDF export** using a server-side renderer (Prism backend; React component → PDF via headless browser or pdfmake/react-pdf)
- **Source coverage notes automatically included**: any degraded source during the report period is noted in the PDF (never silently omit data gaps)
- **Template system** (day-2 enhancement): pre-built templates for MSSP weekly reports, incident reports, executive summaries
- **Scheduled reports** (day-2): email delivery on a cron schedule; recipient list per tenant (RBAC-gated)

---

## Screen 10 — Saved Queries / Query Library

**Traces to:** §11.3.1 (Saved queries/query library); E-UI-CONSOLE-001

### Purpose & Persona

Repository of saved PrismQL queries. Personal + shared (tenant-wide). Parameterized queries. Threat-hunting recipe library (§14.7).

### Layout

```
┌──────────────────────────────────────────────────────────────────┐
│  Query Library    [+ Save current query]   [Import]  [Export]    │
│  ─────────────────────────────────────────────────────────────── │
│  [Search queries…]  [Type ▾: Personal / Shared / Recipes]        │
│  [Tags ▾]  [MITRE Tactic ▾]  [Author ▾]                          │
│                                                                   │
│  ● RECIPES (community / pre-built)                               │
│  ├─ Credential Access — Mimikatz detection [T1003] [Run] [Copy]  │
│  ├─ Lateral Movement — SMB spread [T1021] [Run] [Copy]           │
│  └─ Network Recon — Port scan sweep [T1046] [Run] [Copy]         │
│                                                                   │
│  ● MY QUERIES                                                     │
│  ├─ alice_brute_force_hunt (jsmith, 2026-06-24) [Run] [Edit] [▾] │
│  └─ splunk_high_risk_assets (shared) [Run] [Edit] [▾]            │
└──────────────────────────────────────────────────────────────────┘
```

### Key Components

- **Recipes tab**: curated PrismQL detection and hunt recipes (§14.7); MITRE-tagged; executable + backtested + version-controlled
- **Parameterized queries**: query parameters as `{{entity}}` placeholders; clicking Run prompts for parameter values
- **[→ Rule]**: saved query can be promoted to a detection rule (links to Screen 6)
- **[→ Investigation]**: click Run → opens Investigation Workspace (Screen 4) with query pre-populated

---

## Screen 11 — Cache / Retention Browser

**Traces to:** §3.3 (demand-driven cache / FROM cache.<name>); §11.3.1 (Cache/retention browser); E-UI-CONSOLE-001

### Purpose & Persona

Visibility into the demand-driven retention cache. Shows what is currently cached, TTL, policy source (which detection rule or RETAIN directive is keeping this data). Browse and query cached result sets.

**Persona:** Detection engineer; advanced analyst.

### Layout

```
┌──────────────────────────────────────────────────────────────────┐
│  Cache / Retention Browser                                        │
│  ─────────────────────────────────────────────────────────────── │
│  Total cached: 1.2 GB / 4 GB (configured)  Used: 30%            │
│  [Tier ▾: Hot (RocksDB) / Cold (Iceberg)]                        │
│                                                                   │
│  ┌──────────────────────────────────────────────────────────────┐│
│  │ cache.alice_brute_hunt  Hot  Expires: 4h  Retained by:       ││
│  │   RETAIN 24h AS alice_brute_hunt (query 14:25 UTC, jsmith)  ││
│  │   Size: 42 MB  Rows: 18,432                                  ││
│  │   [Browse: FROM cache.alice_brute_hunt]  [Extend TTL]  [Del] ││
│  ├──────────────────────────────────────────────────────────────┤││
│  │ detection.credential_theft_window  Hot  Expires: 27m         ││
│  │   Retained by: detection rule credential_theft (30m window) ││
│  │   Size: 8 MB  Rows: 3,204  Policy: detection window          ││
│  │   [Browse]                                                   ││
│  └──────────────────────────────────────────────────────────────┘│
│                                                                   │
│  Cache policy overview:                                           │
│  ● Detection-window retention: 3 active rules, 3 windows         │
│  ● Explicit RETAIN: 2 named result sets                          │
│  ● Config defaults: 4 tables with default TTL                    │
└──────────────────────────────────────────────────────────────────┘
```

### Key Components

- Tier toggle (hot = RocksDB; cold = Iceberg; §3.3 addendum)
- Policy source transparency: every cached dataset shows WHY it is being retained (which detection rule, which RETAIN directive, which config default)
- "Browse: FROM cache.X" → opens Investigation Workspace with `SELECT * FROM cache.alice_brute_hunt LIMIT 100` pre-populated
- Eviction controls (admin-gated): extend TTL, delete manually; all deletions logged

---

## Screen 12 — Sources / Connectors

**Traces to:** §13 (static vs dynamic connectors); §11.3.1 (Sources/connectors); §11.2 (configure-schema wizard); E-UI-CONSOLE-001; UI-D4

### Purpose & Persona

Connector operational surface. List + health for all configured sources. Static vs dynamic connector indicator. Configure-schema wizard for dynamic connectors. Credential rotation shortcut (full credential management in U1 Admin). Admin-gated for config changes; all analysts can view connector health.

**Persona:** Connector-Admin role (config); all analysts (health view).

### Layout

```
┌──────────────────────────────────────────────────────────────────┐
│  Sources / Connectors          [+ Add Connector] (admin-gated)  │
│  ─────────────────────────────────────────────────────────────── │
│  [Health ▾: All / Healthy / Degraded / Down]  [Type ▾]          │
│                                                                   │
│  ┌──────────────────────────────────────────────────────────────┐│
│  │ ✓ CrowdStrike (Static)   Healthy    Avg latency: 320ms       ││
│  │   Last query: 2m ago  |  Type: Security sensor (OCSF)       ││
│  │   [View schema]  [Test connection]  [Rotate credentials ↗]  ││
│  ├──────────────────────────────────────────────────────────────┤│
│  │ △ Armis (Static)  DEGRADED since 1h  Avg latency: timeout   ││
│  │   Error: connect timeout (last attempt 14:55 UTC)            ││
│  │   [Diagnose]  [Retry connection]  [Rotate credentials ↗]    ││
│  ├──────────────────────────────────────────────────────────────┤│
│  │ ✓ Splunk (Dynamic)  Healthy  Avg latency: 1.2s              ││
│  │   Schema: 42 tables mapped to OCSF  [View mapping]           ││
│  │   [Configure schema] (admin)  [Test]  [Rotate credentials ↗] ││
│  └──────────────────────────────────────────────────────────────┘│
│                                                                   │
│  [Add Connector] → Choose type wizard:                           │
│  ○ Security sensor (static)  ○ SIEM/Lake (dynamic)              │
│  ○ Identity (dynamic)  ○ Network/OT (static or dynamic)         │
└──────────────────────────────────────────────────────────────────┘
```

- **Static connector**: schema built-in (sensor TOML spec); onboarding = credentials only
- **Dynamic connector**: [Configure schema] wizard → introspect → map fields → preview → save (§13.2)
- **Configure-schema wizard** (dynamic connectors): step-by-step flow: (1) Authenticate, (2) Introspect schema, (3) Map fields to OCSF or native schema-on-read, (4) Preview sample data, (5) Validate + save mapping
- **[Rotate credentials ↗]**: links to U1 Credentials section (full rotation flow is in U1)
- All config actions (add, modify, delete connector) are admin-gated and audited (§11.2)

---

## Screen 13 — Satellite / Topology Health

**Traces to:** §3.2 (Prism Satellite; multi-hop chaining); §11.3.1 (Satellite/topology); E-UI-CONSOLE-001

### Purpose & Persona

Tree view of the Prism Satellite mesh. Per-hop health status. Degraded-subtree indicators. Only visible when satellites are deployed.

**Persona:** Platform operator; SOC manager; Advanced analyst (understanding data completeness for OT sources).

### Layout

```
┌──────────────────────────────────────────────────────────────────┐
│  Satellite / Topology                                            │
│  ─────────────────────────────────────────────────────────────── │
│  Central Prism (hub)  ✓ Healthy                                  │
│  │                                                               │
│  ├── SAT-US-EAST   ✓ 12ms RTT  6 sources                        │
│  │   ├── [CrowdStrike]  ✓  ├── [Claroty-1]  ✓                  │
│  │   └── SAT-OT-ZONE-A  △  (2 sources; connect timeout)         │
│  │       ├── [PLC-Zone-A]  ✗ timed out                          │
│  │       └── [Armis-OT]   △ degraded                            │
│  │                                                               │
│  └── SAT-EU-WEST   ✓ 28ms RTT  4 sources                        │
│      └── [Cyberint-EU]  ✓                                        │
│                                                                   │
│  DEGRADED SUBTREE ALERT: SAT-OT-ZONE-A is degraded. Queries to  │
│  OT sources in this subtree return partial results. [Diagnose]   │
└──────────────────────────────────────────────────────────────────┘
```

- **Tree view**: rendered as a collapsible tree (Cytoscape.js or react-d3-tree); each node shows health status (✓/△/✗)
- **Degraded subtree propagation** (§3.2): if a mid-tree satellite is down, all downstream sources are implicitly degraded; the tree shows this visually with a cascade indicator
- **Per-hop metrics**: RTT, last heartbeat, queue depth (for store-and-forward edges)
- **Diagnose**: opens a diagnostic panel showing the error, last successful heartbeat, satellite log tail

---

## Open Design Questions for PO / Architect

The following questions should be resolved at day-2 morph time before finalizing the UX spec and dispatching E-UI-CONSOLE-001 stories.

1. **Case management depth:** How much of a full ITSM-style case management system is in scope for day-2? (JIRA-style assignment flows, SLA tracking, escalation rules, case merging/splitting?) The current spec assumes moderate depth (assignment + status + notes + evidence linking). Full ITSM depth is a significant scope increase.

2. **S3 agent UX boundaries:** What can the server-hosted S3 agent DO autonomously vs. only suggest? The current spec requires human approval for all impactful actions. Does the product vision allow any fully-autonomous agentic actions? If so, what categories, and what audit/rollback is required?

3. **Triage Queue vs Findings Queue overlap:** These are designed as two screens (Screen 1 = case-centric triage, Screen 7 = detection-output-centric findings). Is this the right split, or should they be unified with a view toggle? In some competing products (Hunters, Elastic) these are one surface with different views.

4. **Entity 360 time range default:** What time range should the Entity 360 default to? (24h? 7d? configurable per entity type?) This directly affects PrismQL query cost and per-source load.

5. **Reporting fidelity:** Is PDF-only sufficient, or does day-2 require: DOCX export, custom report builders, white-label MSSP reports with customer branding, scheduled delivery? MSSP use cases may require more fidelity here.

6. **Configure-schema wizard UX depth:** How much of the connector configure-schema wizard is in scope for S2 vs. U1 Admin? The current spec puts a read-only health view in S2 and the full configuration wizard in U1. Should non-admin analysts be able to see schema mappings in read-only form from S2?

7. **Mobile/responsive scope:** The current spec defines a read-only mode for SM (<768px). Is read-only acceptable, or must key workflows (at minimum: alert triage, case status update) be fully functional on mobile? SOC floor use cases may require this.

8. **Detection recipe library editorial process:** Who authors and curates the built-in recipe library (§14.7)? Is this an open community model (like Sigma rules), a Prism-owned content team, or customer-contributed? The editorial model affects the library UX significantly.

9. **Multi-schema source label UX:** When results come from both OCSF-normalized sources AND native-schema-on-read sources in the same query (§13.6), how should the Results Explorer surface the schema difference? (Same grid with different field labels? Different views? Schema indicator per row?)

10. **Online learning model transparency (§15):** If anomaly scoring is available, should the UI show which model version scored a finding, and allow the analyst to see why? This is a trust-first AI concern (UI-D2) applied to ML, not just language models. Scope needs to be decided before the ML UI is designed.
