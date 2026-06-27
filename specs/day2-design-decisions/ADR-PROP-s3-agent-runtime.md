---
document_type: proposed-adr
status: capture
do_not_execute: true
provenance: "2026-06-26 side-analysis — day-2 design decision capture; PROPOSED, gated on brief-reframe sign-off; separate from live factory."
proposed_number: "ADR-PROP-s3-agent-runtime"
note_on_numbering: "Live ADR registry is at ADR-046 (last allocated). ADR-047 through ADR-054 are already reserved in matured-vision §5.4 / §11.5 checklist for RetentionCache, PrismQL RETAIN semantics, Detection window, Central deployment topology, Per-connection identity, Credential custody, Shared state, and Central operational model. This ADR receives a real number only at morph time via the create-adr skill; the architect must verify no collision."
related_specs:
  - matured-vision §11.3.2 (S3 agent runtime sketch)
  - matured-vision §11.1 (secret broker)
  - disposition: .factory/specs/day2-ui-design/S3-conversational-canvas-disposition.md
  - ADR-051 (per-connection analyst identity, proposed)
  - ADR-052 (credential custody, proposed)
  - AD-017 (AI-opaque credentials, project memory)
---

# ADR-PROP: S3 Server-Hosted Agent Runtime Architecture

## Status

PROPOSED — capture artifact. Gated on brief-reframe sign-off per disposition §9.

**Human decision applied 2026-06-26:** S3 opt-in flag default resolved — `s3_agent_runtime_enabled = false` by default (see Deployment Gating section).

## Context

Prism is currently MCP-native only: the central service acts as an MCP server for BYO agents
(S1). The human-directed adoption of the conversational-canvas paradigm (2026-06-25/26 side
analysis, disposition §2) adds S3: an embedded AI surface inside the browser console where
prism hosts the agent server-side. This is a net-new scope addition noted explicitly in
matured-vision §11.3:

> "The central service gains an LLM-agent orchestration layer (model routing, tool-call
> mediation, output hardening). This is a notable scope addition — flag for architect."

The S3 runtime must satisfy four non-negotiable constraints inherited from the existing
product contracts:

1. Credentials stay AI-opaque (AD-017). The hosted agent must never see raw secret values.
2. Prompt-injection hardening already applies to S1 MCP output and must also cover S3 LLM
   responses (disposition §4.1 item 4).
3. Every tool call must be bound to per-connection analyst identity (matured-vision §11.3.2,
   references ADR-051).
4. The agent runtime must be deployment-gatable — disableable, air-gap-safe, and
   self-hosted-model-compatible (matured-vision §11.3.2 final bullet).

The "single tool contract, two consumers" insight from matured-vision §11.3.2 is a load-bearing
design principle: S3 hosted agents call the SAME MCP tool surface S1 BYO agents use. No
parallel data-fetching layer is created. This constrains the architecture to a thin orchestration
wrapper over the existing tool surface, not a parallel implementation.

## Decision

**Adopt a four-component server-side agent runtime for S3, wrapping the existing MCP tool
surface and inheriting prism's existing security contracts. The runtime is deployment-gated and
per-tenant isolated.**

### Runtime Components

```
Browser console (S2) ──► Agent Orchestrator (server-side, per-tenant session)
                              │
              ┌───────────────┼─────────────────────┐
              ▼               ▼                     ▼
        Model Router    Tool Mediator         Output Hardener
        (per-tenant     (exposes the SAME      (prompt-injection
         model +         MCP tool surface       defense — prism's
         cost budgets;   as S1 BYO clients;     existing output
         model-routing   credentials injected   hardening applied
         skill)          here, AD-017)          before render)
              │               │                     ▲
              ▼               ▼                     │
            LLM         PrismQL / federated  ───────┘
                        query engine
```

#### Agent Orchestrator

- Server-side, per-tenant session manager. One orchestration loop per active chat thread.
- Conversation store: per-tenant, per-session, isolated. No cross-tenant context leakage.
- Uses Vercel AI SDK `streamText` route handlers (already proven in the generative-UI spike)
  as the implementation mechanism for conversation streaming to the browser.
- Conversation history is session-scoped. No server-side persistent database for conversation
  state in S3 v1 — the browser holds Zustand/localStorage state (disposition §4.1 item 5).
  Rationale: ephemeral-first limits blast radius; server-DB persistence deferred to a named
  future story with explicit human authorization.
- Session expiry and idle-timeout enforced server-side, bound to per-connection analyst
  identity (ADR-051).

#### Model Router

- Reuses the existing `model-routing` skill (already designed for per-model selection/fallback).
- Per-tenant configuration: each tenant can have a configured model, model family, and cost
  budget ceiling.
- LLM API keys are configured server-side via the §11.1 secret broker (this is dogfood: the
  broker that manages sensor credentials also manages LLM API keys). Keys are AI-opaque — the
  LLM key value never appears in the LLM's own context (AD-017 generalization).
- Air-gap / self-hosted path: Model Router accepts a configured local-model endpoint (vLLM,
  Ollama, or compatible) in place of a cloud model. The deployment configuration determines
  which backend is active.

#### Tool Mediator

- Exposes the SAME MCP tool surface that S1 BYO agents use. One tool contract, two consumers.
  No new tool definitions are created for S3. This is the "single tool contract" constraint.
- At the I/O boundary: resolves credential references via the §11.1 secret broker immediately
  before executing any sensor fetch tool call. The resolved secret is injected into the HTTP
  client at the adapter boundary and never passed as a parameter to the LLM turn (AD-017).
- Every tool call is tagged with the per-connection analyst identity (ADR-051) for audit.
- Tool call audit records: analyst identity, tool name, input parameters (credentials
  redacted), execution timestamp, duration, result status. Audit records are append-only and
  tenant-scoped.

#### Output Hardener

- Applies prism's existing prompt-injection-hardened output pipeline to S3 LLM responses.
- For the conversational-canvas render path: sits between the LLM's `displayDynamicWidget`
  tool result and the React widget renderer. Widget schemas that fail Zod validation (see
  ADR-PROP-widget-dsl-render-and-schema-validation) are rejected before render.
- Both S1 (outbound tool results to BYO agents) and S3 (inbound LLM responses from hosted
  agent) pass through output hardening. Same contract, both directions.

### Deployment Gating

**DECIDED 2026-06-26 (human): S3 is OPT-IN by default across all deployment types.**

S3 is an **opt-in, deployment-gated capability**:

- A deployment-level configuration flag (`s3_agent_runtime_enabled: bool`) controls whether
  the runtime is active. **Default: `false` (disabled).** A tenant administrator or platform
  operator must explicitly set `s3_agent_runtime_enabled = true` to activate S3 for a
  deployment. This applies to cloud, satellite/edge, and air-gapped deployments alike.
- S1 BYO-agent access (the existing MCP server) is always available regardless of this flag.
  Analysts can bring their own MCP client at any time without S3 being enabled.
- Air-gap and OT-regulated deployments are safe by default: S3 never activates unless
  explicitly opted in and a compatible model backend is configured.
- When disabled, the browser console degrades gracefully to S2 only. No S3 UI controls are
  rendered.
- Air-gapped deployments that opt in: configure `model_backend: local` (vLLM, Ollama, or
  compatible endpoint). No outbound internet traffic is required when a local model backend
  is active.
- Conversation state is browser-local in v1 (Zustand/localStorage). No server-side
  conversation database is included in v1; persistence is a named future story requiring
  explicit human authorization.
- The federated query core (PrismQL, sensor adapters, MCP server) has ZERO dependency on S3
  being present. S3 is an additive layer.

## Consequences

**Positive:**
- Single tool contract maintained — no API surface duplication.
- AD-017 AI-opacity is generalized to cover LLM API keys as well as sensor credentials, which
  strengthens the overall trust posture.
- Deployment gating makes the architecture safe for OT/air-gap customers from day one.
- Reusing the `model-routing` skill avoids reinventing model selection logic.
- No server-side conversation DB in v1 limits blast radius and reduces implementation scope.

**Negative / trade-offs:**
- Conversation history is lost on session expiry (browser state only). Acceptable for v1;
  analysts may want cross-session history in a future cycle — requires explicit human
  authorization before adding server-side persistence.
- Per-tenant model configuration adds operational complexity (credential rotation for LLM keys
  across tenants). Managed by the same broker infrastructure as sensor credentials.
- Vercel AI SDK dependency introduces a TypeScript-side orchestration layer. The Rust backend
  is authoritative for data; the TS layer is authoritative for LLM orchestration. This boundary
  must be kept clean.

## Alternatives Considered

**A. BYO-agent only (no S3 hosted runtime).** Rejected: this was the original product
position, explicitly overridden by human directive 2026-06-25. The investigations console
persona requires an embedded AI without forcing the analyst to run their own MCP client.

**B. Server-side Python agent runtime (LangChain / LangGraph).** Rejected: prism is a
Rust workspace; a Python runtime introduces a second language runtime, separate dependency
tree, and a dependency on a framework with a history of rapid churn. The Vercel AI SDK on the
TypeScript frontend side is a more natural fit given the TS SPA decision (UI-D5, 2026-06-25).

**C. Separate microservice for the agent runtime.** Rejected for v1: adds deployment
complexity without benefit at this scale. The agent runtime is scoped as a module within the
central backend, not a standalone service. Architecture can revisit in a later cycle if
isolation becomes necessary.

**D. Persist conversation history to RocksDB.** Deferred, not rejected: RocksDB is already
in use for other domains. The deferral is a scope decision — session-local storage is
sufficient for v1, and adding persistence is a named future story. Any addition of server-side
conversation storage requires explicit human authorization.

## Open Decisions for Human

1. **Conversation history persistence policy.** ~~S3 v1 stores conversation state in the
   browser (session/localStorage only). If the analyst refreshes or closes the tab, history is
   lost. Is this acceptable for v1, or should a minimal server-side store (e.g., Redis TTL-backed
   session cache, not RocksDB) be included from day one?~~

   **RESOLVED 2026-06-27 (human): server-side conversation store from day one.** Minimal
   per-tenant server-side conversation/history store required; NOT browser-only. Per-tenant-DEK
   encrypted (ties secret-subsystem HD-4 / SS-26 per-tenant DEK). Configurable retention policy.
   Rationale: feeds the C10 GAP-Q2 evidence-package + audit story and gives cross-device
   continuity. Must respect AI-opaque output (AD-017) and C16 entity-masking. Supersedes the
   "browser session/localStorage only for v1" framing in the Deployment Gating section and the
   Consequences/Alternatives sections above — those passages reflected the pre-resolution
   position; this resolution governs for story decomposition.

2. **Per-tenant model budget enforcement.** ~~The design specifies per-tenant cost budgets in
   the Model Router, but does not define the enforcement mechanism (token counter? API-cost
   estimator? hard cutoff vs soft warning?). This needs a concrete design before story
   decomposition.~~

   **RESOLVED 2026-06-27 (human): per-tenant model budget = token+cost accounting with
   soft-warn → hard cutoff.** Per-tenant configurable budget; token counter + API-cost estimator;
   SOFT warning at a configurable threshold (e.g. 80%) then HARD cutoff at 100%. Not
   warning-only; not cutoff-only. Both enforcement stages are required from day one.

3. **S3 default enabled/disabled.** — **RESOLVED 2026-06-26 (human).** S3 is
   `s3_agent_runtime_enabled = false` by default across ALL deployment types (cloud,
   satellite/edge, air-gapped). Tenant/admin opts in per deployment. S1 BYO-agent is
   always available. Air-gap and OT-regulated deployments are safe by default. Conversation
   state is browser-local in v1. See Deployment Gating section for full specification.
