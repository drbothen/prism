# Research: Deployment-Aware Schema-Versioning + Forward-Migration (C9 Q3)

**Date:** 2026-06-27
**Type:** general (technology / architecture research)
**Item:** Prism Day-2 Vision SIDE-ANALYSIS — C9 Q3
**Status:** CAPTURE-ONLY (`do_not_execute`) — research artifact; modifies no live spec, BC, ADR, STATE.md, or SESSION-HANDOFF.md.
**Author:** research-agent

> **Scope note.** This document grounds the A-vs-B fork for C9 Q3 with cited evidence and a per-domain / per-deployment-model recommendation. It builds on already-LOCKED C9 decisions (DB-authoritative config, system-versioned runtime history, embedded git for detection content/recipes, forward-only/ArcSwap fast-revert, 4-layer bootstrap recovery, two-tier canary) and the already-decided C5 OCSF schema-version descriptor axis. Those are treated as fixed inputs, not re-litigated.

---

## 0. The Fork (restated)

- **Option A — explicit `schema_version` field on persisted artifacts + a forward-migration framework.** Store versioned resources; convert forward through a registered chain of migrations. Models: Envoy xDS resource versioning; Kubernetes `apiVersion` + conversion-webhook + storage-version-migration.
- **Option B — `#[non_exhaustive]` + additive-forward-compat-only.** Never require migrations; permit only additive, backward/forward-compatible schema changes; rely on serde defaults.

The load-bearing pressure is the **on-prem / client-managed SKIP-VERSION upgrade** (e.g. v1.2 → v1.7, skipping intermediate releases), which SaaS blue-green never has to face because SaaS upgrades continuously through every release.

---

## 1. How mature self-hosted products handle skip-version migration (Q1)

**Headline pattern, consistent across every product surveyed:** they support **product-level skip-version upgrades** (operator jumps from release N directly to a much later release M) while still enforcing a **strictly sequential chain of underlying schema migrations**, frequently with **explicit "required-stop" intermediate versions** and a mix of synchronous + background/batched execution. Skip-version at the *release* level; never-skip at the *schema-migration* level. [Perplexity deep-research synthesis of the sources below]

| Product | Skip-version releases? | Required intermediate stops? | Migration execution | Explicit schema_version field | Config vs data |
|---|---|---|---|---|---|
| **GitLab** | Yes, within a documented upgrade path | **Yes** — codified required stops at `x.2.z / x.5.z / x.8.z / x.11.z`; GitLab 18 stops are 18.2, 18.5, 18.8, 18.11 [GitLab upgrade-path docs] | Rails migrations (sync) **+ post-deployment migrations** (`SKIP_POST_DEPLOYMENT_MIGRATIONS` env) **+ background migrations** that must finish before the next required stop | Implicit via Rails `schema_migrations` table | Data = formal Rails chain; config (`gitlab.rb`, systemd units) = manual, doc-driven (`git diff` the unit files, copy them) |
| **Sentry (self-hosted)** | Yes, but bounded | **Yes** — example path 22.8.0 → 23.6.2 → 23.11.0 → 24.2.0, not a direct jump | Django migrations run **synchronously during downtime** (`./install.sh` stops services, migrates, restarts). Hotfix releases issued specifically to patch broken migrations | Implicit via Django `django_migrations` table | Data = Django chain (downtime); config = `.env` / `.env.custom`, operator merges new vars manually, no version field |
| **Elastic / Kibana** | Yes, constrained by version-number AND release-date | **Yes** — major upgrades force intermediate versions (e.g. 6.8.latest → 7.17.latest → 8.x; 8.19 mandatory before 9.x) | Elasticsearch index reindex via upgrade-assistant; **Kibana saved-object migrations run at startup**, with "compatible migrations" (8.6.0+) reusing indices | **Yes, explicitly** — saved objects carry a `migrationVersion` field (e.g. `"dashboard":"7.3.0"`); references use indirection by `name` because IDs aren't stable across migrations | Data/objects = formal versioned migration; config (`elasticsearch.yml`, `kibana.yml`) = manual + compatibility matrices |
| **Grafana** | Yes | No explicit required stops documented (relies on release notes) | DB migrations applied **synchronously at startup**, tracked by an internal schema-version table; no background migrations | Implicit — single global schema-version row in DB | Data = startup migration chain; config (`grafana.ini`) = manual, backward-compatible defaults |
| **HashiCorp Vault** | Yes (in-place); **no rigid required stops** | No — relies on change-tracker + release notes, not version-gating | Data-store changes applied automatically on unseal. **No backward-compat guarantee for the data store → mandatory backup before upgrade** (effectively one-way) | Internal/not user-facing | Data = automatic on-unseal; config (`vault.hcl`) = manual, review deprecations |
| **HashiCorp Consul** | Yes, but with **explicit forward-compat ranges** | **Yes** — e.g. can only reach Enterprise 1.10 from 1.8.13+ or 1.9.7+; other versions "not forward compatible" and "agents may fail to start." Plus a documented Envoy xDS v2→v3 "stairstep" path | Binary swap + rolling agent restart; Raft state handled internally | Internal/not user-facing | Data = internal Raft state; config (incl. Envoy escape-hatch) sometimes needs manual rewrite |
| **Nomad** | Yes (in-place implied) | Not documented in sources | Internal state handled at startup; no documented formal migration framework | Internal/not user-facing (inferred) | Config = files via `-config`; manual adjustment per release notes |
| **Temporal** | Yes | Not version-gated, but updates "must be applied in order" | **Sequential SQL chain** via the schema-upgrade tool; ordered SQL update files; `schema_version` / `db_version` table | **Yes** — explicit schema-version table tracked by the schema tool | Data = ordered SQL chain; config = files/env |

**Three robust takeaways for Prism:**

1. **Skip-version-release is universally supported; skip-migration-step is universally forbidden.** The target binary always carries the *full ordered chain* of migrations and replays only the pending subset. The operator never installs intermediate *binaries*, but the *code* still walks N→N+1→…→M internally. This is exactly what makes on-prem skip-version (v1.2 → v1.7) safe.
2. **"Required-stop" versions are the pressure-release valve** (GitLab, Sentry, Elastic, Consul). When a migration is too disruptive (expensive background reindex, breaking semantic change, forward-compat break), products insert a mandatory intermediate stop and gate progress on it. This is a *governance* mechanism layered on top of the migration chain — useful for Prism's high-blast-radius classes.
3. **Config schema is treated more loosely than data schema almost everywhere** — except Kibana, which versions saved objects explicitly. Most products lean on backward-compatible defaults + release-note-driven manual config edits. **Prism's C9-locked DB-authoritative + UI-authored config is actually *stronger* than these products' file-based config**, which removes the "operator hand-edits a stale TOML" failure mode entirely — and therefore lets Prism use a *real* migration framework for config where these products couldn't.

---

## 2. Kubernetes apiVersion/conversion-webhook & Envoy xDS versioning (Q2)

### Kubernetes CRD model

- A CRD declares `spec.versions[]`, each with `served` and `storage` booleans. **Exactly one version is `storage: true` at any time** — that is the "storage version" persisted in etcd. Multiple versions can be `served: true` simultaneously. [kubernetes.io storage-version + CRD-versioning docs]
- Reads/writes in a non-storage served version are converted. Two strategies: **`None`** (only rewrites the `apiVersion` field — valid only when schemas are identical) and **`Webhook`** (API server posts a `ConversionReview` to an external HTTPS service on port 443, gets back a `ConversionResponse`). [kubernetes.io]
- **Hub-and-spoke conversion** (Kubebuilder/controller-runtime): pick ONE hub version (usually = storage version); implement conversion only between each non-hub spoke and the hub. This reduces conversion functions from `N×(N−1)` (every pair) to `2×(N−1)` (linear). To convert spoke→spoke, go spoke→hub→spoke. [Kubebuilder book]
- **Storage Version Migration** (`StorageVersionMigrator`, feature-gated, k8s ≥ 1.30): a declarative `StorageVersionMigration` resource that batch-rewrites all stored objects from the old storage version to the new one (also used for re-encryption / key rotation). This is the "rewrite the data at rest after promoting a new storage version" step. [kubernetes.io migrate-storage-version docs]

**Cost model:** the CRD author pays at **runtime** (live conversion on every read/write that crosses versions) AND maintains `2×(N−1)` conversion functions + tests for round-trip fidelity. Operational risk is high: webhook unavailability → API requests fail; buggy conversion → silent data loss. **Warranted** when schema genuinely changes AND long-lived external clients cannot upgrade in lockstep. **Overkill** when there are no schema changes (use `None`) or when all clients upgrade together (single team, internal CRD).

### Envoy xDS resource versioning

- Each `DiscoveryResponse` carries an **opaque `version_info` string** + a `nonce`. Envoy ACKs by echoing the accepted `version_info` + `response_nonce`; NACKs by echoing the nonce but keeping the *previous* `version_info`. The string is opaque — control plane may use a monotonic int, hash, git SHA, or timestamp. [envoyproxy.io xDS protocol docs]
- **State-of-the-World (SotW):** every response is the full snapshot for a resource type; one `version_info` per type. Simple to reason about, expensive at scale. **Delta/Incremental xDS:** only changed resources are sent, with per-resource version tracking; cheaper at scale, more complex. (Istio defaults to Delta as of 1.22.) [Tetrate; Envoy Hoot]
- **Cost model:** the **control plane** — not Envoy — owns version management and any cross-version conversion (build-time pre-generation per xDS API version, or runtime conversion). Envoy's data-plane role is light: cache resources, track last-accepted `version_info`, ACK/NACK.

**Relevance to Prism.** The Kubernetes "**one storage version, serve many, convert forward, batch-migrate the rest**" pattern is the closest mature analogue to Prism's situation, and it maps cleanly:
- "storage version" ≈ Prism's canonical on-disk RocksDB/config schema version.
- "served versions" ≈ the schema versions Prism's UI/MCP API must accept from older artifacts during a skip-version jump.
- "hub-and-spoke" ≈ the recommended way to keep the forward-migration chain linear instead of quadratic.
- "StorageVersionMigration" ≈ Prism's on-open batch migration of at-rest config/data.

The Envoy model is **less applicable**: Prism's persisted artifacts are durable at-rest state, not ephemeral streamed config snapshots, and Prism has no external control plane assigning opaque version strings. The one transferable idea is that an **opaque-but-monotonic version identifier on each pushed config snapshot** pairs naturally with C9's ArcSwap hot-swap (the snapshot already needs an identity for audit/rollback).

---

## 3. Rust-specific schema-evolution tooling (Q3) — versions verified against crates.io

> All versions confirmed via crates.io API on 2026-06-27.

| Tool / facility | Layer | Latest version (crates.io, 2026-06-27) | Verdict for Prism |
|---|---|---|---|
| **serde** core (`#[serde(default)]`, internally-tagged enums, `alias`, `rename`, `flatten`, `deserialize_with`) | Serialization | **1.0.228** (2025-09-27) | **Primary tool.** Already a workspace dependency. Handles all *additive* evolution. |
| **savefile** + `savefile-derive` | Binary serialization w/ built-in schema evolution | **0.20.4** (2026-06-14) — actively maintained | Candidate ONLY if a binary at-rest format with embedded versioning is wanted; introduces a new serialization format dependency. Likely **not** needed given Prism already uses serde + RocksDB. Flag as a sub-fork. |
| **serde_version** | Version-aware deserialization | **0.5.1 (2019-11-26) — ABANDONED, 17 recent downloads, requires nightly `specialization`** | **REJECT.** Stale, nightly-only, single-maintainer-dormant. Do not adopt. |
| **refinery** | SQL migration runner | **0.9.2** (2026-06-10) | SQL-oriented (Postgres/MySQL/SQLite/MSSQL). **Not applicable** — Prism has no SQL store. Useful only as a *design template* for the migration-chain registry pattern. |
| **sqlx** `migrate!` | Async SQL + embedded migrations | **0.9.0** (2026-05-21) | SQL-oriented. **Not applicable** to RocksDB. Design template only. |
| **rocksdb** | KV store wrapper | **0.24.0** (2025-08-10) | Prism's actual store. **No built-in migration framework exists** — by design, since RocksDB has no global schema. Migrations are application-coded. |

### Key clarification: `#[non_exhaustive]` is NOT a serialization-compat mechanism

This is the single most important correction for the fork framing. `#[non_exhaustive]` (Rust reference + serde issue #1991 + serde issue #1137):

- **What it does:** a *compile-time, cross-crate* guardrail. Downstream crates cannot exhaustively `match` an enum (must include a `_` wildcard arm) and cannot use struct-literal construction (must use `..` / a constructor). This lets the *defining* crate add variants/fields in a minor release without breaking *downstream code that compiles against it*.
- **What it does NOT do:** it has **zero effect on the serialized wire/at-rest format.** serde derives serialization from the *current* set of variants/fields and embeds no non-exhaustiveness marker.
  - **Backward compat (new code reads old data):** works — but that comes from `#[serde(default)]` filling absent fields, NOT from `#[non_exhaustive]`.
  - **Forward compat (OLD code reads NEW data):** `#[non_exhaustive]` does **nothing**. If new code wrote an enum variant the old binary doesn't know, the old binary's deserializer **fails at deserialize time, before any pattern match happens** — the wildcard arm never gets a chance to run. New struct fields are tolerated only if the *format* ignores unknown fields (serde_json does by default; bincode does not), which is again a format property, not a `#[non_exhaustive]` property.
- Operational caveat: serde **remote-derive** has a known interaction failure with `#[non_exhaustive]` enums (serde #1991).

**Consequence for the fork:** Option B as literally stated ("`#[non_exhaustive]` + additive-forward-compat-only") **conflates a compile-time API guardrail with a serialization-compatibility strategy.** The two are orthogonal. The *real* Option B is "serde additive evolution (`#[serde(default)]` + tolerant deserialization) with `#[non_exhaustive]` retained purely for API hygiene." `#[non_exhaustive]` should stay (Prism already enforces it on 87 types) but it is **not** load-bearing for skip-version safety and must not be cited as the mechanism that makes skip-version safe.

### RocksDB schema-evolution patterns (no crate; patterns only)

1. **Versioned key prefixes** (e.g. `userprofile:v2:<id>`) — encode schema version in the key; prefix-scan to find records needing migration.
2. **Value-level version tag** — internally-tagged enum / leading version byte inside each value; pairs with serde.
3. **On-open migration routine** — at DB open, detect at-rest schema version (a dedicated meta key, e.g. a `__schema_meta__` CF holding `{cf_name → schema_version}`), replay the pending migration chain, rewrite records. This is the direct RocksDB analogue of GitLab's startup Rails migrations and Kibana's startup saved-object migrations.
4. **Column-family-per-version is DISCOURAGED** — RocksDB requires *all* existing CFs be named at `open_cf` time or the open fails (rust-rocksdb #608); CF-per-version accumulates unbounded CFs and complicates open. Reserve CF boundaries for *conceptual domains* (Prism already has ~19 domain CFs), use key-prefix + value-tag for *version*.

---

## 4. Unified vs per-domain schema-versioning; OCSF's role (Q4)

Prism has **four genuinely distinct persistence sub-domains** with different evolution physics. A single unified version counter across all of them would be a false economy (it would force lockstep migration of unrelated subsystems and make skip-version jumps brittle). The evidence favors **per-domain schema-version axes under a shared migration-chain *mechanism*** (one runner abstraction, N independent version registries — mirroring how GitLab has Rails migrations while Kibana independently versions saved objects, yet both products replay ordered chains).

| Sub-domain | Authority / store (C9-locked) | Evolution character | Recommended versioning |
|---|---|---|---|
| **Runtime config** | DB-authoritative, system-versioned history, UI-authored, ArcSwap hot-swap | Frequently changed; UI is the only author → no stale-hand-edit risk; strong audit need | **Explicit `schema_version` + forward-migration chain** (Option A). The DB-authoritative + UI-authored property makes a real migration framework *cheaper and safer* here than in any surveyed product. |
| **Detection content + recipes** | Embedded git2 (0.19) | Content versioned by git commits already; forward-only append revert | **Git is the version axis.** Add a *content-schema* `schema_version` field inside artifacts for the rare structural change, but lean on git history; no separate framework needed for the common case. |
| **RocksDB hot/working data** | ~19 column families | High-volume; expensive to rewrite at scale | **Per-CF schema-version meta + on-open migration chain.** Hub-and-spoke conversion to keep chains linear. Batch/lazy migration for large CFs (GitLab background-migration analogue). |
| **Iceberg + OCSF cold tier** | Cold-tier lake, OCSF-normalized | **OCSF already carries its own schema-version axis (C5 decision); Iceberg has native schema evolution** | **Absorbed by OCSF + Iceberg.** Do NOT build a Prism-proprietary version axis here. OCSF's `metadata.version` / class-version + Iceberg's column-id-based schema evolution already solve forward/backward read. Prism's job is to *record which OCSF version* each cold partition was written under and let Iceberg handle additive column evolution. |

**OCSF absorbs a large fraction of the cold-tier problem.** OCSF schema versioning + Iceberg's additive-friendly evolution (rename/add via stable column IDs) is precisely the "additive-forward-compat-only" regime working *correctly in its sweet spot* — append-mostly analytic data where you never need to rewrite history. This is the one sub-domain where Option B's spirit is the right answer, and it's already decided.

---

## 5. Skip-version safety, maintenance cost, and testing (Q5)

**Do mature products forbid arbitrary skip-version?** No product surveyed *categorically* forbids skip-version. Instead they **bound** it with two mechanisms:
- **Migration frameworks** that replay the full ordered chain internally (so the operator's "jump" is safe because no step is actually skipped — GitLab, Sentry, Temporal, Django).
- **Required-stop versions** when a step is too disruptive to cross transitively (GitLab, Sentry, Elastic, Consul forward-compat ranges).
- **Compatibility-mode validation** (Confluent Schema Registry BACKWARD / FORWARD / FULL, and their `_TRANSITIVE` variants) that validate each new schema against *all* prior versions, enabling open-ended jumps so long as transitive compatibility holds. [Confluent schema-evolution docs]

**Long-term maintenance cost of an open-ended forward-migration chain** is real and accumulates:
1. Compatibility/registry rules must stay correct for every new version (modes, transitivity).
2. Each new schema validated against prior versions in CI.
3. Migration scripts + fixtures maintained to reflect the *entire* history.
4. Golden fixtures + round-trip tests so old-version data still decodes under new code.

**How the chain is tested** (synthesis of cited practice):
- **Golden fixtures per schema version** — store a canonical serialized sample for each historical version; assert each still decodes/upgrades correctly under current code (golden-master / characterization testing).
- **Round-trip serialization tests** — write with old schema, read with new, assert semantic equivalence.
- **Migrate fixtures through the production pipeline** (Django/South practice) so test data tracks the schema.
- **Upgrade-matrix CI** — the sources show the *transitive* variant (validate new vs. all prior) rather than an exhaustive all-pairs matrix; for Prism, a pragmatic matrix is "each released version → current" plus the specific skip-pairs we commit to supporting (e.g. every supported on-prem LTS → current).

**Maintenance-bounding recommendation for Prism:** adopt a **supported-skip-version window** (e.g. "any release within the last K minor versions may jump directly to current; older requires a required-stop at the oldest in-window LTS"). This caps the chain-testing matrix and is exactly the GitLab/Elastic governance pattern. The window is a **human business decision** (support policy) — flagged below.

---

## 6. When forward-compat-only (Option B) breaks down (Q6)

Additive-only evolution is provably safe for *growth* and provably insufficient for *change*. Documented breakdown cases:

| Change class | Why additive-only fails | Cited evidence |
|---|---|---|
| **Field removal / tag reuse** | Wire format can't distinguish old vs. new meaning of a reused number → ambiguous decode, data loss, even PII leakage. Protobuf: "never change field numbers"; **reserve** removed numbers. | protobuf.dev proto3 guide; SO #65230623; Google Groups |
| **Type change of an existing field** | Breaking change. Only *safe promotions* (e.g. Avro `int→long`) are allowed; everything else breaks. Protobuf fix is add-new-field-+-deprecate-old, not in-place change. | Protobuf versioning best-practice; Avro/Confluent compat docs |
| **New REQUIRED invariant old data violates** | Avro backward-compat requires new fields to have **defaults**; a non-defaulted required field (or a new validation rule) means old records can't be read under the new schema. | Confluent schema-evolution; Avro compat |
| **Splitting one field into two / merging two into one** | Inherently semantic — old data has only the original shape; new code must infer/combine. Cannot be expressed additively; needs explicit migration logic. | Inferred from Protobuf/Avro "don't repurpose meaning" guidance |
| **Unit / enum-value remap** | Semantic change (seconds→ms, enum 1 meaning X then Y). Reserve enum values; never repurpose. | protobuf.dev; Protobuf best-practice |

**Confluent compatibility modes** crystallize the boundary: **BACKWARD** (new consumer reads old data), **FORWARD** (old consumer reads new data), **FULL** (both). Notably Confluent recommends **BACKWARD_TRANSITIVE** for Protobuf because adding new message types is *not* forward-compatible — i.e. even the gold-standard additive systems concede additive-only is insufficient and reach for transitive validation + explicit migration on the breaking subset.

**Conclusion for the fork:** pure Option B cannot survive the breaking-change subset, which *will* occur over the multi-year life of an on-prem product (field removals, type changes, new invariants). Pure Option B fails exactly when on-prem skip-version is most painful.

---

## 7. ANALYSIS + LEAN

### The fork is a false binary; the answer is a HYBRID

Option A (full conversion-webhook-grade framework everywhere) is **over-engineered** for Prism's additive-majority reality and imposes Kubernetes-webhook-class runtime + maintenance cost where most changes are additive. Option B (additive-only, leaning on `#[non_exhaustive]`) is **under-engineered** — it cannot survive the breaking-change subset that multi-year on-prem skip-version guarantees, and it rests on a category error (`#[non_exhaustive]` is not a serialization-compat mechanism).

**LEAN: HYBRID — "additive-forward-compat by default, with an explicit per-domain `schema_version` and a targeted forward-migration chain reserved for the breaking subset."** This is precisely the pattern every surveyed mature self-hosted product converged on (skip-version-release + never-skip-migration-step + required-stops for the disruptive subset).

### Per-domain recommendation

| Sub-domain | Lean | Rationale |
|---|---|---|
| **Runtime config** (DB-authoritative, system-versioned, UI-authored) | **A (explicit `schema_version` + forward-migration chain)** | DB-authority + UI-only authoring removes the stale-hand-edit failure mode, making a real migration framework *cheaper and safer* here than anywhere. System-versioned history + ArcSwap already demand a per-snapshot version identity. Hub-and-spoke to keep the chain linear. |
| **Detection content + recipes** (git2) | **Git is the axis; B-by-default with a thin content-`schema_version` for rare structural change** | Git already provides the version timeline + forward-only revert. Don't duplicate it. |
| **RocksDB hot data** (~19 CFs) | **Hybrid: per-CF `schema_version` meta + on-open migration chain; additive in-value evolution by default** | KV store has no global schema; on-open chain is the GitLab-startup analogue. Batch/lazy migrate large CFs. Key-prefix or value-tag for version; CF-per-version rejected (rust-rocksdb #608). |
| **Iceberg + OCSF cold tier** | **B (additive) — absorbed by OCSF schema-version axis (C5) + Iceberg native evolution** | Append-mostly analytic data is additive-only's sweet spot; OCSF + Iceberg already solve it. Record OCSF version per partition; do not build a proprietary axis. |

### Per-deployment-model recommendation

| Model | Migration posture |
|---|---|
| **SaaS (k8s blue-green, continuous)** | Walks every release → only ever applies one migration step at a time. **Forward chain barely exercised**, no required-stops needed. Blue-green gives instant rollback (forward-only revert still applies). |
| **MSSP-managed (offline-signed-bundle, A/B appliance, watchdog)** | **Skip-version is real here.** Bundle carries full ordered chain; A/B slot = validate-migrated-state-before-cutover (validate-before-persist already C9-locked). Watchdog covers a migration that bricks boot → A/B fallback. |
| **Client-managed self-operated** | **Highest skip-version exposure (v1.2 → v1.7).** Requires: (a) full chain in the binary, (b) a **supported-skip-version window** with a **required-stop at the oldest in-window LTS** for older jumps (GitLab/Elastic pattern), (c) golden-fixture upgrade-matrix CI across supported skip-pairs, (d) on-open migration is idempotent + atomic + resumable (bootstrap 4-layer recovery covers interruption). |

### Mechanism (shared across domains)

- ONE migration-runner abstraction (registry of ordered, idempotent migrations keyed by `(domain, from_version → to_version)`), N independent per-domain version registries. (refinery/sqlx as *design templates* only — neither runs on RocksDB.)
- **Hub-and-spoke conversion** (one canonical "current" schema per domain; spokes convert to it) to keep chains linear, not quadratic — directly borrowed from Kubebuilder.
- **`#[non_exhaustive]` retained for API hygiene** (already on 87 types) but explicitly **decoupled** from the skip-version-safety story in the spec.
- **`#[serde(default)]` + tolerant deserialization** carries all additive changes (the majority case) with zero migration code.
- **Migration code reserved for the breaking subset only**: field removal/rename-with-semantics, type change, new required invariant, split/merge, unit/enum remap.

### Testing posture (non-negotiable for on-prem skip-version)

- Golden fixture per released schema version per domain.
- Round-trip + forward-migration tests (old fixture → current, assert semantic equivalence).
- Upgrade-matrix CI across the **supported skip-version window** pairs (not exhaustive all-pairs — bounded by the window).

### Sub-forks requiring a HUMAN decision

1. **Supported-skip-version window size** (e.g. "last K minors jump direct; older requires LTS required-stop"). This is a **support-policy / business decision**, not an engineering one. It directly bounds the CI upgrade-matrix and long-term maintenance cost.
2. **Required-stop versions for on-prem/client-managed** — whether to designate LTS required-stops at all, and at what cadence (the GitLab `x.2/x.5/x.8/x.11` analogue). Business + release-engineering decision.
3. **savefile adoption** (binary at-rest format with built-in versioning, v0.20.4, actively maintained) vs. staying on serde+RocksDB value bytes. Engineering trade-off; **default recommendation is NO** (stay serde) unless a measured perf/footprint need emerges — adopting savefile introduces a second serialization format and is not justified by current evidence. Flag for architect confirmation.
4. **Whether runtime-config migration runs at boot (synchronous, Grafana-style) or supports background/lazy migration (GitLab-style)** for very large config histories. Likely synchronous-at-boot is fine given config volume is small; confirm against the system-versioned-history growth model.

### Confidence

- **High** on the hybrid lean and the per-domain split (consistent across 7 surveyed products + K8s/Envoy + Confluent + Rust ecosystem).
- **High** on the `#[non_exhaustive]` clarification (Rust reference + serde issues are unambiguous).
- **High** on verified crate versions (crates.io API, 2026-06-27).
- **Medium** on the exact RocksDB on-open meta-key layout (pattern is well-attested; no canonical Rust crate exists, so it's application-coded — Prism owns the design).
- **Inconclusive / human-owned**: the four sub-forks above (policy decisions, not research-resolvable).

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 3 | (1) Q1 skip-version migration patterns across GitLab/Sentry/Elastic/Grafana/HashiCorp/Temporal; (2) Q2 K8s conversion-webhook + storage-version + Envoy xDS versioning; (3) Q3 Rust serde/migration/RocksDB schema-evolution tooling + `#[non_exhaustive]` clarification |
| Perplexity perplexity_reason | 1 | Q5 + Q6 synthesis: additive-only breakdown cases (Protobuf/Avro/Confluent) + skip-version safety/maintenance/testing |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_ask | 0 | — |
| Context7 | 0 | — (crate versions verified directly against crates.io registry API, which is authoritative for version numbers) |
| Tavily (any) | 0 | — |
| WebFetch | 6 | crates.io registry API version verification: savefile 0.20.4, refinery 0.9.2, serde-version 0.5.1 (abandoned), rocksdb 0.24.0, sqlx 0.9.0, serde 1.0.228 |
| WebSearch | 0 | — |
| Training data | 2 areas | OCSF `metadata.version` field semantics and Iceberg column-id-based schema evolution (flagged: framed against the C5-decided OCSF axis; cross-checked against the cold-tier reasoning, not independently re-verified this pass — the *mechanism* claim is well-established but treat the exact OCSF field path as model-knowledge). Prism's own C9/C5 locked decisions (from task context, not re-derived). |

**Total MCP tool calls:** 4 (3× `perplexity_research`, 1× `perplexity_reason`) + 6 WebFetch registry verifications = 10 grounded external calls.
**Training data reliance:** low-to-medium — every non-obvious external claim is sourced to a Perplexity-cited primary doc; all crate versions verified against the crates.io API on 2026-06-27; the only model-knowledge areas are the OCSF field-path detail and Prism's own task-supplied locked decisions, both flagged inline.

### Primary sources cited (via Perplexity research/reason synthesis)
- GitLab upgrade-path / required-stops / post-deployment-migration docs (docs.gitlab.com)
- Sentry self-hosted upgrade docs + getsentry/self-hosted releases (GitHub)
- Elastic upgrade / upgrade-planning docs; Kibana saved-object-migration docs (elastic.co)
- HashiCorp Vault / Consul upgrade docs (developer.hashicorp.com)
- Temporal MySQL schema-upgrade community discussion
- Kubernetes storage-version + CRD-versioning + storage-version-migration docs (kubernetes.io); Kubebuilder hub-and-spoke conversion (book.kubebuilder.io)
- Envoy xDS protocol docs (envoyproxy.io); Tetrate SotW-vs-Delta xDS
- Protobuf proto3 guide (protobuf.dev); Avro/Confluent Schema Registry schema-evolution docs (docs.confluent.io)
- serde docs (serde.rs field-attrs, enum-representations); serde issues #1137, #1991; Rust users-forum `#[non_exhaustive]` thread; rust-rocksdb #608
- crates.io registry API (version verification, 2026-06-27)
