---
document_type: proposed-adr
status: proposed
do_not_execute: true
decided: "2026-06-26 (human)"
candidate_adr_slots:
  - "ADR-050: Central deployment topology (transport selection)"
  - "ADR-051: Per-connection analyst identity model"
  - "ADR-052: Central credential custody design (shared with SS-26)"
  - "ADR-053: Shared state access (alerts/cases across analysts)"
  - "ADR-054: Central service operational model (scaling, fairness, graceful shutdown)"
produced_by: architect
timestamp: "2026-06-26"
traces_to:
  - matured-vision-day2-requirements.md §3.1 (central deployment pivot; access layer gap)
  - matured-vision-day2-requirements.md §11.1 (SS-26 secret broker; hybrid credential store)
  - matured-vision-day2-requirements.md §11.2 (central config store)
  - matured-vision-day2-requirements.md §5.2 (DI-017 amendment; DI-NEW-006)
  - matured-vision-day2-requirements.md §5.3 (NFR-CONCURRENCY-FAIRNESS)
  - matured-vision-day2-requirements.md §11.3 (multi-surface UI; four surfaces over one central backend)
  - day2-design-decisions/secret-subsystem-sketch.md (SS-26 SecretBackend trait; per-tenant DEK)
  - day2-design-decisions/ADR-PROP-storage-engine-taxonomy.md (Postgres control-plane for case/alert state)
  - domain-spec/invariants.md (DI-017 single-process → single-central-service; DI-NEW-006 cross-analyst isolation)
  - research/central-deployment-access-layer-2026-06-26.md (primary research basis — all five topics)
---

# ADR-PROP — Central Deployment & Multi-Analyst Access Layer (C1)

> **STATUS: PROPOSED — DECIDED 2026-06-26 (human).** This is a CAPTURE artifact.
> `do_not_execute: true`. It does NOT modify live ADR files, ARCH-INDEX.md, or any live factory
> artifact. The real ADR numbers (ADR-050..054) and formal ARCH-INDEX.md rows are deferred to the
> morph execution (post-demo, post-T14, gated on brief-reframe sign-off §5.1).

> **Research basis:** `research/central-deployment-access-layer-2026-06-26.md` (fetched sources:
> MCP spec 2025-06-18 transports + authorization; rmcp 1.7.0 Context7 docs; SOC concurrency
> prior art; scale/fairness patterns). All load-bearing claims are source-grounded in that doc.

---

## Context

### The pivot is access-layer-only

prism's DATA-PLANE is already fully multi-tenant. `OrgId` / `OrgSlug` / `OrgRegistry` / `Arc-DI`
threading are wired and correct. DI-017 ("serves a single analyst") describes the per-analyst
STDIO deployment, which remains valid and is retained.

The GAP between the per-analyst model and a central multi-analyst service is entirely in the
ACCESS LAYER:

| Capability | Status today | What C1 adds |
|---|---|---|
| Tenant model (OrgId/OrgSlug/OrgRegistry) | EXISTS — data-plane fully multi-tenant | Nothing. Reused as-is. |
| Per-tenant config/credential scoping | EXISTS (reference-based, AD-017) | Resolution backend moves analyst-local → server (SS-26); contract unchanged. |
| Transport | EXISTS — stdio, single-analyst (DI-017) | ADDITIVE: Streamable HTTP transport option; stdio retained. |
| Per-connection identity | PARTIAL — session exists; identity assumed = single OS user | ADDITIVE: identity captured from transport (OAuth bearer → analyst), not assumed. |
| AuthN/AuthZ | Implicit (local process trust) | ADDITIVE: OAuth 2.1 RS + per-connection capability gate + OrgId scope binding. |
| Audit identity | EXISTS (BC-2.05.002) but bound to single process user | AMEND: bind to per-connection analyst identity. |
| Cross-analyst isolation | N/A (single analyst) | NEW invariant DI-NEW-006. |
| Shared case/alert state | Does not exist as multi-analyst store | NEW (ADR-053). |
| Stateful-service ops/scaling/fairness | Single-process laptop model | NEW (ADR-054). |

This document records the DECIDED posture for each of the five C1 sub-decisions (transport,
identity, credentials, shared state, ops). The decisions are inter-dependent and are captured
together so the morph can produce ADR-050..054 from a single coherent source.

---

## Decision

### C1-D1: Transport — ADR-050

**DECIDED 2026-06-26 (human): MCP Streamable HTTP (spec revision 2025-06-18) + keep stdio.**

**Mechanism:**
- Adopt **MCP Streamable HTTP** as the central multi-analyst transport. This transport
  *replaced* the deprecated HTTP+SSE transport (deprecated 2025-03-26 revision); HTTP+SSE
  must NOT be implemented.
- **Keep stdio** for single-analyst / local deployment. Stdio is unchanged. It is not
  deprecated by this decision.
- Implement via the in-tree **`rmcp 1.7.0` `StreamableHttpService`** — NOT new protocol code.
  The transport pivot is a wiring + axum mount + middleware exercise.

**rmcp `StreamableHttpService` configuration:**
- `stateful_mode: true` — create and maintain a session per `Mcp-Session-Id` header.
- `allowed_hosts` / `allowed_origins` — set explicitly for the central deployment endpoint;
  the default loopback-only restriction does NOT ship for the central service.
- `session_store: Option<Arc<dyn SessionStore>>` — back with a shared store from day one
  if multi-instance scaling is in scope (avoids sticky-init re-architecture later).
- `cancellation_token` — hooks the graceful-shutdown drain (C1-D5 below).

**SSE resumability:** SSE events carry per-stream `id`; clients reconnect with `Last-Event-ID`
and the server MAY replay. This eliminates sticky sessions — any instance can serve a
reconnecting client if the session store is shared.

**Spec revision note:** rmcp 1.7.0 targets spec revision 2025-06-18. A newer 2025-11-25 revision
exists; confirm rmcp support level at morph time before committing to the newer revision.

**DI-017 amendment framing (§5.2):**
"Single-process" → "single-central-service; the stdio transport constrains to single-analyst
per-process; the central (Streamable HTTP) transport enables multi-analyst per-process. The
single logical session per transport connection is preserved."

### C1-D2: Identity — ADR-051

**DECIDED 2026-06-26 (human): MCP server = OAuth 2.1 Resource Server (MCP Authorization spec
2025-06-18); bearer token → analyst identity → OrgId scope → per-connection capability gate.**

**Mechanism:**
- prism's central MCP server implements the **OAuth 2.1 Resource Server** role per the MCP
  Authorization spec (revision 2025-06-18).
- Server MUST validate bearer tokens per request; MUST enforce token audience (RFC 8707 Resource
  Indicators — tokens issued for other resources are rejected); MUST publish
  **Protected Resource Metadata (RFC 9728)** at the well-known URL; MUST respond with
  `WWW-Authenticate` + 401 on unauthorized requests.
- Token → **analyst identity** (subject claim) → **OrgId tenant scope** (token claim or
  directory lookup) → **per-connection capability gate** — gates against the existing
  write-gate feature-flag model. The write-gate model is NOT replaced; the capability gate
  uses it as the authoritative source for which operations are gated.
- **Stdio transport stays env-credential** (unchanged). The MCP Authorization spec states
  stdio SHOULD NOT use OAuth — Prism aligns with this.
- **DI-NEW-006:** a connection's in-flight query state and confirmation tokens are
  per-connection; cross-analyst access is explicitly forbidden. `Mcp-Session-Id` is the
  enforcement key.
- **Audit identity (BC-2.05.002 amendment):** bind to the validated per-connection analyst
  identity from the token, not the OS process user.

**Authorization Server posture (DECIDED 2026-06-26):**
- **Built-in OAuth 2.1 AS + external IdP (OIDC / SAML)** — hybrid. Mirrors §11.1 hybrid
  secret-store stance: ship a first-party built-in AS for self-contained / air-gap
  deployments AND integrate with the customer's external IdP.
- The AS is not in scope for ADR-051 itself (ADR-051 is the RS contract); the AS is a
  separate architectural surface adjacent to SS-26 and SSO (see `ADR-PROP-sso-identity.md`).

### C1-D3: Credentials — ADR-052 (+ SS-26)

**DECIDED 2026-06-26 (human): SS-26 design stands. The multi-analyst access layer adds only
per-connection-analyst AUDIT BINDING; the core credential custody model is unchanged.**

The credential custody design is already captured in `day2-design-decisions/secret-subsystem-sketch.md`
(SS-26 Secret Broker, `SecretBackend` trait, per-tenant-DEK envelope, hybrid built-in + external
backends). This section records only the access-layer-additive delta:

**Delta from C1:**
1. **Per-connection analyst identity on every resolution:** every credential resolution is
   now attributable to *which analyst's connection* triggered it. The audit record carries
   `{analyst_id, session_id, OrgId, credential_ref, timestamp}` — no secret value, per AD-017.
2. **Per-tenant DEK isolation is a SECURITY BOUNDARY, not just a config nicety:** with many
   analysts/orgs sharing one process, cryptographic per-OrgId isolation means a compromised
   DEK is scoped to one org. Binds to the same OrgId boundary the data-plane already enforces.
3. **Concurrent resolution under load:** many analysts → many concurrent broker calls. The
   broker must be safe under fan-out and sits inside the per-tenant fairness budget (C1-D5).

SS-26 design questions (HD-1..HD-5) remain in `secret-subsystem-sketch.md` and are NOT
re-decided here.

**Satellite/residency interaction unchanged:** at OT/edge enclaves, the `SecretBackend` is
satellite-local; secrets resolve at the satellite; only sanitized OCSF results transit to
central (§3.2 §11.1).

### C1-D4: Shared State (Alerts / Cases) — ADR-053

**DECIDED 2026-06-26 (human): FULL case-management on BUNDLED PostgreSQL (central-only),
with OPTIMISTIC CONCURRENCY (per-record version/ETag CAS) + SOFT OWNERSHIP +
PRESENCE HINTS; no hard pessimistic record locks.**

**Workload rationale (reconciled with §14.3):**
§14.3 rejected PostgreSQL for the ephemeral correlation/detection path (RocksDB-native). That
ruling stands. Case-management is a categorically different workload: long-lived collaborative
records, multi-analyst edits, secondary-index queries, CAS-on-version, audit history. The storage
taxonomy ADR (`ADR-PROP-storage-engine-taxonomy.md`) records this as a CONSCIOUS workload-lane
decision, not a reversal of §14.3.

**Concurrency model (SOC/TheHive/SOAR prior art):**
- **Optimistic concurrency (version/ETag CAS):** `UPDATE case SET status=…, version=version+1
  WHERE case_id=… AND version=N`. Returns `409 Conflict` on stale write; client re-reads and
  retries. No long-held row locks.
- **Soft ownership / assignment:** a case's `assigned_to` field is a recommendation, not an
  edit lock. Any authorised analyst can update the case; the `assigned_to` field reflects who
  "owns" it semantically. Hard pessimistic locks are NOT implemented.
- **Presence hints:** when analyst A is actively editing a case, the server broadcasts a
  presence event to other analysts viewing the same case (`resources/updated` SSE notification
  via rmcp). This is advisory UX, not a blocking lock.
- **Conflict UX:** `409` + full current state returned in the error body. Client shows the
  analyst the conflicting version; analyst chooses to overwrite or merge. Default: show conflict,
  require explicit re-submit.

**Full case-management scope (DECIDED 2026-06-26):**
Status / assignment / case-wall notes / inter-case links / disposition — all on PostgreSQL.
This is the §11.3.1 Findings/Alerts screen surface (`status`, `notes`, assignment, replay link).

**Store:** BUNDLED PostgreSQL (see `ADR-PROP-storage-engine-taxonomy.md` §Decision — Postgres
central-only, never external/cloud, never at a Satellite).

### C1-D5: Operations — ADR-054

**DECIDED 2026-06-26 (human): stateless-leaning front + shared state; SSE resumability
(no sticky sessions); readiness-flip + connection-drain graceful shutdown;
per-analyst/per-tenant fairness via nested Tokio Semaphores + weight-aware fan-out scheduler.**

**Scaling posture:**
- **Stateless-leaning front:** MCP/Streamable HTTP request handling is stateless per request;
  session state is in the shared `SessionStore` (C1-D1). Identical instances can handle any
  request for a session. Horizontal scale = add instances + shared state store.
- **SSE resumability via `Last-Event-ID`:** no sticky sessions needed. A reconnecting client
  lands on any available instance; the instance restores session context from the `SessionStore`.
- **Shared state:** case/alert store on BUNDLED PostgreSQL (C1-D4); session store (rmcp
  `SessionStore`) on a shared backend; notification fan-out via `resources/updated` SSE +
  optionally a shared pub/sub for cross-instance broadcast.

**Fairness (§5.3 NFR-CONCURRENCY-FAIRNESS):**
- No single analyst's queries may consume >50% of `MAX_FANOUT_CONCURRENCY` for >10 consecutive
  seconds.
- Mechanism: **per-analyst `Semaphore` nested under the existing global
  `HTTP_SEMAPHORE_PERMITS = 200`** and `MAX_FANOUT_CONCURRENCY = 10` budgets.
- **Do NOT conflate** with the 8/8 prism-operations scheduler split (ADR-022 §D, D-209) — that
  is a separate subsystem. Per-analyst fairness applies to the central multi-analyst MCP path only.
- **Weight-aware fair-queue:** weighted fair queueing over per-tenant/per-analyst queues so
  large tenants cannot starve small ones. Implementable as Tokio bounded channels with a
  weight-based scheduler.

**Memory budget under multi-analyst load (DC-004):**
- The 512MB/200MB laptop default is superseded by a configurable server-sized budget (DC-004
  §4 and §5.3 NFR-015 amendment). The per-query 200MB budget × N concurrent analysts is
  admission-controlled; under contention, new queries are queued or rejected with
  `E-QUERY-NNN` (query-budget-exceeded).

**Graceful shutdown:**
1. Flip readiness endpoint false → removed from load-balancer / Claude Code client pool.
2. Stop accepting new MCP connections.
3. Drain in-flight requests with a bounded timeout (configurable; default 30s).
4. Emit a "shutting down" SSE notification to connected analysts so clients can reconnect
   elsewhere.
5. Invoke rmcp `cancellation_token` → terminates all active sessions + stops the listener.
6. Exit.

**Health / readiness endpoints:** standard HTTP `/healthz` (liveness) and `/readyz` (readiness)
over the same axum mount as the MCP endpoint. Liveness = process alive. Readiness = all
critical dependencies (PostgreSQL, RocksDB) reachable and healthy.

---

## Open Questions (NOT blocking morph; flagged for architect at ADR-050..054 authorship)

| # | Question | ADR | Notes |
|---|----------|-----|-------|
| OQ-1 | Target MCP spec revision 2025-06-18 vs 2025-11-25? | ADR-050 | Gate on rmcp 1.7.0 support level for the newer revision. |
| OQ-2 | Single-instance central v1 → multi-instance later, or multi-instance from day one? | ADR-050 / ADR-054 | Feature-ordering lever (Canonical Principle Rule 2). Forces shared `SessionStore` + shared notification bus immediately if multi-instance v1. |
| OQ-3 | OrgId scope source — token claim vs server-side analyst→org directory? | ADR-051 | Affects multi-org analysts (MSSP operator seeing all client orgs). |
| OQ-4 | Capability granularity: per-tool, per-table, per-source? | ADR-051 | Must reconcile with existing write-gate feature-flag model — NOT a parallel RBAC. |
| OQ-5 | Confirmation-token isolation (DI-NEW-006): binding to `Mcp-Session-Id` lifetime? | ADR-051 | Tied to `WATCH…UNLESS` per-session confirmation model. |
| OQ-6 | Notification fan-out: MCP `resources/updated` SSE is sufficient for same-instance broadcast; cross-instance broadcast needs a shared pub/sub. What backing (Redis-class)? Does this cut against air-gap self-sufficiency? | ADR-054 | The recurring built-in-vs-external axis (§3 of the research consolidated open questions). |
| OQ-7 | Per-analyst (vs per-tenant) credential-resolution rate budget for compromised/abusive connection detection? | ADR-052 | Extends SS-26 §HD-2 (rotation) scope. |
| OQ-8 | Case conflict UX: 409-then-merge vs last-write-wins-with-audit? | ADR-053 | UX decision; lean is 409-then-merge per SOC prior art. |

---

## Consequences

### Positive
- The data-plane REMAINS unchanged. Zero risk to existing sensor adapters, query engine,
  prism-operations, and all live BCs.
- Transport adoption (ADR-050) is low-risk: in-tree rmcp 1.7.0 implements Streamable HTTP;
  the pivot is wiring + axum mount. Not new protocol code.
- Identity adoption (ADR-051) is standards-based: MCP Authorization spec 2025-06-18 +
  OAuth 2.1. The RS validation + RFC 9728 metadata endpoints are net-new application code
  but small, well-specified, and well-understood.
- Credential custody (ADR-052) is already designed (SS-26). C1 adds only the per-connection
  audit binding — one field in the audit record.
- Shared state (ADR-053) on PostgreSQL eliminates the app-built optimistic-CAS + index
  burden that a RocksDB-native case store would have imposed.

### Costs / risks
- **ADR-053 (shared case state)** is the largest genuinely new engineering surface: schema
  design, optimistic-CAS plumbing, presence-hint SSE, `409` conflict handling, PostgreSQL
  lifecycle management in the central service.
- **ADR-054 (stateful-service ops)** introduces a service-level operational surface prism has
  not had before: health/readiness endpoints, connection-drain logic, multi-analyst fairness
  semaphores, and (for multi-instance) a shared session store and notification bus.
- **DI-017 amendment** has downstream BC ripples: BC-2.10.001 / BC-2.10.006 (transport),
  BC-2.05.002 (audit identity), BC-2.04.002 / BC-2.04.011 (credential). These are PO scope
  at morph time (§5.5 of matured-vision).
- **Thesis preservation check:** all five C1 decisions stay inside the ephemeral / federated /
  residency-first / AI-opaque thesis. The one area to guard is the notification pub/sub bus
  (OQ-6) — a careless "just add Redis" would cut against air-gap self-sufficiency; the
  built-in-first stance applies here as it does to secrets (SS-26) and identity (SSO).
