---
document_type: research
produced_by: research-agent
status: capture
do_not_execute: true
timestamp: "2026-06-27"
provenance: "day-2 vision SIDE-ANALYSIS (OUT-OF-BAND, SEPARATE from the live VSDD factory pipeline); decision-gating head-to-head. Does NOT modify vision/specs/STATE/SESSION-HANDOFF/live-ADR-registry/prior-research."
topic: "Cold analytic tier storage format decision: KEEP Apache Iceberg (committed in ADR-PROP-storage-engine-taxonomy) vs SWITCH to Hive-partitioned OCSF Parquet (the Amazon Security Lake layout) for Prism's OWN self-managed cold tier"
decision_gated: "ADR-PROP-storage-engine-taxonomy.md §Decision row 2 (Apache Iceberg cold ANALYTIC tier) — amend or keep"
feeds: "storage-taxonomy ADR-PROP amendment — DISCUSSION input only; the lean is decision input for the human + architect, NOT an executed change."
non_contradiction_read:
  - "research/siem-lake-federation-2026-06-27.md (C5 — Security Lake = Hive-Parquet not Iceberg; iceberg-rust pre-1.0 read/write maturity; OCSF version skew 1.1->1.6; append-only write-path finding. NOT re-derived; this pass tests whether C5's two-providers-one-engine framing should collapse to one provider.)"
  - "specs/day2-design-decisions/ADR-PROP-storage-engine-taxonomy.md (the committed four-engine taxonomy; the five stated reasons Iceberg was chosen for cold — each tested below.)"
  - "research/capability-descriptor-pushdown-2026-06-26.md (C3 — DataFusion Exact/Inexact/Unsupported pushdown contract honored by BOTH ListingTable/Parquet and IcebergTableProvider. NOT re-derived.)"
---

# Cold Tier: Apache Iceberg vs Hive-Partitioned OCSF Parquet — Head-to-Head

**Side-analysis / decision-gating comparison — NOT a spec or ADR change.** This document produces the cited head-to-head needed to decide whether Prism's OWN self-managed COLD analytic tier should KEEP Apache Iceberg (as committed in `ADR-PROP-storage-engine-taxonomy.md`) or SWITCH to Hive-partitioned OCSF Parquet (the exact `region=/accountId=/eventDay=` layout Amazon Security Lake uses). `do_not_execute: true`. The lean is decision input for the human + architect; nothing here is executed.

> **The key insight under test.** C5 established that Amazon Security Lake is Hive-partitioned OCSF Parquet, NOT Iceberg. So a Hive-Parquet cold tier would UNIFY to ONE DataFusion `ListingTable` provider serving BOTH Prism's cold tier AND Security Lake, whereas Iceberg keeps TWO providers (`IcebergTableProvider` for cold + `ListingTable` for Security Lake). This pass weighs the one-provider win against the Iceberg capabilities it forfeits.

> **Scope fence.** The DataFusion pushdown contract (Exact/Inexact/Unsupported) is OWNED by C3 and is referenced, not re-derived. Security Lake's storage shape, OCSF version skew, and iceberg-rust pre-1.0 write maturity are OWNED by C5 and referenced. This file's job is the FORMAT-vs-FORMAT comparison along seven axes, the decision matrix, and the flip conditions.

> **Currency note (load-bearing, supersedes C5 on one point).** C5 (same day) cited the iceberg-rust `0.9.1 / 0.10.0` family from crates.io. Context7's live mirror of the Apache Iceberg site carries a **`2026-03-10` blog post announcing the iceberg-rust 0.9.0 release** with *"significantly expanded"* DataFusion integration: **DDL via SQL (create/drop table), limit pushdown, expanded predicate pushdown (Boolean, IsNaN, Timestamp, Binary, string pattern matching), and insert with automatic sort-based clustering for partitioned writes."* [Ctx7-iceberg-090] The read/write path is therefore meaningfully MORE capable today than the C5 snapshot implied — but it is still 0.x (pre-1.0). Both facts are load-bearing below. The exact crates.io version-string discrepancy (C5's 0.9.1/0.10.0 vs the site's 0.9.0 release post) is flagged [INCONCLUSIVE on the precise current patch] — the *capability* set is what matters and it is confirmed.

---

## Executive Summary (~16 lines)

1. **This is genuinely close, and the deciding axis is SCHEMA EVOLUTION under OCSF drift — the one axis where Hive-Parquet is materially weaker and the workload provably hits the weakness.** OCSF drifts 1.1->1.6 with added/deprecated/RETYPED fields (C5 confirmed 1.4.0 alone carried ~12 deprecations + ~140 net changes; Security Lake is pinned 1.1/1.3, upstream is 1.6.0). A multi-year store WILL contain files written under multiple OCSF versions. [C5][query-OCSF]
2. **Iceberg wins schema evolution DECISIVELY** via table-level field-ID schema authority: add/rename/drop/reorder + type-promotion rules enforced at write time, with old snapshots preserving historical schemas. [Iceberg-spec][Ctx7-iceberg-schema] Hive-Parquet has NO table-level schema authority — each Parquet file embeds its own schema and reconciliation is the reader's problem. [DF-ListingOptions]
3. **BUT the Hive-Parquet schema-merge gap is narrower than C5's framing implied, due to current DataFusion internals.** DataFusion 51.0.0/52.0.0 introduced an explicit `TableSchema` abstraction separating `file_schema` from `table_schema` + partition columns, and a changelog entry **"Handle merged schemas in parquet pruning"** confirms cross-file schema handling exists. [Ctx7-df-tableschema][Ctx7-df-mergedschema] Added columns (1.6 fields absent from 1.1 files) null-fill cleanly when an explicit table schema is supplied. The HARD failure is RETYPED fields (int->struct, string->struct): Databricks documents that Parquet directories with type-conflicting files fail schema unification and must be split into separate tables. [DB-schema-mismatch] Iceberg forbids those same structural retypes too — so the difference narrows to "Iceberg gives you a managed migration path (add new field-ID, deprecate old); Hive-Parquet gives you a directory-of-mixed-schemas you must reconcile by hand or by per-version tables."
4. **DataFusion READ maturity favors Hive-Parquet TODAY, but the gap CLOSED sharply in 2026.** `ListingTable` is DataFusion's most-exercised path (Hive partition inference defaults on, row-group + page-index pruning, filter/projection/limit pushdown). [DF-custom-tp][DF-parquet-blog] iceberg-rust's `IcebergTableProvider` is pre-1.0 but, per the 2026-03-10 0.9.0 post, now has limit + broad predicate pushdown + manifest pruning and is proven in production (Cheetah, 60ms Iceberg-over-Rust). [Ctx7-iceberg-090][Cheetah] The gap is now "mature-and-default" vs "capable-and-pre-1.0," not "mature vs experimental."
5. **WRITE/append/retention favors Hive-Parquet for SINGLE-writer-per-table append-only** (just PUT a Parquet object into the partition dir; retention = delete partition dirs). [HN-parquet-append] Iceberg adds catalog-commit + manifest overhead that buys little under single-writer. For CONCURRENT writers Iceberg wins decisively (ACID compare-and-swap commit, snapshot isolation). [Iceberg-acid] Prism's cold tier is single-writer-per-table by design (C5 SQ6 lean), so this axis leans Hive-Parquet — conditionally.
6. **CONCURRENCY/CONSISTENCY favors Iceberg structurally** (readers pin a snapshot, never see partial commits) but the practical gap is SMALL for a cold, batch-written, append-only tier where writes don't overlap heavy reads. The Hive-Parquet race (reader sees a partially-written object / mid-delete partition) is mitigated by write-to-temp-then-atomic-PUT + the fact that S3 object PUT is atomic (the object is invisible until complete). The unmitigated residue is the directory-LISTING race during retention deletes. [object_store][Iceberg-acid]
7. **PRUNING granularity favors Iceberg** (manifest-list -> per-file column stats -> row-group, plus it AVOIDS object-store LIST entirely — "Iceberg was born to avoid listing large tables"). [Dremio-metadata][Onehouse-metadata] For TIME-BOUND queries the practical delta is MODEST: eventDay partition-prune + time-sorted row-group stats already skips most data. [DF-parquet-blog] For EQUALITY predicates on high-cardinality OCSF fields (principal, resource) Iceberg's per-file stats + bucket transforms prune finer than directory partitioning. At multi-year/many-file scale the LIST-avoidance matters for latency + S3 request cost; below that scale it is inferential, not benchmarked. [INCONCLUSIVE on magnitude in DataFusion/Rust — no published head-to-head benchmark.]
8. **TIME-TRAVEL is NOT genuinely needed for the cited backtest/model-audit use IF the store is append-only — and it is.** On an append-only store, "table as of T" is approximable by an `ingest_time <= T` predicate (data is never mutated, so snapshot-as-of ~= ingest-time-filter). The difference bites ONLY on late-arriving data / backfills / corrections — and Prism routed the mutation-heavy workload to Postgres, keeping the cold tier append-only. Critically, **Iceberg SQL-level time-travel (`AS OF`) is NOT surfaced through iceberg-datafusion today** — snapshot selection is API-level only — so Iceberg's headline time-travel advantage is partly UNAVAILABLE in Prism's exact stack regardless. [Onehouse-metadata][Spark-iceberg-tt][Athena-iceberg-tt] This neutralizes ADR-PROP's time-travel rationale more than expected.
9. **UNIFICATION VALUE is real and quantifiable: ONE `ListingTable` provider serving both cold tier + Security Lake collapses a dependency tree (drop pre-1.0 iceberg-rust entirely), one schema-on-read OCSF reconciliation, one partition-projection code path, one thing to test/secure/`#[non_exhaustive]`-audit.** The counterweight — "teams who chose plain-Parquet later regretted it" — is real BUT the documented regret drivers are multi-writer ACID, frequent updates/deletes, and small-file/streaming pain. [Dremio-scaling][Decube] For a single-writer, append-only, batch-written cold tier those drivers DON'T fire; the one regret driver that DOES apply is schema evolution at scale (axis 1). The reverse prior art is equally documented: Iceberg is "overkill" for single-writer append-only datasets — "just use Parquet." [Decube-overkill][Iceberg-overkill]
10. **Overall lean: HYBRID, leaning SWITCH-to-Hive-Parquet for the cold tier — CONDITIONAL on solving schema evolution with a per-OCSF-version table strategy.** The unification win + the neutralized time-travel rationale + the single-writer-append-only fit outweigh Iceberg's advantages FOR THIS WORKLOAD, PROVIDED Prism adopts a **per-`(source-class, schema-class, OCSF-version)` table/partition key** (which ADR-PROP and C5 SQ4 already proposed) so that within any one `ListingTable` the file schemas are homogeneous and the RETYPE problem never arises. That keying turns Hive-Parquet's worst axis into a non-issue and is the load-bearing condition. If Prism CANNOT commit to version-homogeneous tables, KEEP Iceberg.
11. **The answer FLIPS to KEEP-ICEBERG if any of:** (a) the cold tier becomes multi-writer-per-table; (b) row-level corrections/deletes become a real workload (GDPR/retention-redaction of specific records, not whole partitions); (c) true SQL `AS OF` snapshot time-travel becomes a hard requirement AND iceberg-datafusion surfaces it; (d) version-mixed files within one table become unavoidable (can't enforce version-homogeneous tables). [synthesis]
12. **Honest residue:** no published DataFusion-Rust head-to-head benchmark exists for IcebergTableProvider vs ListingTable on this workload [INCONCLUSIVE]; DataFusion's exact null-fill-vs-error behavior for explicit-table-schema-over-mixed-files is documented-by-inference, not by an authoritative statement [INCONCLUSIVE — `Handle merged schemas in parquet pruning` changelog confirms merge support exists but not the precise type-conflict semantics]; and iceberg-rust remains pre-1.0 with the version-string discrepancy noted above.

---

## What ADR-PROP claimed, and how each reason holds up

The committed taxonomy (`ADR-PROP-storage-engine-taxonomy.md` §Decision "Apache Iceberg — cold analytic tier", lines 92-106) gave FIVE reasons for Iceberg. Testing each:

| # | ADR-PROP stated reason (verbatim anchor) | Holds up under this pass? |
|---|------------------------------------------|----------------------------|
| R1 | "Columnar + partition-pruned: `event_time`/`eventDay` predicate pushdown cuts I/O... zstd-in-Parquet" | **NEUTRAL — applies equally to BOTH.** Hive-Parquet is *also* columnar zstd Parquet with eventDay partition pruning. This is not an Iceberg-specific advantage; it's a Parquet advantage both formats inherit. The Iceberg-specific increment is manifest/file-stat pruning (axis 5), which is finer but modest for time-bound queries. [DF-parquet-blog][Dremio-metadata] |
| R2 | "Schema evolution: OCSF is versioned; Iceberg schema-evolution absorbs OCSF version drift without migration" | **HOLDS — this is Iceberg's strongest and the decisive axis (axis 1).** But "without migration" is the load-bearing phrase: Hive-Parquet CAN match it IF Prism keys tables by OCSF version (then no in-table drift to absorb). The reason holds; the question is whether the version-keyed-table workaround is acceptable. [Iceberg-spec][Ctx7-iceberg-schema] |
| R3 | "Multi-schema tables keyed by `(source-class, schema, schema-version)`" | **HOLDS for Iceberg AND is exactly the Hive-Parquet escape hatch.** ADR-PROP already proposes version-keyed tables. Under Hive-Parquet, that key becomes the partition/table boundary that keeps each `ListingTable` schema-homogeneous. The keying ADR-PROP wrote FOR Iceberg is precisely what makes Hive-Parquet viable. [ADR-PROP §Decision][C5-SQ4] |
| R4 | "Unified cold-cache / Security Lake read path: Security Lake IS OCSF-as-Iceberg. ...same DataFusion + Iceberg TableProvider — one mechanism, not two" | **FALSE PREMISE (C5 already corrected this).** Security Lake is Hive-Parquet, NOT Iceberg. So Iceberg gives TWO providers; Hive-Parquet gives ONE. This reason, as written, actually argues FOR switching to Hive-Parquet — the "one mechanism" goal is achieved by Hive-Parquet, not Iceberg. This is the single most important reversal. [C5-#1][SecLake-store] |
| R5 | "Time-travel / snapshot: cold-tier replay for backtesting; model-state versioning for ML audit" | **WEAK — partly neutralized (axis 6).** (a) Append-only stores approximate "as of T" with `ingest_time <= T`; (b) iceberg-datafusion does NOT surface SQL `AS OF` today (API-level snapshot selection only), so the headline feature is unavailable in Prism's exact stack regardless of format. The backtest use is real but does not require Iceberg snapshots on an append-only store. [Onehouse-metadata][Spark-iceberg-tt] |

**Net:** of ADR-PROP's five reasons, R1 is format-neutral, R4 is a FALSE premise that now argues the other way, R5 is largely neutralized, and only R2/R3 (schema evolution) genuinely favor Iceberg — and even those are matchable by version-keyed Hive-Parquet tables.

---

## Axis 1 — Schema Evolution under OCSF version drift (THE DECIDING AXIS)

### Evidence
- **Iceberg:** table-level schema with stable integer **field IDs**; meaning is tied to ID not name/position, so add/rename/drop/reorder are safe; type evolution is constrained (numeric widening allowed; primitive->struct and struct->primitive FORBIDDEN); validation at write time guarantees each snapshot is schema-valid; old snapshots preserve historical schemas for point-in-time reads. [Iceberg-spec][Ctx7-iceberg-schema] Context7 confirms: *"Schemas in Iceberg can be evolved by promoting primitive types, or by adding, deleting, renaming, or reordering fields... a new schema version identified by a unique ID."* [Ctx7-iceberg-schema]
- **Hive-Parquet via DataFusion:** NO table-level schema authority — each Parquet file embeds its own schema; partition columns are path-derived, not in-file. [DF-ListingOptions] DataFusion 51/52 added an explicit `TableSchema` (separates `file_schema` from full `table_schema` + partition cols), and a changelog entry **"Handle merged schemas in parquet pruning"** confirms cross-file schema-merge handling exists in the engine. [Ctx7-df-tableschema][Ctx7-df-mergedschema] The deprecated `SchemaAdapter` (now `PhysicalExprAdapterFactory`) was the old reconciliation seam. [DF-schema-adapter]
- **Concrete OCSF 1.1+1.6-coexist-in-one-tree behaviors:**
  - **Added field (1.6 adds, 1.1 lacks):** with an explicit table schema covering the union, 1.1 files null-fill the new column. DataFusion already null-fills path-derived partition columns absent from files, so column-add-via-null-fill is the expected behavior — **confirmed-by-inference, not by an authoritative statement.** [DF-ListingOptions][INCONCLUSIVE on the exact authoritative null-fill guarantee]
  - **Deprecated/dropped field:** trivial in Hive-Parquet (stop writing it; old files still carry it; readers null-fill for new files). Iceberg marks deprecated then drops the field-ID; old snapshots retain it. Both handle this fine.
  - **RETYPED field (int->long OK-ish; string->struct / int->struct = HARD):** Databricks documents that Parquet directories with type-conflicting files FAIL schema unification ("timestamp vs int" cited) and must be corrected or split into separate tables. [DB-schema-mismatch] **Iceberg FORBIDS the same structural retypes** (primitive<->struct disallowed). [Iceberg-spec] So neither format makes an in-place string->struct retype safe; the difference is Iceberg gives a managed path (add new field-ID, deprecate old, both visible) whereas Hive-Parquet requires a per-version table split.

### LEAN
**Iceberg wins this axis, DECISIVELY on the bare comparison** — table-level field-ID authority is purpose-built for exactly this multi-year-drift problem and Hive-Parquet has no equivalent. **BUT the win COLLAPSES to a tie if Prism keys cold-tier tables by OCSF version** (`(source-class, schema-class, OCSF-version)`, which ADR-PROP R3 already proposes): version-homogeneous tables never contain mixed schemas, so the retype-conflict failure never arises and added/dropped fields are handled by table boundaries instead of in-table evolution. The decisive question is therefore NOT "which format evolves better" but **"can Prism commit to version-homogeneous tables?"** If yes -> tie, unification breaks the tie for Hive-Parquet. If no -> Iceberg wins decisively.

---

## Axis 2 — DataFusion read maturity in Rust TODAY

### Evidence
- **ListingTable (Hive-Parquet):** DataFusion's most-exercised path. Hive partition inference defaults ON (`infer_partitions_from_hive_compliant` config, Context7-confirmed). [Ctx7-df-config] Row-group + page-index + Bloom-filter pruning, projection + filter pushdown, file-level pruning, metadata caching via `ParquetFileReaderFactory`. [DF-parquet-blog][DF-custom-tp] Real issues/fixes around partitioned reads confirm broad usage. [DF-issue-1139]
- **iceberg-rust `IcebergTableProvider`:** pre-1.0 (0.9 family). The **2026-03-10 0.9.0 release post** confirms *significantly expanded* DataFusion integration: DDL-via-SQL, **limit pushdown**, predicate pushdown for Boolean/IsNaN/Timestamp/Binary/string-pattern, insert with sort-based clustering. [Ctx7-iceberg-090] Production-proven for low-latency (Cheetah: 60ms Iceberg queries via manifest/metadata LRU caching, ~100x warm-cache planning speedup). [Cheetah] Manifest-list + per-file column-stat pruning + partition transforms are core. [Iceberg-spec][Dremio-metadata]

### LEAN
**Hive-Parquet `ListingTable` wins on maturity/stability TODAY, but the 2026 gap is "mature-default vs capable-pre-1.0," not "mature vs experimental."** The iceberg-rust 0.9.0 (Mar 2026) DataFusion integration is real and improving fast. For a Rust-only, pre-1.0-averse codebase that already pins exact versions, the maturity edge is a genuine but DIMINISHING reason to prefer Hive-Parquet. Decisiveness: MODERATE (was strong before the 0.9.0 release; now moderate).

---

## Axis 3 — Write / append / compaction / retention (append-mostly, single-writer)

### Evidence
- **Hive-Parquet:** write = PUT a new Parquet object into `region=/accountId=/eventDay=/`. Object-store PUT is atomic (object invisible until complete). [object_store] Retention = delete partition directories (free DELETEs on S3). [S3-pricing] Compaction is NOT built-in — must be a custom Rust job (read small files, rewrite bin-packed, delete originals via DataFusion COPY + `object_store`). [DF-format-options][object_store] Small-file proliferation is the classic risk if writes are frequent/tiny. [Decube]
- **Iceberg:** append = catalog compare-and-swap commit referencing new data files. [Iceberg-acid] Built-in `rewrite_data_files` (bin-pack compaction) + `rewrite_manifests` + `expire_snapshots` + `remove_orphan_files`. [Iceberg-maintenance][Onehouse-metadata] iceberg-rust 0.9.0 supports partitioned-write insert with sort-clustering; compaction tooling maturity in Rust is less documented. [Ctx7-iceberg-090][INCONCLUSIVE on Rust-native compaction tooling completeness]

### LEAN
**Hive-Parquet wins for SINGLE-writer-per-table append-only** (the workload Prism designed — C5 SQ6) — the write path is trivially simple and atomic; Iceberg's commit/manifest machinery buys little. **Iceberg wins DECISIVELY for CONCURRENT writers** (OCC commit conflicts handled; C5 flagged iceberg-rust concurrent-commit behavior as undocumented anyway). The caveat: Hive-Parquet's small-file/compaction burden becomes Prism's own engineering — but for a cold, batch-written tier (hourly/daily flushes, not per-event), file sizes are controllable at write time and the burden is low. Decisiveness for single-writer: MODERATE lean Hive-Parquet.

---

## Axis 4 — Concurrency & consistency (readers during writes)

### Evidence
- **Iceberg:** ACID. Atomic CAS on the metadata pointer; readers pin a snapshot and NEVER see partial commits; OCC for writers (conflicting commit fails + retries); maintenance ops warned to coordinate with in-flight writes to avoid corruption. [Iceberg-acid][Iceberg-maintenance]
- **Hive-Parquet:** no table-level transaction. Reader discovers files at plan time via LIST; consistency = object-store semantics. Atomic object PUT prevents partial-FILE reads, but the directory-LISTING race remains (a reader mid-retention-delete may try to GET a just-deleted object; a reader may miss/see files appearing mid-scan). Mitigations: write-to-temp-then-atomic-PUT, `_SUCCESS` markers, deferred deletes. [object_store][HN-parquet-append] DataFusion errors if a planned file vanishes mid-query (standard behavior, not documented authoritatively). [INCONCLUSIVE on DataFusion's exact missing-file error path]

### LEAN
**Iceberg wins structurally, but the practical gap is SMALL for a cold, batch-written, append-only tier where writes are scheduled off heavy-read windows.** The realistic residual risk is the retention-delete-vs-active-read race, mitigable by deferring partition deletes behind a grace window (delete eventDay=N only after no active query references it) and by atomic PUT. Decisiveness: LOW-MODERATE lean Iceberg (would be DECISIVE under continuous-ingest-during-live-reads, which is not the cold-tier profile).

---

## Axis 5 — Pruning granularity & query cost

### Evidence
- **Iceberg 4-stage pruning:** snapshot resolve -> manifest-list prune (partition-summary stats) -> manifest-file prune (per-file column min/max, null counts) -> Parquet row-group/page prune. Dremio: *"By the time a query engine begins scanning actual Parquet files, Iceberg's metadata has already eliminated 90-99% of the files."* [Dremio-metadata] Critically, Iceberg **AVOIDS object-store LIST** — Onehouse: *"Iceberg was born to solve: scaling table metadata to avoid listing large tables."* [Onehouse-metadata] Partition transforms (day/hour/bucket/truncate) + table sort orders align files with predicates. [Iceberg-spec]
- **Hive-Parquet:** eventDay directory-prefix partition prune (coarse, partition-level) + Parquet row-group stats (tight on eventTime IF time-sorted, per Security Lake's sort requirement) + page-index + Bloom. [DF-parquet-blog][SecLake-store] Requires object-store LIST to discover files in matching partitions (LIST billed at PUT rate on S3). [S3-pricing] DataFusion can cache listings/metadata to amortize. [DF-parquet-blog]

### LEAN
**Iceberg wins, but MODESTLY for time-bound queries and MORE for high-cardinality equality predicates.** For `eventDay BETWEEN ...` + time-sorted row groups, Hive-Parquet already skips most data; the Iceberg increment is the LIST-avoidance (matters at multi-year/millions-of-files scale for latency + S3 request cost) plus finer per-file equality pruning (principal/resource fields) via bucket transforms. **No published DataFusion-Rust head-to-head benchmark exists** [INCONCLUSIVE on magnitude]. Decisiveness: MODERATE lean Iceberg, scaling with file count + equality-predicate selectivity. The mandatory time-bound (C3 Topic 4) blunts the worst case for both formats.

---

## Axis 6 — Time-travel / backtesting necessity

### Evidence
- **Iceberg snapshot semantics:** "table state as of commit time T" = all writes committed at/before T, INCLUDING late-arriving rows with earlier eventTime; EXCLUDING rows ingested after T. [Iceberg-spec][Onehouse-metadata] SQL `TIMESTAMP AS OF` / `VERSION AS OF` exist in Spark + Athena. [Spark-iceberg-tt][Athena-iceberg-tt][Ctx7-iceberg-tt]
- **Critical gap:** iceberg-datafusion surfaces snapshot selection at the **API level only** — NO SQL `AS OF` syntax in DataFusion's Iceberg integration today. [Onehouse-metadata + research synthesis][C5-SQ2-Open-Q] So Prism would manage snapshot IDs programmatically regardless of format.
- **Append-only Hive-Parquet approximation:** since data is NEVER mutated, "as of T" ~= `WHERE ingest_time <= T`. Equivalent for the common case; DIFFERS only on late-arriving/backfill/correction rows (Iceberg's snapshot excludes late rows by commit-time; ingest-time filter does the same IF an `ingest_time` column is recorded). Corrections that REWRITE rows break the append-only premise — but Prism routed mutations to Postgres, keeping the cold tier append-only.

### LEAN
**Hive-Parquet (ingest_time filtering) is SUFFICIENT for the cited backtest/model-audit use, given an append-only store.** Iceberg's snapshot time-travel is genuinely needed ONLY if row-level corrections/deletes enter the cold tier — which the architecture explicitly prevents. And the headline SQL `AS OF` is unavailable in iceberg-datafusion anyway. **This axis NEUTRALIZES ADR-PROP R5.** Decisiveness: LOW lean Iceberg (semantically cleaner) but the requirement is satisfiable by Hive-Parquet + an `ingest_time` column at near-zero cost. Recommend Prism record `ingest_time` per row regardless of format.

---

## Axis 7 — Unification value (one provider vs two)

### Evidence FOR unification (Hive-Parquet for both)
- C5 #2 established Security Lake = Hive-Parquet -> Iceberg cold tier = TWO providers (`IcebergTableProvider` + `ListingTable`); Hive-Parquet cold tier = ONE `ListingTable` for both. [C5-#2][SecLake-store]
- Quantified simplification: (a) DROP the pre-1.0 iceberg-rust + iceberg-datafusion dependency entirely (no 0.x API-churn maintenance, no catalog dependency); (b) ONE OCSF schema-on-read reconciliation code path; (c) ONE partition-projection (`region=/accountId=/eventDay=`) implementation; (d) ONE provider to `#[non_exhaustive]`-audit, security-review, and test; (e) ONE pushdown-descriptor binding (C3) instead of two exactness profiles.

### Counterweight — "plain-Parquet-lake regret" prior art
- Documented regret drivers when teams abandoned raw Parquet for Iceberg/Delta/Hudi: schema-evolution pain at scale, small-file/streaming proliferation, lack of multi-writer ACID, slow LIST at scale, no time-travel. [Dremio-scaling][Conduktor-migrate][Decube] **BUT** — the strong regret drivers (multi-writer ACID, frequent updates/deletes, streaming small-files) DO NOT fire for a single-writer, append-only, batch-written cold tier. The one regret driver that DOES apply is schema evolution (axis 1), addressed by version-keyed tables. No first-person named-company "regretted plain Parquet for THIS exact profile" retrospective was found [INCONCLUSIVE — sources are vendor blogs, not company post-mortems].
- **Reverse prior art (Iceberg as overkill):** multiple sources explicitly state Iceberg is overkill for single-writer/append-only/stable-schema/small-or-cold datasets — "just use Parquet"; Iceberg's snapshot maintenance + catalog is "added complexity without a corresponding payoff" for that profile; "adopt Iceberg later... migration path is natural since Iceberg sits on Parquet anyway." [Decube-overkill][Iceberg-overkill][PuppyGraph-iceberg-parquet]

### LEAN
**The one-provider win is real and meaningful, and the regret counterweight does NOT fire for Prism's workload profile** (single-writer, append-only, batch-written) EXCEPT on schema evolution — which the version-keyed-table strategy neutralizes. The reverse prior art (Iceberg-as-overkill) describes Prism's profile almost exactly. The natural "adopt Iceberg later" migration path (Iceberg sits on Parquet) means switching to Hive-Parquet now is NOT a one-way door. Decisiveness: STRONG lean Hive-Parquet, contingent on the version-keyed-table condition (axis 1) holding.

---

## DECISION MATRIX

| Axis | Iceberg | Hive-Parquet | Winner | Decisiveness | Note |
|------|---------|--------------|--------|--------------|------|
| 1. Schema evolution (OCSF drift) | field-ID authority, managed migration | no table authority; per-version tables needed | **Iceberg** (bare) / **TIE** (if version-keyed) | DECISIVE bare; NEUTRAL if version-keyed | The deciding axis; version-keyed tables collapse the gap |
| 2. DataFusion read maturity (Rust, 2026) | pre-1.0 0.9.0, capable+improving | mature default path | **Hive-Parquet** | MODERATE (was strong; gap closing) | iceberg-rust 0.9.0 (Mar 2026) narrowed it |
| 3. Write/append/retention (single-writer) | catalog-commit overhead | trivial atomic PUT + dir-delete | **Hive-Parquet** | MODERATE | Iceberg wins DECISIVELY if multi-writer |
| 4. Concurrency/consistency | ACID snapshot isolation | object-store semantics + mitigations | **Iceberg** | LOW-MODERATE | small gap for cold batch-written tier |
| 5. Pruning granularity / cost | 4-stage + LIST-avoidance | partition + row-group + Bloom | **Iceberg** | MODERATE | modest for time-bound; bigger for equality + at scale |
| 6. Time-travel / backtest | snapshot AS-OF (API-only in DF) | ingest_time filter (append-only) | **Iceberg** (semantics) | LOW | NEUTRALIZED: append-only + no SQL AS OF in iceberg-datafusion |
| 7. Unification (one provider) | TWO providers + pre-1.0 dep | ONE provider, drop iceberg-rust | **Hive-Parquet** | STRONG | the architectural payoff; regret counterweight doesn't fire for this profile |

**Tally (bare):** Iceberg leads axes 1,4,5,6; Hive-Parquet leads 2,3,7. But weighting by decisiveness AND by whether the advantage actually fires for THIS workload: axis 1 is neutralizable (version-keyed), axis 6 is neutralized, axis 4 is low-gap-for-cold, axis 5 is modest-for-time-bound — while axis 7 (Hive-Parquet) is strong-and-fires, axis 3 fires (single-writer), and axis 2 favors Hive-Parquet.

---

## OVERALL LEAN

**HYBRID, leaning SWITCH-TO-HIVE-PARQUET for Prism's self-managed cold tier — CONDITIONAL on adopting version-homogeneous tables.**

Rationale: ADR-PROP's strongest stated reason (R4 unification via "Security Lake IS Iceberg") is a FALSE premise that, once corrected, argues FOR Hive-Parquet. R5 (time-travel) is neutralized by append-only semantics + the absent SQL `AS OF` in iceberg-datafusion. R1 is format-neutral. Only R2/R3 (schema evolution) genuinely favor Iceberg — and they are neutralizable by keying cold-tier tables on `(source-class, schema-class, OCSF-version)`, which ADR-PROP ALREADY proposed. With schema evolution neutralized, the unification win (one provider, drop pre-1.0 iceberg-rust, one OCSF-on-read path) + single-writer-append-only fit + the documented "Iceberg-is-overkill-for-this-profile" prior art tip the balance to Hive-Parquet. And it is not a one-way door: Iceberg sits on Parquet, so a later adoption is a natural migration.

**The load-bearing CONDITION:** Prism must enforce **version-homogeneous tables** (one `ListingTable` never spans OCSF versions). If that cannot be guaranteed, the mixed-schema RETYPE problem resurfaces and **KEEP ICEBERG.**

---

## Conditions under which the answer FLIPS to KEEP-ICEBERG

1. **Cold tier becomes multi-writer-per-table** (axis 3/4 flip DECISIVE to Iceberg's ACID OCC).
2. **Row-level corrections/deletes enter the cold tier** — e.g., GDPR/privacy redaction of SPECIFIC records (not whole eventDay partitions), or supersession of corrected events. Breaks append-only; Iceberg merge-on-read/copy-on-write + snapshot history wins (axis 6 flips).
3. **True SQL `AS OF` snapshot time-travel becomes a hard requirement AND iceberg-datafusion surfaces it.** (Today it doesn't, so Iceberg can't deliver it in Prism's stack either — but if both change, Iceberg wins axis 6 decisively.)
4. **Version-homogeneous tables prove infeasible** (a single logical table must mix OCSF versions, e.g. mid-day version cutover within one eventDay partition with retype conflicts). Then axis 1 flips DECISIVE to Iceberg.
5. **Many-tiny-file ingest profile** (per-event or sub-minute flushes) without an owned compaction job — Iceberg's built-in compaction + the natural small-file handling becomes worth the table-format overhead (axis 3 flips).

---

## Open Design Questions

1. **Can Prism guarantee version-homogeneous cold-tier tables?** This is THE load-bearing question. Lean: yes — key tables/partitions by `(source-class, schema-class, OCSF-version)` per ADR-PROP R3 + C5 SQ4, with the OCSF-version as a partition-tree level or a table-name component. If a source bumps OCSF version, new data lands in a new version-keyed table; cross-version queries UNION the per-version tables (DataFusion handles the union; each leg is schema-homogeneous).
2. **Record `ingest_time` per row regardless of format** — required for the Hive-Parquet append-only time-travel approximation (axis 6). Cheap; do it either way.
3. **Compaction ownership under Hive-Parquet:** who runs the bin-pack job, on what cadence, with what grace window vs active reads? Lean: a prism-managed retention/compaction process (mirrors C5 SQ6's icepick-style cleanup lean, minus Iceberg). Cold batch writes keep file sizes controllable at write time, lowering compaction urgency.
4. **Retention-delete-vs-active-read race mitigation** (axis 4): defer eventDay partition deletes behind a grace window so no in-flight query references the partition; atomic PUT for writes. Confirm DataFusion's missing-file error path is acceptable [INCONCLUSIVE — verify].
5. **Does the C5 "two-providers-one-engine" reframe of the vision flagship sentence get superseded by "ONE provider, ONE format"?** If this lean is accepted, the vision sentence simplifies further than C5 proposed: not "one engine, two providers" but "one engine, one Hive-Parquet provider, two stores (cold tier + Security Lake)."
6. **Cross-format query during a migration window:** if some sources are mid-transition, can DataFusion UNION a per-version table set transparently behind the C3 descriptor? Lean: yes, modeled as one logical connector with per-version `[[tables]]` legs.
7. **Bucket/sort-order design for equality-predicate pruning (axis 5):** without Iceberg partition transforms, does Hive-Parquet need additional partition levels (e.g., a hash bucket on a hot equality field) or in-file sort orders to match Iceberg's equality pruning? Lean: time-sort within files (Security Lake already mandates this) + accept coarser equality pruning; add a bucket partition level only if a specific high-cardinality equality predicate dominates query cost.

## Honest Costs & Caveats

- **No published DataFusion-Rust head-to-head benchmark** (IcebergTableProvider vs ListingTable) on this workload — axis 5 magnitude is inferential. [INCONCLUSIVE]
- **DataFusion's exact null-fill-vs-error behavior** for an explicit table schema over mixed-schema files is documented-by-inference; the "Handle merged schemas in parquet pruning" changelog confirms merge support EXISTS but the precise type-conflict semantics are not authoritatively stated. [Ctx7-df-mergedschema][INCONCLUSIVE]
- **iceberg-rust is pre-1.0** with an unresolved version-string discrepancy (C5's 0.9.1/0.10.0 vs the site's 0.9.0 release post). The CAPABILITY set (DataFusion DDL, limit + predicate pushdown, sort-clustered insert) is confirmed as of the 2026-03-10 release post; the exact current patch is [INCONCLUSIVE]. Switching to Hive-Parquet SIDESTEPS this risk entirely (a genuine point in favor of switching).
- **Switching is reversible** (Iceberg sits on Parquet); KEEPING Iceberg and later finding it overkill is also reversible but carries the pre-1.0 maintenance tax in the interim.
- **The lean is workload-specific.** It rests on single-writer + append-only + batch-written + version-homogeneous-tables. Every flip condition above is a real way the workload could change; the architect must confirm the four load-bearing assumptions before committing.
- **This contradicts nothing in C5 or C3** — it EXTENDS C5's "Security Lake = Hive-Parquet, two-providers-one-engine" finding by asking whether the cold tier should ALSO be Hive-Parquet to collapse to one provider. It updates C5 only on the iceberg-rust currency point (0.9.0 / 2026-03-10), explicitly flagged.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 2 | (1) Axes 1-4: schema evolution under OCSF drift, DataFusion read maturity, write/append/compaction/retention, concurrency/consistency — Iceberg vs Hive-Parquet for a Rust/DataFusion append-only cold tier (`reasoning_effort=high`, 66KB, read in full). (2) Axes 5-7: pruning granularity/cost, time-travel/backtest necessity, unification one-vs-two-provider + regret/overkill prior art (`reasoning_effort=high`, 86KB; ~70% read directly covering axes 5/6 + start of 7; axis-7 counterweight cross-validated independently via WebSearch). |
| Perplexity perplexity_reason | 0 | — |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_ask | 0 | — |
| Context7 | 4 | `resolve-library-id` x2 (DataFusion -> `/apache/datafusion`; Iceberg -> `/apache/iceberg`, v1.10.1) + `query-docs` x2: (a) DataFusion ListingTable/TableSchema/partition-inference/schema-merge — CONFIRMED `TableSchema` (file vs table schema) in 51/52, `infer_partitions_from_hive_compliant` default-true, "Handle merged schemas in parquet pruning" changelog; (b) iceberg-rust DataFusion integration + time-travel + schema evolution — CONFIRMED the **2026-03-10 iceberg-rust 0.9.0 release** with expanded DataFusion integration (DDL, limit + predicate pushdown, sort-clustered insert), SQL `AS OF` shown for Spark NOT DataFusion, field-ID schema-evolution rules. LOAD-BEARING currency verification. |
| Tavily (all) | 0 | — |
| WebSearch | 2 | Cross-validate axis-7 counterweight independently (filled the unread ~30% of research file #2): (1) plain-Parquet-lake regret / migration-to-Iceberg drivers; (2) Iceberg-as-overkill for single-writer append-only — "just use Parquet." Both returned consistent, citable vendor/engineering sources. |
| WebFetch | 0 | — |
| Read | 5 | 3 non-contradiction context files (C5, ADR-PROP, C3) + 2 persisted research outputs. |
| Grep | 3 | Locate sections within the oversized single-line research JSON #2 (regret/migration/overkill/unification/conclusion). |
| Training data | ~1 area | The version-keyed-table-as-schema-evolution-workaround synthesis (combining ADR-PROP R3 + C5 SQ4 + DataFusion union behavior) is my own synthesis over cited inputs, flagged as synthesis not external prior art. |

**Total MCP tool calls:** 6 (2x `perplexity_research` high-effort + 4x Context7). Plus 2 WebSearch for independent axis-7 cross-validation.
**Training data reliance:** low — every axis verdict is web-sourced + citation-backed; the two load-bearing CURRENT-state facts (DataFusion `TableSchema`/merged-schema-pruning, iceberg-rust 0.9.0 2026-03-10 DataFusion integration) were verified via Context7 against the live Apache mirrors, NOT training data. The only synthesis (version-keyed-table strategy) is built on cited inputs and flagged.

### Citation key

**Apache Iceberg / iceberg-rust:**
- **[Iceberg-spec]** https://iceberg.apache.org/spec/ — field-ID schema evolution, type-promotion rules (primitive<->struct forbidden), partition transforms, manifests/metrics, snapshot model.
- **[Ctx7-iceberg-090]** Context7 /apache/iceberg, https://github.com/apache/iceberg/blob/main/site/docs/blog/posts/2026-03-10-iceberg-rust-0.9.0-release.md — iceberg-rust 0.9.0 (2026-03-10): expanded DataFusion integration (DDL via SQL, limit pushdown, predicate pushdown Boolean/IsNaN/Timestamp/Binary/string-pattern, sort-clustered partitioned insert). **Currency-verification, supersedes C5's version snapshot.**
- **[Ctx7-iceberg-schema]** Context7 /apache/iceberg spec.md — schema evolution via promote/add/delete/rename/reorder, unique schema-version IDs.
- **[Ctx7-iceberg-tt]** Context7 /apache/iceberg branching.md + spec-queries — `VERSION AS OF` / `TIMESTAMP AS OF` (shown for Spark).
- **[Iceberg-acid]** "How Iceberg ACID transactions work" (Tim Berglund), https://www.youtube.com/watch?v=AbdCpu3x31s — atomic CAS metadata pointer, snapshot isolation, OCC, write-time validation.
- **[Iceberg-maintenance]** https://iceberg.apache.org/docs/latest/maintenance/ — rewrite_data_files, rewrite_manifests, expire_snapshots, remove_orphan_files, corruption warning on short orphan-retention.
- **[Dremio-metadata]** https://www.dremio.com/blog (Iceberg metadata performance) — 4-stage pruning pipeline; "metadata eliminated 90-99% of files"; partition transforms + sort orders.
- **[Dremio-scaling]** https://www.dremio.com/blog/scaling-data-lakes-moving-from-raw-parquet-to-iceberg-lakehouses/ — raw-Parquet pain points (schema evolution, no ACID, slow listing, no time-travel) driving migration.
- **[Onehouse-metadata]** Onehouse blog (managing Iceberg metadata) — "Iceberg was born to solve: scaling table metadata to avoid listing large tables"; expire_snapshots procedure; metadata caching.
- **[Cheetah]** "Bringing Iceberg to low-latency workloads" talk, https://www.youtube.com/watch?v=FQl2HadBJWE — Cheetah: Arrow Flight + iceberg-rust, 60ms queries, manifest/metadata LRU cache ~100x warm planning.
- **[Conduktor-migrate]** https://www.conduktor.io/glossary/migrating-to-apache-iceberg-from-hive-or-parquet — migration drivers/guidance.

**DataFusion / Parquet / object_store:**
- **[Ctx7-df-tableschema]** Context7 /apache/datafusion upgrading/51.0.0.md + 52.0.0.md — `TableSchema` (file_schema vs table_schema vs partition_cols) abstraction.
- **[Ctx7-df-mergedschema]** Context7 /apache/datafusion changelog 8.0.0 — "Handle merged schemas in parquet pruning" (cross-file schema-merge support exists).
- **[Ctx7-df-config]** Context7 /apache/datafusion configs.md — `infer_partitions_from_hive_compliant` defaults true (Hive partition inference).
- **[DF-ListingOptions]** https://docs.rs/datafusion/latest/datafusion/datasource/listing/struct.ListingOptions.html — partition columns path-derived not in-file; explicit-schema vs inference.
- **[DF-schema-adapter]** https://docs.rs/datafusion/latest/datafusion/datasource/schema_adapter/index.html — SchemaAdapter deprecated -> PhysicalExprAdapterFactory.
- **[DF-parquet-blog]** https://datafusion.apache.org/blog/2025/08/15/external-parquet-indexes/ — file/row-group/page hierarchical pruning + metadata caching.
- **[DF-custom-tp]** https://datafusion.apache.org/library-user-guide/custom-table-providers.html — ListingTable as TableProvider exemplar; filter/projection pushdown.
- **[DF-format-options]** https://datafusion.apache.org/user-guide/sql/format_options.html — COPY/INSERT for Parquet (compaction-via-rewrite path).
- **[DF-issue-1139]** https://github.com/apache/datafusion/issues/1139 — partitioned read in listing table (Hive partitioning priority/maturity).
- **[object_store]** https://docs.rs/object_store — uniform ObjectStore trait; atomicity = backend semantics.
- **[S3-pricing]** AWS S3 pricing docs — LIST billed at PUT rate; DELETE free; GET billed.

**Hive-Parquet / table-format trade-offs / regret + overkill prior art:**
- **[DB-schema-mismatch]** https://kb.databricks.com/data-sources/schema-mismatch-issue-while-reading-parquet-files — Parquet directories with type-conflicting files fail schema unification; split into separate tables (timestamp-vs-int).
- **[HN-parquet-append]** Hacker News practitioner thread — "Parquet is built for append-only"; partition-by-week + hash-bucket layout.
- **[Decube]** https://www.decube.io/post/what-is-apache-iceberg-versus-parquet — Iceberg vs Parquet complementary layers; when Iceberg is worth it.
- **[Decube-overkill]** (same) — Iceberg overkill for single-writer/append-only/stable schema; "just use Parquet."
- **[Iceberg-overkill]** https://olake.io/blog/iceberg-vs-parquet-table-format-vs-file-format/ + https://www.phoenixdata.ai/glossary/apache-parquet-vs-apache-iceberg — Parquet ideal for read-heavy/stable-schema/static; Iceberg metadata overhead; adopt-later natural migration.
- **[PuppyGraph-iceberg-parquet]** https://www.puppygraph.com/blog/apache-iceberg-vs-parquet — file-format vs table-format distinction; concurrency/evolution as the Iceberg payoff.

**Amazon Security Lake (referenced from C5; format-defining for the unification axis):**
- **[SecLake-store]** AWS Security Lake docs — OCSF Parquet, `region=/accountId=/eventDay=` Hive layout, zstd, time-sorted records, Glue catalog (NOT Iceberg).

**Prism artifacts (read, not web):**
- **[ADR-PROP §Decision]** `.factory/specs/day2-design-decisions/ADR-PROP-storage-engine-taxonomy.md` — five-reason Iceberg cold-tier rationale (R1-R5) + `(source-class, schema, schema-version)` keying.
- **[C5-#1][C5-#2][C5-SQ2][C5-SQ4][C5-SQ6]** `.factory/research/siem-lake-federation-2026-06-27.md` — Security Lake = Hive-Parquet; two-providers-one-engine; iceberg-rust maturity; OCSF version skew; append-only write path.
- **[C3]** `.factory/research/capability-descriptor-pushdown-2026-06-26.md` — DataFusion Exact/Inexact/Unsupported pushdown contract (honored by both providers); mandatory time-bound (Topic 4).
- **[query-OCSF]** OCSF version-state (current GA 1.6.0; 1.4.0 ~12 deprecations) — established in C5 SQ4 (web-sourced there).
