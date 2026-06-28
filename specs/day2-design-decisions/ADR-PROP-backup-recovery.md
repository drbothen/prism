---
document_type: proposed-adr
status: capture
do_not_execute: true
decided: "2026-06-27 (human)"
candidate_adr_slots:
  - "ADR-PROP-C17-1: Per-store backup/restore mechanics — pgBackRest WAL for Config DB; git bundle for detection content; RocksDB Checkpoint+BackupEngine for working state; Iceberg catalog+metadata+data for cold tier; authoritative slice for KG+vector; sealed-blob for keys; config-DB-class vs RocksDB-class for ARO"
  - "ADR-PROP-C17-2: Cross-store coherent point-in-time — logical-watermark + per-store time-travel as primary; T = AS OF KNOWN <T> C8 watermark; backup-set manifest binding per-store snapshot IDs to one T; selective physical freeze for tightly-coupled core only"
  - "ADR-PROP-C17-3: KEY escrow/recovery under Option-3 per-tenant CMEK — tenant-held recovery key default; optional M-of-N threshold escrow tier; crypto-shred as erasure + zero-access reconciliation; operator stores only sealed/wrapped blobs it cannot unwrap"
  - "ADR-PROP-C17-4: Per-tenant + nested backup granularity — cluster backup + per-tenant logical export (SOLVED); per-tenant PITR = restore-to-side-instance + selective re-ingestion (GENUINELY HARD); nested parent/subtree/child scopes; silo escape-hatch"
  - "ADR-PROP-C17-5: Satellite recovery — reconstruct-from-central default; local data buffer backup; air-gap nodes get signed+encrypted offline-media bundles verifiable without online control plane"
  - "ADR-PROP-C17-6: DR tier ladder per deployment-profile/contract — Backup-Restore / Pilot-Light / Warm-Standby / Active-Active; SaaS default multi-AZ + optional multi-region; on-prem VM-snapshot + config-as-data + optional HA-pair"
  - "ADR-PROP-C17-7: Unified integrity model — detached signatures + content-hash/Merkle + customer-managed-key encryption across satellite/SaaS/on-prem"
  - "ADR-PROP-C17-8: CIP-009 recovery evidence first-class — Prism generates timestamped restore-test runs + integrity-verification records + post-restore CIP-010 baseline diff; RSAW export packaging consolidated in C20"
produced_by: architect
timestamp: "2026-06-27"
provenance: >
  Day-2 out-of-band SIDE-ANALYSIS capture. Decision item C17 — backup & recovery as a
  first-class architectural citizen. Human-confirmed decisions 2026-06-27. CAPTURE ONLY.
  Does NOT modify any live spec, ADR-registry artifact (specs/architecture/), BC, story,
  STATE.md, or SESSION-HANDOFF.md. No git operation performed. Real ADR numbers and formal
  ARCH-INDEX.md rows deferred to the morph execution cycle.
  touches_no_live_artifacts: true
seeded_from:
  - research/backup-recovery-2026-06-27.md (PRIMARY — per-store mechanics, cross-store
    logical-watermark, key-escrow, per-tenant/nested, satellite, DR tiers, CIP-009,
    verified crate versions, sub-forks)
cross_refs:
  - specs/day2-design-decisions/ARCHITECTURE-DESIGN-SYSTEM.md (INV-ADS-02 operator-zero-access;
    P-ADS-02/04; PAT-ADS-03 signed bundle; Section C.2 conformance checklist; PAT-ADS-15/16
    backup patterns; INV-ADS-10 recoverability-preserves-zero-access)
  - specs/day2-design-decisions/ADR-PROP-prismql-deliverables.md (C8 bitemporality AS OF
    KNOWN <T> — the watermark T that C17 reuses as cross-store recovery point)
  - specs/day2-design-decisions/ADR-PROP-entity-masking.md (C16 per-tenant vault/DEK custody
    + crypto-shred context)
  - specs/day2-design-decisions/ADR-PROP-nested-tenancy.md (C19 nested backup scopes, Option-3
    child-keyed CMEK, closure table)
  - specs/day2-design-decisions/ADR-PROP-compliance-profiles.md (DR-tier + key-custody as
    Profile settings axes)
  - specs/day2-design-decisions/secret-subsystem-sketch.md (SS-26 per-tenant DEK hierarchy —
    key escrow reuses this infrastructure)
  - matured-vision-day2-requirements.md §16.4 (C17 decision log bullet)
---

# ADR-PROP — Backup & Recovery as a First-Class Citizen (C17)

> **STATUS: DECIDED 2026-06-27 (human).** Full decision record for C17 — backup & recovery
> as a first-class architectural concern covering per-store mechanics, cross-store coherent
> point-in-time, key escrow/recovery under Option-3 CMEK, per-tenant + nested granularity,
> satellite recovery, DR tier ladder, unified integrity model, and CIP-009 recovery evidence.
> CAPTURE artifact (`do_not_execute: true`). Real ADR numbers and formal ARCH-INDEX.md rows
> deferred to morph execution. Seeded from `research/backup-recovery-2026-06-27.md`.

---

## 1 — Context and Scope

Backup & recovery is not a bolt-on concern for Prism; it intersects every storage tier,
every tenancy boundary, and the operator-zero-access invariant (INV-ADS-02). An MSSP
platform that cannot recover a tenant's data — or that requires the operator to hold
a usable key to perform recovery — has a fundamental architectural defect.

C17 resolves the design space across six axes:

1. Per-store backup/restore mechanics (Config DB, detection content git, RocksDB working
   state, Iceberg cold tier, KG+vector, per-tenant keys, ARO state)
2. Cross-store coherent point-in-time (tied to C8 bitemporality via the `AS OF KNOWN <T>`
   watermark)
3. KEY escrow/recovery preserving operator-zero-access (the hardest axis, under Option-3
   per-tenant CMEK)
4. Per-tenant + nested-tenant backup granularity (C19)
5. Satellite state recovery (ties C9 + C2)
6. Per-deployment-model DR and unified integrity model (ties CIP-009 / C20)

**Settled context (not re-litigated here).** Stores to protect are: config DB (PostgreSQL
DB-authoritative, system-versioned), detection content (embedded git2), RocksDB working
state (~19 CFs), Iceberg cold tier (C5), Context KG+vector (indradb+usearch+lancedb,
C12), per-tenant DEK/CMEK keys (SS-26 + Option-3), ARO state (C15). Operator-zero-access
is a hard architectural invariant (INV-ADS-02) the backup design must preserve. Bootstrap
config recovery for restart-class keys is covered separately in C9; this document covers
DATA-STORE + KEY backup/recovery.

---

## 2 — Decisions

### D-C17-Q1 — Per-Store Backup/Restore Mechanics

#### 2.1 Config DB — PostgreSQL PITR

**Decision:** Postgres continuous WAL archiving via **pgBackRest 2.55** (verified
2026-06-27: latest stable ~2026-05). Time-based retention (`repo-retention-full`),
client-side encryption, and `--type=time` restore to a `recovery_target_time`. The
bitemporal history tables in the Config DB coexist with coarse cluster PITR: cluster
PITR is the epoch rewind; the bitemporal history table is the fine-grained AS-OF axis.
Per-tenant PITR within a pooled store requires a restore-to-side-instance + selective
re-ingestion workflow (see D-C17-Q3). **[VERIFIED — pgbackrest.org; crates.io research]**

#### 2.2 Detection Content — Embedded git (git2 0.21.0)

**Decision:** Back up via **`git bundle`** (NOT `git clone` — `git clone` is not a backup;
it omits server-side metadata and cannot guarantee a point-in-time image) or an atomic
filesystem snapshot of `.git` taken when `git gc` is not mid-run. git objects are
immutable; an atomic FS snapshot is consistent. Back up periodically — `git gc` and
reflog expiry prune unreachable objects and may destroy recoverability if the bundle
predates the last GC. This is the backup/restore complement to C9's fast-revert design;
C9 governs revert of detection content; C17 governs archive/restore to a new host.

**Crate:** `git2` **0.21.0** (2026-05-18, crates.io — verified). Bundle/pack backup API
coverage requires confirmation at morph — the crate API docs do not enumerate this surface
explicitly. Use `run_git_bundle` via shell or libgit2 raw API if the safe high-level binding
is absent. **[INCONCLUSIVE — confirm git2 bundle API at morph; git bundle mechanism itself
is VERIFIED: `research/backup-recovery-2026-06-27.md` §1.2]**

#### 2.3 RocksDB Working State (~19 Column Families)

**Decision:** Two mechanisms used deliberately:
- **Checkpoint API** (`rocksdb::Checkpoint::create_checkpoint`) — fast, consistent,
  O(SST-file-count), hard-linked live SSTs + MANIFEST + WAL copy. Best for hot backup
  and seeding a follower. Run with memtable flush before snapshot for a fully self-contained
  SST image.
- **BackupEngine API** (`rocksdb::backup`) — retained incremental backup set, hard-links
  unchanged SSTs across backups (storage-efficient). Best for the historical backup set
  with selective prune.

**Both APIs are confirmed exposed** in `rocksdb` **0.24.0** (2025-08-10, crates.io —
verified via docs.rs `rocksdb/latest` — modules: `checkpoint`, `backup`). This resolves the
inconclusive flag from the research transcript.

**PITR-like:** pair with `Options::WAL_ttl_seconds` for replay within the TTL window
(obsolete WAL moves to `archive/` then deletes after TTL); beyond TTL, PITR granularity
drops to the backup cadence.

**Prism note:** RocksDB is the working/cache tier. Back up only NON-RECONSTRUCTIBLE CFs.
Pure-cache CFs are disposable. The split between authoritative and disposable CFs must be
explicitly catalogued at morph. **[VERIFIED — rust-rocksdb docs.rs + research §1.3]**

#### 2.4 Iceberg Cold Tier (C5)

**Decision:** Back up ALL THREE parts: (1) the **catalog** (Hive/REST/JDBC — via its native
RDBMS backup); (2) all **metadata files + manifest lists** (the snapshot history within
retention); (3) the **data files** (object store — replicate/back up). Copying data files
alone is insufficient; metadata + catalog pointer are required to reconstruct. Manage
retention via `expire_snapshots` (age/count); pin survivors with branches/tags so they
survive expiry. Drive maintenance (`expire_snapshots`, `deleteOrphanFiles`) via the
canonical Java/Spark procedures or a verified Rust path.

**Crate:** `iceberg` (iceberg-rust) **0.9.1** (2026-05-06, crates.io — verified). Pre-1.0.
The Rust binding has not been confirmed to expose snapshot-expiration/maintenance ops. Do
NOT assume Rust coverage; prefer the canonical Java/Spark procedures for maintenance ops
until the Rust path is production-verified. Build a backup+restore verification harness
before production-trusting this surface (production-grade default, CLAUDE.md §Canonical
Principle Rule 1). **[VERIFIED — iceberg.apache.org spec; iceberg-rust 0.9.1 data-access
binding verified; maintenance op Rust-coverage INCONCLUSIVE — flag at morph]**

#### 2.5 KG + Vector (C12: indradb + usearch + lancedb)

**Decision (partial — INCONCLUSIVE until morph):** Back up the **authoritative
(non-recomputable) embeddings + graph edges**. ANN indexes (usearch) are rebuildable from
the vector corpus — treat as disposable; do NOT back up the index file itself if the
vector corpus is backed up. LanceDB is itself a versioned columnar store (supports its own
versioning/time-travel); indradb holds graph state in RocksDB; usearch is the index.

**Sub-decision:** the authoritative vs rebuildable classification for each of these stores
is **INCONCLUSIVE** — direct per-store backup docs for indradb / usearch / lancedb were
not in the research corpus. A dedicated per-store pass at morph must confirm each store's
native snapshot/backup API before the classification is finalized.

**OQ-C17-001:** Confirm indradb, usearch, and lancedb native backup/snapshot APIs at morph.
Classify each as: (a) has native snapshot — use it; (b) RocksDB-backed (indradb) — use
the RocksDB Checkpoint/BackupEngine path via the underlying RocksDB options; (c) columnar
store (lancedb) — use its native versioning; (d) index-only (usearch) — treat as
rebuildable from vector corpus, no backup required.

#### 2.6 Per-Tenant DEK / CMEK Keys — the Hardest Target

Full treatment in D-C17-SF1 (§2.8). Keys are the highest-consequence backup target.
**Back up keys as sealed/wrapped blobs** (operator-stored, operator-unable-to-unwrap);
recovery capability is held by the tenant and/or threshold custodians.

#### 2.7 ARO State (C15)

**Decision (partial — INCONCLUSIVE until morph):** Classify ARO state by the authoritative
vs derived taxonomy.

**OQ-C17-002:** Confirm the ARO state split at morph:
- **Authoritative ARO config/decisions** (what the operator-resolved-objects represent as
  configuration) → Config-DB-class backup (PostgreSQL PITR, system-versioned tables,
  continuous WAL archiving via pgBackRest).
- **In-flight/runtime ARO scratch** (transient state, in-progress recommendation queues)
  → RocksDB-class working state (back up only the non-reconstructible portion; purely
  runtime scratch is disposable).

Until the ARO split is confirmed at morph, default to: back up ALL ARO state via the
Config-DB-class path; revisit to classify and exclude only after the split is confirmed.

---

### D-C17-SF2 — Cross-Store Coherent Point-in-Time (PRIMARY MECHANISM DECIDED)

**Decision (SF-2): LOGICAL-WATERMARK + PER-STORE TIME-TRAVEL as the primary cross-store
consistent backup strategy. NOT a global physical application-consistent freeze.**

**Mechanism:**
1. Stamp a single **Hybrid Logical Clock (HLC) transaction-time T** at the start of a
   coordinated backup. HLCs are central to logical-watermark backup strategies and are
   causally ordered across distributed components.
2. Each store takes its native snapshot and is then restored (or queried) AS-OF ≤ T:
   - Postgres: `recovery_target_time = T` (WAL replay to T)
   - RocksDB: checkpoint at sequence# ≤ T + WAL replay to T
   - Iceberg: `AS OF TIMESTAMP T` (snapshot-as-of-timestamp)
   - LanceDB: version corresponding to T
   - Detection content git: commit at T
3. A **BACKUP-SET MANIFEST** binds the per-store snapshot identifiers to a single recovery
   point T: (Postgres backup-label/LSN, RocksDB checkpoint sequence#, Iceberg snapshot-id,
   git bundle/commit SHA, KG+vector version) → one T.
4. Restore = read the manifest, bring each store to its recorded identifier (or to ≤T) via
   each store's own time-travel/PITR mechanism.

**CRITICAL alignment with C8 bitemporality:** T is the same `AS OF KNOWN <T>` watermark
that the C8 query engine exposes (ADR-PROP-prismql-deliverables.md §3 D-C8-2). The backup
recovery point and the queryable `AS OF KNOWN T` are the same T. This is not a coincidence
— it is a deliberate synthesis: a recovered cluster is temporally coherent on the exact same
axis the query engine uses, enabling a recovered cluster to serve queries consistent with
the pre-failure state without any special logic.

**Physical freeze reserved for tightly-coupled core only:**

**OQ-C17-003:** At morph, the architect must identify which components (if any) are
tightly-coupled enough at the write-path level to warrant a physical application-consistent
freeze (fsfreeze + quiesce + multi-volume snapshot group) rather than the logical-watermark
approach. Candidates: any two stores that share a write path with sub-second synchrony
requirements and have no independent time-travel mechanism. The research is explicit:
for a heterogeneous multi-store system, a global physical freeze "does NOT scale across
independent stores with unsynchronized commit/flush timelines." Reserve it narrowly.

**Retention floor:** Set retention collectively to a common floor bounded by the shortest
per-store PITR window. If RocksDB WAL TTL is 1 day but Postgres WAL is 7 days and Iceberg
retains 30 days, the effective cross-store RPO is 1 day unless all retention windows are
tuned up to the same floor. The common retention floor is a per-deployment configurable
(Compliance-Profile DR-tier axis, see D-C17-SF4).

**[VERIFIED — research/backup-recovery-2026-06-27.md §2; HLC pattern; xstore-transcript
§4.4, §6.2; the research characterizes this as the "more realistic" primary strategy for
"low-RPO distributed systems with diverse storage technologies" — verbatim from transcript]**

---

### D-C17-SF1 — KEY Escrow/Recovery Under Option-3 CMEK (HARDEST DECISION, DECIDED)

**The core paradox (research-stated):** "Recoverability and zero-knowledge pull in opposite
directions. Recoverability requires that someone, somewhere, have sufficient information to
reconstruct keys. Zero-knowledge demands that the operator not have such information."

**Decision (SF-1): the TENANT-HELD RECOVERY KEY as the default escrow model, with an
optional M-of-N threshold escrow tier for tenants wanting operator-assisted recoverability.**

**Envelope (unchanged from Option-3):** per-tenant DEK (AES-GCM, encrypts tenant data)
wrapped by per-tenant CMEK/KEK (SS-26 HD-1 + HD-4). This envelope is unchanged; what C17
adds is the RECOVERY PATH for lost keys.

**DEFAULT — Tenant-held recovery key:** the tenant's DEK/CMEK hierarchy is backed up as
**sealed/wrapped blobs** encrypted under a key the tenant controls (on-prem HSM, external
key manager, or tenant-side recovery key). The operator stores these sealed blobs in the
backup set but **CANNOT unwrap them** — it does not hold the wrapping key. On recovery, the
operator facilitates transport of the blobs; the tenant unwraps. The operator can never
recover tenant data unilaterally. This is the "no unilateral operator access" zero-access
promise.

**OPTIONAL — M-of-N threshold escrow tier (Shamir split):** for tenants wanting
operator-assisted recoverability (e.g., regulated entities that must demonstrate audit-able
recovery capability with independent custodians), the tenant may opt into a Shamir split:
the recovery/root key is split into `n` shares, threshold `k`; shares distributed across
tenant + auditor + regulator + (optionally) the operator. No single party (including the
operator) can reconstruct alone. This is the HashiCorp Vault recovery-key model.
Audited break-glass: every use is logged, reviewed, dual-controlled. The operator's share
(if any) is just one of `k` required shares — still "no unilateral operator access."

**Configurability:** the escrow model is a per-tenant or per-Compliance-Profile setting
(ties P-ADS-13 Configurable-Not-Prescriptive and the key-custody axis in
ADR-PROP-compliance-profiles.md). The zero-access PROMISE wording is:
**"no unilateral operator access"** (the operator can never recover alone). This is the
correct and precise phrasing — "no access under any circumstance" would be false in the
M-of-N tier (by design), and is overly strong for MSSP managed-service contexts where
authorized break-glass may be contractually required.

**Crypto-shredding = the erasure primitive AND the zero-access reconciliation:** destroying
the tenant's CMEK key renders ALL encrypted data in ALL stores (including old backups)
unreadable ciphertext. This is Prism's GDPR right-to-erasure mechanism for pooled stores,
its tenant offboarding mechanism, and the clean architectural reconciliation with
operator-zero-access: the operator holds ciphertext everywhere; once the key is destroyed,
the data is cryptographically dead even if the bytes persist in backup media.
**[VERIFIED — research §4; key-escrow-transcript §4.1..4.5; HashiCorp Vault recovery keys
model, NIST SP 800-57 key lifecycle; Google Workspace CSE zero-access precedent]**

**Aligns with:** INV-ADS-02, P-ADS-02, NERC CIP-011-3 entity-held-key zero-access
(C20/C16), INV-ADS-10 (new — see Section C).

**OQ-C17-004:** At morph, the architect confirms: is the Shamir M-of-N escrow tier
implemented as a Vault-based feature (Vault recovery keys pattern) or as a native Prism
key-escrow module? If Vault, the Vault-based model references HashiCorp Vault **2.0.3**
(2026-06-17, verified; 1.21.x also maintained). If native, the Shamir Secret Sharing crate
needs selection and version-pinning. Either path requires a dedicated story in E-BACKUP-
RECOVERY-001.

---

### D-C17-Q3 — Per-Tenant + Nested Backup Granularity

#### SOLVED: Cluster backup + per-tenant logical export + crypto-shred

For Prism's pooled-store model (shared infrastructure, per-tenant logical partitioning):

- **Cluster backup** (the primary): full cluster PITR backup as described in D-C17-Q1 per
  store. Covers all tenants.
- **Per-tenant logical export** (additive): RLS-filtered / namespace-scoped logical export
  per tenant for portable, verifiable-complete tenant data bundles. Usable for tenant
  offboarding, GDPR export requests, and per-tenant SLA reporting.
- **Crypto-shredding for erasure:** destroy the tenant's CMEK key → data unrecoverable
  everywhere, including in old cluster backups. This is the GDPR right-to-erasure answer
  for pooled stores. Per-tenant crypto-shred must be applied in ALL regions where the
  tenant's data resides (C19 residency → OQ-DEPLOY-2 axis).

#### GENUINELY HARD: Per-tenant PITR in a pooled store

Strict per-tenant point-in-time restore INSIDE a pooled store without touching other
tenants is **NOT natively supported by DB engines** — confirmed by the research as
"genuinely difficult … usually implemented via side restores and selective re-ingestion."

**Decision:** the restore workflow is a **restore-to-side-instance + selective re-ingestion
operational workflow:**
1. Restore the full cluster PITR backup to a SIDE instance (separate DB cluster, not
   touching production).
2. Extract the one tenant's as-of-T slice from the side instance (RLS-filtered logical
   export, scoped to `OrgSlug`).
3. Validate the extracted slice for FK integrity, shared-reference-data consistency, and
   ID collision risks.
4. Re-ingest the validated slice into the live store under a coordinated migration.

This workflow must be **built and heavily tested**. It is NOT a built-in restore. Pitfalls:
foreign keys to shared reference tables, ID collisions on re-ingest, partial-visibility
windows during re-ingest, and export format versioning across schema migrations. The
export format must be consistent and versioned.

**Silo escape-hatch:** offer more isolated storage (`isolation_tier = silo`) at higher
tiers or contract grades for tenants with stringent restore requirements (C19
ADR-PROP-C19-2). The silo model makes per-tenant PITR trivial (each tenant has its own DB)
at the cost of operational overhead.

#### Nested tenancy backup scopes (C19)

**Decision:** explicit backup scope selectors per operation:
- **PARENT-ONLY** — back up / restore / crypto-shred the parent node's data only, NOT
  children.
- **SUBTREE** — parent + all descendants (closure-table enumerated, consistent with C19's
  closure table in ADR-PROP-nested-tenancy.md).
- **CHILD-ONLY** — single child tenant, isolated.

All scope operations use **consistent hierarchical identifiers** (OrgSlug + closure-table
path) across all stores to ensure no store is missed. Crypto-shred for nested tenants must
propagate to ALL stores and ALL regions where that tenant's data resides.

**[VERIFIED — research §3; per-tenant-transcript §4.2..4.5, §5.2..5.4, §6.3; C19 closure
table — ADR-PROP-nested-tenancy.md D-C19-1]**

---

### D-C17-Q5 — Satellite State Recovery (DECIDED)

**Decision:** RECONSTRUCT-FROM-CENTRAL by default.

Satellites are headless data-plane appliances (P-ADS-01). Their config, detection content,
and policy are all defined at Central and delivered as signed bundles (PAT-ADS-03). A
satellite that needs recovery should, by default, be re-provisioned from Central: re-enroll
(join-token OOB), receive its config bundle, receive its detection-content bundle, and
resume operation. This is the same "config-as-code deterministic reprovision" principle
that governs SaaS GitOps DR.

**Local state that warrants backup:**
- Local **data buffers** (collected telemetry not yet shipped to Central that would be lost
  on re-provision). Back these up to a Central-addressable destination or to local media.
- Any genuinely **local non-reconstructible state** (satellite-generated state with no
  Central copy). Classify and minimise; ideally design all non-reconstructible state to
  ship to Central before backup is required.

**Air-gap nodes:** for satellites that cannot dial home, use **signed + encrypted offline-
media bundles**. The bundle must be verifiable WITHOUT an online control plane — carry the
verification metadata (Ed25519 signatures, content-hash/Merkle tree) with the bundle. The
same signing infrastructure as PAT-ADS-03.

**Complements C9 A/B dual-slot self-recovery:** C9's A/B dual-slot (PAT-ADS-09) handles
software-update rollback (restart-class recovery). C17 satellite recovery handles
data-buffer archival and full re-provision from Central (site-failure-class recovery). The
two are complementary, not overlapping.

**[VERIFIED — research §5; dr-transcript §2.1..2.7; industry bias toward reconstruct-
from-central confirmed by Proxus, AWS IoT Greengrass, Azure IoT Edge, osquery patterns]**

---

### D-C17-SF4 — DR Tier Ladder (DECIDED)

**Decision:** Offer the FULL TIER LADDER. Tier selected per deployment-profile/contract.

| Tier | RPO/RTO | Mechanism |
|------|---------|-----------|
| Backup-Restore | hours | backups + cross-region copy; cheapest |
| Pilot-Light | minutes–low | core data replicated, minimal standby infra, scale on failover |
| Warm-Standby | minutes | scaled-down-but-running replica, scale on failover |
| Active-Active | near-zero | full multi-region live, automatic failover |

**SaaS default:** multi-AZ baseline + optional multi-region (pilot-light/warm-standby). The
tier is a **Compliance-Profile / contract setting** (ties PAT-ADS-12 and P-ADS-13) — the
human must determine at morph which DR tier is the default SaaS commitment and what
up-tier requires.

**On-prem/MSSP appliance default:** VM-snapshot + config-as-data deterministic
re-provisioning + optional HA-pair. The appliance rebuilds from its config export after
hardware replacement.

**OQ-C17-005 (business decision at morph):** What RTO/RPO does each contract grade (SaaS
standard, SaaS enterprise, MSSP-managed, on-prem appliance) commit to? This is a cost and
business decision, not a pure architecture decision. The architecture supports the full
ladder; the business selects which tier each contract grade activates.

**[VERIFIED — research §6; dr-transcript §4.1..4.7, §8.2..8.3; AWS DR whitepaper 4-tier
taxonomy; multi-AZ baseline + optional multi-region as SaaS default confirmed]**

---

### D-C17-Q6 — Unified Integrity Model (DECIDED)

**Decision:** Signed + tamper-evident + customer-managed-key encrypted across
satellite/SaaS/on-prem.

Mechanism:
- **Detached signatures** (Ed25519, same as PAT-ADS-03) on every backup artifact.
- **Content-addressable hashing + Merkle trees** for tamper-detection (TUF/Sigstore prior
  art). The same signing/hashing infrastructure used for backup integrity, BCSI-at-rest
  (C16), and CIP-013 software-integrity (C20).
- **Customer-managed-key encryption** throughout: backups are encrypted under the same
  per-tenant CMEK hierarchy (SS-26 HD-1); the SaaS operator stores encrypted backup blobs
  it cannot decrypt. This extends operator-zero-access (INV-ADS-02) to the backup channel
  end-to-end.

**Tooling candidate:** restic **0.19.0** (2026-06-09, verified) for content-addressable,
encrypted, deduplicated snapshots of the file/directory-backed stores. Per-store native
mechanisms for the stores that have them (pgBackRest for Postgres, RocksDB Checkpoint/
BackupEngine for RocksDB, Iceberg snapshot for the cold tier). Restic is a candidate for
the unified wrapper layer; confirm at morph whether it integrates cleanly with
per-tenant CMEK key rotation.

**[VERIFIED — research §6.3; dr-transcript §6.1..6.3; TUF, Sigstore/cosign, restic 0.19.0
integrity model; PAT-ADS-03 Signed-Offline-Bundle (this pattern reuses that infrastructure)]**

---

### D-C17-CIP009 — CIP-009 Recovery Evidence (FIRST-CLASS, DECIDED)

**Decision:** Recovery evidence is FIRST-CLASS in C17. Prism generates recovery-test
evidence NOW — as part of the backup-recovery subsystem, not as an afterthought.

**What Prism generates:**
1. **Timestamped restore-test runs** — periodic, automated, logged. A full restore-test
   (or restore-to-side-instance + validation) is triggered on a configurable schedule.
   The test log is a signed, integrity-verified artifact.
2. **Integrity-verification records** — the hash+signature verification result for each
   backup artifact, logged and queryable.
3. **Post-restore config baseline diff vs CIP-010 baseline** — after a restore-test, Prism
   computes a diff of the restored config state against the last known-good CIP-010
   baseline. A clean diff = known-good restore evidence. A diff with deviations = a
   finding that requires explanation.

**Prism is itself a BCS/EACMS candidate:** if Prism is classified as a BES Cyber System
or Electronic Access Control/Monitoring System, it needs its own CIP-009 recovery plan
covering Prism's config, audit/log data, and integrations. D-C17-CIP009 ensures C17
produces the evidence needed for that plan.

**RSAW export packaging consolidated in C20:** the NERC-specific Reliable Standard Audit
Worksheet (RSAW) export that packages recovery-test evidence for an audit submission is
in C20's scope (nerc-cip-support-2026-06-27.md §10.4 fork-1). C17 produces the raw
evidence records; C20 packages and exports them. This avoids duplication.

**[VERIFIED — research §7; nerc-cip-support-2026-06-27.md §5 + §10.2; CIP-009-6 requires
documented recovery plans, integrity verification of backups, and periodic recovery testing
with retained evidence; C20 RSAW export coordinates the audit package]**

---

## 3 — Open Questions

| ID | Question | Blocking? | Resolves-at |
|----|----------|-----------|-------------|
| OQ-C17-001 | Confirm indradb / usearch / lancedb native backup/snapshot APIs. Classify each: (a) native snapshot, (b) RocksDB-backed → Checkpoint/BackupEngine, (c) columnar versioned (lancedb), (d) index-only rebuildable (usearch). | NO — default: FS snapshot of all authoritative data at T | morph |
| OQ-C17-002 | Confirm ARO state authoritative-vs-derived split. Which CFs/tables are authoritative (Config-DB-class) vs runtime scratch (RocksDB-class, disposable)? | NO — default: back up all ARO state via Config-DB-class until split confirmed | morph |
| OQ-C17-003 | Which components (if any) warrant physical application-consistent freeze rather than logical-watermark? Candidates: tightly-coupled write-path pairs with no independent time-travel. | NO — default: logical-watermark for all; freeze = empty set until identified | morph |
| OQ-C17-004 | Shamir M-of-N escrow tier implementation: Vault-based (Vault 2.0.3 recovery keys) or native Prism module? Select crate + version if native. | NO — default: Vault-based; defer native module to follow-up story | morph |
| OQ-C17-005 | DR tier per contract grade: which tier (Backup-Restore / Pilot-Light / Warm-Standby / Active-Active) maps to SaaS standard, SaaS enterprise, MSSP-managed, on-prem? Business decision. | NO — architecture supports full ladder; tier selection is contract policy | morph (business decision) |

---

## 4 — Invariants

### PIV-C17-001 — Operator Stores Only Sealed/Wrapped Key Blobs

In every backup path, the operator's backup infrastructure MUST NOT hold a key that enables
unilateral decryption of any tenant's data. Keys are backed up as sealed/wrapped blobs
under tenant-controlled wrapping keys (tenant-held recovery key default) or M-of-N split
blobs where the operator holds at most one share. Violation = cross-tenant isolation defect
(P-ADS-06) + zero-access violation (INV-ADS-02). Enforced structurally: the key escrow
module has no decrypt-path wired for the operator's service account.

### PIV-C17-002 — Backup Recovery Point = C8 `AS OF KNOWN <T>` Watermark

The HLC transaction-time T used as the backup-set manifest's coherence point MUST be the
same type and resolution as the `AS OF KNOWN <T>` watermark the query engine exposes
(C8 bitemporality). A recovered cluster MUST be able to serve `AS OF KNOWN T` queries
consistent with the pre-failure state without additional post-recovery transforms.

### PIV-C17-003 — Crypto-Shred Must Be Applied in All Residency Regions

When a tenant is offboarded or a data erasure is requested, crypto-shred (destroy the
tenant's CMEK key) MUST be applied in ALL regions where the tenant's data resides. A
crypto-shred that is only applied in the primary region while backup replicas remain in a
secondary region is an erasure defect. The per-tenant crypto-shred workflow must enumerate
ALL residency locations from C19 hierarchical metadata before issuing the key-destroy.

### PIV-C17-004 — Per-Tenant PITR is a Restore-to-Side-Instance Workflow, NOT a DB-Native Feature

No implementation may represent per-tenant PITR in a pooled store as a DB-native restore
operation. It is an operational workflow (restore full cluster to side instance → extract
tenant slice → validate → re-ingest). Any story or spec that assumes DB-native per-tenant
PITR is incorrect and must be revised before acceptance.

---

## 5 — ADS Conformance Checklist

```
CONFORMANCE CHECKLIST — ADR-PROP-backup-recovery.md (C17) — 2026-06-27

P-ADS-01: Central-Sole-Surface
  [YES] Backup orchestration and recovery-test scheduling are authored at Central only.
  [YES] No satellite has a user-facing backup/restore UI surface; satellites receive
        reconstruct-from-central instructions via the signed bundle path (PAT-ADS-03).

P-ADS-02: Operator-Zero-Access-At-Rest
  [YES] Backup artifacts (data stores) are encrypted under the same per-tenant CMEK
        hierarchy (SS-26 HD-1). Operator stores ciphertext; tenant holds the wrapping key.
  [YES] Key escrow stores only sealed/wrapped blobs the operator cannot unwrap.
        "No unilateral operator access" is the zero-access promise; the operator
        can facilitate blob transport but cannot perform unilateral recovery.
  — Note: this conformance item is the direct driver for INV-ADS-10 (new — see C.1).

P-ADS-03: Derived-Results-Only-At-Central
  [YES] Backup artifacts (data + keys) never transit Central in raw form; they transit
        via the normal store paths (PITR WAL archives, RocksDB checkpoint files, Iceberg
        object storage — all encrypted at rest under tenant CMK).
  [N/A] No opt-in path that requires consent governance applies to backup mechanics.

P-ADS-04: Tenant-Keyed-Central-Persistence
  [YES] RocksDB (hot) and Iceberg (cold) are the backup targets for working state and
        cold tier respectively. PostgreSQL is Config-DB (backup via WAL archiving, not
        query-result caching). This is consistent with P-ADS-04's PostgreSQL-is-control-
        plane-only boundary.

P-ADS-06: Per-Tenant-Isolation
  [YES] Per-tenant logical export uses OrgSlug-scoped RLS filtering.
  [YES] Per-tenant crypto-shred propagates across all regions.
  [YES] Nested backup scopes (PARENT-ONLY / SUBTREE / CHILD-ONLY) are per-tenant-isolated.

P-ADS-07: AI-Opaque
  [YES] No backup path transmits credentials or raw sensitive data to AI context.
        Backup orchestration is a data-plane operation, not an AI-reasoning operation.

P-ADS-08: OCSF-Normalize-At-Boundary
  [N/A] Backup mechanics operate on store-native formats (WAL, SSTs, Iceberg metadata);
        OCSF normalization governs the DATA content flowing into those stores.

P-ADS-09: Config-DB-Authoritative
  [YES] Backup schedules, retention policies, and DR-tier configuration are authored
        at Central, stored in the Config DB, pushed as signed bundles. No satellite or
        edge node authors backup policy locally.

P-ADS-10: Idempotent-Gated-Actions
  [YES] Backup initiation and restore operations carry idempotency keys.
  [YES] Break-glass key recovery operations are gated by M-of-N threshold (operator
        alone cannot initiate) + RBAC approver role + audited dual-control.

INV-ADS check (all nine + new INV-ADS-10):
  [YES] INV-ADS-01: no raw sensor data at Central — backups are of derived/encrypted data
  [YES] INV-ADS-02: operator zero-access at rest — CMK-encrypted backups + sealed-blob keys
  [YES] INV-ADS-03: per-tenant isolation enforced — OrgSlug-scoped export; crypto-shred in all regions
  [YES] INV-ADS-04: config authored only at Central — backup/DR config in Config-DB
  [YES] INV-ADS-05: actions gated and idempotent — restore ops carry idempotency keys; break-glass gated
  [YES] INV-ADS-06: AI-opaque — backup ops are data-plane, not AI-path
  [YES] INV-ADS-07: OCSF normalization at all boundaries — N/A to backup mechanics (governs data content)
  [YES] INV-ADS-08: air-gap deployment is valid reference profile — satellite reconstruct-from-central
        works without internet; air-gap nodes use signed+encrypted offline-media bundles
  [YES] INV-ADS-09: authorization decisions are logged — break-glass key recovery is fully audited
  [YES] INV-ADS-10 (NEW): recoverability preserves operator-zero-access — CMK-encrypted backups,
        sealed-blob key escrow, crypto-shred erasure; operator cannot recover tenant data alone
```

All items PASS. C17 is fully conformant with ADS v1.4.

---

## 6 — Proposed Epic

**E-BACKUP-RECOVERY-001 — Backup & Recovery as a First-Class Citizen (C17)**

Status: PROPOSED. Not in STORY-INDEX. Morph execution will decompose into stories.

High-level scope:
- Per-store backup orchestrator (Config DB / git / RocksDB / Iceberg / KG+vector / keys /
  ARO)
- Backup-set manifest generation + cross-store coherent T stamping (HLC integration with
  C8 transaction-time axis)
- Tenant-held recovery key generation + sealed-blob key escrow infrastructure (SS-26
  extension)
- M-of-N threshold escrow tier (Shamir + Vault integration, OQ-C17-004 resolution)
- Per-tenant logical export + versioned export format
- Restore-to-side-instance workflow tooling + FK/ID-collision validation
- Satellite local-buffer backup + air-gap signed bundle
- DR tier activation per deployment-profile (Backup-Restore through Active-Active ladder)
- Recovery-test scheduler + integrity-verification records + CIP-010 baseline diff
- INV-ADS-10 enforcement: CMK-encrypt-backup + sealed-blob audit trail

Pre-morph prerequisite: OQ-C17-001..005 closed; iceberg-rust 0.9.1 maintenance-op
coverage confirmed or Java/Spark path scoped; git2 0.21.0 bundle API confirmed or
shelled out; rocksdb 0.24.0 confirmed sufficient (already confirmed).

---

## 7 — Cross-Wiring

| Feature | Cross-link |
|---------|-----------|
| **C8 bitemporality** | T = `AS OF KNOWN <T>` watermark — backup recovery point is C8 transaction-time; same HLC axis |
| **C9 bootstrap/restart-class recovery** | Complementary: C9 handles software-update rollback (A/B dual-slot, PAT-ADS-09); C17 handles data-backup/restore and full re-provision |
| **C16 vault/DEK custody + crypto-shred** | Per-tenant DEK hierarchy (SS-26) is the key envelope; crypto-shred is the erasure primitive shared with C16 offboarding |
| **C19 nested backup scopes, Option-3 child-keyed** | PARENT-ONLY / SUBTREE / CHILD-ONLY scopes; child-keyed CMEK informs per-tenant key backup; closure table enumerates subtree members |
| **C2 satellite mesh** | Satellite reconstruct-from-central; local-buffer backup; air-gap signed bundles use PAT-ADS-03 |
| **C20 CIP-009 evidence + RSAW export** | C17 produces timestamped restore-test runs + baseline diff; C20 packages into RSAW audit export |
| **C5 Iceberg cold tier** | Back up catalog + metadata + data files; retention tuning (`expire_snapshots`) affects cross-store retention floor |
| **C12 KG+vector** | Back up authoritative embeddings + graph edges; usearch ANN index is rebuildable; lancedb versioned; indradb RocksDB-backed |
| **C15 ARO state** | Split authoritative config (Config-DB-class PITR) vs runtime scratch (RocksDB-class, non-reconstructible portion only) — OQ-C17-002 |
| **Option-3 / SS-26** | Per-tenant DEK wrapped by per-tenant CMEK; operator stores only sealed/wrapped blobs (sealed-blob escrow pattern) |
| **Compliance-Profiles (C18)** | DR-tier + key-custody model as profile settings axes (PAT-ADS-12); tighten-only; SaaS default vs MSSP vs on-prem tier selected via profile |
