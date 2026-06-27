---
document_type: proposed-adr
status: proposed
do_not_execute: true
decided: "2026-06-26 (human)"
proposed_adr_slot: "ADR-047..054 already reserved; this decision lands AFTER those — real number deferred to architect at morph (ADR-055+); use ADR-PROP-storage-engine-taxonomy as the stable capture ID until then"
produced_by: architect
timestamp: "2026-06-26"
traces_to:
  - matured-vision-day2-requirements.md §3.3 (RocksDB RetentionCache hot tier)
  - matured-vision-day2-requirements.md §14.3 (correlation/detection state = RocksDB-native; PostgreSQL rejected for this path)
  - matured-vision-day2-requirements.md §11.2 (central config store, RBAC, audit, tenant/user)
  - matured-vision-day2-requirements.md §3.1 (central deployment, shared alert/case state)
  - matured-vision-day2-requirements.md §3.2 (Satellite mesh, edge-local control-plane)
  - matured-vision-day2-requirements.md §15.3 (three retention tiers; model-state CF)
  - day2-design-decisions/secret-subsystem-sketch.md (SS-26 credential/DEK store on Postgres control-plane)
  - domain-spec/invariants.md (DI-017 single-central-service; DI-NEW-001..004 RetentionCache)
  - research/central-deployment-access-layer-2026-06-26.md §Topic 4 (shared case/alert store choice; RocksDB-vs-relational tension)
  - research/ingestion-open-subthreads-2026-06-26.md §3 (sub-thread 3 checkpoint cadence — RocksDB state-backend; §4 pcap; §6 edge compute; §7 residency)
---

# ADR-PROP — Storage Engine Taxonomy (Four-Engine Model)

> **STATUS: PROPOSED — DECIDED 2026-06-26 (human).** This is a CAPTURE artifact.
> `do_not_execute: true`. It does NOT modify live ADR files, ARCH-INDEX.md, or any live factory
> artifact. The real ADR number and the formal ARCH-INDEX.md ADR row are deferred to the
> morph execution (post-demo, post-T14, gated on brief-reframe sign-off §5.1).

---

## Context

The 2026-06-24 vision session established that prism is maturing from a per-analyst stdio process
into a central multi-tenant service with a Satellite mesh. As that maturation proceeded across
§§3, 11, 14, 17, a recurring store-choice question resurfaced in multiple forms:

- **§14.3** explicitly rejected PostgreSQL for correlation/detection state and chose RocksDB-native.
- **§11.2** introduced a central config store with RBAC, audit, versioned change control, and
  rollback — a workload that is relational by nature.
- **§3.1 / ADR-053** surface area: shared alert/case state across analysts (multi-key transactions,
  secondary indexes, optimistic CAS, collaborative records) — also relational.
- **§3.2** (Satellite mesh): edge nodes need a lightweight local control-plane for enrollment/identity
  state, local policy, and operational metadata — without a heavy datastore.
- **§3.3 addendum** (day-2 2026-06-25): RetentionCache cold tier = Apache Iceberg
  (Parquet-on-object-store + catalog) for long-baseline OCSF + native event/metadata storage,
  days → multi-year.

Each of these is a **different workload class**. Applying a single storage engine uniformly across
all of them is a category error. The research in
`research/central-deployment-access-layer-2026-06-26.md §Topic 4` makes the workload-class
distinction explicit:

> "Case-management state is a **different workload:** long-lived collaborative records,
> multi-analyst edits, secondary-index queries (by assignee/status/severity/time),
> CAS-on-version, audit history. These two [§14.3 correlation state and case-management] are
> **not the same store question** and conflating them would be an error."

The human decision (2026-06-26) crystallised a four-engine taxonomy, each engine in its
workload lane, and explicitly reconciled the §14.3 no-PostgreSQL ruling with the new control-plane
posture.

---

## Decision

**DECIDED 2026-06-26 (human): prism operates a FOUR-ENGINE storage taxonomy. Each engine owns
a distinct workload lane; no engine crosses into another's lane.**

| Engine | Lane | Tier / Scope | Nodes |
|--------|------|--------------|-------|
| **RocksDB** | Ephemeral/hot DATA-PLANE | Correlation & detection state; RetentionCache hot tier; continuous-operator window/sequence state; store-and-forward queues | Central AND every Satellite |
| **Apache Iceberg** | Cold ANALYTIC tier | Long-baseline OCSF + native event/metadata; `RETAIN` multi-year; columnar; partition-pruned on `event_time`/`eventDay` | Central / regional (NOT at edge Satellites by default) |
| **PostgreSQL (BUNDLED in the central appliance — NEVER external/cloud)** | Relational CONTROL-PLANE | Case-management + alerts; central config store (§11.2); RBAC; audit log; tenant/user; identity/AS state; result-cache METADATA | **Central-only** |
| **SQLite (embedded)** | Satellite-local CONTROL-PLANE | Local config; enrollment/identity state; local policy + operational metadata | Satellite / edge (NOT at central) |

### Per-Engine Workload-Fit Rationale

#### RocksDB — ephemeral/hot data-plane (central + every Satellite)

RocksDB is the right engine for prism's ephemeral, high-write, key-range-scan workload:

- **Write throughput:** LSM-tree absorbs bursty sensor fan-out writes. CFs provide logical
  isolation at negligible overhead. Existing 19-CF pattern (prism-core/storage.rs `ALL_DOMAINS`).
- **Short-TTL KV lookups:** RetentionCache hot tier (seconds → hours/days); correlation windows;
  operator checkpointing (incremental SSTable-based, §17.7 / ingestion-open-subthreads §3).
- **Store-and-forward:** Satellite queues; intermittent-connectivity buffering (§3.2).
- **No join / secondary-index requirement:** ephemeral detection state is keyed by
  `(rule_id, match_key, window)` — point lookup + range scan, not relational query.
- **Single-binary / air-gap:** RocksDB embeds in the Rust binary. Satellites carry no external
  process dependency. This is load-bearing for the OT/Purdue placement (§3.2, §17.13).

#### Apache Iceberg — cold analytic tier (central / regional)

Iceberg is the right engine for long-baseline OCSF + native event storage:

- **Columnar + partition-pruned:** `event_time`/`eventDay` predicate pushdown cuts I/O on
  multi-year scans. zstd-in-Parquet for object-storage cost.
- **Schema evolution:** OCSF is versioned (1.1, 1.3, …); Iceberg schema-evolution absorbs OCSF
  version drift without migration (§13.6 G-16).
- **Multi-schema tables:** keyed by `(source-class, schema, schema-version)` — OCSF-vN tables
  AND native schema-on-read tables for cached non-security data (§3.3 addendum, §13.6).
- **Unified cold-cache / Security Lake read path:** Amazon Security Lake IS OCSF-as-Iceberg.
  The cold-cache read path and the lake read path are the **same DataFusion + Iceberg
  TableProvider** — one mechanism, not two (§3.3 addendum, §3.5).
- **Time-travel / snapshot:** cold-tier replay for backtesting (§14.3). Model-state versioning
  for ML audit (§15.5).

**Why NOT Iceberg for case-management:**
Iceberg is optimised for OLAP append-mostly workloads. Case-management is OLTP collaborative:
- Table-level snapshot commits are expensive under concurrent multi-analyst writes (multi-second
  round-trips; commit conflicts under contention).
- No point-lookup indexes: retrieving one case by ID requires a scan or an external catalog.
- Multi-row CAS ("update case status where version = N") requires app-built logic on top.
- Expensive row updates: Iceberg's row-level deletes are implemented as merge-on-read or
  copy-on-write rewrites — not single-row in-place updates.
- Transaction conflicts under concurrent multi-analyst case edits are a first-class failure mode
  for SOC workflows (the SOC/TheHive/SOAR prior art, `central-deployment-access-layer-2026-06-26.md
  §Topic 4`, shows optimistic CAS over short OLTP transactions is the canonical pattern).

#### PostgreSQL (BUNDLED, central-only) — relational control-plane

PostgreSQL is the right engine for prism's relational control-plane workload:

- **Full-text + secondary indexes:** case-management queries by assignee / status / severity / time
  are natural relational queries, not range scans over a KV store.
- **Optimistic CAS:** `UPDATE … WHERE version = N RETURNING …` with `409` on conflict is
  PostgreSQL's native pattern. Multi-row transactions under concurrent multi-analyst edits
  are safe and cheap.
- **RBAC, audit, tenant/user, identity/AS:** strictly relational; referential integrity between
  entities (user → role → permission → resource) is textbook PostgreSQL territory.
- **Result-cache METADATA:** coverage annotations, freshness watermarks, policy records — small,
  structured, queried with joins. NOT the cache payload itself (that stays in RocksDB/Iceberg).
- **Config store with versioned change control + rollback (§11.2):** Git-backed or DB-backed
  config history both naturally map to relational row-versioning.
- **SS-26 secret-broker credential/DEK metadata:** encrypted secret envelope records +
  per-tenant-DEK references are small relational rows; the actual DEK/ciphertext may be in
  the same store or a separate CF, but the metadata layer is relational.

**BUNDLED constraint — non-negotiable:**
PostgreSQL is deployed as a component of the **central appliance** (like SQLite is embedded in
the Satellite binary). It is NEVER an external/cloud-managed dependency (RDS, Cloud SQL, Azure
PostgreSQL). This preserves:
- **Air-gap compatibility** for on-prem central deployments.
- **Self-sufficiency** — the central service carries everything it needs.
- **Operational simplicity** — no external managed DB endpoint to configure, secure, or pay for.

PostgreSQL packaging: bundled via the `prism start` launch sequence, data directory inside the
prism data root, lifecycle managed by the prism service supervisor (or a sidecar process under
the same systemd/container unit). Exposed only on the loopback interface.

**Central-only constraint — non-negotiable:**
PostgreSQL does NOT deploy to Satellites. Edge nodes carry SQLite for their local control-plane.
Mixing PostgreSQL into edge deployments would break:
- Single-binary / minimal-footprint Satellite deployment (OT/Purdue, air-gap enclaves).
- Satellite self-sufficiency under connectivity loss.
- The lean, mesh-compatible operational model.

#### SQLite (embedded) — satellite-local control-plane

SQLite is the right engine for Satellite-local control-plane state:

- **Zero external dependencies:** pure Rust SQLite bindings (`rusqlite`) embed in the Satellite
  binary. No separate process. No setup.
- **Enrollment/identity state:** the Satellite's trust anchor, parent endpoint, chain-depth,
  loop-prevention ID set — a handful of small relational records.
- **Local policy + operational metadata:** per-instance config overrides, local TOML
  binding/exception state (§17.8 Q2 policy), receiver-mode config — not a high-write workload.
- **Local collector / receiver state:** locus-node config; active buffer pointers; per-source
  durability contract (§17.4).

SQLite does NOT hold analytics data (that goes to RocksDB hot tier at the edge node).
SQLite does NOT hold correlation state (that goes to RocksDB, also at the edge node).
SQLite is strictly the control-plane: enrollment, identity, policy, metadata. Small, durable,
reads dominate.

---

## §14.3 Reconciliation (the load-bearing nuance)

§14.3 explicitly REJECTED PostgreSQL and chose RocksDB-native for correlation/detection state.
That decision was about **correlation/risk/campaign state over the RetentionCache window** —
short-lived, ephemeral, federated, append/scan-shaped, high-write. The ruling stands and is
unchanged: **the DATA-PLANE (ephemeral correlation path) stays RocksDB-native.**

Introducing PostgreSQL for the CONTROL-PLANE is a **different workload class** and is NOT a
silent reversal of §14.3. The distinction is:

| Dimension | §14.3 DATA-PLANE (RocksDB) | New CONTROL-PLANE (PostgreSQL) |
|-----------|---------------------------|-------------------------------|
| Lifetime | Ephemeral (seconds → days, TTL-driven) | Persistent (weeks → years) |
| Write pattern | High-rate append / point-upsert | Low-rate, analyst-paced mutations |
| Access pattern | Range scan + point lookup by key | Secondary-index queries, JOINs, CAS |
| Concurrency | Single-tenant or fan-out isolation | Multi-analyst collaborative records |
| Transactions | Single-CF WriteBatch (prism-native) | Multi-row ACID transactions |
| Workload shape | OLAP/KV (optimized for sensor fan-out) | OLTP/relational (case-mgmt, RBAC, audit) |
| Deployment scope | Central + every Satellite | Central-only |

**This is a CONSCIOUS decision, not a scope creep.** The no-PostgreSQL ruling in §14.3 was
explicitly scoped to the ephemeral correlation path. The human decision of 2026-06-26 introduces
PostgreSQL in a different workload lane (control-plane) where it is manifestly the right tool.
The production-grade default (CLAUDE.md) demands the correct engine for the workload, not
dogmatic uniformity.

---

## Ripple Effects (must be picked up at morph time)

| Affected area | Ripple |
|---------------|--------|
| **C9 config-management (§11.2)** | Central config store lands on PostgreSQL (central-only). Per-tenant config rows, versioned mutations, rollback history, config-change audit log. |
| **§11 admin RBAC / audit / identity** | All RBAC tables, audit-log rows, tenant-user-role mappings land on PostgreSQL (central-only). |
| **A#2 result-cache METADATA** | Coverage annotations, freshness watermarks, per-tier replication metadata land on PostgreSQL (central-only). NOT the cache payload (RocksDB/Iceberg). |
| **ADR-053 (shared case/alert state)** | The store-choice tension documented in `central-deployment-access-layer-2026-06-26.md §Topic 4` is resolved: BUNDLED PostgreSQL with optimistic CAS + soft ownership + presence hints (see ADR-PROP-central-deployment-access-layer.md §Decision). |
| **SS-26 secret broker (secret-subsystem-sketch.md)** | Credential envelope records + per-tenant-DEK metadata land on PostgreSQL. The DEK ciphertext itself may live in the same store or a dedicated CF — to be resolved at ADR-052 morph. |
| **Satellite binary** | SQLite added as a dev-dependency. RocksDB stays. No PostgreSQL at the edge. Satellite `Cargo.toml` gains `rusqlite`. |
| **Central service binary** | PostgreSQL client added (`tokio-postgres` or `sqlx`). The bundled Postgres instance is managed by the service supervisor (systemd/container). |
| **BC families** | New BC families needed (at PO authorship time): case-management CRUD + CAS; RBAC CRUD; audit-log append + query; config-store versioning + rollback. |
| **ADR authorship** | Real ADR number allocated at morph. This proposed ADR supersedes the ad-hoc PostgreSQL-rejected note in §14.3 by explicitly scoping each engine to its workload lane. |

---

## Alternatives Considered

### Alternative A: RocksDB-everywhere (no PostgreSQL at all)

Consistent with §14.3's RocksDB-native posture. Operationally simpler (one storage engine
everywhere). Prior prism principle.

**Rejected because:** building optimistic CAS, secondary indexes (by assignee/status/severity/time),
multi-key atomicity, and versioned audit history on top of RocksDB for the collaborative case-management
workload is a substantial engineering burden with no off-the-shelf support. RocksDB `TransactionDB`
is feasible but is app-built DBMS-lite. The `central-deployment-access-layer-2026-06-26.md §Topic 4`
research explicitly flags this: "real engineering + correctness burden." For the control-plane workload
(relational, collaborative, long-lived, low write-rate), the correct tool is a relational engine.

The RocksDB-everywhere choice would also conflict with the production-grade default (CLAUDE.md):
"the correct mechanism, not the cheapest mechanism." Forcing case-management onto RocksDB because
we already have it is the cheapest mechanism.

### Alternative B: External / cloud-managed PostgreSQL

Use a managed PostgreSQL endpoint (RDS, Cloud SQL, etc.) instead of a bundled instance.

**Rejected because:** it breaks air-gap compatibility for on-prem and OT deployments, adds an
external dependency that must be configured and secured, and conflicts with the self-sufficiency
thesis that anchors prism's Satellite model. The "BUNDLED in the appliance" constraint is
non-negotiable for the same reason SSH-ingress into an OT enclave is not an option.

### Alternative C: Iceberg for case-management

Use the Iceberg cold tier (already present for analytics) to store case-management records.

**Rejected because:** Iceberg is OLAP/append-mostly. The failure modes for collaborative
case-management are exactly the Iceberg anti-patterns: table-level snapshot commits are expensive
under contention; there are no point-lookup indexes; multi-row CAS requires out-of-band logic;
row updates are expensive (merge-on-read or copy-on-write). See "Why NOT Iceberg for case-management"
in the Decision section above.

---

## Honest Cost

Prism now operates FOUR storage engines:

- **2 central-only:** PostgreSQL (control-plane) + Apache Iceberg (analytics cold tier).
- **1 everywhere:** RocksDB (data-plane, ephemeral/hot).
- **1 edge-only:** SQLite (Satellite local control-plane).

**Operational cost:**
- Central service: 3 engines (RocksDB + PostgreSQL + Iceberg access). The Postgres process is
  bundled and supervisor-managed; the Iceberg storage is backed by an object store (local or cloud).
  Two processes (prism + postgres) in the central deployment unit.
- Satellite: 2 engines (RocksDB + SQLite). Both embedded. One process.
- No Satellite carries Postgres or Iceberg. The mesh stays lean.

**Engineering cost:**
- `tokio-postgres` / `sqlx` added as central-service dependencies.
- `rusqlite` added as Satellite dependency.
- Schema migrations for the PostgreSQL control-plane (versioned, reversible — applies §11.2 GitOps
  config discipline to the DB schema itself).
- Iceberg DataFusion TableProvider integration already planned for the cold-cache + Security Lake path.

**Risk mitigation:**
- PostgreSQL is well-understood; the BUNDLED constraint removes the operational surface area.
- SQLite is already in the Rust ecosystem (`rusqlite`); minimal new dependency.
- The four-lane taxonomy makes the engine choice a DECISION, not an accident, reducing future
  "why is this on the wrong engine" confusion.
