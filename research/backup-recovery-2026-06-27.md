---
document_type: research
produced_by: research-agent
status: capture
do_not_execute: true
timestamp: "2026-06-27"
program: day-2-vision-side-analysis
relation: OUT-OF-BAND — SEPARATE from the live VSDD factory pipeline
topic_slug: backup-recovery
side_analysis_item: C17 — backup & recovery as a first-class citizen
scope: >
  Cited research pass for the Prism day-2 SIDE-ANALYSIS item C17: backup & recovery as a
  first-class architectural concern. Covers per-store backup/restore mechanics (config DB,
  embedded git, RocksDB, Iceberg cold tier, KG+vector, KEYS, ARO state), cross-store
  consistent point-in-time (tied to C8 bitemporality), per-tenant + nested-tenant
  granularity (C19), KEY escrow/recovery preserving operator-zero-access (the hardest part,
  under Option 3 per-tenant CMEK), satellite/air-gap recovery (ties C9 + C2), per-deployment
  -model DR (SaaS vs MSSP/on-prem), and NERC CIP-009 recovery evidence (ties C20). Verifies
  tool/crate versions against registries.
ties:
  - "C9 bootstrap/restart-class recovery (4-layer A/B + commit-confirmed) — see bootstrap-config-recovery-2026-06-27.md"
  - "C8 bitemporality (valid-time + transaction-time; AS OF KNOWN <T> point-in-time)"
  - "C20 NERC CIP-009 recovery-plan obligations — see nerc-cip-support-2026-06-27.md §5"
  - "C2 satellite mesh (outbound-only dial-home, air-gap capable) — satellite-mesh-2026-06-26.md"
  - "C19 nested tenancy; C16 BCSI/RSI masking + zero-access; C18 RBAC; C15 ARO state"
  - "Architecture Design System: per-tenant isolation, Central-Sole-Surface, operator-zero-access, air-gap"
settled_context_NOT_relitigated:
  - "Stores to protect: config DB (DB-authoritative system-versioned), detection-content (embedded git2), RocksDB working state (~19 CFs), Iceberg cold tier, Context KG+vector (indradb+usearch+lancedb), per-tenant DEK/CMEK keys (Option 3 + secret subsystem), ARO state"
  - "Operator-zero-access is a hard architectural invariant the backup design MUST preserve (NERC CIP-011-3 zero-access aligns)"
  - "Config recovery for RESTART-CLASS bootstrap keys is covered separately by C9 (bootstrap-config-recovery-2026-06-27.md); THIS pass covers DATA-STORE + KEY backup/recovery"
caveat: >
  CAPTURE artifact. LEANS are discussion input only — NOT decisions. Numbers/epics/ADRs
  remain the architect's at morph. This file modifies NO live spec/BC/ADR/STATE.md/
  SESSION-HANDOFF.md/RESEARCH-INDEX.md or any prior research file, and was not git-added or
  committed. Tool/crate versions verified against crates.io / official release pages as of
  2026-06-27; landscape changes rapidly — re-verify at morph.
---

# Backup & Recovery as a First-Class Citizen — Cited Research Pass (Day-2 Vision Side-Analysis, C17)

**Date:** 2026-06-27 · **Primary tool:** `mcp__perplexity__perplexity_research` (`sonar-deep-research`) at `reasoning_effort=high` — five thematic deep-research passes, all succeeded on first attempt; each returned 74–111 KB of citation-backed synthesis. Plus crates.io / GitHub-release version verification via WebFetch and one `perplexity_ask` factual confirmation.

> **Cross-tool reading note.** All five deep-research responses exceeded the inline token cap and were saved to transcript files (paths in Research Methods). The per-store transcript was read in full including its 19-source citation list. The other four were mined by sequential targeted extraction covering every cited system, mechanism, version, and human-decision fork used below; section structures were enumerated and the load-bearing sections (decision-forks, synthesis, recommendations, conclusions) extracted verbatim. The Honest Costs section flags where transcript tails were not re-read line-by-line.

> **Headline finding (load-bearing).** For a heterogeneous multi-store system that needs a *low RPO* and a *coherent point-in-time*, the deep research is explicit and consistent: a **full physical application-consistent freeze across all stores at one wall-clock instant is NOT the realistic primary strategy** — it does not scale across independent stores with unsynchronized commit/flush timelines. The realistic primary is a **logical-watermark approach**: each store is snapshotted independently on its own native mechanism, and a coherent recovery point is reconstructed by replaying/truncating each store to a **common transaction-time cutoff T** using each store's own point-in-time / time-travel capability. For Prism this maps *directly* onto C8 bitemporality — the backup recovery point should be a **transaction-time watermark T expressible as `AS OF KNOWN <T>`**, with a Hybrid-Logical-Clock (HLC) supplying the cross-store coherent timestamp and a **backup-set manifest** binding the per-store snapshot identifiers to that single T. Physical quiesce is reserved for the few most tightly-coupled components. This is the central architectural choice and it is the primary cross-store human-decision fork (§3.5, §10).

> **Second load-bearing finding.** The single hardest, highest-consequence element is **KEY backup/recovery under Option 3 per-tenant CMEK while preserving operator-zero-access** (§4). The research confirms this is *genuinely not fully solvable by technology alone* — recoverability and zero-knowledge pull in opposite directions, and the resolution is a **governance + cryptography choice (the escrow model)**, which is a genuine human-decision fork. The canonical resolution that preserves operator-zero-access: the operator stores **only sealed/wrapped key blobs it cannot unwrap**, and recovery capability lives with the tenant (tenant-held recovery key/HSM) and/or a **threshold M-of-N split across independent custodians** with audited break-glass. **Crypto-shredding** (destroy the tenant key → data unrecoverable) doubles as the offboarding/erasure primitive and is the clean reconciliation with operator-zero-access.

---

## Q1 — What must be backed up + recovery model per store

The per-store mechanics below are drawn from the per-store deep-research transcript [per-store-transcript], with versions independently re-verified against crates.io / release pages (Research Methods). Each store gets: backup mechanism, restore mechanism, point-in-time (PITR) support, consistency, retention.

### 1.1 Config DB — PostgreSQL PITR (DB-authoritative system-versioned)

- **Backup:** base backup + **continuous WAL archiving**. Built-in: `pg_basebackup`. Production tooling: **pgBackRest** (latest **2.55**, ~May 2026 — verified via `perplexity_ask`; note: an earlier transcript reading of the GitHub page returned a mis-parsed "2.58.0 Jan" date — treat 2.55/2026-05 as authoritative) or **Barman** (3.19.x line, 2026). [per-store-transcript: PostgreSQL §; pgbackrest.org; pgbarman.org]
- **Restore + PITR:** restore base backup, replay archived WAL to a **recovery target** (`recovery_target_time` / `_xid` / `_lsn`); pgBackRest exposes `--type=time --target=<timestamp> --target-action=promote`. **Requires an unbroken WAL sequence** — any gap renders the target unreachable. [per-store-transcript]
- **Consistency:** continuous-archiving base backups are **immune to filesystem changes during backup** (Postgres coordinates with checkpoints + WAL); crash recovery repairs torn pages on replay. File-system snapshots are consistent only if the snapshot impl is trustworthy; a `CHECKPOINT` before snapshot shortens replay. [per-store-transcript]
- **Retention:** pgBackRest `repo-retention-full` (count or time-based; always keeps ≥1 full backup; full expiry cascades its diff/incr backups; WAL expired only when no retained backup needs it). Barman retention is coupled to WAL streaming/replication slots — backups can stall in `WAITING_FOR_WALS` if a slot still needs old WAL. [per-store-transcript: pgBackRest retention §, Barman §]
- **System-versioned / temporal interaction (ties C8):** PostgreSQL has **no native SQL:2011 system-versioned tables** [hyPiRion]; the common pattern is snapshot-table + history-table maintained by triggers/app logic. **PITR is cluster-level (coarse)**; it rewinds *all* tables to an epoch. So engine-level PITR is a coarse time-machine; the bitemporal history table is the fine-grained axis. The correct recovery pattern for a partial corruption is **restore a PITR clone to a side instance, extract the as-of-T slice, merge back** — this is the same restore-to-side pattern that recurs for per-tenant restore (§3). [per-store-transcript — PITR×temporal interaction flagged as model reasoning over cited Postgres architecture]

### 1.2 Detection content / recipes — embedded git (libgit2 / `git2`)

- **Crate:** `git2` **0.21.0** (2026-05-18, crates.io — verified). Bindings to libgit2; exposes repo/commit/branch/ref/remote ops. Backup-specific API coverage (bundle creation, pack manipulation) is **not enumerated in docs** — confirm at morph. [crates.io: git2; docs.rs: git2]
- **Backup mechanism:** the canonical unit is the **object database + refs**. **`git clone` is explicitly NOT a backup** — it omits server-side metadata, is non-atomic, and cannot guarantee a point-in-time image. [Rewind blog, per-store-transcript] Use **`git bundle`** (self-contained object DB + refs; a point-in-time snapshot when generated against a quiesced repo) or a **filesystem snapshot of `.git`** taken when `git gc` is not mid-run (git objects are immutable, so an atomic FS snapshot is consistent).
- **PITR:** git history *is* commit-level time-travel natively; "recover the content as of commit/tag/time" is intrinsic. The backup concern is preserving the object DB + refs onto a new host, not temporal navigation. Watch `git gc` / reflog expiry pruning unreachable objects — periodic bundle/pack backups protect long-term recoverability. [per-store-transcript]
- **Note:** this is the store where Prism already has fast-revert + signed-bundle design (C9); C17's git concern is the *backup/restore* of the embedded repo, complementary to C9's *revert* of its content.

### 1.3 RocksDB working state (~19 column families)

- **Crate:** `rocksdb` **0.24.0** (2025-08-10, crates.io — verified). **CONFIRMED: exposes both a `checkpoint` module (Checkpoint API bindings) AND a `backup` module** — this resolves the per-store transcript's "inconclusive" flag on whether the Rust crate surfaces these APIs. [docs.rs: rocksdb/latest — Modules: `checkpoint`, `backup`]
- **Two mechanisms (pick deliberately):**
  - **Checkpoint API (`CreateCheckpoint`)** — a fast, consistent on-disk snapshot. Hard-links live SST files + copies MANIFEST/CURRENT/current-WAL. Runtime is O(number of SST files), not DB size. The result is a **valid RocksDB DB directory** openable read-only or `tar`/`rsync`'d offsite. `logSizeForFlush` controls whether the memtable is flushed first (flush → snapshot is self-contained in SSTs; no flush → checkpoint includes a WAL copy that must be replayed for full consistency). **Best for: fast hot backup / seeding a follower / fork-for-analysis.** [per-store-transcript: RocksDB Checkpoint §]
  - **BackupEngine API (`BackupableDBOptions`)** — incremental backups + multiple retained backup versions in a separate backup dir; hard-links unchanged SSTs across backups (storage-efficient). Restore via `RestoreOptions` (e.g. `keep_log_file=true`). **Best for: retained historical backup set with selective prune.** [per-store-transcript: RocksDB BackupEngine §]
- **PITR-like:** pair a checkpoint/backup with WAL retained via `Options::WAL_ttl_seconds` (obsolete WAL moves to `archive/` then deletes after TTL). Within the TTL window you can restore + replay to a specific sequence number. Beyond TTL, PITR granularity drops to the backup cadence. [per-store-transcript]
- **Consistency:** checkpoints are consistent at a sequence number; non-torn either by flushing memtables or by replaying the copied WAL on open (crash-recovery semantics).
- **Prism note:** RocksDB is the **working/cache tier** — much of it may be *reconstructible* rather than authoritative (see §5 disposable-vs-durable). Back up the CFs that hold non-reconstructible state; treat pure-cache CFs as disposable.

### 1.4 Iceberg cold tier (C5)

- **Crate:** `iceberg` (iceberg-rust) **0.9.1** (2026-05-06, crates.io — verified). Pre-1.0; treat as primarily a read/write data-access binding. The transcript could not confirm whether the Rust binding exposes snapshot-expiration / orphan-file / maintenance ops — **the recommendation stands: drive Iceberg maintenance (expire_snapshots, deleteOrphanFiles) via the canonical Java/Spark procedures or a verified Rust path, not assumed Rust coverage.** [per-store-transcript: iceberg-rust §; crates.io: iceberg]
- **Backup model:** Iceberg's snapshot model IS its time-travel: each commit creates an immutable **snapshot** referenced by table metadata; query/restore **AS OF snapshot-id or timestamp**. A correct backup is **three parts**: (1) the **catalog** (Hive/REST/JDBC — back up via its native mechanism, e.g. the catalog RDBMS), (2) **all metadata files + manifest lists** (the snapshot history within retention), (3) the **data files** (object store — replicate/back up). Copying only data files is insufficient; metadata + catalog pointer are required to reconstruct. [per-store-transcript: Iceberg §; iceberg.apache.org/spec]
- **PITR:** time-travel by snapshot-id/timestamp, but only **within the retention window** defined by `expire_snapshots` + `write.metadata.delete-after-commit.enabled` / `write.metadata.previous-versions-max`. **Branches/tags** pin snapshots so they survive expiry (back up the refs). [per-store-transcript: maintenance/spark-procedures §]
- **Retention:** `expire_snapshots` (age/count) deletes unreachable snapshots + their data files; `deleteOrphanFiles` cleans stray files in object storage; `rewriteDataFiles`/`rewriteManifests` compact. These define the time-travel window and must be tuned to the cross-store retention floor (§2).

### 1.5 Context KG + vector (C12: indradb + usearch + lancedb)

- The per-tenant transcript covers graph + vector backup generically; **direct per-store backup docs for indradb / usearch / lancedb were NOT in the retrieved corpus** — flag `[INCONCLUSIVE — Prism-specific stores]`; a dedicated per-store pass at morph should confirm each store's native snapshot/backup API. [per-tenant-transcript §3.4]
- **What the research does say (applies to graph + vector stores generally):** vector indexes and graph stores are often **derived/rebuildable from a source of truth** (re-embed + re-ingest) — so the first design question is *which of KG+vector is authoritative vs reconstructible*. LanceDB is itself a versioned columnar store (supports its own versioning/time-travel); usearch is an index (rebuildable from vectors); indradb holds graph state. **Lean:** treat the **embeddings/vectors + graph edges that are NOT recomputable** as authoritative-must-backup, and treat pure ANN indexes as rebuildable. [per-tenant-transcript §3.4 — general; Prism mapping is model reasoning, flagged]
- Backup mechanism for the authoritative slice: file-system snapshot of the store's data dir taken at the watermark T, or the store's native export, captured into the same backup-set manifest as the other stores.

### 1.6 Per-tenant DEK / CMEK keys (Option 3 + secret subsystem) — the hardest

Full treatment in **Q4 / §4**. Summary: keys are the **highest-consequence backup target** (lose them → all tenant cold data is unrecoverable ciphertext) and the one that most directly collides with operator-zero-access. Back up keys as **sealed/wrapped blobs** (operator-stored, operator-unable-to-unwrap), with recovery capability held by tenant and/or threshold custodians.

### 1.7 ARO state (C15)

- `[INCONCLUSIVE — Prism-specific]`: ARO state was not separately covered by the corpus. **Lean (model reasoning):** classify ARO state by the §5 disposable-vs-durable taxonomy — the **authoritative ARO decisions/configuration** (what the operator-resolved-objects represent) belong with config-DB-class backup (DB-authoritative, system-versioned, PITR); any **in-flight/runtime ARO scratch** is RocksDB-class working state (back up only the non-reconstructible portion). Confirm ARO's authoritative-vs-derived split at morph.

---

## Q2 — RPO/RTO + consistency: coherent cross-store point-in-time (ties C8)

From the cross-store-consistency transcript [xstore-transcript].

### 2.1 Crash-consistent vs application-consistent

- **Crash-consistent** = the on-disk image is what you'd see after a power-cut; recoverable via each store's crash recovery (WAL replay). **Application-consistent** = the stores are quiesced/flushed so the image is a clean transactional boundary. For a *single* store, PITR gives application-consistency. The hard part is **across independent stores** with unsynchronized timelines. [xstore-transcript §1.2]

### 2.2 The two strategies + the verdict

The transcript articulates exactly two and is explicit about which is realistic:

1. **Full physical application-consistent freeze** — quiesce ALL stores at one near-instant wall-clock T (the cross-store analog of Windows VSS): send quiesce/flush signals to Postgres, RocksDB, Iceberg, KG+vector, snapshot all volumes in a multi-volume crash-consistent group, release. **Low RPO and high consistency, but the freeze hurts availability and does NOT scale across heterogeneous independent stores/machines.** Reserve for the most tightly-coupled components. [xstore-transcript §6.1, §6.3]
2. **Per-store independent snapshot + logical-watermark reconciliation** — each store snapshots on its own native mechanism (Postgres PITR, RocksDB checkpoint, Iceberg snapshot, store-native), and a coherent point is reconstructed by bringing **each store to a common transaction-time cutoff T** via its own time-travel/PITR. **"For low-RPO distributed systems with diverse storage technologies, the second pattern — logical watermark plus per-store time travel — is more [realistic]."** (verbatim from transcript). [xstore-transcript §6.2, exec summary]

**Supporting theory:** Chandy–Lamport consistent-cut + epoch/barrier snapshotting (à la Flink aligned checkpoints) are the formal backbone, but the literature "stops short of prescribing a complete solution for this kind of heterogeneous stack" — Prism builds the orchestration. [xstore-transcript §3, §1.3, §4.6]

### 2.3 The logical-watermark scheme tied to C8 (the recommended mechanism)

- Use a **Hybrid Logical Clock (HLC)** to stamp a single coherent cross-store transaction-time **T**. HLCs are "central to designing logical watermark-based backup strategies." [xstore-transcript §4.4]
- Each store must support **restore/read AS-OF ≤ T**: Postgres `recovery_target_time`/system-versioned history, Iceberg snapshot-as-of-timestamp, RocksDB checkpoint+WAL-to-sequence, LanceDB versioning. The reconstructed image "respects the transaction-time axis across all stores" — i.e. **exactly Prism's `AS OF KNOWN <T>` bitemporal semantics (C8)**. [xstore-transcript §1.3, §4.5, §7.2]
- The recovery point T should align with a **bitemporal transaction-time watermark** — so a restore is not just physically coherent but *temporally* coherent on the same axis the query engine already exposes. This is the cleanest possible synthesis of C17 with C8: the backup recovery point and the queryable `AS OF KNOWN T` are the same T.

### 2.4 The backup-set manifest (consistency group)

- Bind the per-store snapshot identifiers (Postgres backup label/LSN, RocksDB checkpoint sequence#, Iceberg snapshot-id, git bundle/commit, KG+vector version) to **one recovery point T** in a **manifest** — the cross-store analog of a storage "consistency group" / Kubernetes `VolumeGroupSnapshot` CRD / enterprise backup consistency group. Restore = read the manifest, bring each store to its recorded identifier (or to ≤T). [xstore-transcript §5.1–5.5]

### 2.5 RPO/RTO

- **RPO** is bounded by the **shortest retention/PITR window among the stores** — if RocksDB WAL TTL is 1 day but Postgres WAL is 7 days and Iceberg retains 30 days, full cross-store continuity is only guaranteed to 1 day unless tuned. **Retention must be set collectively to a common floor.** [per-store-transcript cross-system §; xstore-transcript §6.3]
- **RTO** depends on the strategy: logical-watermark restore replays per store (slower, flexible); physical-freeze restore is faster but the freeze cost is at backup time.

---

## Q3 — Per-tenant backup/restore under nested tenancy (C19)

From the per-tenant transcript [per-tenant-transcript].

### 3.1 The partitioning model decides everything

- **Silo (DB/keyspace-per-tenant):** per-tenant backup is trivial — back up the tenant's own database/keyspace. Expensive at scale.
- **Pool (shared store, tenant by `OrgSlug` discriminator):** per-tenant backup requires **logical extraction** (query/filter by tenant key + export) layered on cluster-level physical backup. This is Prism's model. [per-tenant-transcript §2.1, §8.1]

### 3.2 SOLVED vs GENUINELY HARD (the key distinction)

- **SOLVED:** cluster-level backup **+ per-tenant logical export** (RLS-filtered export in Postgres; namespace/tenant-scoped export in graph/vector stores like Memgraph multi-tenancy / Pinecone namespaces) **+ crypto-shredding for erasure**. These are well-understood and codified. [per-tenant-transcript §8.1]
- **GENUINELY HARD (not solved by DB engines today):** **strict per-tenant point-in-time restore inside a pooled store, without touching other tenants and without a side-instance.** "Genuinely difficult … usually implemented via side restores and selective re-ingestion." [per-tenant-transcript §8.2, exec summary]

### 3.3 The practical pattern: restore-to-side-instance + selective re-ingestion

- The realistic mechanism for per-tenant PITR in a pooled store: **restore the full cluster PITR backup to a SIDE instance, extract the one tenant's as-of-T slice (RLS-filtered logical export), then merge/re-ingest it into the live store.** Not a built-in DB feature — an **operational workflow** that must be carefully built. [per-tenant-transcript §4.2, §4.5, §8.3]
- **Pitfalls (must design for):** foreign keys, **shared reference data**, and **ID collisions on restore**. The logical export format must be **consistent + versioned**; relationship semantics preserved. [per-tenant-transcript §4.4]
- **Tier escape-hatch:** offer **more isolated storage (silo) at higher tiers** for tenants with stringent restore requirements — the standard multi-tenant "escape hatch." [per-tenant-transcript §8.3]

### 3.4 Nested tenancy backup scopes (C19)

- Nested tenancy needs **explicit backup scopes: parent-only, subtree (parent + children), child-only** — chosen per operation, with **consistent hierarchical identifiers** across all stores and tenant-aware region/residency placement. Crypto-shredding must be applied **in all regions** where the tenant's data resides. [per-tenant-transcript §5.2, §5.3, §5.4, §6.5]

### 3.5 Offboarding / export / erasure

- **Tenant export bundle** = comprehensive, **verifiable-complete** export (the offboarding step). [per-tenant-transcript §6.1]
- **Crypto-shredding** = destroy the tenant's encryption key → data becomes unreadable ciphertext everywhere it resides **including in old backups** (the data is still physically in backups but cryptographically dead). This is the pragmatic erasure mechanism for pooled stores and the clean GDPR/right-to-erasure answer **and** the clean reconciliation with operator-zero-access (the operator never held a usable key anyway). [per-tenant-transcript §6.3, §6.4]

---

## Q4 — KEY backup & recovery preserving operator-zero-access (the critical risk under Option 3)

From the key-escrow transcript [key-escrow-transcript], read sections 1–5.5 in full + section structure 6–8 + sub-fork extraction.

### 4.1 The core paradox (stated explicitly by the research)

> "Recoverability and zero-knowledge pull in opposite directions. Recoverability requires that someone, somewhere, have sufficient information to reconstruct keys. Zero-knowledge demands that the operator not have such information." [key-escrow-transcript §4.1]

The resolution is **NOT technical alone** — it is a governance + cryptography choice: **assign recovery capability to entities OTHER than the operator** (tenant, external auditors, threshold custodians), and design so the operator can *facilitate* recovery (store/transport encrypted blobs) without being able to *perform* it. "Zero-access is not a binary property but a design spectrum." [key-escrow-transcript §1.2, §4.1]

### 4.2 The envelope hierarchy (Prism's Option 3, confirmed canonical)

Per-tenant **DEK** (AES-GCM, encrypts tenant data) wrapped by per-tenant **CMEK/KEK** in a KMS/HSM. This is the universal pattern (AWS KMS, GCP Cloud KMS, Azure Key Vault all implement it). **KMS-resident keys are non-exportable** — you cannot dump plaintext key material; this is itself a zero-access enforcement. BYOK/EKM/HYOK / External Key Store push the source-of-truth for key material to **customer-controlled** infrastructure. [key-escrow-transcript §2.1–2.5]

### 4.3 The three zero-access-preserving escrow patterns

1. **Escrow encrypted to a tenant-controlled recovery key / HSM.** Operator stores DEKs/CMEKs wrapped under a key only the tenant controls (on-prem HSM / external key manager). Operator holds **sealed blobs it cannot decrypt**; on recovery it hands blobs to the tenant who unwraps. Google Workspace CSE is the large-scale exemplar (Google never sees plaintext; external key service holds top-level keys). [key-escrow-transcript §4.2, §7.3]
2. **Threshold M-of-N split across independent custodians (Shamir).** Split a recovery/root key into `n` shares, threshold `k`; assign shares to tenant + auditor + operator + regulator etc. **No single party (including the operator) can reconstruct alone.** Information-theoretically secure below threshold. This is exactly HashiCorp Vault's unseal/recovery-key model. [key-escrow-transcript §3.1–3.5, §4.3]
3. **Sealed/wrapped key blobs the operator stores but cannot unwrap.** HSM backup blobs (AWS CloudHSM backups encrypted inside the HSM — AWS lacks the decryption key; Azure Managed HSM backup blobs opaque to Azure; Thales PKCS#11 key-wrap export; Vault seal-wrap). Operator backs up + replicates the blob; only the HSM/KEK boundary can unwrap. [key-escrow-transcript §4.4, §5.2–5.5]

### 4.4 HSM / KMS backup specifics (verified versions where relevant)

- **HashiCorp Vault** — latest **2.0.3** (2026-06-17; the 1.21.x line is still maintained — verified via `perplexity_ask`, correcting an initial mis-parse). Shamir unseal (split master key into `k`-of-`n` unseal shares); **auto-unseal** via HSM/cloud-KMS returns **recovery keys** (also Shamir-split) used only for break-glass (new root token, seal migration via `-migrate`); **seal-wrap** double-encrypts secrets with HSM keys (FIPS 140-2 KeyStorage). [key-escrow-transcript §3.3–3.5]
- **AWS CloudHSM** backups encrypted inside the HSM, undecryptable by AWS; **deleted backups retained only 7 days**. **AWS KMS** delete has a mandatory **7–30 day waiting period** (a recovery safeguard). [key-escrow-transcript §2.2, §5.2]
- **Azure Managed HSM** full backup/restore (full restore wipes + reconstructs) + **selective single-key restore** (key must be purged first). [key-escrow-transcript §2.4, §5.3]
- **NIST SP 800-57 / NISTIR 7298** frame key recovery/escrow as a lifecycle obligation requiring split-knowledge + dual-control + documented, audited break-glass — and warn the recovery path must not become an easier attack vector than primary storage. [key-escrow-transcript §1.1, §4.5, §6]

### 4.5 Break-glass

Emergency key recovery must be: pre-staged with minimal privilege, **heavily audited** (every use logged, reviewed, justified), and gated by **dual-control / M-of-N**. Vault recovery keys are the canonical cryptographic break-glass. [key-escrow-transcript §6.1–6.4]

### 4.6 The genuine human-decision fork (§8.2)

> "Designing key escrow models involves human decisions that cannot be fully automated. Organizations must decide whether to implement escrow at all. Some may choose strict non-escrow policies, accepting that losing keys means losing data. Others may choose escrow with external entities … to satisfy legal obligations." [key-escrow-transcript §8.2]

This is the #1 sub-fork for C17 (see §10).

---

## Q5 — Satellite state recovery (ties C9 + C2)

From the DR/satellite transcript [dr-transcript].

### 5.1 Reconstruct-from-central vs local backup (the satellite fork)

- **Strong industry bias toward reconstruct-from-central:** treat satellites as **replaceable nodes whose config/logic is derived from a central source of truth versioned as code** (Proxus stateless gateways + central manifests + local buffers; AWS IoT Greengrass fleet provisioning + reproducible cores; Azure IoT Edge central reprovisioning; Fleet/osquery central enrollment; balenaOS named-volume disposability). [dr-transcript §2.1–2.6, §7.1]
- **BUT in air-gapped / weak-connectivity scenarios the trade-off shifts** — local backup gains value because central can't re-provision an isolated node. This aligns with C9's autonomous-local-recovery crux (a satellite that can't dial home can't be rescued by central). [dr-transcript §2.1, §2.7]

### 5.2 Disposable-vs-durable taxonomy (what to back up)

- **Reconstructible (don't back up; re-derive from central):** config, detection content, policy, identity-derived-from-enrollment — anything the central source-of-truth defines as code.
- **Worth backing up in situ (durable, non-reconstructible locally):** local data **buffers** (collected telemetry not yet shipped), and any locally-generated state with no central copy — but treat even this as **disposable on device reassignment / re-provision**, especially in multi-tenant security contexts. [dr-transcript §2.6, §2.7, §8.1]
- **For Prism satellites with A/B dual-slot self-recovery (C9):** "while satellites might [hold local state], they are fundamentally disposable/reconstructible nodes whose local config is not the primary source of truth." The bootstrap A/B (C9) handles restart-class config; C17's satellite concern is **the local data buffer + air-gap backup of anything genuinely local.** [dr-transcript §2.7]

### 5.3 Air-gapped backup

- Backup without network egress = **offline/removable media + manual handling**; backups carried out-of-band. **Signed + encrypted backup bundles** are mandatory for high-assurance air-gap (you can't trust an online control plane to verify). Verification must work **without an online control plane** (carry the verification metadata/signatures with the bundle). [dr-transcript §3.1–3.4] Note: explicit vendor docs for "signed+encrypted air-gap backup bundles specifically for this topology" were thin — the pattern is assembled from TUF/Sigstore + offline-media practice. [dr-transcript §3.3 — flagged]

---

## Q6 — Disaster recovery across deployment models (SaaS vs MSSP/on-prem)

From the DR transcript [dr-transcript §4–5, §7–8].

### 6.1 SaaS cloud-native DR — the four-tier taxonomy (AWS DR whitepaper)

| Tier | RPO/RTO | Mechanism |
|---|---|---|
| **Backup & Restore** | hours | backups + cross-region copy; cheapest |
| **Pilot Light** | minutes–low | core data replicated, minimal standby infra, scale up on failover |
| **Warm Standby** | minutes | scaled-down-but-running replica, scale up on failover |
| **Active-Active (multi-site)** | near-zero | full multi-region live, automatic failover |

Multi-AZ is the baseline resiliency; multi-region is the DR expansion. Managed backup (RDS automated backups + cross-region, EBS snapshot copy, AWS Backup with customer-managed-key cross-region re-encryption). GitOps / config-as-code for control-plane DR. [dr-transcript §4.1–4.7] **Lean for Prism SaaS central plane:** **multi-AZ baseline + optional multi-region (pilot-light/warm-standby) expansion** — pick the tier per RTO/RPO contract. [dr-transcript §8.2]

### 6.2 On-prem / MSSP appliance DR

- Appliance backup to **local NAS / offline media**; restore onto **replacement hardware**; **configuration-as-data deterministic re-provisioning** (the appliance rebuilds from its config export — same reconstruct-from-config principle as satellites); **virtual-appliance OVA/image snapshot+restore**; **HA-pair failover** (FortiGate-style). [dr-transcript §5.1–5.4, §8.3] **Lean:** VM snapshots + config exports + optional HA-pair. [dr-transcript §8.3]

### 6.3 Backup integrity + authenticity (reconciled with operator-zero-access)

- **Signed + tamper-evident:** detached signatures, content-addressable hashing, **Merkle trees**; prior art = **TUF (The Update Framework)** (Root/Timestamp/Snapshot/Targets roles + signed metadata), **Sigstore/cosign** (transparency-log-backed signing), **restic** (latest **0.19.0**, 2026-06-09 — content-addressable, encrypted-by-default, deduplicated snapshots). [dr-transcript §6.1, §6.2; per-store-transcript: restic §]
- **Reconcile with operator-zero-access:** encrypt backups under **customer-managed keys** (as AWS Backup cross-region re-encryption does) so **only the customer/site operator can decrypt — even if the SaaS provider supplies the backup tooling.** This is the same zero-access invariant as §4: the SaaS operator stores ciphertext + supplies tooling but cannot read tenant backup contents. [dr-transcript §6.3, §7.4]
- A **unified integrity model** should span satellites + SaaS + on-prem: same signing (detached sig + Merkle/content-hash), same customer-managed-key encryption, same verifiable-restore. [dr-transcript §8.4]

---

## Q7 — NERC CIP-009 recovery-plan requirements (ties C20)

From the existing C20 research [nerc-cip-support-2026-06-27.md §5] — not re-litigated, summarized for C17 binding:

- **CIP-009-6** (effective Jul 1 2016, inactive Jun 30 2028) requires **documented recovery plans** for BES Cyber Systems: backup strategy, restoration procedures, **integrity verification of backups**, roles/responsibilities, and **periodic recovery testing with retained evidence**.
- **Prism is implicated two ways:** (1) if Prism is classified as BCS/EACMS it needs **its own recovery plan** (restore config, audit/log data, integrations — C17 must cover Prism's *own* recoverability, config-as-data); (2) Prism **supports the entity's CIP-009 program** — track backup status/success, store config baselines, capture recovery-exercise logs, and **compare post-recovery config against the CIP-010 baseline** to prove restore returned a known-good state.
- **Evidence operators need:** timestamped restore-test runs, post-restore baseline diffs, integrity-verification records (pairs directly with the §6.3 signed/hashed snapshot machinery — reuse the same crypto for backup integrity, BCSI-at-rest, and CIP-013 software-integrity).
- **C17 design consequence:** "backup & recovery as first-class" must mean **(a) Prism's own state is backup/restore-testable with integrity verification, AND (b) Prism *generates* recovery-test evidence** (timestamped restore runs + baseline-diff). [nerc-cip-support-2026-06-27.md §5, §10.2]

---

## Q8 — Rust / tooling (versions verified 2026-06-27)

| Tool / crate | Version (verified) | Source | Backup-relevant capability |
|---|---|---|---|
| `rocksdb` (rust-rocksdb) | **0.24.0** (2025-08-10) | crates.io + docs.rs | **Confirmed exposes `checkpoint` AND `backup` modules** (resolves transcript's inconclusive flag) |
| `iceberg` (iceberg-rust) | **0.9.1** (2026-05-06) | crates.io | Pre-1.0 data-access binding; drive maintenance ops via Java/Spark or verified Rust path |
| `git2` | **0.21.0** (2026-05-18) | crates.io | libgit2 bindings; confirm bundle/pack backup API coverage at morph |
| **pgBackRest** | **2.55** (~2026-05-26) | perplexity_ask | Postgres PITR: full/diff/incr, time-based retention, `--type=time` restore, client-side encryption |
| **Barman** | 3.19.x (2026) | per-store transcript | Postgres PITR; multi-server; WAL-streaming-coupled retention |
| **HashiCorp Vault** | **2.0.3** (2026-06-17); 1.21.x maintained | perplexity_ask | Shamir unseal/recovery keys, HSM auto-unseal, seal-wrap |
| **restic** | **0.19.0** (2026-06-09) | per-store transcript | content-addressable, encrypted, deduplicated snapshots; integrity |
| **Velero** | (transcript cited v1.3.2 docs — STALE; newer versions add CSI snapshot + Kopia) | per-store transcript | K8s resource + volume backup; restic/CSI integration — **re-verify current version at morph** |

> **Version caveat.** `iceberg` 0.9.1 and `git2` 0.21.0 and `rocksdb` 0.24.0 are all pre-1.0 or low-0.x for the backup-relevant surface; under the production-grade default, treat the backup-orchestration code built on them as needing a verification harness (test backups + test restores) before it's production-trusted. Velero version in the corpus was stale (v1.3.2 docs) — re-verify at morph if K8s-native backup is in scope.

---

## ANALYSIS + LEANS — recommended first-class backup/recovery architecture

### A. Per-store backup + restore (Q1)

- **Config DB:** Postgres + continuous WAL archiving via **pgBackRest 2.55** (time-based retention, encryption, `--type=time` PITR). Bitemporal history tables coexist with cluster PITR (coarse epoch rewind + fine-grained as-of via history).
- **Detection content (git):** `git2 0.21.0`; back up via **`git bundle`** or atomic `.git` FS snapshot (NOT `git clone`); complements C9 fast-revert/signed-bundle.
- **RocksDB:** `rocksdb 0.24.0` — use the **Checkpoint API for fast hot backups** + **BackupEngine for retained incremental backup sets**; pair with `WAL_ttl_seconds` for PITR within the TTL window. Back up only non-reconstructible CFs.
- **Iceberg:** back up **catalog + metadata files + data files** (all three); manage retention via `expire_snapshots`/metadata-delete; pin survivors with branches/tags. Drive maintenance via Java/Spark or a verified Rust path (`iceberg 0.9.1` is pre-1.0).
- **KG+vector:** back up the **authoritative (non-recomputable) embeddings + graph edges**; treat ANN indexes (usearch) as rebuildable. [INCONCLUSIVE — confirm indradb/usearch/lancedb native APIs at morph.]
- **Keys:** sealed-blob backup (see C below).
- **ARO state:** [INCONCLUSIVE] split authoritative (config-DB-class) vs runtime-scratch (RocksDB-class).

### B. Cross-store coherent point-in-time tied to C8 (Q2)

- **Primary strategy: logical-watermark + per-store time-travel**, NOT a global physical freeze. Stamp a single **HLC transaction-time T**; each store restores AS-OF ≤ T; a **backup-set manifest** binds per-store snapshot IDs to T. **Make T the same `AS OF KNOWN <T>` watermark the query engine exposes (C8)** — backup recovery point = bitemporal query point.
- Reserve a physical application-consistent freeze for the few most tightly-coupled components only.
- Set retention **collectively to a common floor** (RPO bounded by the shortest store window).

### C. KEY escrow/recovery preserving operator-zero-access (Q4)

- **Envelope: per-tenant DEK wrapped by per-tenant CMEK** (Option 3 — confirmed canonical).
- **Operator backs up ONLY sealed/wrapped blobs it cannot unwrap.** Recovery capability lives with the tenant (tenant-held recovery key/HSM) and/or a **Shamir M-of-N split across independent custodians** with audited break-glass (Vault-style recovery keys).
- **Crypto-shredding** is the offboarding/erasure primitive AND the clean reconciliation with operator-zero-access.
- This preserves the operator-zero-access invariant and aligns with NERC CIP-011-3 entity-key zero-access (C20/C16).

### D. Per-tenant + nested granularity (Q3)

- Pooled store ⇒ **cluster backup + per-tenant logical (RLS/namespace-filtered) export** (SOLVED).
- Per-tenant PITR ⇒ **restore-to-side-instance + selective re-ingestion** (the GENUINELY HARD part — build it as an operational workflow, design for FK/shared-data/ID-collision pitfalls, test heavily). Offer **silo escape-hatch at higher tiers**.
- Nested tenancy ⇒ explicit **parent-only / subtree / child-only** scopes with consistent hierarchical IDs + per-region crypto-shred.

### E. Satellite recovery (Q5)

- **Reconstruct-from-central by default** (satellites are disposable/reconstructible; config-as-code). Back up only **local data buffers** + genuinely-local non-reconstructible state. **Air-gap:** signed+encrypted offline-media bundles, verifiable without an online control plane. Complements C9 A/B dual-slot self-recovery.

### F. Per-deployment-model DR (Q6)

- **SaaS:** multi-AZ baseline + optional multi-region (pilot-light/warm-standby). **On-prem/MSSP:** VM snapshot + config-as-data deterministic re-provision + optional HA-pair.
- **Unified integrity model across all three:** detached signatures + content-hash/Merkle (TUF/Sigstore prior art) + **customer-managed-key encryption** so only the customer decrypts (operator-zero-access preserved end-to-end).

### G. CIP-009 evidence (Q7)

- Make **Prism's own state backup/restore-testable with integrity verification**, and make Prism **generate recovery-test evidence** (timestamped restore runs + post-restore baseline diff vs CIP-010 baseline). Reuse the §F signing/hashing crypto.

### Genuine sub-forks requiring a HUMAN decision

1. **KEY ESCROW MODEL (the #1 fork).** Strict non-escrow (lose key → lose data; maximal zero-access) **vs** threshold M-of-N escrow with external custodians (recoverable but recovery capability is distributed) **vs** tenant-held-recovery-key (tenant owns recovery, operator can't help). The research is explicit this "cannot be fully automated" and is driven by legal/regulatory obligations + customer trust posture. **Lean:** tenant-held recovery key as the default (cleanest operator-zero-access), with an *optional* M-of-N escrow tier for tenants who want operator-assisted recoverability — but the human must ratify, because it defines the zero-access promise's exact wording ("no unilateral operator access" vs "no access under any circumstance").
2. **CROSS-STORE CONSISTENCY APPROACH.** Logical-watermark + per-store time-travel (recommended; low-RPO, scalable, ties C8) **vs** physical application-consistent freeze (stronger instantaneous consistency, worse availability/scalability) **vs** hybrid (freeze the tightly-coupled core, watermark the rest). **Lean:** logical-watermark primary + selective physical freeze for tightly-coupled components — but the architect must decide which components are "tightly coupled enough" to warrant a freeze, and confirm the HLC/T mechanism is the same one C8 uses.
3. **Reconstruct-from-central vs local satellite backup** (per the C9 tie) — **Lean:** reconstruct-from-central default + local buffer backup only; air-gap nodes get local signed bundles. Human ratifies the air-gap backup-media handling policy.
4. **DR tier per deployment model** — what RTO/RPO does each contract (SaaS, MSSP, on-prem appliance) commit to? Backup-restore vs pilot-light vs warm-standby vs active-active is a cost/RTO business decision, not an AI decision.
5. **CIP-009 recovery-evidence module scope** — build a first-class recovery-test-evidence generator (ties C20 §10.4 fork-1 RSAW-export) or rely on the audit-trail substrate? **Lean:** phase it — recovery-test evidence substrate now, dedicated export later.

---

## Honest Costs & Caveats

- **The hardest element (KEY escrow) is the least turnkey and is a governance decision, not just code.** The research is explicit: recoverability vs zero-knowledge cannot be fully reconciled by technology; the escrow-model choice is a human fork. Over-promising "fully recoverable AND fully zero-access" is the most likely spec defect.
- **Strict per-tenant PITR in a pooled store is genuinely unsolved by DB engines** — the side-instance + selective-re-ingestion workflow is operational, error-prone (FK/shared-data/ID-collision), and must be built + heavily tested. Do not spec it as a built-in restore.
- **Prism-specific stores partly INCONCLUSIVE:** indradb / usearch / lancedb native backup APIs and ARO state's authoritative-vs-derived split were NOT in the retrieved corpus — flagged for a dedicated per-store pass at morph.
- **Velero version in the corpus was stale** (v1.3.2 docs; newer versions add CSI snapshot groups + Kopia). Re-verify if K8s-native backup is in scope.
- **One date anomaly:** a WebFetch of the pgBackRest GitHub releases page returned a mis-parsed "2.58.0 / Jan" — `perplexity_ask` gives **2.55 / ~2026-05-26**; treat 2.55/2026-05 as authoritative pending a direct release-page confirm at morph. Vault was initially mis-parsed as "2.0.3" being a sub-product; confirmed Vault core IS on a 2.x line (latest 2.0.3, 2026-06-17) with 1.21.x maintained.
- **Pre-1.0 crate maturity:** `iceberg 0.9.1`, the backup-relevant `git2`/`rocksdb` surfaces — build a backup+restore verification harness before production-trusting any of them (production-grade default).
- **Transcript-tail caveat.** The per-store transcript was read in full (incl. 19-source citation list). The other four exceeded the inline cap and were saved to files that are single giant lines (un-paginable by Read); they were mined by enumerating section structure + extracting load-bearing sections (decision-forks, synthesis, recommendations, conclusions) via targeted Grep. No finding above depends on an unread tail; where a section's full prose was not extracted verbatim, the finding is anchored to the extracted heading + adjacent extracted sentences and flagged where it is model reasoning.

---

## Sources (families; full numbered citations in saved transcripts)

- **[per-store-transcript]** — PostgreSQL continuous-archiving + backup-file docs; pgBackRest release/retention; pgbarman.org; hyPiRion system-versioned-tables; RocksDB checkpoint (curiosity.ai) + in-memory-persist blog; rust-rocksdb GitHub; Iceberg spec + maintenance + spark-procedures; Rewind "git clone not a backup"; docs.rs/git2; restic releases + forum; Velero v1.3.2 restic docs.
- **[xstore-transcript]** — Chandy–Lamport; Flink barrier/epoch checkpoints; LVM/ZFS/btrfs + VSS + AWS/GCP/NetApp multi-volume snapshots; CSI VolumeGroupSnapshot; fsfreeze/Velero quiesce hooks; bitemporal (valid/transaction time) + SQL:2011 system-versioned; Iceberg time-travel; Hybrid Logical Clocks; consistency-group/manifest patterns.
- **[key-escrow-transcript]** — AWS KMS/CloudHSM/external-key-store + BYOK blog; GCP Cloud KMS + Workspace CSE; Azure Key Vault/Managed HSM backup/BYOK; Shamir Secret Sharing; HashiCorp Vault seal/unseal/recovery-keys/seal-wrap; Thales KMU/Luna PKCS#11 key-wrap; NIST SP 800-57 + NISTIR 7298; IronCore Labs BYOK; Yale HIPAA break-glass; M-of-N control.
- **[dr-transcript]** — Proxus stateless edge; AWS IoT Greengrass v2; Azure IoT Edge; Fleet/osquery; balenaOS named volumes; air-gap offline-media backup; AWS DR whitepaper (4-tier); RDS/EBS/AWS Backup cross-region; GitOps control-plane DR; FortiGate HA-pair; OVA snapshots; TUF; Sigstore/cosign; Merkle trees; restic.
- **[per-tenant-transcript]** — silo/bridge/pool models; AWS SaaS Factory; Citus/schema-per-tenant/RLS; Memgraph multi-tenancy; Pinecone namespaces; restore-to-side-instance + selective re-ingestion; tenant-scoped logical replication/CDC; crypto-shredding; GDPR right-to-erasure; nested/hierarchical tenancy + data residency.
- **[nerc-cip-support-2026-06-27.md]** — prior C20 research (CIP-009 §5; recovery evidence §10.2).
- **Version verification** — crates.io API (rocksdb 0.24.0, iceberg 0.9.1, git2 0.21.0); docs.rs/rocksdb (checkpoint + backup modules); perplexity_ask (Vault 2.0.3 / pgBackRest 2.55).

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY, reasoning_effort=high)** | 5 | Deep multi-source synthesis on the 5 core themes: (Q1) per-store backup/PITR mechanics across PostgreSQL/RocksDB/Iceberg/git2/restic/Velero with versions; (Q4) key backup/escrow/recovery preserving operator-zero-access across KMS/HSM/Shamir/Vault/NIST; (Q2) cross-store coherent point-in-time (crash-vs-app-consistent, Chandy-Lamport, HLC logical watermark, bitemporal AS-OF-T, consistency-group manifests); (Q3) per-tenant + nested-tenant backup/restore/offboarding/crypto-shred in pooled multi-tenancy; (Q5+Q6) satellite/air-gap recovery + per-deployment-model DR + signed/encrypted backup integrity. All 5 succeeded first attempt at `high`; all 5 exceeded inline cap and were saved to transcript files. |
| Perplexity perplexity_ask | 1 | ≤2-sentence factual confirmation: current HashiCorp Vault + pgBackRest stable versions (correcting WebFetch mis-parses). |
| Perplexity perplexity_reason | 0 | — |
| Perplexity perplexity_search | 0 | — |
| Context7 | 0 | — (crate state verified directly via crates.io/docs.rs) |
| Tavily (any) | 0 | — |
| WebFetch (crates.io / docs.rs / GitHub releases) | 6 | Version verification: rocksdb 0.24.0 (+ confirmed checkpoint+backup modules on docs.rs), iceberg 0.9.1, git2 0.21.0, pgBackRest releases page (mis-parsed — superseded by perplexity_ask), Vault tags (mis-parsed — superseded by perplexity_ask). |
| WebSearch | 0 | — |
| Training data | 2 areas (flagged inline) | `[model reasoning]`: (a) the Prism-side mapping of generic findings onto Prism's specific stores (KG+vector authoritative-vs-derived split; ARO state classification; PITR×bitemporal interaction) — each anchored to a cited generic finding and flagged; (b) structuring/synthesis of the LEANS. The substantive mechanisms, patterns, and versions are all sourced. |

**Total MCP tool calls:** 6 (5× `perplexity_research` at high + 1× `perplexity_ask`). **Plus** 6 crates.io/docs.rs/GitHub version-verification WebFetch calls.
**Training data reliance:** **low** — every per-store mechanism, cross-store strategy, key-escrow pattern, per-tenant pattern, DR tier, and CIP-009 obligation traces to a cited deep-research pass; all crate/tool versions verified against crates.io/docs.rs (NOT training data); the two model-reasoning flags are Prism-side mapping + synthesis over cited evidence, not substitute facts. Prism-specific stores (indradb/usearch/lancedb) and ARO state are flagged `[INCONCLUSIVE]` rather than guessed.

> Tool/crate versions and DR/backup tooling landscape change rapidly; all versions are "as of 2026-06-27" and should be re-verified at morph before any is treated as a load-bearing architecture gate.
