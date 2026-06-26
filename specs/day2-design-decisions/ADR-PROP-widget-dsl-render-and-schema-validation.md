---
document_type: proposed-adr
status: capture
do_not_execute: true
provenance: "2026-06-26 side-analysis — day-2 design decision capture; PROPOSED, gated on brief-reframe sign-off; separate from live factory."
proposed_number: "ADR-PROP-widget-dsl-render-and-schema-validation"
note_on_numbering: "Real number allocated by architect at morph time via create-adr skill. ADR-047–054 are reserved in matured-vision §5.4. Do not reuse."
related_specs:
  - spike: /Users/jmagady/Dev/aletheon_2/spike/DSL_SPECIFICATION.md
  - disposition: .factory/specs/day2-ui-design/S3-conversational-canvas-disposition.md §3 §4.1
  - ADR-PROP-s3-agent-runtime (companion)
  - ADR-PROP-sandboxed-expression-evaluator (companion, governs expression evaluation layer)
  - matured-vision §11.3.2 (render layer position in architecture)
---

# ADR-PROP: Widget DSL Render Layer and Schema Validation

## Status

PROPOSED — capture artifact. Gated on brief-reframe sign-off per disposition §9.

## Context

The S3 conversational-canvas surface (human-decided ADOPT, 2026-06-25/26) generates UI widgets
on the fly: the server-side hosted agent emits a JSON widget schema as the result of a
`displayDynamicWidget` tool call, and the browser recursively renders that schema to React
components on an infinite draggable/pinnable canvas.

This creates a novel attack surface for prism: attacker-influenceable OCSF data (event titles,
hostnames, process command lines, observable values) can influence the LLM's output, including
the widget schemas it produces. If the render layer renders schemas without validation, a
prompt-injection attack could produce schemas with:

- Unknown primitive types that trigger render-path exceptions or undefined behavior.
- Props that escape expected value domains (e.g., injecting arbitrary JavaScript into a string
  prop that a React component passes to `dangerouslySetInnerHTML`).
- Deeply nested structures that exhaust the call stack or trigger quadratic render behavior.
- Widget schemas that reference credential-adjacent fields by name, exposing them in the
  rendered output.

The spike (`/Users/jmagady/Dev/aletheon_2/spike`) already contains a `validateWidgetSchema()`
function and Zod tool definitions per widget primitive. The disposition §4.1 item 1 mandates:
"validation MUST be on the hot path (not optional)."

The spike DSL has 54 primitives organized into categories: layout (container, grid, flex, tabs,
accordion, card, divider, spacer), text (heading, paragraph, badge, tag, code, tooltip,
callout), data display (table, list, keyValue, metric, progress, timeline, treeView,
comparison), 13 chart types (bar, line, area, pie, scatter, gauge, heatmap, radar, treemap,
funnel, waterfall, bubble, candlestick), form inputs, overlays, media, reactive containers,
animation, and conditional rendering.

The disposition §3 mandates ENHANCE with: OCSF-aware primitives (severity bands, OCSF category
labels, sensor-source badges, investigation-drilldown links) and multiple-option viz generation.
These additions must fit within the same validation-first render architecture.

## Decision

**Adopt the aletheon spike's 54-primitive widget DSL as prism's S3 render layer, with strict
Zod schema validation on the hot render path as a mandatory security gate, OCSF-aware
primitive extensions, multiple-option generation support, ephemeral-only canvas/widget state,
and a clear upgrade protocol for adding new primitives.**

### Core Principle: Validation-First Render

The render pipeline is:

```
LLM emits displayDynamicWidget tool result
        │
        ▼
[Zod schema validation gate]   ← MANDATORY, hot-path, not optional
    FAIL → reject schema, surface error to analyst UI
    PASS ↓
Output Hardener (per ADR-PROP-s3-agent-runtime)
        │
        ▼
Widget Renderer (recursive renderNode() in React)
    uses ANTLR4 evaluator for {{expr}} bindings
    (per ADR-PROP-sandboxed-expression-evaluator)
```

No schema reaches the React renderer without passing Zod validation. This is an invariant, not
a best practice. The validation gate rejects any schema with:

- An unknown `type` field (strict allowlist of the 54 primitives + any registered OCSF
  extensions — see Primitive Upgrade Protocol below).
- Props that do not match the registered Zod schema for that primitive type.
- Nesting depth exceeding a configurable limit (default: 20 levels). Rationale: recursive
  renderers are vulnerable to stack exhaustion; 20 levels is sufficient for all known
  legitimate layouts.
- Array children whose length exceeds a configurable limit (default: 1000 items per node).
  Rationale: prevents render-time quadratic complexity from attacker-influenced data shapes.

### Primitive Allowlist

The 54 spike primitives are the initial allowlist. Every primitive has:

- A registered Zod schema defining its allowed props and prop types.
- A prop allowlist that explicitly enumerates every prop the component accepts.
- No `dangerouslySetInnerHTML` or equivalent raw HTML injection anywhere in the render
  path. Text props are rendered as React text nodes, not injected HTML.
- No event handlers (`onClick`, `onMouseOver`, etc.) emitted from widget schemas. Interaction
  handlers are hardcoded in the React component implementations, not schema-driven.
  Rationale: a schema-driven event handler would allow the LLM to inject arbitrary JS execution
  paths via handler-prop values.

### OCSF-Aware Primitive Extensions

Prism retargets the spike's OT-specific domain terms to SOC/MSSP workflows. New OCSF-aware
primitives added to the allowlist (each requires a Zod schema registration before use):

| Primitive | Purpose | OCSF binding |
|---|---|---|
| `ocsfSeverityBadge` | Renders OCSF severity numeric to labeled colored band | `severity_id` / `severity` fields |
| `ocsfCategoryLabel` | Renders OCSF class name and class_uid | `class_name`, `class_uid` |
| `sensorSourceChip` | Source badge: sensor name + health indicator | prism sensor metadata |
| `investigationDrilldown` | Link chip that opens S2 investigation for an observable | observable value + type |

These four primitives constitute the minimum OCSF-aware extension for S3 Phase 1. Additional
OCSF primitives are added via the Primitive Upgrade Protocol below.

### Multiple-Option Generation

The LLM may emit multiple `displayDynamicWidget` calls per turn to offer visualization
alternatives (napkin.ai principle #3, disposition §3). The render layer handles this by:

- Treating multiple widget schemas in a single response as a "multi-option response."
- Rendering all options side-by-side in the canvas with an analyst-facing selector
  ("Option A / Option B / Option C").
- Each option schema is independently validated through the Zod gate. A failed option is
  skipped (surfaced as "Option N unavailable: invalid schema"), not fatal to the others.
- The analyst pins one option to the canvas; unselected options are dismissed.

### Ephemeral Canvas and Widget State

Canvas state (widget positions, pinned widgets, active conversation thread) is stored in
browser-local state only:

- Zustand stores for in-session state.
- `localStorage` for session persistence (survives page refresh, clears on tab close or
  explicit "clear canvas" action).
- No server-side persistence of widget schemas or canvas layout in S3 v1.
- Widget schemas themselves are not stored after render unless pinned by the analyst; pinned
  widgets store only the validated schema (never the raw LLM response).

This scope boundary limits the blast radius of a stored-XSS or schema-injection attack. There
is no backend database that could be poisoned with malicious widget schemas.

### Primitive Upgrade Protocol

Adding a new primitive requires all of the following before the primitive ships:

1. Zod schema authored and merged to the validated-primitives registry.
2. React component implementation reviewed for `dangerouslySetInnerHTML`, event handler
   injection, and prototype pollution vectors.
3. Expression binding props (those accepting `{{expr}}` syntax) explicitly flagged in the Zod
   schema so the ANTLR4 evaluator is applied rather than direct string interpolation.
4. Nesting depth and array-length bounds set for any children or data array props.
5. Security review sign-off (per CLAUDE.md routing table: `vsdd-factory:security-reviewer`).

## Consequences

**Positive:**
- Validation-first render is a defense-in-depth layer that is independent of LLM output
  quality. Even if the LLM is tricked into emitting a malicious schema, the Zod gate rejects it
  before any React rendering occurs.
- Ephemeral-only state drastically reduces the stored-injection attack surface.
- The 54-primitive spike DSL is production-capable and already tested; adoption is a port and
  retarget, not a rewrite.
- OCSF-aware extensions are a small, bounded addition that directly serves the SOC analyst
  persona.

**Negative / trade-offs:**
- Schema validation adds latency on the hot render path. Expected overhead for a typical
  widget schema (depth ≤ 5, 10–50 nodes) is sub-millisecond with compiled Zod schemas.
- The strict allowlist means any new primitive requires the Primitive Upgrade Protocol — a
  deliberate friction that protects the security invariant.
- Multiple-option generation increases LLM token consumption per turn. This is a cost model
  concern for the per-tenant budget tracked by the Model Router.
- No cross-session canvas persistence means analysts lose their canvas on hard refresh. This is
  a known UX trade-off; server-side persistence is a named future story.

## Alternatives Considered

**A. Runtime schema validation on a best-effort basis (log-and-continue on failure).** Rejected:
the disposition §4.1 item 1 explicitly mandates the validation gate "MUST be on the hot path
(not optional)." Log-and-continue would allow partially invalid schemas to reach the renderer,
defeating the purpose.

**B. Sanitize unknown props rather than reject unknown primitives.** Rejected: sanitization of
unknown field values is fundamentally weaker than a strict allowlist. The attacker's goal is
to introduce a primitive or prop that exercises an unreviewed code path in the React renderer.
Sanitizing unknown props does not eliminate that risk; rejecting unknown primitives does.

**C. HTML/Markdown render instead of a widget DSL.** Rejected: raw HTML emission from an LLM
is a well-documented prompt-injection vector (LLM-generated `<script>` tags, event handlers,
CSS injection). A restricted JSON DSL with an explicit allowlist is structurally safer than
HTML because the attack surface is a schema type check, not a full HTML parser.

**D. Server-side persist canvas to RocksDB for cross-session continuity.** Deferred to a named
future story. The blast-radius argument for ephemeral-only state is correct for v1. Persistence
can be added with proper encryption and access control in a future cycle.

## Open Decisions for Human

1. **Nesting depth and array-length limits.** The decision defaults to depth 20 and 1000
   items per node. Are these the right bounds? (A depth of 20 handles all known real layouts;
   1000 items is generous for a widget render. Tighter bounds reduce DoS surface but may
   break legitimate high-density data tables.)

2. **Cross-session canvas persistence scope.** Confirmed as deferred for v1. When this ships
   in a future story, should it use browser-local export/import (analyst-controlled JSON file)
   or server-side storage? The disposition §7 Phase 3 mentions "canvas state export/import as
   JSON (analyst-local file, not server DB)" — this is the preferred direction unless the
   human overrides.

3. **Multi-option generation default.** Should the LLM be prompted to always offer multiple
   visualization options, or should single-option generation be the default with multi-option
   as an explicit analyst request? Multi-option increases cost and rendering complexity;
   single-option is simpler but less exploratory. This is a product UX call.
