---
document_type: disposition
status: capture
do_not_execute: true
provenance: "2026-06-25/26 side-analysis — conversational-canvas disposition; separate from live factory; HUMAN-DECIDED ADOPT."
verdict: ADOPT
surface: S3
related_surfaces: [S1, S2]
cross_links:
  - matured-vision: "§11.3, §11.3.2, §16.2"
  - mockups: ".factory/specs/day2-ui-design/mockups/"
  - spike_source: "/Users/jmagady/Dev/aletheon_2/spike"
---

# S3 Conversational Canvas — Disposition Record

## 1. Context

During a 2026-06-25/26 side-analysis session (separate from the live Phase 3 factory pipeline), a
generative-UI spike was evaluated for adoption as prism's S3 embedded-AI surface. The spike lives at
`/Users/jmagady/Dev/aletheon_2/spike` (read-only reference; not part of the prism workspace).

**What the spike is.** A chat-as-interface generative UI: the user asks in natural language, a
server-side Claude call invokes fetch tools, then emits a `displayDynamicWidget` tool result
containing a JSON widget schema. The frontend recursively renders that schema to React components on
an infinite draggable/pinnable canvas. Key implementation artifacts:

- `spike/docs/GENERATIVE-UI-DESIGN.md` — UX vision + architecture diagram
- `spike/DSL_SPECIFICATION.md` — 54-primitive composable JSON widget DSL (layout/text/data/13
  chart types/lists/forms/overlays/animation/conditional)
- `spike/dashboard/src/stores/` — Zustand canvas, chat, drilldown, RDP, terminal stores
- `spike/dashboard/src/dsl/evaluator.ts` — ANTLR4-based expression evaluator (see Security section)
- `spike/dashboard/src/dsl/grammar/BindingExpr.g4` — formal ANTLR4 grammar for `{{expr}}` bindings
- Working demo with real Claude model via Vercel AI SDK (streamText/useChat + Zod tool definitions)

**Orthogonality.** The widget DSL is a UI-GENERATION language, orthogonal to PrismQL. PrismQL
fetches and queries OCSF data; the DSL renders it. They compose — they do not compete.

---

## 2. Human-Decided Verdict

**ADOPT the conversational canvas as prism's S3 embedded-AI surface — enhanced and hardened.**

S3 is a distinct AI-native mode that COMPLEMENTS (does NOT replace) the structured S2 console
screens. This is the multi-surface / multi-persona model declared in matured-vision §16.2 #1 and
§11.3:

> S1 MCP/BYO-agent · S2 full browser console · S3 server-hosted embedded AI · S4 browser extension
> · U1 admin console

---

## 3. Per-Component Disposition Table

| Component | Verdict | Rationale |
|---|---|---|
| **Conversational-canvas UX paradigm** (chat IS the interface, infinite draggable canvas, widget persistence via pin) | **ADOPT** as S3 mode | Directly realizes matured-vision §11.3 E-UI-EMBEDDED-AI-001 (S3 server-hosted copilot/agent). Differentiates prism from static-dashboard SOC tools. |
| **Widget-generation DSL (54 primitives)** | **ENHANCE** | Core DSL is production-capable. Needs: (a) OCSF-aware primitives (severity bands, OCSF category labels, sensor-source badges, investigation-drilldown links); (b) multiple-option viz generation pattern (napkin.ai principle #3 — offer alternatives when ambiguous); (c) replace `Function()`-based expression evaluator path with the ANTLR4 sandboxed evaluator (see Security section). |
| **Vercel AI SDK + component-registry pattern** (streamText/useChat + Zod tool defs per widget type) | **ADOPT** | The Zod tool definitions map directly to prism's existing MCP tool surface. This realizes the §11.3.2 "single tool contract, two consumers" insight: S1 BYO-agent and S3 hosted-agent call the same underlying MCP tools. No duplicate data-fetching layer needed. |
| **PostgreSQL + Kafka data layer** (spike's persistent event store + stream broker) | **DROP** | Contradicts prism's ephemeral/federated architecture. Prism has no server-side event lake. Widgets fetch live via PrismQL queries executed over MCP tools. Canvas and widget state persist in session/local storage only — NO server database. |
| **ANTLR4 grammar (`BindingExpr.g4`) + evaluator.ts** | **RECONSIDER / ADOPT for SECURITY** | The spike ships an ANTLR4-based evaluator as an *alternative* to the `Function()`/`new Function()` path in `expressionEvaluator.ts`. For prism S3, the ANTLR4 path is MANDATORY (see Security section). The spike already has the grammar and generated parser — this is a configuration choice, not a rewrite. |
| **OT-specific domain terms** (ARO classification, asset-graph/AGE, PLC/HMI/RTU device classes, plant-operator personas) | **DROP / RETARGET** | Spike was built for OT/ICS operations. Prism targets SOC analysts + MSSP. Primitives are retargeted to OCSF categories, SOC workflows, sensor-source context, and analyst personas. No PLC/RTU/SCADA terminology in prism S3. |
| **ui-tars (UI-automation grounding model)** | **DEFER** | Not core to S3 v1. Requires additional model routing and significantly expands scope. Candidate for a later S3 feature cycle. |

---

## 4. Security Constraints (First-Class)

Generative UI introduces a NEW prompt-injection surface in prism: an LLM translating
attacker-influenceable OCSF data (event titles, hostnames, process names) into RENDERABLE widget
schemas. This is not a secondary concern — it gates S3 adoption.

### 4.1 Mandatory Hardening Requirements

**(1) Strict schema validation before render.** Every widget schema emitted by the model MUST be
validated against the allowlisted primitive/prop schema (Zod validation) before any React rendering
occurs. A schema that does not match a known primitive type must be rejected, not rendered. The
spike already has `validateWidgetSchema()` — this validation MUST be on the hot path (not optional).

**(2) Sandboxed expression evaluator — NO eval() / Function().** The `{{expr}}` binding syntax in
reactive container nodes must be evaluated via the ANTLR4 grammar-parsed evaluator
(`evaluator.ts`), NOT via `new Function()` or `eval()`. The spike contains BOTH paths; prism S3
uses only the ANTLR4 path. This eliminates the JavaScript-execution escape vector in LLM-generated
widget schemas.

**(3) Credentials remain AI-opaque (AD-017).** Sensor credentials, API keys, and auth tokens must
never appear in widget schema data, in chat messages, or in canvas store state. The S3 agent
runtime (Tool Mediator layer in §11.3.2) injects credentials at the I/O boundary using prism's
existing broker pattern — they do not flow through the LLM turn.

**(4) Output hardening applies to BOTH agent paths.** Prism's existing output-hardening applies to
S1 (BYO agent, outbound tool results) and must also apply to S3 (hosted agent, inbound LLM
responses). The Output Hardener in §11.3.2 sits between the model response and the widget renderer.

**(5) Canvas/conversation state is ephemeral.** No server-persisted dashboards in S3 v1. Widget
schemas and conversation history are session-local (Zustand + localStorage). This limits the blast
radius of a stored-XSS or schema-injection attack — there is no persistent backend store to
corrupt.

### 4.2 Render Layer Position in §11.3.2 Architecture

```
Browser console (S2)
  └─► Agent Orchestrator (server-side, per-tenant session)
          │
     ┌────┴────────────────────────────────────┐
     Model Router   Tool Mediator   Output Hardener
                         │                │
                 (credentials injected   (widget schema
                  here — AD-017)          validated here)
                                              │
                                    Widget Renderer (React)
                                    [ANTLR4 evaluator only]
```

The widget render layer sits AFTER Output Hardener. No schema reaches the renderer without passing
validation.

---

## 5. Architecture Mapping

### 5.1 Mapping to §11.3.2 S3 Server-Hosted Agent Runtime

| §11.3.2 Component | Canvas Realization |
|---|---|
| Agent Orchestrator (server-side, per-tenant session) | Vercel AI SDK `streamText` route handler, one session per chat thread |
| Model Router | Existing prism model-routing skill (already designed for S3); routes to on-prem model for air-gapped deployments |
| Tool Mediator | MCP tool surface — same tools used by S1 BYO-agent; credentials injected by broker (AD-017) |
| Output Hardener | Zod schema validation gate on `displayDynamicWidget` tool result before React render |
| Widget Render Layer | Recursive `renderNode()` React renderer; ANTLR4 evaluator for `{{expr}}` bindings |

### 5.2 "Single Tool Contract, Two Consumers" (§11.3.2)

The `displayDynamicWidget` tool (and the fetch-data MCP tools it delegates to) are identical
between S1 and S3:

- **S1 (BYO agent):** analyst's own Claude/GPT client connects via MCP; calls fetch tools directly.
- **S3 (hosted agent):** prism's server-side agent runtime calls the same tools; emits widget
  schemas back to the browser canvas instead of plain text.

This means: one tool spec definition serves both surfaces. No parallel data-fetching layer.

### 5.3 AD-017 Credential Boundary

The S3 hosted agent inherits prism's credential opaqueness contract (AD-017):

- Sensor credentials are resolved by the Tool Mediator at the moment a fetch tool executes.
- Credentials are not passed as parameters to the LLM turn.
- Widget schema data (the LLM's output) must not contain credential values; validated by Output
  Hardener.
- The canvas/conversation Zustand stores never serialize credentials.

---

## 6. Candidate ADRs (Numbers TBD at Morph Time)

These ADRs do not yet exist in `.factory/specs/architecture/adr/`. Numbers are placeholders; the
architect allocates collision-free IDs via the `create-adr` skill at morph time.

| Candidate ADR | Scope | Decision Point |
|---|---|---|
| ADR-TBD: Server-Hosted Agent Runtime (S3) | Defines the server-side agent orchestrator architecture, session model, deployment-gating (air-gap/on-prem), and integration with model-routing skill | Already noted in matured-vision §11.3 as a pending ADR |
| ADR-TBD: Widget DSL Render Layer + Schema Validation | Mandates Zod validation on the hot render path; defines the allowlisted primitive set; defines upgrade policy for adding new primitives | Security-gating ADR for the entire canvas approach |
| ADR-TBD: Sandboxed Expression Evaluator (ANTLR4) | Prohibits `eval()`/`Function()` for widget binding expressions; mandates ANTLR4-parsed evaluator; defines the allowlisted built-in function set | Required before any reactive `{{expr}}` binding ships |

---

## 7. Phased Adoption Plan

This is a PORT + RETARGET, not a rewrite. The spike already has the runtime, DSL renderer, Zustand
stores, and ANTLR4 evaluator. Effort estimate: weeks, not months.

### Phase 1 — NL → PrismQL + Single-Widget Render over MCP Tools

**Goal:** Analyst types a natural-language question in the S3 chat panel; the hosted agent calls
existing MCP tools to execute a PrismQL query; result is rendered as a single widget (table, metric
card, or timeline) on the canvas.

- Drop PostgreSQL/Kafka data layer; wire `displayDynamicWidget` to MCP tool results.
- Implement Output Hardener (Zod schema validation gate).
- Configure ANTLR4 evaluator as the ONLY expression evaluation path.
- Minimum OCSF-aware primitives: event-severity badges, OCSF category label, sensor-source chip.
- AI-opaque credential wiring via Tool Mediator (AD-017 compliance from day 1).
- Canvas: basic infinite canvas, widget pin/dismiss, conversation thread.

**Effort rough order of magnitude:** 3–5 weeks (port + retarget + security hardening)

### Phase 2 — Multiple-Option Generation + Editable Widgets + OCSF-Aware Primitives

**Goal:** Realize napkin.ai principle #3 (offer alternatives); allow analyst to refine widgets
inline; expand OCSF-aware primitives to cover full S2 investigation workflow equivalence.

- Multiple-option generation: model emits 2–3 widget schema alternatives; analyst picks.
- Editable widgets: analyst can drag/resize/reconfigure rendered widgets.
- Expand OCSF primitive set: networkGraph retargeted to prism asset topology; timelineChart wired
  to alert/event timelines; radarChart for security posture dimensions.
- Drilldown: `drilldown` primitive links canvas widgets back to S2 investigations console screens.

**Effort rough order of magnitude:** 4–6 weeks

### Phase 3 — Sandboxed-Eval Hardening + RDP/VNC Diagnostic Widgets

**Goal:** Full security hardening pass; optional remote-access diagnostic primitives for MSSP
workflows.

- Formal security audit of ANTLR4 evaluator surface; built-in function allowlist review.
- RDP/VNC/SSH diagnostic widget primitives (already prototyped in spike's `rdpStore.ts` and
  terminal store) — gated on operator-permission model and explicit analyst action.
- Session-continuity: canvas state export/import as JSON (analyst-local file, not server DB).
- Performance hardening: canvas with 50+ widgets, reactive bindings, SSE streams.

**Effort rough order of magnitude:** 3–4 weeks

---

## 8. Cross-Links

| Reference | Path/Section |
|---|---|
| S3 architectural add (server-hosted agent runtime) | matured-vision §11.3 |
| S3 runtime architecture diagram + single-tool-contract insight | matured-vision §11.3.2 |
| Multi-surface / multi-persona decision | matured-vision §16.2 #1 |
| S2 screen inventory (S3 complements these) | matured-vision §11.3.1 |
| Session decisions confirmed 2026-06-25/26 | matured-vision §16 |
| S2 console mockups (S3 canvas mockup TBD) | `.factory/specs/day2-ui-design/mockups/` |
| Credential opaqueness | AD-017 (project memory `project_ai_opaque_credentials.md`) |
| Spike source (read-only) | `/Users/jmagady/Dev/aletheon_2/spike` |

---

## 9. Adoption Gate

This document is a CAPTURE artifact. Execution is gated on:

1. **Brief-reframe sign-off** — PO ratifies value-prop #5 rewrite (§11.3) that positions S3
   correctly alongside S1/S2/S4.
2. **ADR allocation** — architect allocates the three candidate ADR IDs and produces ADR bodies.
3. **S3 canvas mockup** — UX designer produces the S3 canvas screen in the mockups set
   (`.factory/specs/day2-ui-design/mockups/`) alongside existing S2 screens.
4. **Story decomposition** — story-writer decomposes Phase 1 adoption into implementable stories
   targeting the correct wave.

Until those gates pass, no factory pipeline files are modified and no implementation begins.

---

*do_not_execute: true — morph gated on brief-reframe sign-off per §9 above.*
