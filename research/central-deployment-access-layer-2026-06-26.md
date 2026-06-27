---
document_type: research
produced_by: research-agent
status: capture
do_not_execute: true
timestamp: "2026-06-26"
provenance: >
  side-analysis discussion input; does not modify vision/specs. Cited research pass to
  inform a DISCUSSION on prism's CENTRAL DEPLOYMENT & multi-tenant access layer
  (matured-vision-day2-requirements.md §3.1). NOT a spec/vision change. No live factory
  file modified; no git operation performed.
traces_to:
  - matured-vision-day2-requirements.md §3.1 (central deployment pivot)
  - matured-vision-day2-requirements.md §11.1 (server credential custody)
  - matured-vision-day2-requirements.md §11.3 (multi-surface UI / central HTTP transport)
  - matured-vision-day2-requirements.md §14.3 (RocksDB-native correlation state)
  - .factory/specs/day2-design-decisions/secret-subsystem-sketch.md (SS-26 secret broker)
  - domain-spec/invariants.md DI-017 (single-process → single-central-service)
  - project memory AD-017 (AI-opaque credentials)
  - candidate ADR-050..054, DI-NEW-006
---

# Central Deployment & Multi-Tenant Access Layer — Cited Research (Side-Analysis)

> PROPOSED discussion input. Status: capture. Not a spec, not an ADR, not a vision change.
> Section numbering is internal to this document.

## Executive Summary (~12 lines)

1. **The pivot is access-layer-only and the MCP spec already provides the primitives.** prism's data-plane is already multi-tenant (OrgId/OrgSlug/OrgRegistry/Arc-DI). The gap — multi-analyst transport, per-connection identity, central credential custody, shared case/alert state — maps cleanly onto existing standards; nothing here requires re-architecting the data-plane.
2. **Transport: adopt MCP Streamable HTTP (revision 2025-06-18), keep stdio.** Streamable HTTP *replaced* the deprecated HTTP+SSE transport (deprecated as of the 2025-03-26 revision; HTTP+SSE was the 2024-11-05 transport) [MCP-TRANSPORTS]. A single MCP endpoint (POST + optional GET/SSE) with an `Mcp-Session-Id` header gives per-connection sessions out of the box. This is the lean for **ADR-050**.
3. **The Rust SDK (rmcp, pinned 1.7.0 in-tree) already ships this.** `StreamableHttpService` with `stateful_mode`, `Mcp-Session-Id` issuance, `allowed_hosts`/`allowed_origins` (DNS-rebinding defense), `session_store: Option<Arc<dyn SessionStore>>` for cross-instance recovery, and `AuthClient` bearer-token plumbing [RMCP-DOCS]. The transport pivot is largely a **wiring + axum mount + middleware** exercise, not new protocol code.
4. **Identity: the MCP server becomes an OAuth 2.1 Resource Server.** The MCP Authorization spec (2025-06-18) makes the server an OAuth 2.1 RS that MUST validate bearer tokens for *its own* audience (RFC 8707), MUST publish Protected Resource Metadata (RFC 9728), and uses `WWW-Authenticate`/401 discovery [MCP-AUTHZ]. This is the standards basis for **ADR-051**. Token → analyst identity → OrgId scope → per-connection capability gate.
5. **Credential custody is already sketched (SS-26) and unchanged in design; only the *audit binding* is additive.** The broker-injects-at-I/O-boundary, AI-opaque (AD-017) contract is preserved; what's new is binding every resolution to the per-connection analyst identity from ADR-051. Do not redo §11.1/SS-26.
6. **Shared case/alert state is the one genuinely new design surface — and a store-choice tension.** Industry SOC tools (TheHive, SOAR) use *soft ownership + optimistic concurrency (version/ETag CAS)*, rarely hard pessimistic locks [SOC-CONCURRENCY]. The honest tension: RocksDB is excellent as a KV engine but multi-key transactions, secondary indexes, and CAS for *collaborative* records are app-built on top — a relational store is the textbook fit [SOC-CONCURRENCY]. §14.3 already *rejected* PostgreSQL for correlation state and is RocksDB-native; case-management state is a *different* workload and the choice must be reconciled, not assumed.
7. **Ops model: lean stateless-front + shared-state where possible; treat live SSE sessions as budgeted stateful capacity** [SCALE-PATTERNS]. SSE resumes via `Last-Event-ID` so it needs no sticky sessions; readiness-flip + connection-drain is the standard graceful-shutdown play. Multi-tenant fairness (§5.3 NFR) → per-tenant Tokio `Semaphore`/bounded-channel budgets + weighted fair queueing.
8. **Net:** transport + identity are low-risk standards adoption (ADR-050/051) backed by in-tree SDK support; credential custody is already designed (ADR-052/SS-26); the real open design work is **shared state store choice (ADR-053)** and **stateful-service scaling/fairness (ADR-054)**.

---

## Read-coverage note (honesty)

- **Read in-repo:** matured-vision §3.1, §5.2 (DI-NEW-006), §5.3 (NFR fairness), §5.4 (ADR-050..054 + SS registry), §11.1, §14.3; secret-subsystem-sketch.md frontmatter + §1–3; rmcp-subscribe-notify research (rmcp pinned **1.7.0**); grep of `crates/` transport surfaces (prism-mcp server.rs, boot.rs).
- **NOT exhaustively read:** full BC bodies (BC-2.10.x transport, BC-2.05.002 audit identity, BC-2.04.x credential) — cited from §3.1's amendment list, not from the BC text itself. Full SS-26 §4–7 (DEK hierarchy) — frontmatter + problem statement read; design details deferred to that doc by scope.
- **External sources:** MCP spec pages fetched directly (authoritative); rmcp API via Context7 (docs.rs); SOC-concurrency and scaling patterns via Perplexity with citations. Two large `perplexity_research` deep-research calls completed but their raw output exceeded the readable-token cap and was not fully parsed — superseded by direct-source fetches (MCP spec) and focused cited `perplexity_ask` calls, which is the *stronger* evidence path for version-specific claims. Flagged in Research Methods.

---

## The additive-vs-existing boundary (the core reconciliation — Topic 6)

| Capability | Status today | What the pivot adds |
|---|---|---|
| Tenant model (OrgId/OrgSlug/OrgRegistry) | **EXISTS** — data-plane fully multi-tenant, Arc-DI wired | Nothing. The cryptographic/isolation boundary is reused as-is. |
| Per-tenant config/credential scoping | **EXISTS** (reference-based, AD-017) | Resolution *backend* moves analyst-local → server (SS-26); contract unchanged. |
| Transport | **EXISTS** — stdio, single-analyst (DI-017) | **ADDITIVE**: Streamable HTTP transport option; stdio retained. |
| Per-connection identity | **PARTIAL** — per-connection session exists; identity *assumed* = single OS user | **ADDITIVE**: identity *captured from the transport* (OAuth bearer → analyst), not assumed. |
| AuthN/AuthZ | implicit (local process trust) | **ADDITIVE**: OAuth 2.1 RS + per-connection capability/authZ + OrgId scope binding. |
| Audit identity | **EXISTS** (BC-2.05.002) but bound to single process user | **AMEND**: bind to per-connection analyst identity from ADR-051. |
| Cross-analyst isolation | n/a (single analyst) | **NEW invariant DI-NEW-006**: a connection cannot reach another analyst's in-flight query state / confirmation tokens. |
| Shared case/alert state | **does not exist as a multi-analyst store** | **NEW** (ADR-053) — the largest genuinely-new surface. |
| Stateful-service ops/scaling/fairness | single-process laptop model | **NEW** (ADR-054) — health/readiness, draining, per-tenant fairness NFR. |

**DI-017 amendment framing (confirms §5.2 line 571).** "single-process" → "single-central-service; the *stdio* transport constrains to single-analyst per process, the *central (Streamable HTTP)* transport enables multi-analyst per process. The single logical session per transport connection is preserved." This is consistent with the MCP spec's own framing: a "session" is the logically-related interaction set keyed by `Mcp-Session-Id`, and the server "operates as an independent process that can handle multiple client connections" [MCP-TRANSPORTS].

---

## Topic 1 — MCP transport for a central multi-analyst service

**Prior art (authoritative, verified against the spec).**
- The current MCP spec (revision **2025-06-18**) defines **two** standard transports: **stdio** and **Streamable HTTP**. Streamable HTTP **"replaces the HTTP+SSE transport from protocol version 2024-11-05"**; HTTP+SSE is **deprecated** [MCP-TRANSPORTS]. (The intermediate 2025-03-26 revision introduced Streamable HTTP; a newer 2025-11-25 spec revision exists per the spec index, but 2025-06-18 is the revision rmcp 1.7.0 targets and is the safe anchor — flag for architect: confirm whether to target 2025-11-25 at build time.)
- **Streamable HTTP mechanics** [MCP-TRANSPORTS]:
  - Single MCP endpoint path (e.g. `/mcp`) supporting **POST and GET**.
  - Client POSTs JSON-RPC; server returns either `application/json` (single response) **or** `text/event-stream` (SSE upgrade for streaming/server-initiated messages). Notifications/responses → `202 Accepted`.
  - Client `GET` opens a server→client SSE stream (server-initiated requests/notifications).
  - **Session model:** server MAY issue `Mcp-Session-Id` on the `InitializeResult`; client MUST echo it on all subsequent requests. Server MAY terminate (then 404 → client re-initializes). Client SHOULD `DELETE` to end a session.
  - **Resumability:** SSE events carry per-stream `id`; client reconnects with `Last-Event-ID`; server MAY replay *on the same stream*. This makes reconnection cheap and **stateless across instances** if the event log is shared.
  - **Security MUSTs:** validate `Origin` (DNS-rebinding), bind localhost when local, authenticate connections.
- **Serving many analysts:** "the server operates as an independent process that can handle multiple client connections." Each analyst = one (or more) `Mcp-Session-Id`. Per-connection identity is layered via the Authorization spec (Topic 2), not the transport itself.

**rmcp (the Rust SDK, in-tree pinned 1.7.0) already implements this** [RMCP-DOCS]:
- `StreamableHttpService` / `StreamableHttpServerConfig` with:
  - `stateful_mode: bool` — "create a session for each request and keep it alive"; session id in `Mcp-Session-Id` header.
  - `json_response: bool` — return `application/json` directly when stateless (spec-allowed, 2025-06-18) to skip SSE framing for simple request/response tools.
  - `allowed_hosts` / `allowed_origins` — Host/Origin validation; **defaults to loopback-only** (DNS-rebinding defense) — public deployment MUST override.
  - `cancellation_token` — cancels all sessions + stops accepting (graceful-shutdown hook).
  - `session_store: Option<Arc<dyn SessionStore>>` — **external session store for cross-instance recovery**; persists the client's `initialize` params on handshake, restores transparently when a request lands on an instance with no in-memory session. This is the exact primitive for horizontal scaling without sticky init.

**Lean (ADR-050):** Adopt **Streamable HTTP, target spec revision 2025-06-18, mount via axum, keep stdio for local single-analyst.** Use rmcp `StreamableHttpService` with `stateful_mode=true`. Set `allowed_hosts`/`allowed_origins` explicitly for the central deployment (do NOT ship loopback defaults). Plan to back `session_store` with a shared store from day one if multi-instance is in scope.

**Open Qs:**
- Target 2025-06-18 or the newer 2025-11-25 revision? (Confirm rmcp support level before committing.)
- Single-instance first (in-memory sessions) vs multi-instance with shared `SessionStore` from the start? (Couples to ADR-054 scaling lean.)
- Streamable HTTP vs a future WebSocket custom transport for the S2 browser console — SSE is sufficient for one-way streaming + server-initiated notifications and avoids sticky sessions [SCALE-PATTERNS]; reserve WS only if bidirectional high-frequency is needed.

---

## Topic 2 — Per-connection analyst identity (authN/authZ)

**Prior art (MCP Authorization spec, revision 2025-06-18, fetched directly)** [MCP-AUTHZ]:
- Authorization is **OPTIONAL** in MCP, but HTTP-transport implementations **SHOULD** conform; **stdio SHOULD NOT** use it (credentials come from the environment — matches prism's current stdio model).
- **The MCP server acts as an OAuth 2.1 Resource Server.** The MCP client is the OAuth client; the Authorization Server is separate (may be co-hosted or external — out of MCP's scope).
- Hard requirements relevant to prism:
  - Server **MUST** implement **Protected Resource Metadata (RFC 9728)** with `authorization_servers`.
  - Server **MUST** use `WWW-Authenticate` on `401` to advertise RS-metadata URL.
  - AS **MUST** provide Authorization Server Metadata (**RFC 8414**); DCR (**RFC 7591**) SHOULD be supported.
  - Client **MUST** send `Authorization: Bearer <token>` on **every** HTTP request (even within a session); tokens **MUST NOT** be in the query string.
  - Server **MUST** validate the token **audience** is itself (**RFC 8707** Resource Indicators) — reject tokens issued for other resources; **token passthrough is forbidden** (confused-deputy defense). When the MCP server calls upstream APIs it uses a *separate* token and MUST NOT forward the client's.
  - Error codes: `401` (unauthorized/invalid), `403` (insufficient scope), `400` (malformed).
  - PKCE **MUST** be used; tokens SHOULD be short-lived; refresh-token rotation for public clients.
- rmcp ships the client side (`AuthClient` auto-fetches/injects bearer tokens, supports `delete_session` with bearer) [RMCP-DOCS]; the **server-side RS validation + metadata endpoints are the part prism must implement** (rmcp provides the transport; audience validation + capability mapping is application logic).

**Mapping to prism's model:**
- bearer token → **analyst identity** (subject claim) → **OrgId tenant scope** (claim or directory lookup) → per-connection **capability/authZ** enforcement against the existing feature-flag/write-gate model (project memory: writes gated behind feature flags).
- **DI-NEW-006** (§5.2) is the isolation invariant: a session's in-flight query state and confirmation tokens are per-connection; cross-analyst access is forbidden. Streamable HTTP's per-session `Mcp-Session-Id` is the enforcement key; the audit identity (BC-2.05.002 amendment) is derived from the validated token, not the OS process user.

**Lean (ADR-051):** **MCP server = OAuth 2.1 Resource Server.** Validate bearer per request, enforce audience (RFC 8707), publish RFC 9728 metadata + `WWW-Authenticate`. Derive analyst identity from the validated token; resolve OrgId scope from a tenant claim; gate capabilities per-connection. Treat the AS as pluggable/external (customer IdP via OIDC) with a built-in option for self-contained deployments — mirrors the §11.1 "built-in + external" stance for secrets.

**Open Qs:**
- Bring-your-own IdP (OIDC) only, or ship a built-in AS for air-gapped/satellite deployments (parallels the §11.1 hybrid secret-store decision)?
- Where does OrgId scope live — token claim, DCR-time binding, or a server-side analyst→org directory? (Affects multi-org analysts.)
- Capability model granularity: per-tool, per-table, per-source? Reconcile with the existing write-gate feature-flag system rather than inventing a parallel RBAC.
- Confirmation-token isolation (DI-NEW-006) lifetime + binding to `Mcp-Session-Id`.

---

## Topic 3 — Central credential custody at multi-analyst scale

**This design is already captured (§11.1 + SS-26 secret-subsystem-sketch). Do NOT redo it.** The relevant *multi-analyst access* deltas only:

- The custody **contract is unchanged**: `CredentialRef` in → resolved secret injected by the broker **at the HTTP I/O boundary**, never into PrismQL results, MCP output, logs, or agent context (AD-017 preserved/hardened). Pluggable `SecretBackend` (built-in envelope-encrypted per-tenant-DEK store + external Vault/AWS-SM/GCP-SM/Azure-KV).
- **What multi-analyst changes:**
  1. **Audit identity on every resolution** is now the **per-connection analyst identity** (from ADR-051), not a single process user. Every credential resolution is attributable to *which analyst's connection* triggered it. This is the one place the access-layer pivot reaches into SS-26.
  2. **Per-tenant DEK isolation is reinforced as a security boundary, not just a config nicety** — with many analysts/orgs sharing one process, cryptographic per-OrgId isolation means a compromise scoped to one org's DEK cannot decrypt another's. This binds to the same OrgId boundary the data-plane already enforces.
  3. **Concurrent resolution under load:** many analysts → many concurrent broker calls; the broker must be safe under fan-out (it sits behind the same per-tenant fairness budget as Topic 5).
  4. **Satellite/residency:** at OT/edge enclaves, the `SecretBackend` is satellite-local; secrets resolve at the satellite and never transit to central — only sanitized OCSF results flow upward (§3.2). Unchanged by multi-analyst central transport.

**Lean:** SS-26 design stands. The only access-layer addition is **per-connection-analyst audit binding** + treating per-tenant-DEK isolation as the cryptographic complement to OrgId data-plane isolation. Route credential-custody design questions to SS-26/ADR-052, not here.

**Open Qs (access-layer only; crypto/DEK Qs belong to SS-26 HD-1..HD-5):**
- Does the audit record store analyst-id + session-id + OrgId + ref + timestamp (no secret value)? (Confirm against BC-2.05.002 amendment.)
- Is there a per-analyst (vs per-tenant) credential-resolution rate budget to bound a compromised/abusive connection?

---

## Topic 4 — Shared alert/case state across analysts (the genuinely new surface)

**Prior art** [SOC-CONCURRENCY]:
- **TheHive** is explicitly built for multi-analyst collaboration: multiple analysts work the *same case simultaneously* with live activity updates. Public docs show **per-object discrete updates** (cases/tasks/observables) — **no hard record-locking**; behavior is consistent with **optimistic concurrency at the API level**, not long-held pessimistic locks.
- **Modern SOAR/SIRP** (Splunk SOAR, Cortex XSOAR): multiple analysts view/act on one incident; updates are **small field-level transactions** (status, owner, note, artifact); **"assigned-to" is soft ownership**, not an edit lock. Hard pessimistic UI locks are uncommon.
- **Concurrency-control best practice for incident records:** **optimistic concurrency (version/ETag CAS) at the record/field level**, returning `409 Conflict` on stale writes, over short DB transactions; pessimistic locking only for high write contention. Add UX presence hints ("Alice is editing") rather than blocking.
- **Store choice tension (the honest cost):** an embedded KV store (RocksDB) gives high write throughput but **secondary indexes, multi-key transactions, and cross-key CAS must be app-built**; a **relational/transactional store (e.g. PostgreSQL) is the textbook fit** for collaborative records needing rich querying, multi-row updates, audit/history, and built-in CAS [SOC-CONCURRENCY].

**The reconciliation with §14.3 (the load-bearing nuance).** §14.3 explicitly **rejected PostgreSQL** and is **RocksDB-native** — but that decision was about **correlation/risk/campaign state over the RetentionCache window** (short-lived, ephemeral, federated, append/scan-shaped). **Case-management state is a different workload:** long-lived collaborative records, multi-analyst edits, secondary-index queries (by assignee/status/severity/time), CAS-on-version, audit history. These two are **not the same store question** and conflating them would be an error.

Three honest options to surface (do not pre-decide — this is ADR-053's job):
- **(A) RocksDB-native case store** (consistent with §14.3 RocksDB-native posture; existing 19-CF storage layer; no new datastore). Cost: build optimistic-CAS (per-record version CF), secondary indexes (index CFs), and multi-key atomicity (RocksDB `WriteBatch`/`TransactionDB`) yourself. RocksDB *does* offer `TransactionDB` with optimistic & pessimistic modes — feasible but is app-built DBMS-lite. Honest cost: real engineering + correctness burden.
- **(B) Relational store for case/alert state only**, RocksDB retained for cache/correlation. Cost: introduces a datastore §14.3 rejected *for correlation* — must be argued as a *different* workload; adds an operational dependency that cuts against the air-gap/satellite self-sufficiency thesis unless an embedded SQL engine (e.g. SQLite-class) is acceptable.
- **(C) Embedded transactional engine** (embedded SQL, e.g. SQLite/`redb`-class) — keeps single-binary/air-gap self-sufficiency while getting CAS/indexes/transactions natively. Middle path; worth evaluating against the RocksDB-native posture.

**Lean (ADR-053):** Use **optimistic concurrency (per-record version CAS, `409` on conflict) + soft ownership/assignment + presence hints** for the access semantics regardless of store — that part is settled by SOC prior art. **Defer the store choice to ADR-053 as an explicit open decision**, framing it as a *distinct* workload from §14.3 correlation state. Do not silently assume RocksDB-native carries over; do not silently introduce PostgreSQL. Lean **toward (A) or (C)** to preserve the air-gap/satellite/single-binary thesis, with the cost of optimistic-CAS + index plumbing made explicit.

**Open Qs:**
- Is case/alert state in scope for the *central* pivot now, or does it follow the alerting/findings model in §14.5 (Alert{} with statuses) on its own timeline?
- Real-time presence/notification fan-out: reuse the MCP `resources/updated` + `notifications/*` SSE path (already wired per rmcp-subscribe-notify research) rather than a new websocket layer?
- Per-analyst vs shared visibility: shared within OrgId; what about cross-org analysts (MSSP operators) — does an operator see all orgs' cases, scoped by authZ?
- Conflict UX: 409-then-merge vs last-write-wins-with-audit?

---

## Topic 5 — Central service operational model

**Prior art** [SCALE-PATTERNS]:
- **Stateless front + shared state scales best**; keep state in external stores, scale by adding identical replicas. **Stateful** designs buy locality/latency at the cost of scaling/failover complexity — justified only for high-update in-memory coordination.
- **Long-lived streams:** **SSE is stateless across reconnects** (`Last-Event-ID` lets any instance resume) → **no sticky sessions needed**, unlike websockets which usually pin app state to a TCP connection. This is a strong argument to keep the analyst-facing stream on **Streamable HTTP/SSE** (Topic 1) rather than raw websockets.
- **Scaling streams:** shard connections across pods, broadcast via a shared pub/sub (Redis/Kafka-class), keep per-connection memory small, expose per-tenant connection metrics for autoscaling/backpressure.
- **Multi-tenant fairness:** per-tenant concurrency budgets (refuse/queue at limit), **weighted fair queueing** (per-tenant queues + weights so big tenants don't starve small ones), and per-tenant rate limits (conn creation, msgs/sec, fan-out size). In Rust/Tokio this maps to **per-tenant `Semaphore`s / bounded channels** + a weight-aware scheduler.
- **Health vs readiness + draining:** *liveness* = restart-if-stuck; *readiness* = remove-from-LB-but-keep-running. Graceful shutdown for long-lived streams = **flip readiness false → stop accepting new conns → drain/close in-flight with an explicit "shutting down" event so clients reconnect elsewhere → exit after timeout.** rmcp's `cancellation_token` is the in-process hook for "terminate all sessions + stop accepting" [RMCP-DOCS].

**Mapping to prism:**
- The **NFR-CONCURRENCY-FAIRNESS** idea (§5.3: no analyst's queries consume >50% of `MAX_FANOUT_CONCURRENCY` for >10s) is exactly a per-tenant/per-analyst fairness budget — implementable as a nested per-analyst `Semaphore` layered under the existing global `HTTP_SEMAPHORE_PERMITS=200` and `MAX_FANOUT_CONCURRENCY=10` budgets (do **not** conflate with the 8/8 prism-operations scheduler split per CLAUDE.md concurrency note).
- **Memory budget (§5.3 NFR-015 amendment):** parameterized — 512MB laptop default, GB-range central server. The fairness budget interacts with the 200MB-per-query budget under multi-analyst load.

**Lean (ADR-054):** Architect prism central as **stateless-leaning front (MCP/Streamable HTTP) + shared state** so it can scale horizontally; back `session_store` and the case store (Topic 4) with shared stores; rely on **SSE resumability to avoid sticky sessions**. Implement **per-analyst/per-tenant fairness via nested Tokio `Semaphore`s + a weight-aware fan-out scheduler**; wire **liveness/readiness endpoints + readiness-flip-then-drain graceful shutdown** using rmcp's `cancellation_token`.

**Open Qs:**
- Single-instance (stateful, simplest) for v1 of central, with multi-instance scale-out gated to a later wave? (Feature-order lever, not a shortcut — explicit phasing.)
- Shared pub/sub for cross-instance notification fan-out — what backing (Redis-class) and does that cut against air-gap self-sufficiency? (Mirrors the secret-store + IdP "built-in vs external" tension.)
- Fairness scheduler: hard per-tenant cap (simple, can waste capacity) vs work-conserving WFQ (better utilization, more complex)?
- Per-query 200MB budget × N concurrent analysts vs total process budget — admission control policy under contention.

---

## Consolidated open design questions (for the discussion)

**Cross-cutting / decision-level (human/architect):**
1. Single-instance central v1 → multi-instance later (explicit feature-order phasing), or multi-instance from day one (forces shared `session_store` + shared notification bus + shared case store immediately)?
2. The recurring **"built-in vs external" axis** appears three times — secret store (§11.1, decided: hybrid), IdP/AS (Topic 2, open), notification/pub-sub bus (Topic 5, open). Air-gap/satellite self-sufficiency argues for built-in defaults everywhere with external integration optional. Decide the axis once, apply consistently.
3. **Shared case/alert store choice (ADR-053)** — the single largest genuinely-new design decision. Frame explicitly as a *different workload* from §14.3 correlation state; do not inherit RocksDB-native by default nor reach for PostgreSQL by default.

**Spec-mechanical (answerable in scope at morph time):**
4. Target MCP spec revision (2025-06-18 vs 2025-11-25) — gate on rmcp 1.7.0 support.
5. OrgId scope source (token claim vs directory) and capability granularity, reconciled with the existing write-gate feature-flag model.
6. DI-NEW-006 enforcement binding (confirmation-token ↔ `Mcp-Session-Id` lifetime).
7. BC amendments enumerated in §3.1 (BC-2.10.001/006 transport, BC-2.05.002 audit identity, BC-2.04.002/011 credential) — product-owner scope.

## Honest costs / risks

- **Lowest risk:** transport (ADR-050) and identity (ADR-051) — standards adoption with in-tree SDK support (rmcp 1.7.0 `StreamableHttpService` + `AuthClient`). Server-side OAuth RS validation + RFC 9728 metadata endpoints are net-new application code but small and well-specified.
- **Already designed:** credential custody (ADR-052/SS-26) — only per-connection-analyst audit binding is additive.
- **Real engineering cost:** shared case/alert store (ADR-053) — optimistic-CAS + secondary indexes + multi-key atomicity is a meaningful build regardless of store; the store choice carries an air-gap-self-sufficiency-vs-built-it-yourself tradeoff with no free option.
- **Moderate cost / well-understood:** stateful-service scaling + fairness (ADR-054) — standard patterns, but multi-instance scale-out pulls in a shared session store + notification bus, which reopens the built-in-vs-external axis.
- **Thesis preservation:** every lean above stays inside the ephemeral / federated / residency-first / AI-opaque thesis. The one place to guard is ADR-053's store choice — a careless "just add Postgres" would cut against air-gap/satellite self-sufficiency; flag it as a first-class decision, not an implementation detail.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 2 | Deep multi-source on (a) MCP transport+authorization spec and (b) stateful multi-tenant Rust/Tokio scaling + SOC shared-state. Both completed but raw output exceeded the readable-token cap; superseded by direct-source fetches + focused asks (stronger version-grounded evidence). |
| Perplexity perplexity_ask | 2 | Focused cited synthesis: SOC case-concurrency (TheHive/SOAR, optimistic vs pessimistic, RocksDB vs relational); stateless-front/fairness/draining patterns. |
| Context7 | 2 (resolve + query) | rmcp Rust SDK — `StreamableHttpService`, `StreamableHttpServerConfig` (`stateful_mode`, `json_response`, `allowed_hosts/origins`, `session_store`, `cancellation_token`), `AuthClient` bearer integration. Verified against docs.rs. |
| WebFetch | 2 | Authoritative MCP spec pages (2025-06-18): transports + authorization. Direct quotes of MUST/SHOULD, headers, status codes, RFC numbers. |
| Read / Grep | several | In-repo: matured-vision §3.1/§5.x/§11.1/§14.3, secret-subsystem-sketch, rmcp version pin (1.7.0), transport surfaces in crates/. |
| Training data | 1 area | General OAuth 2.1 / Tokio Semaphore mechanics framing only — all load-bearing claims are externally sourced. |

**Total MCP tool calls:** 6 (2 perplexity_research + 2 perplexity_ask + 2 Context7) plus 2 WebFetch on the authoritative spec.
**Training data reliance:** low — transport/auth claims verified against the MCP spec directly; rmcp API verified via Context7/docs.rs; SOC + scaling patterns carry inline citations.

### Source key
- [MCP-TRANSPORTS] modelcontextprotocol.io/specification/2025-06-18/basic/transports (fetched 2026-06-26).
- [MCP-AUTHZ] modelcontextprotocol.io/specification/2025-06-18/basic/authorization (fetched 2026-06-26): OAuth 2.1 RS, RFC 9728/8414/7591/8707, RFC 9068.
- [RMCP-DOCS] docs.rs/rmcp (Context7 `/websites/rs_rmcp`, 2026-06-26): `StreamableHttpService`, `StreamableHttpServerConfig`, `AuthClient`. In-tree pin: rmcp 1.7.0 (Cargo.lock; rmcp-subscribe-notify-api.md).
- [SOC-CONCURRENCY] perplexity_ask cited set incl. docs.strangebee.com (TheHive), event-driven.io, bytebytego.com, databricks.com/blog/concurrency-control, stackoverflow optimistic-vs-pessimistic.
- [SCALE-PATTERNS] perplexity_ask cited set incl. aerospike.com, getstream.io (websocket vs SSE), highscalability.com (stateful services), redhat.com (stateful vs stateless), reddit r/softwarearchitecture (horizontal scaling stateful).
