---
document_type: proposed-adr
status: capture
do_not_execute: true
provenance: "2026-06-26 side-analysis — day-2 design decision capture; PROPOSED, gated on brief-reframe sign-off; separate from live factory."
decision_source: "Human decision 2026-06-25 (UI-D5 RESOLVED); matured-vision-day2-requirements.md §11.3 web-stack bullet; research/ui-requirements-2026-06-25.md §6"
real_adr_number_pending: true
reserved_range: "ADR-047..054 already assigned; architect to allocate from ADR-055+ at morph time"
traces_to:
  - "matured-vision-day2-requirements.md §11.3 (UI-D5 RESOLVED, web-stack bullet)"
  - "matured-vision-day2-requirements.md §11.0 (multi-surface UI scope)"
  - "matured-vision-day2-requirements.md §3.1 (E-CENTRAL-TRANSPORT-001)"
  - "research/ui-requirements-2026-06-25.md §6 (web-stack decision)"
  - "research/ui-requirements-2026-06-25.md §7 (admin/RBAC/SSO)"
  - "ADR-051 (per-connection analyst identity — security binding)"
binds_to_epics:
  - E-UI-ADMIN-001
  - E-UI-CONSOLE-001
  - E-UI-EMBEDDED-AI-001
  - E-UI-EXTENSION-001
  - E-CENTRAL-TRANSPORT-001
---

# ADR-PROP: UI Web-Stack Selection — TypeScript SPA (React) + Rust (Axum/Tokio/DataFusion) Backend

## Status

**CAPTURE** — decision is settled (human-decided 2026-06-25, UI-D5 RESOLVED). This ADR formalizes
the already-resolved choice; it is PROPOSED until morphed into the live ADR registry at morph time.
Not to be executed before brief-reframe sign-off and architect ADR number allocation.

---

## Context

Prism's matured day-2 vision (§11.3, human directive 2026-06-25) adds a full multi-surface browser
experience to the existing MCP-native (stdio) product. The four new surfaces are:

- **S2** — Full browser investigations console (PrismQL editor, entity/event grid, federated results,
  detection findings, dashboards)
- **S3** — Server-hosted embedded AI (copilot/NL-to-PrismQL, conversational-canvas mode — §11.3.2)
- **S4** — Browser extension (IOC right-click federated pivot)
- **U1** — Admin/Ops console (tenant management, connector config, credential rotation, RBAC, audit)

All four surfaces run over the E-CENTRAL-TRANSPORT-001 HTTP transport (Axum/Tokio backend). They
share a single Rust backend but require a frontend framework.

### The Core Trade-off

At decision time, two serious options existed:

**Option A — TypeScript SPA (React) + Rust backend + OpenAPI codegen**
The frontend is React/TypeScript; the Rust backend exposes an HTTP/SSE API; shared types flow via
OpenAPI → openapi-typescript codegen. Standard in the SOC/SIEM product market.

**Option B — Rust-native frontend (Leptos or Dioxus) + Rust backend**
Full Rust from browser WASM to server. One language; shared crate types. Architecturally
appealing for an all-Rust shop. Explored and set aside (see Alternatives below).

### Deciding Factors

The decision was made in the 2026-06-25 side-analysis walkthrough after explicit human trade-off
review. The deciding factors were:

1. **Monaco is the PrismQL editor.** The PrismQL editor requires syntax highlighting, autocomplete
   over the OCSF schema, and inline linting. Monaco Editor (VS Code's editor core) is the
   industry-standard choice for code editors in SOC products (Panther uses it; it is used throughout
   security-tool UIs). Monaco is a JavaScript library. In a Rust-WASM frontend it becomes an
   awkward JS island — the unified-Rust benefit collapses exactly where the UI is most complex.

2. **Data-dense SOC UI component ecosystem is JS-native.** The SOC console requires:
   - **AG Grid / TanStack Table + TanStack Virtual** for virtualized 10k+-row OCSF event grids
   - **ECharts / visx** for MITRE ATT&CK heatmaps, risk distribution dashboards
   - **Cytoscape.js / sigma.js** for relationship graphs and entity canvas
   - These are JavaScript libraries with mature production track records in security products.
     Leptos/Dioxus equivalents are research-grade or require wrapping these same libraries,
     defeating the single-language benefit.

3. **Ecosystem velocity matters for the investigations console.** Prism is entering a market
   (SOC/SIEM consoles) where the incumbent UX patterns are JavaScript-native. Research from the
   2026-06-25 UI pass confirms Snowsight, Grafana, Panther, and all major comparable products
   use JS frontend stacks. Practitioners on that stack know how to build this UI idiomatically.

4. **Type boundary is solvable, not eliminated.** The type-boundary concern (TS ↔ Rust mismatch)
   is addressed by generating TypeScript types from the OpenAPI spec at build time via
   `openapi-typescript`. This is not zero-overhead, but it is tractable and eliminates
   manual drift between frontend and backend type definitions.

5. **WASM escape hatch for perf-critical modules.** If specific client-side modules (e.g.,
   PrismQL parse-and-highlight, OCSF record diffing) need Rust-grade performance, they can be
   compiled to WASM and called from the TypeScript layer. This preserves the option without
   requiring the entire frontend to be Rust.

---

## Decision

**Option A is adopted. The Prism frontend is TypeScript SPA (React) + Rust (Axum/Tokio/DataFusion)
backend.**

This decision is human-made and settled (UI-D5 RESOLVED 2026-06-25). It is not subject to further
architectural deliberation within the day-2 morph.

### Specific technology choices (binding)

| Layer | Technology | Rationale |
|-------|-----------|-----------|
| Frontend framework | React (TypeScript) | Market-dominant for SOC product UIs; practitioner knowledge; rich ecosystem integration |
| PrismQL editor | Monaco Editor | VS Code core; syntax highlight, autocomplete, linting; standard for code-heavy security consoles |
| Virtualized tables | AG Grid Community / TanStack Table + TanStack Virtual | 10k+ row OCSF event grids; AG Grid handles column pinning, sorting, filtering at scale |
| Charts / dashboards | ECharts or visx | MITRE ATT&CK heatmaps, risk distribution, detection trends; both support large dataset rendering |
| Relationship graphs | Cytoscape.js or sigma.js | Entity canvas, entity-to-entity edges, cross-source coverage; sigma.js faster for large graphs |
| Frontend↔Backend types | OpenAPI → openapi-typescript codegen | Eliminates manual TS/Rust type drift; generated at build time from the Axum-served OpenAPI spec |
| Backend HTTP layer | Axum + Tokio | Already the E-CENTRAL-TRANSPORT-001 transport; serves static SPA assets + API routes |
| Backend data plane | DataFusion (existing) | No change; federated query engine is unaffected by frontend choice |
| WASM (optional) | wasm-pack + wasm-bindgen | For perf-critical client modules only (PrismQL highlight, OCSF diff); not default |
| Streaming protocol | Server-Sent Events (SSE) or WebSocket over Axum | Partial-result streaming (§3.6) and real-time detection firing to the console |

### Module boundaries (TypeScript ↔ Rust)

The boundary rule is: **business logic, query execution, data access, and credential resolution
stay in Rust; rendering, interaction, and client-side state management are TypeScript.** No
business logic leaks into the TypeScript layer. The TypeScript layer calls Rust via the HTTP API
and renders results.

Exception: PrismQL syntax highlighting and autocomplete MAY be compiled to WASM and called from
Monaco's language-server protocol. This is an optimization path, not required at day-2 launch.

### Type generation pipeline (binding)

```
Axum routes (Rust)
  → utoipa / aide OpenAPI spec generation (auto, at build time)
  → openapi.json (committed to repo or generated artifact)
  → openapi-typescript CLI → api-types.ts
  → imported by React components
```

This pipeline runs as part of the frontend build step. A CI check verifies the generated types
are in sync with the backend spec to prevent drift.

### Security canon binding (non-negotiable)

The TypeScript SPA introduces a web security surface that did not exist in the stdio-only product.
The following security properties bind to this ADR and are non-negotiable:

1. **CSP (Content Security Policy):** strict CSP header served by Axum; script-src restricted to
   self + Monaco CDN bundle hash (or self-hosted Monaco). Blocks XSS injection into the investigations
   console via attacker-controlled OCSF data.

2. **XSS prevention:** no `dangerouslySetInnerHTML` with OCSF field values or analyst-supplied strings.
   All OCSF-sourced data rendered via React's JSX escaping (structural default) or explicit sanitization
   for formatted output. This is especially critical for S2 (investigations console) where OCSF data
   originates from attacker-influenceable sources.

3. **CSRF protection:** all state-mutating API calls use an `X-CSRF-Token` header (double-submit cookie
   pattern) or SameSite=Strict cookie on the session token. Axum middleware enforces this.

4. **Clickjacking (X-Frame-Options / frame-ancestors):** Axum serves `X-Frame-Options: DENY` and
   `Content-Security-Policy: frame-ancestors 'none'` unless embedding is explicitly configured.

5. **HTTP-only / SameSite cookies:** session/identity tokens are HTTP-only, Secure, SameSite=Strict.
   Not accessible from JavaScript. Binds to per-connection analyst identity propagation (ADR-051).

6. **Session expiry and idle timeout:** session tokens carry a max-age and an idle-timeout enforced
   server-side. The console displays a countdown / re-auth prompt before expiry. Binds to ADR-051.

7. **Signed tenant tokens:** multi-tenant views carry a signed tenant context token (e.g., JWT with
   `org_id` claim, signed by the Axum server). The frontend cannot forge or modify tenant context —
   every API call is validated against the token by the Axum handler, not trusted from the request body.
   Binds to ADR-051 per-connection identity.

8. **Prompt-injection hardening (S2 + S3):** OCSF results displayed in the console are rendered as
   structured data (field: value tables, JSON viewers), NOT as raw markdown or HTML blobs the user's
   browser renders verbatim. This is the client-side complement to Prism's existing server-side
   output-hardening. S3 (embedded AI, §11.3.2) applies a second hardening pass before widget rendering.

9. **S4 browser extension token flow (no session-staleness footgun):** the browser extension (S4)
   authenticates to the central service via a proper OAuth-style token flow against the Axum API —
   NOT by piggybacking on the S2 console session (which breaks when stale, as documented in Query.io's
   known issues). Extension tokens are scoped, short-lived, and revocable independently of the console
   session.

All nine points are enforced properties, not guidelines. Violations are P1 findings in adversarial
review.

---

## Consequences

### Positive

- Monaco, AG Grid, Cytoscape.js, ECharts: all drop-in with no wrapping work; full feature access.
- React + TypeScript is the known hiring and consulting pool for SOC console UIs.
- Type boundary solved structurally by codegen; not eliminated but tractable.
- WASM escape hatch available for perf-critical modules without committing the whole frontend.
- Security canon (CSP, XSS, CSRF, signed tenant tokens) is well-understood in the React ecosystem;
  OWASP guidance, battle-tested libraries (DOMPurify, helmet.js equivalent in Axum).

### Negative / Risks

- **Two-language codebase.** Prism becomes Rust + TypeScript. The factory pipeline now exercises
  frontend-specific agents (ux-designer, design-system-bootstrap, accessibility-auditor,
  visual-reviewer, e2e-tester, ui-quality-gate). Material process cost acknowledged.
- **Type drift risk.** OpenAPI spec generation must be kept current. CI gate (spec-drift check)
  is mandatory — not optional hygiene.
- **CSP + Monaco CDN.** Monaco loaded from CDN requires a CSP hash or self-hosting. Self-hosting
  Monaco is the production-grade choice (avoids CDN dependency at runtime, supports air-gap).
  The web-stack implementation ADR (live, after morph) must specify Monaco self-hosting as the default.
- **S3 conversational-canvas (§11.3.2 2026-06-26 addendum):** generative UI over attacker-influenceable
  OCSF data is a new attack surface. Widget schemas MUST be validated against an allowlist; expression
  evaluation MUST use a sandboxed/grammar-parsed evaluator (no `eval()`/`Function()`). This is a
  binding constraint on E-UI-EMBEDDED-AI-001, not deferred cleanup.

### Neutral

- Rust backend architecture (Axum, Tokio, DataFusion, Arc-DI) is unchanged.
- PrismQL grammar and execution engine are unchanged.
- Credential model (AI-opaque, reference-based, AD-017) is unchanged. The TypeScript layer never
  receives or stores credential values.

---

## Alternatives Considered

### Option B — Rust-native frontend (Leptos or Dioxus)

**What it offers:** one language (Rust) end-to-end; shared crate types between frontend and backend
without codegen; philosophically coherent for an all-Rust shop.

**Why set aside:**

1. **Monaco becomes a JS island.** Leptos can embed JavaScript islands, but Monaco is a large, opinionated
   JS library with its own bundler assumptions. Wrapping it in a Rust-WASM frontend requires a JS interop
   boundary that reintroduces the exact cross-language complexity the option was meant to avoid — at the
   worst possible location (the most-used, most-complex UI component).

2. **AG Grid, Cytoscape.js, sigma.js: same problem.** These are battle-tested JS libraries with no Rust
   equivalents at equivalent maturity. Wrapping them via `web-sys` / JS interop is feasible but requires
   significant maintenance overhead. Writing native equivalents in Rust-WASM is research-grade effort.

3. **Leptos and Dioxus are not production-proven at SOC-product scale.** Research confirms the
   practitioners' consensus: "Leptos is appropriate when the team is overwhelmingly Rust-based AND it is
   closer to an internal tool than wide-market SaaS." Prism is wide-market SaaS targeting MSSP
   competition. Betting the investigations console on an ecosystem that is not yet proven at that scale
   introduces unacceptable adoption risk.

4. **The all-Rust-shop argument is partially mitigated by codegen.** The main objection to Option A
   (type boundary friction) is solved by `openapi-typescript` codegen. The unified-language advantage
   of Option B is real but smaller than it initially appears, given codegen.

**Decision:** Option B was weighed seriously and set aside for the above four reasons. The decision is
not "Rust native is bad" — it is "Leptos + Monaco + AG Grid is the wrong tradeoff for a production SOC
console in 2026." Revisit if Leptos ecosystem matures to production-grade for data-dense UIs.

### Option C — No frontend (MCP-native only)

This was the original product architecture and remains valid for S1 (BYO agent, power users). The
human directive (2026-06-25 §11.3) explicitly overrides the recommendation to stay frontend-free.
Option C is not available as a day-2 scope choice.

---

## Open Decisions for Human

The following decisions are NOT resolved by this ADR. Each requires explicit human input before
the relevant implementation epic begins.

| # | Question | Stakes | Recommendation |
|---|----------|--------|----------------|
| OD-1 | **Monaco self-hosted vs CDN?** The security canon above specifies self-hosting as the default for air-gap and CSP cleanliness. However, self-hosting Monaco adds ~5 MB to the frontend build and requires a webpack/vite configuration step. Human confirmation that self-hosting is the correct default OR that CDN + SRI hash pinning is acceptable. | Security / air-gap support | **Recommend: self-hosted.** Air-gap (OT/satellite deployments) cannot reach a CDN at runtime. Self-hosting is the only option that works universally. |
| OD-2 | **React version and bundler?** React 18 (stable) vs React 19 (RC as of 2026-06). Vite vs webpack vs Turbopack for the bundler. This affects build toolchain, SSR support (relevant for S3 conversational canvas), and long-term maintenance. | Build toolchain lock-in | **Recommend: React 18 + Vite (stable, well-supported, fastest dev builds).** React 19 when it reaches stable. |
| OD-3 | **State management?** For S2/S3 the investigations console needs cross-component state (current query, active case, session identity). Candidates: Zustand (minimal), Jotai (atomic), React Query (server-state), Redux Toolkit (heavy but type-safe). Human preference or delegation to ux-designer. | Developer experience, testability | **Recommend: React Query (TanStack Query) for server state + Zustand for local UI state.** React Query handles the streaming SSE partial-result pattern natively. |
| OD-4 | **S3 model provider.** The conversational-canvas (§11.3.2, 2026-06-26 addendum) requires an LLM API call from the server-side agent runtime. Options: Anthropic Claude API (direct), LiteLLM-style model router (multi-provider, fallback). The AI-opaque credential model (§11.1) applies — model API keys are broker-resolved. Human: which model(s) are in scope for day-2 S3? Is model-routing (multi-provider fallback) required at launch or deferred? | Cost, vendor lock-in, air-gap | **Recommend: Anthropic Claude API at launch (aligns with existing factory model stack); LiteLLM-compatible abstraction layer so the backend can swap models without frontend changes. Air-gapped deployments must be able to disable S3 or point at a self-hosted model.** |
| OD-5 | **Browser extension manifest version.** S4 targets Manifest V3 (Chrome, Edge) and Firefox (WebExtensions API, MV3-compatible). Apple Safari MV3 support is partial in 2026. Scope: Chrome/Edge only at day-2, or cross-browser? | Engineering scope | **Recommend: Chrome/Edge (MV3) at day-2 launch. Firefox and Safari as follow-on.** |
