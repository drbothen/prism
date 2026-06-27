---
document_type: research
produced_by: research-agent
status: capture
do_not_execute: true
timestamp: "2026-06-27"
provenance: "side-analysis discussion input (OUT-OF-BAND, SEPARATE from the live VSDD factory pipeline); does not modify vision/specs/STATE/SESSION-HANDOFF/ADR registry."
topic: "C5 — SIEM / Security-Lake federation (Amazon Security Lake flagship); Iceberg read/write path from Rust/DataFusion; SIEM-as-queryable-store; OCSF lingua-franca; cost/egress/residency guards"
feeds: "matured-vision-day2-requirements.md §3.3 addendum / §3.5 / §17.5 + storage taxonomy (ADR-PROP-storage-engine-taxonomy.md) — DISCUSSION input only"
engine: "DataFusion + Apache Iceberg (iceberg-rust); ephemeral/federated query thesis"
non_contradiction_read:
  - "research/capability-descriptor-pushdown-2026-06-26.md (C3 — owns descriptor/pushdown/join-guard/DataFusion-50.x mechanics; NOT re-derived here)"
  - "specs/day2-design-decisions/ADR-PROP-storage-engine-taxonomy.md (four-engine taxonomy; Iceberg cold tier; the 'one mechanism' claim at §Decision lines 102-104)"
  - "C4 dynamic-schema-connectors-2026-06-27.md (PARALLEL — owns generic connector-authoring/schema-discovery; NOT covered here)"
---

# C5 — SIEM / Security-Lake Federation (Amazon Security Lake Flagship)

**Side-analysis / discussion input — NOT a spec or vision change.** This document gathers cited prior art and offers *leans* on how prism federates against external SIEMs and security-lakes as queryable non-security Connector stores, with Amazon Security Lake as the flagship case. It tests the day-2 vision's load-bearing claim that *"Amazon Security Lake IS OCSF-as-Iceberg, so the cold-cache read path and the lake read path are the SAME DataFusion + Iceberg TableProvider — one mechanism, not two"* (ADR-PROP-storage-engine-taxonomy.md §Decision, lines 102-104). It does not modify the vision, any spec, STATE.md, SESSION-HANDOFF.md, the live ADR registry, or prior research. `do_not_execute: true`.

> **Scope fence vs C3/C4.** The pushdown/descriptor/exactness/join-guard/DataFusion-50.x mechanics are OWNED by C3 (`capability-descriptor-pushdown-2026-06-26.md`) and are *referenced*, not re-derived, here. Generic connector-authoring and schema-discovery mechanics are OWNED by the parallel C4 pass. This file covers only lake/SIEM-federation-SPECIFIC concerns: the lake/SIEM source shapes, the Iceberg read/write path reality, OCSF-as-lingua-franca, and lake-specific cost/egress/residency guards.

> **Read-coverage honesty.** As with the C3 pass, I could NOT directly read `matured-vision-day2-requirements.md` (not located on `develop` via the prior pass; the brief cites §3.3 addendum / §3.5 / §17.5). I DID directly read `ADR-PROP-storage-engine-taxonomy.md`, which restates the relevant §3.3-addendum content verbatim (Iceberg cold tier, the "one mechanism" claim, OCSF-versioned-tables, `event_time`/`eventDay` partition pruning). All §3.3/§3.5/§17.5 anchors below trace to the ADR-PROP capture + the brief's paraphrase, NOT to a direct vision-file read. A reviewer with the vision file open should reconcile the §-anchors.

---

## Executive Summary (~14 lines)

1. **The flagship claim has a load-bearing factual error that must be confronted: Amazon Security Lake is NOT Iceberg.** As of 2024–2026 AWS documentation, Security Lake stores **OCSF Parquet in Hive-style partitioned layout** (`region=/accountId=/eventDay=`), cataloged in **AWS Glue Data Catalog** and governed by **AWS Lake Formation**. AWS docs never mention Iceberg for Security Lake; Iceberg + the Glue Iceberg REST endpoint appear in the *separate* S3 Tables / CloudWatch-Logs-lakehouse context. [SecLake-store][SecLake-custom][S3Tables-CWL] The vision's "Security Lake IS OCSF-as-Iceberg" is therefore **incorrect on storage format** — but the *engine-level* "one mechanism" thesis can still hold (see #2).
2. **The "one DataFusion mechanism" thesis survives — but as "one ENGINE, two TableProviders," not "one TableProvider."** DataFusion reads self-managed Iceberg via `IcebergTableProvider`, and reads Security Lake's Hive Parquet via the *separate* `ListingTable`/Glue-Parquet provider. Same DataFusion optimizer, expression system, and execution engine; **two distinct TableProvider scan implementations.** [DF-iceberg][SecLake-store] The unification is real at the query-engine layer; it is NOT a single code path at the storage-provider layer.
3. **iceberg-rust is pre-1.0 (crates.io ~0.9.1/0.10.0 family as of late 2025–2026) but its READ path is production-usable.** It implements a DataFusion `IcebergTableProvider` with `scan(projection, filters, limit)` + `supports_filters_pushdown`, static predicate pushdown, partition pruning, and column-statistics pruning — confirmed both by Context7's public-api dump and by the `icepick` CLI running it against AWS S3 Tables + Cloudflare R2 in production-like settings. [Ctx7-DF][IRust-icepick][IRust-468]
4. **Direct AWS Glue catalog access from iceberg-rust is the weakest link for the Security Lake path.** The `apache/iceberg-rust` Glue catalog exists but is noted "not all catalog implementations are fully complete"; community guidance is to front Glue with a REST-catalog shim rather than a built-in Glue client. [Ctx7-cat][IRust-2186] For Security Lake specifically this is moot if prism reads via **data-access subscriber (raw S3 Parquet)** + its own partition projection — sidestepping Glue/Iceberg entirely (#6).
5. **Two distinct Rust Iceberg stacks exist — do not conflate.** `apache/iceberg-rust` (the `iceberg` + `iceberg-datafusion` crates) is the ASF project; `JanKaul/iceberg-rust` (`iceberg-rust`/`datafusion_iceberg`) is an unofficial "batteries-included" stack advertising direct Glue catalog, DataFusion integration, and equality-deletes. The architect must pick ONE lineage. [IRust-crates][IRust-jankaul]
6. **Every SIEM that is NOT a security-lake is a PUSHDOWN-QUERY-API source, never a bulk object-store read.** Splunk (SPL/REST + explicitly-private SmartStore buckets), Elastic/OpenSearch (ES|QL/SQL/PPL + private searchable-snapshot format), Sentinel (KQL/Azure Monitor), Google SecOps/Chronicle (UDM search) all expose rich query APIs but their object-store backings are proprietary and vendor-documented as off-limits to external engines. [Splunk-rest][Splunk-smartstore][Elastic-esql][Elastic-frozen][OS-sqlppl][Sentinel-lake][GSO-udm] Only **Snowflake/Databricks** (general-purpose lakes) realistically support BOTH pushdown SQL AND bulk Iceberg/Delta read. [Snowflake-jdbc][Databricks-sqlexec]
7. **Time-range + equality predicates push down on EVERY surveyed platform.** This is the universal floor and validates C3's mandatory-time-bound generalization as the primary cost control across the lake/SIEM federation surface. [Splunk-rest][Elastic-esql][Sentinel-lake][GSO-udm][Snowflake-jdbc]
8. **OCSF is the federation lingua franca only for the OCSF-native lakes (Security Lake, the emerging Sentinel data lake).** Other SIEMs are NOT OCSF — they are SPL/KQL/UDM-native. Reconciliation is prism's per-(table, schema-class) descriptor problem (C3 Topic 5): an OCSF predicate pushes down ONLY where it maps to a concrete native field on the SIEM. [SecLake-ocsf][Sentinel-lake]
9. **OCSF version skew is real and BITES at the minor-version level.** Current OCSF GA is **1.6.0** (Aug 2025); Security Lake custom sources are pinned to **≤1.3** and native AWS sources annotate `metadata.version = 1.1.0`. OCSF minor bumps are NOT guaranteed backward-compatible — 1.4.0 alone carried ~12 deprecations. A query expressed against 1.5/1.6 field shapes can mis-bind against 1.1-stamped Security Lake data. [OCSF-repo][OCSF-14][SecLake-ocsf]
10. **The mandatory time-bound (C3 Topic 4) is the single most important cost control on a multi-year Iceberg lake or a Security Lake S3 scan** — it maps directly onto Iceberg `days()`/`hours()` partition transforms and Security Lake's `eventDay` partition, pruning files before any S3 GET. Without it, a query can scan years of Parquet at S3-GET + Athena-scan cost. [SecLake-store][IRust-icepick][C3-§4]
11. **Residency-by-construction (C2 satellite-mesh decision; matured-vision §17.5) constrains lake federation hard:** Security Lake is per-Region, subscribers see only their selected Region(s), and cross-Region rollup is an explicit opt-in. Cross-region egress + Lake-Formation/RAM cross-account grants are the residency seam. A residency-bound prism deployment must refuse to query a lake in a disallowed region/tenant *at plan time*, before any S3 GET. [SecLake-subs][SecLake-xacct][C2-residency]
12. **The WRITE-path symmetry claim (§3.3 `RETAIN <dur>` writes Iceberg, `FROM cache.<name>` reads it) is REALISTIC TODAY for append-only, with a documented maturity ceiling.** iceberg-rust's writer appends data files + partitioned writes + transactional catalog commits (proven by icepick's commit/compact/snapshot-cleanup). It does NOT yet support row-level mutation (UPDATE/DELETE/MERGE via copy-on-write/merge-on-read). For an append-mostly cold cache this is the right shape; for any path needing row updates it is a maturity risk. [IRust-writer][IRust-450][IRust-icepick]
13. **Read-and-write against the SAME Iceberg stack is realistic** — the same `iceberg` crate provides both the scan builder and the writer module, and DataFusion maps `INSERT INTO` to append via `IcebergTableProvider::insert_into`. The shared-stack risk is concentrated in (a) pre-1.0 API churn and (b) commit-conflict handling under concurrent writers, which the sources do not document for iceberg-rust. [Ctx7-DF][IRust-writer]
14. **Honest bottom line:** the vision's *engine* unification (one DataFusion) is sound and buildable; the *storage-format* claim (Security Lake = Iceberg) is wrong and should be reworded to "Security Lake = OCSF Parquet read by the same DataFusion engine via a Glue/Hive-Parquet TableProvider, distinct from the self-managed Iceberg TableProvider." Pre-1.0 iceberg-rust + the Glue-catalog gap + append-only write ceiling are the three real maturity risks.

---

## Sub-Question 1 — Amazon Security Lake as a Federated Read Source

### Prior art (verified)

**Storage shape.** Security Lake centralizes security telemetry as **OCSF-normalized Apache Parquet on Amazon S3** (one bucket per Region in the customer account), cataloged in **AWS Glue Data Catalog** and governed by **AWS Lake Formation**. For natively supported AWS services Security Lake performs the OCSF conversion itself; for custom sources the provider must convert to OCSF before ingest. [SecLake-store][SecLake-custom]

**The OCSF table-per-class model.** AWS does not use the literal phrase "table-per-class," but it maps each Security Lake source to a specific OCSF event class (DNS Activity, SSH Activity, Authentication, findings, etc.) and creates a Glue/Lake-Formation table + a Glue crawler per custom source. In practice this yields a de-facto **table-per-(source, OCSF event class)** model queryable by class. [SecLake-store][SecLake-custom]

**Partition layout (confirmed for custom sources).** The canonical S3 path is:
`/ext/<custom-source-name>/region=<region>/accountId=<accountID>/eventDay=<YYYYMMDD>`
Partition key names are `region`, `accountId`, `eventDay`. `eventDay` is an 8-char `YYYYMMDD` string (AWS docs contain a self-contradictory "truncated to hour" note, but the example paths and format are day-granularity; **no `eventHour` partition key appears in published examples**). Native-AWS-source path layouts are not documented in equal detail but third-party integrators (Zscaler) confirm the same `region/accountId/eventDay` shape. [SecLake-custom][Zscaler-seclake]

**Subscriber access models (the load-bearing distinction).** Two `accessTypes` on `CreateSubscriber`:
- **`S3` (data access).** Subscriber reads raw Parquet objects directly from S3; notified of new objects via an HTTPS endpoint OR by polling an SQS queue. The consumer does its own schema handling. Compatible with **any engine that can read Parquet from S3 with IAM creds — including a Rust/DataFusion engine.** [SecLake-subs]
- **`LAKEFORMATION` (query access).** Subscriber queries the Glue/Lake-Formation tables via Athena (primary), Redshift Spectrum, or Spark SQL (Glue-metastore-integrated). Lake Formation enforces table/column grants. [SecLake-subs][SecLake-query]

**Cross-account / auth paths.** Cross-account sharing is via **AWS Lake Formation grants + AWS Resource Access Manager (RAM)** — either LF-tag-based access control (LF-TBAC) or named-resource grants. Same-Org shares are immediate; cross-Org shares require RAM invitation acceptance. Integrated engines (Athena/Redshift) need **resource links** to query shared tables. Subscribers are Region-scoped (see only the selected Region; rollup Region aggregates) and capped at 10 sources each. Security Lake auto-creates/exchanges subscriber credentials. [SecLake-xacct][SecLake-subs]

**How external engines read it.** Athena/Redshift Spectrum/Spark via Glue + Lake Formation. Trino/Presto via Glue-metastore + Hive/Parquet connector. **DataFusion (or any Rust engine) is not named by AWS, but the data-access (`S3`) model makes it feasible: read Parquet from S3 with IAM creds, infer partitions from the `region/accountId/eventDay` path, apply OCSF field semantics.** [SecLake-store][SecLake-query]

**OCSF version state (2025–2026).** AWS docs explicitly cap custom sources at **OCSF 1.3 and earlier**; native AWS sources annotate `metadata.version = 1.1.0` in mappings. No AWS material confirms Security Lake adopting OCSF 1.5/1.6 across the board. (Current upstream OCSF GA is 1.6.0 — see SQ4.) [SecLake-custom][SecLake-ocsf]

**Iceberg / S3 Tables question (CRITICAL).** **Security Lake is NOT exposed as Iceberg tables.** AWS docs consistently describe Parquet + Hive-style partitioning + Glue/Lake-Formation, and never mention Iceberg or the Glue Iceberg REST endpoint in the Security Lake context. Iceberg appears only in the *separate* S3 Tables service (e.g., CloudWatch Logs → S3 Tables lakehouse uses the Apache Iceberg format + Glue Iceberg REST catalog). No evidence (incl. re:Invent 2024 recap, which highlighted the OpenSearch zero-ETL integration) of Security Lake migrating to Iceberg or integrating S3 Tables. [SecLake-store][S3Tables-CWL][SecLake-reinvent]

### Discussion LEAN

- **Reword the vision's flagship sentence.** "Security Lake IS OCSF-as-Iceberg" is factually wrong on format. The accurate, still-powerful framing: *"Security Lake is OCSF Parquet in Glue/Lake-Formation; prism reads it with the same DataFusion engine it uses for its own Iceberg cold tier, via a distinct Hive-Parquet/Glue TableProvider. One engine, two providers."* This preserves the architectural win (no second query engine) without the false storage equivalence.
- **Prefer the `S3` data-access subscriber path for prism.** It sidesteps the iceberg-rust Glue-catalog gap (SQ2) entirely: prism reads raw OCSF Parquet from S3 via DataFusion's `ListingTable`-style Parquet provider with partition projection on `region/accountId/eventDay`. The `LAKEFORMATION` path adds an Athena/Glue dependency and Lake-Formation grant plumbing that the data-access path avoids — at the cost of doing partition pruning + OCSF interpretation in prism rather than in Athena. For an ephemeral federated engine that already owns its OCSF normalization boundary, doing it in prism is the dogfood-consistent choice.
- **The `eventDay` partition is the cost guard's anchor** (SQ5): a time-bound query prunes to a handful of `eventDay=` prefixes before any S3 GET.

### Open Qs (SQ1)
- Data-access (`S3`) vs query-access (`LAKEFORMATION`) as prism's default Security Lake binding? Lean: **`S3` data-access** for the ephemeral/dogfood reasons above; offer `LAKEFORMATION` as an opt-in for deployments that mandate Lake-Formation-governed access.
- Does prism's connector descriptor (C3) model Security Lake as ONE table-per-OCSF-class connector, or one connector exposing many class-tables? Lean: one connector, many `[[tables]]` blocks (one per OCSF class prism cares about), each with its own pushdown descriptor — consistent with C3 Topic 5 per-(table, schema-class).

---

## Sub-Question 2 — Iceberg Read Path from Rust / DataFusion (LOAD-BEARING)

### Prior art (verified against crates.io + GitHub + Context7)

**Version state.** The ASF `apache/iceberg-rust` project publishes the `iceberg` crate (+ `iceberg-datafusion`) on crates.io at the **0.9.1 / 0.10.0 family as of late 2025–2026 — i.e., pre-1.0 / 0.x.** Context7's mirror snapshot tags v0.7.0, but the live crates.io releases (0.9.1, 0.10.0) are ahead; treat it as actively-evolving pre-1.0. [IRust-crates][IRust-icepick]

**DataFusion TableProvider (CONFIRMED).** `iceberg-datafusion` provides `IcebergTableProvider` implementing the DataFusion `TableProvider` trait. Context7's public-api dump confirms the exact methods: `scan(state, projection, filters, limit)`, `supports_filters_pushdown(&[&Expr]) -> Vec<TableProviderFilterPushDown>`, `schema()`, `insert_into(...)`, `table_type()`. This is the *same* `supports_filters_pushdown` Exact/Inexact/Unsupported contract C3 documented for DataFusion 50.x. [Ctx7-DF][DF-iceberg]

**Catalog support.** iceberg-rust's catalog enum covers **`Rest`, `Glue`, `Memory`, `HMS` (HiveMetaStore), `S3Tables`, `SQL`** — with the explicit caveat *"not all catalog implementations are fully complete."* [Ctx7-cat]
- **REST catalog: implemented** (ships demo docker-compose test fixtures; community moving token-endpoint → OAuth2). [IRust-468]
- **S3 Tables (Iceberg REST endpoint): implemented and exercised in production-like use** by the `icepick` CLI against AWS S3 Tables + Cloudflare R2. [IRust-icepick]
- **AWS Glue Data Catalog: present but the WEAK LINK.** A Glue `CatalogBuilder` exists (Context7 shows a Glue+OpenDAL-S3 example), but community guidance (issue #2186) is that there is **no fully-featured built-in direct Glue client**; the recommended pattern is to **front Glue with a service exposing the Iceberg REST interface** rather than talk Glue natively. [Ctx7-cat][IRust-2186]
- SQL/JDBC catalog: `iceberg-sql-catalog` crate (Postgres/MySQL backends). [IRust-sqlcat]

**Predicate / partition pushdown (CONFIRMED).** The iceberg-rust reader implements **static predicate pushdown**: a scan-builder applies filters at plan time using manifest + partition metadata + column statistics to discard irrelevant files. A DataFusion runtime-filter issue notes "the Trino model fits iceberg-rust's reader cleanly because the reader already has the static-predicate plumbing." icepick exercises "partition pruning and column statistics" / "pruning stats with filters" in production. Partition transforms (identity, bucket, truncate, `days()`, `hours()`) map timestamps to partition keys for pruning — **so a time-bound query DOES prune on an `event_time`/`days()`/`hours()` partition.** [IRust-468][IRust-icepick][DF-runtimefilter]

**Schema evolution + time travel (read).** The Iceberg spec provides schema evolution (add/rename/drop via stable field IDs) and time travel (read snapshot-as-of commit-id/timestamp); iceberg-rust exposes snapshot selection at the API level. **Caveat: DataFusion-level SQL surfacing of time-travel/`AS OF` is not fully wired** — the `TableProvider` needs knobs to select a snapshot, and DataFusion needs to translate `AS OF` syntax; this is identified as not-yet-complete. Schema-evolution read is a core spec capability the reader honors via field IDs. [Iceberg-spec][IRust-timetravel][IRust-468]

**Two-stack caveat.** A SECOND, unofficial Rust Iceberg stack exists: `JanKaul/iceberg-rust` (crate `iceberg-rust` + `datafusion_iceberg`), advertising a more "batteries-included" experience — **direct Glue catalog, DataFusion integration, equality-deletes** — distinct from the ASF `apache/iceberg-rust`. The architect must consciously choose ONE lineage. [IRust-jankaul][IRust-crates]

### Discussion LEAN

- **The "one mechanism" (engine-level) claim is BUILDABLE.** DataFusion + `IcebergTableProvider` gives prism a real Iceberg read path with the same Exact/Inexact/Unsupported pushdown contract C3 already designed prism's descriptor around. The self-managed cold tier reads through `IcebergTableProvider`; Security Lake reads through a *separate* Glue/Hive-Parquet provider — **same engine, two providers** (SQ1 lean).
- **Pin to the ASF `apache/iceberg-rust` lineage** as the default, accepting the Glue-catalog gap, UNLESS the Security Lake `LAKEFORMATION` path (which needs Glue) becomes mandatory — in which case the JanKaul stack's direct-Glue support OR a REST-shim-over-Glue becomes load-bearing. Lean: ASF lineage + REST-shim-over-Glue if Glue is needed; revisit JanKaul only if equality-deletes/direct-Glue become hard requirements. This is a real architect decision, not mechanical.
- **Pre-1.0 API churn is a genuine cost** — pin an exact iceberg-rust version, and budget for breaking changes across 0.x bumps. Flag as a maturity risk, not a blocker (icepick proves the core read/write paths work in production-like settings).

### Open Qs (SQ2)
- ASF `apache/iceberg-rust` vs `JanKaul/iceberg-rust` lineage — which, and why? (Glue support, equality-deletes, maintenance cadence, license, `#[non_exhaustive]`-compatibility with prism's pub-type discipline.)
- For the Security Lake Glue path specifically: REST-shim-over-Glue, native Glue client, or skip Glue via the `S3` data-access subscriber (SQ1 lean)? Lean: skip Glue.
- Does prism need DataFusion-surfaced time-travel (`AS OF`) on the cold tier for backtesting (ADR-PROP cites time-travel for backtesting/model-state audit), and is the not-yet-complete DataFusion `AS OF` wiring a gap prism must fill itself? [INCONCLUSIVE — API-level snapshot selection exists; SQL-level surfacing is incomplete.]

---

## Sub-Question 3 — SIEM-as-a-Queryable-Store Federation (Beyond Security Lake)

### Prior art (verified, per platform)

| Platform | Query/access API (2025–2026) | Realistic integration | Pushdown predicate classes | Bulk read of its object store? |
|---|---|---|---|---|
| **Splunk** | REST `services/search/jobs` + `/export`, SPL; XML/JSON/CSV output [Splunk-rest] | **Pushdown only.** Translate prism predicates → SPL. | time-range (SPL `earliest`/`latest`, *recommended* to avoid alltime), equality, IN, range, full-text | **NO.** SmartStore S3 buckets are proprietary index format; docs explicitly say do not share with other tools. "Federated Search for Amazon S3" is Splunk-as-consumer-of-S3, not Splunk-as-readable-store. [Splunk-smartstore][Splunk-fss3] |
| **Elastic** | ES\|QL `_query`; SQL REST + JDBC [Elastic-esql][Elastic-sql][Elastic-sqlrest] | **Pushdown only.** | time-range, equality, IN (`terms`), range, full-text (`match`) → DSL | **NO.** Searchable-snapshots / frozen-tier on S3 are Elastic's internal snapshot format, not Parquet; not for external reads. [Elastic-frozen] |
| **OpenSearch** | SQL `_sql` + PPL `_ppl` (+ `_explain` shows DSL) [OS-sqlppl][OS-ppl] | **Pushdown only.** | time-range→`range`, equality→`term`, IN→`terms`, full-text→`match` | **NO.** Snapshot repos are OpenSearch-internal. |
| **Microsoft Sentinel** | KQL over Azure Monitor Query API (Log Analytics); the 2025 **Sentinel data lake** on ADLS, ASIM + emerging OCSF [Sentinel-lake][Sentinel-blog][Sentinel-soc] | **Primarily pushdown (KQL).** Data-lake bulk read **plausible but UNCONFIRMED** for third-party engines. | time-range, equality, IN, range, pattern (`has`/`contains`/regex), full-text (`search`) | **UNCONFIRMED.** "Open-format" ADLS lake messaging hints at Parquet/Delta but no documented external OCSF/Parquet export endpoint. [Sentinel-lake][Sentinel-blog] |
| **Google SecOps / Chronicle** | UDM search (absolute/relative time-range, result-limit, sampling, UDM-field aggregation) [GSO-udm] | **Pushdown (UDM search).** BigQuery export **widely assumed but UNCONFIRMED** in cited docs. | time-range (explicit), equality (UDM fields), aggregation/group-by; IN + full-text likely but not doc-confirmed | **UNCONFIRMED.** BigQuery backing assumed; no documented external table export in the cited UDM search docs. [GSO-udm] |
| **Snowflake** | SQL via JDBC/ODBC/REST; "AI Data Cloud for Cybersecurity" [Snowflake-jdbc][Snowflake-cyber] | **Pushdown SQL AND bulk read** (external tables / Iceberg). | time-range, equality, IN, range, basic text | **YES (conditional).** External + Iceberg tables readable by other engines if governance permits. |
| **Databricks** | SQL Statement Execution API 2.0 on SQL warehouses; Delta + Unity Catalog [Databricks-sqlexec] | **Pushdown SQL AND bulk read** (Delta/Unity, Delta Sharing). | time-range, equality, IN, range, text; file-level pushdown when reading Delta directly | **YES (conditional).** Delta tables on S3/ADLS/GCS readable by Spark/Trino/Flink with creds. |

**Universal floor:** time-range + equality push down on every platform. [all-above]

### Mapping onto C3's capability-descriptor model

Each SIEM/lake becomes a C3 connector with a per-(table, schema-class) pushdown descriptor (C3 Topic 1/5). Concretely:

- **Splunk/Elastic/OpenSearch/Sentinel/SecOps (pushdown-API sources):** descriptor declares `pushdown_target = source`; predicate classes that the native query language evaluates are `exact` (time-range, equality, IN, full-text where the language supports it); aggregation/group-by *may* be `exact` IF prism's adapter issues a native aggregate query (DataFusion's TableProvider won't negotiate aggregation per C3 #4 — the adapter must emit the native aggregate and present pre-reduced rows). **Join is ALWAYS central** (no SIEM joins across to prism's other sources) — exactly C3 Topic 3's cross-source join guard. The full SPL/KQL/UDM result, post-pushdown, is fetched and any residual re-checked centrally.
- **Security Lake / self-managed Iceberg / Snowflake-external / Databricks-Delta (bulk-read sources):** descriptor declares the Iceberg/Parquet partition + column-stat pushdown profile; time-range pushes as a partition prune (`exact`), equality/IN on stats-bearing columns push as file-prune (often `inexact` → C3's central re-check). These are the cases where prism's DataFusion engine does the scan itself.
- **What is ALWAYS central:** cross-source joins, OCSF-derived/computed predicates with no native field (C3 Topic 5), full-text on lakes that lack a search index, and any predicate class the descriptor leaves undeclared (C3 fail-closed default).

### Discussion LEAN

- **Two adapter archetypes, not N bespoke ones.** (1) **Pushdown-API adapter** (Splunk/Elastic/OpenSearch/Sentinel/SecOps): translate prism's pushed predicate set → native query string, fetch results, re-check residuals centrally. (2) **Object-store/lake adapter** (Security Lake/Iceberg/Snowflake-external/Databricks-Delta): DataFusion scans Parquet/Iceberg/Delta directly with partition + stat pruning. The pushdown-API archetype is "fetch-then-filter-residual"; the lake archetype is "prune-then-scan." Both fit C3's descriptor model; they differ only in `pushdown_target` and which predicate classes are `exact`.
- **Do NOT attempt to read SIEM proprietary object stores** (SmartStore, Elastic frozen snapshots, Sentinel ADLS without a confirmed export). Vendor docs say off-limits; doing so is fragile/unsupported. The lake-read archetype applies ONLY to OCSF/Parquet/Iceberg/Delta lakes with documented external read.
- **OCSF normalization at the adapter boundary holds** (prism's existing OCSF-normalization rule, CLAUDE.md): the pushdown-API adapter normalizes SPL/KQL/UDM result rows to OCSF *after* fetch; the lake adapter reads OCSF (Security Lake) or schema-on-read native (Snowflake/Databricks) and normalizes per C3 Topic 5.

### Open Qs (SQ3)
- Does prism build the pushdown-API adapter for all five SIEM languages (SPL/ES\|QL/SQL/KQL/UDM) as spec-driven TOML translators, or as code-connectors? (C4 owns connector-authoring mechanics — defer the *how*; the *which-predicates-push* is captured above.)
- For Sentinel/SecOps lake bulk-read: do we wait for a confirmed OCSF/Parquet export, or treat them as pushdown-only until then? Lean: **pushdown-only until a vendor-documented external lake read exists** — do not bet on undocumented ADLS/BigQuery formats.

---

## Sub-Question 4 — OCSF as the Federation Lingua Franca

### Prior art (verified)

- **OCSF-native lakes:** Security Lake is OCSF-native (it converts to OCSF on ingest). The emerging **Microsoft Sentinel data lake** advertises ASIM + emerging OCSF support. These are the only surveyed sources where the on-disk schema IS OCSF. [SecLake-ocsf][Sentinel-soc]
- **Non-OCSF SIEMs:** Splunk (SPL/sourcetypes), Elastic/OpenSearch (ECS-ish/custom mappings), Sentinel Log Analytics (ASIM), SecOps (UDM). Reconciling an OCSF-expressed query against these requires a schema mapping layer — exactly prism's OCSF-normalization boundary. [Splunk-rest][Sentinel-soc][GSO-udm]
- **OCSF version state.** Current upstream GA is **OCSF 1.6.0 (Aug 1, 2025)**; milestones: 1.0.0 (Sep 2023), 1.3.0 (Aug 2024), 1.4.0 (Jan 2025, ~140 net changes + ~12 deprecations), 1.5.0 (2025), 1.6.0 (Aug 2025). OCSF uses semantic versioning but **minor bumps are NOT guaranteed field-level backward-compatible** — 1.4.0's deprecations are the proof. Security Lake custom sources are pinned ≤1.3; native sources annotate `metadata.version = 1.1.0`. [OCSF-repo][OCSF-14][SecLake-ocsf]

### Where OCSF version skew bites

1. **Security Lake data is 1.1/1.3-stamped; a prism query authored against 1.5/1.6 field shapes can mis-bind.** New fields added in 1.4–1.6 do not exist in 1.1 payloads (nulls/failures if relied upon); fields deprecated in 1.4+ still appear in 1.1 data (strict 1.6 validators may reject them).
2. **Cross-source OCSF queries hit heterogeneous versions** — a query joining Security Lake (1.1) with a self-managed Iceberg cold tier written at a different OCSF version sees field skew at the join/projection.
3. **The C3 per-(table, schema-class) descriptor must be per-(table, schema-class, schema-VERSION).** This is the SQ4 amendment to C3 Topic 5: OCSF version is a third axis. The ADR-PROP taxonomy already anticipates this — it keys Iceberg cold-tier tables by `(source-class, schema, schema-version)` (ADR-PROP §Decision, lines 104-106).

### Discussion LEAN

- **Pin a target OCSF version per prism deployment + carry `metadata.version` through.** Stable, present-since-1.0 fields (`class_uid`, `class_name`, `time`, basic identity) are the safe pushdown anchors across versions; version-specific fields push down only when the source's stamped version matches. Map older inbound (1.1 Security Lake) → prism's target version at the normalization boundary, OR keep version-aware parsers and route by `metadata.version`. [OCSF-repo]
- **The descriptor's exactness flag (C3) must account for version skew:** an OCSF predicate on a field that exists in the query's target version but NOT in the source's stamped version is `unsupported` (no native field to push to) → central, returns null/empty appropriately, and the agent-harness must be TOLD (ties C3 Topic 4's disclosure lean). An OCSF predicate on a field present in both but with changed semantics across versions is `inexact` → push widened, re-check centrally (the C3 transform-exactness rule, now also a version-skew rule).
- **Iceberg schema-evolution is the cold-tier mitigation** (ADR-PROP cites it): writing OCSF-vN data into an Iceberg table whose schema evolved across OCSF versions lets stable field-IDs absorb additive drift — but renames/deprecations still need mapping. This is a real benefit of the Iceberg cold tier for OCSF specifically, and a reason the lake (not RocksDB) holds long-baseline OCSF.

### Open Qs (SQ4)
- What is prism's target OCSF version, and does it track upstream (1.6) or AWS-Security-Lake's lag (1.1/1.3)? Lean: target the highest version prism's own cold tier writes, normalize all inbound (incl. 1.1 Security Lake) up to it at the boundary, carry `metadata.version` for audit/disclosure.
- Does the descriptor's schema-class key formally add an OCSF-version axis? Lean: yes — per-(table, schema-class, schema-version), matching ADR-PROP's `(source-class, schema, schema-version)` Iceberg table key.

---

## Sub-Question 5 — Cost / Egress / Residency Guards (Lake + SIEM Specific)

### Prior art (verified)

- **Security Lake cost surface:** S3 GET per object + (query-access path) Athena/Redshift scan cost billed by bytes scanned. Hive partition pruning on `region/accountId/eventDay` + Parquet column projection are the documented cost levers. Object-size + time-sorted-records requirements exist precisely "to optimize Security Lake for query performance." [SecLake-store][SecLake-custom]
- **Iceberg cost lever:** partition transforms (`days()`/`hours()`) + manifest/column-stat pruning skip files before read; icepick's "pruning stats with filters" is the proof in Rust. [IRust-icepick][Iceberg-spec]
- **Residency surface (Security Lake):** per-Region buckets; subscribers see ONLY their selected Region(s); cross-Region access is an explicit rollup-Region opt-in; cross-account is Lake-Formation + RAM grants. [SecLake-subs][SecLake-xacct]
- **C2/C3 anchors:** C3's mandatory-time-bound (Topic 4) is the generalized cost control; the C2 satellite-mesh residency-by-construction decision + matured-vision §17.5 constrain *where* a query may reach.

### Discussion LEAN

- **The mandatory time-bound (C3 Topic 4) IS the primary cost control on a multi-year lake.** On Iceberg it prunes via `days()`/`hours()` partition transforms; on Security Lake it prunes `eventDay=` prefixes — both BEFORE any S3 GET. Without it, an unbounded query scans years of Parquet at S3-GET + Athena-scan cost. This is the strongest argument for C3's "inject-default-window-with-disclosure" lean: on a lake, no-time-bound = scan-everything = unbounded cost. Lean: for lake/Security-Lake sources, the default-window injection should be *tighter* than for live sensor APIs (a multi-year lake makes an unbounded scan catastrophically expensive, not merely slow).
- **Egress is a first-class guard for cross-region/cross-account lake reads.** S3 cross-region GET incurs egress; reading another tenant's Security Lake via RAM-shared Lake Formation incurs cross-account data-transfer. Prism should surface estimated bytes-scanned/egress in the query plan (agent-harness disclosure) and enforce a configurable egress ceiling.
- **Residency-by-construction must gate at PLAN TIME, before any S3 GET.** If a prism deployment's residency policy forbids reaching a region/tenant, a query targeting a lake in that region must be **rejected at PrismQL plan-time** (the same pre-DataFusion guard pass C3 uses for join-reject + time-bound). This is a *reject*, not a *degrade* — matching C3's join-guard posture and the C2 residency decision. Security Lake's per-Region/per-subscriber scoping is the natural enforcement seam: prism only configures subscriber bindings for residency-permitted regions; a query referencing an out-of-region lake table fails descriptor resolution.
- **Result-limit (C3 Topic 4) is the second cost ceiling** — default + max rows, pushed as Iceberg/Parquet `scan(limit)` where supported, residual-capped centrally. On pushdown-API SIEMs, map to the native result-limit (SecOps exposes 1k–1M; Splunk via SPL; etc.).

### Open Qs (SQ5)
- Egress/scan-cost ceiling: configurable per-deployment, per-tenant, or per-query-with-override? Lean: per-deployment default + per-tenant override + audited per-query escape hatch (mirrors C3's join-override lean).
- Does the residency reject happen at descriptor-resolution (the out-of-region table simply isn't bound) or as an explicit plan-time policy check? Lean: BOTH — descriptor binding only includes residency-permitted regions (fail-closed), AND an explicit plan-time check emits a structured residency-denied audit event for observability.
- How does prism estimate bytes-scanned pre-execution for the disclosure envelope? Iceberg manifests carry per-file row/byte counts usable for a pre-scan estimate; Security Lake Hive-Parquet needs partition-count × avg-object-size heuristic. [INCONCLUSIVE on Security-Lake pre-estimate precision — Iceberg manifests are richer than Hive listings.]

---

## Sub-Question 6 — Write Path Symmetry (`RETAIN <dur>` → Iceberg, `FROM cache.<name>` ← Iceberg)

### Prior art (verified)

- **iceberg-rust writer (CONFIRMED, append-mature):** the `iceberg` crate's writer module appends data files and generates delete files per the Iceberg spec; includes a **partition writer** (partitioned writes) and a delta writer (incremental). icepick commits Parquet files (auto-detecting Hive-style partitions), runs bin-pack compaction, and snapshot cleanup against AWS S3 Tables + Cloudflare R2 — proving **transactional catalog commits work in practice** in those catalogs. [IRust-writer][IRust-icepick]
- **Append vs row-level mutation (the ceiling):** GitHub issue #450 (copy-on-write / merge-on-read) confirms *"iceberg-rust can append data files but has no support for row-level mutations."* It can *produce* delete files but the higher-level apply-via-COW/MOR + compaction coordination for full UPDATE/DELETE/MERGE is **incomplete as of late 2025–2026.** [IRust-450]
- **DataFusion DML integration:** `IcebergTableProvider::insert_into` maps `INSERT INTO` → append via the writer. More complex DML (delete-file application, snapshot rewrites) is a work-in-progress; DataFusion write integration is "not yet complete." [Ctx7-DF][IRust-468]
- **Commit-conflict handling:** sources do NOT document how iceberg-rust handles concurrent-writer commit conflicts. icepick's single-writer commit/compact pattern works; multi-writer conflict/retry behavior is unverified. [IRust-icepick] [INCONCLUSIVE]
- **Read-and-write same stack (CONFIRMED structurally):** the SAME `iceberg` crate provides both the scan builder (read) and the writer module (write); icepick reads (scan + prune) and writes (commit + compact) through one stack. So `RETAIN` (write) and `FROM cache.<name>` (read) sharing one Iceberg+DataFusion stack is realistic. [IRust-writer][IRust-icepick]

### Discussion LEAN

- **The write-path symmetry claim is REALISTIC TODAY for an append-mostly cold cache** — which is exactly the ADR-PROP cold-tier workload ("append-mostly OLAP," explicitly contrasted with the OLTP case-management workload that was routed to Postgres, NOT Iceberg). `RETAIN <dur>` writing rows to an Iceberg table via the partition writer (partitioned on `event_time`/`days()`) + committing to a catalog, then `FROM cache.<name>` reading via the same `IcebergTableProvider`, is buildable on iceberg-rust 0.9.x/0.10.x. [ADR-PROP §Decision][IRust-writer]
- **The maturity ceiling is the right shape for the workload, not a blocker.** A cold OCSF cache is append + compact + expire-by-snapshot — never row-update. The missing COW/MOR row-level-mutation support is irrelevant to the `RETAIN`/`FROM cache` path. The ADR-PROP already routed the row-update-heavy workload (case-management) to Postgres precisely to keep Iceberg append-only. The architectures are aligned.
- **The two REAL write-side risks to flag:** (1) **pre-1.0 API churn** in the writer/commit API across 0.x bumps (pin exact version, budget for breakage); (2) **commit-conflict handling under concurrent writers is undocumented** — if multiple prism workers `RETAIN` into the same Iceberg table concurrently, conflict/retry behavior must be validated (a dtu-validator-style test against the chosen catalog), not assumed. For a single-writer-per-table cold-cache design this risk is minimal; for concurrent multi-writer it is open.
- **Catalog choice cascades from SQ2.** The write path commits to whatever catalog prism's cold tier uses (REST or S3 Tables — both proven by icepick; Glue is the gap). Lean: self-managed cold tier on a **REST catalog or S3 Tables** (icepick-proven write path), NOT Glue (write-path Glue maturity is even less certain than read).

### Open Qs (SQ6)
- Single-writer-per-Iceberg-table cold-cache design (sidesteps the undocumented commit-conflict risk), or concurrent multi-writer (needs conflict/retry validation)? Lean: single-writer-per-table partition where feasible; validate concurrent-commit behavior before relying on it.
- Catalog for the self-managed cold tier: REST vs S3 Tables vs Glue? Lean: REST or S3 Tables (icepick-proven); avoid Glue for the write path.
- Does `RETAIN <dur>` map cleanly to Iceberg snapshot-expiry / partition-expiry for the `<dur>` TTL, or does prism manage expiry out-of-band via icepick-style snapshot cleanup? Lean: Iceberg snapshot/partition expiry driven by a prism-managed retention process (icepick demonstrates snapshot cleanup by retention policy). [IRust-icepick]

---

## Consolidated Open Design Questions

1. **Reword the flagship vision sentence.** "Security Lake IS OCSF-as-Iceberg" is factually wrong — Security Lake is Hive-partitioned OCSF Parquet in Glue/Lake-Formation, NOT Iceberg. Restate as "one DataFusion ENGINE, two TableProviders (self-managed Iceberg + Security-Lake Glue/Hive-Parquet)." This is the single most important correction. [SecLake-store][S3Tables-CWL]
2. **iceberg-rust lineage:** ASF `apache/iceberg-rust` vs unofficial `JanKaul/iceberg-rust` — choose one (Glue support, equality-deletes, maintenance, `#[non_exhaustive]` fit). [IRust-crates][IRust-jankaul]
3. **Security Lake binding:** `S3` data-access subscriber (skip Glue, prism does partition projection + OCSF interp — dogfood-consistent) vs `LAKEFORMATION` query-access (needs Glue/Athena). Lean: `S3` data-access. [SecLake-subs]
4. **Glue catalog gap:** if Glue is ever needed, REST-shim-over-Glue vs native Glue client vs JanKaul direct-Glue. Lean: REST-shim or skip-Glue. [IRust-2186]
5. **Two adapter archetypes confirmed:** pushdown-API ("fetch-then-residual": Splunk/Elastic/OpenSearch/Sentinel/SecOps) and lake-bulk-read ("prune-then-scan": Security Lake/Iceberg/Snowflake/Databricks). Both map onto C3's descriptor with different `pushdown_target` + exactness profiles.
6. **OCSF version axis:** make C3's per-(table, schema-class) descriptor per-(table, schema-class, schema-VERSION); target OCSF version per deployment; normalize 1.1/1.3 Security Lake data up to target at the boundary, carry `metadata.version`. [OCSF-repo][SecLake-ocsf]
7. **Cost guards:** mandatory time-bound (tighter default window for lakes than for live sensors) + egress ceiling + result-limit, all enforced at PrismQL plan-time. [C3-§4]
8. **Residency:** reject (not degrade) out-of-region/out-of-tenant lake queries at plan-time; enforce via fail-closed descriptor binding + an explicit plan-time policy check emitting a residency-denied audit event. [C2-residency][SecLake-subs]
9. **Write path:** append-only `RETAIN` → Iceberg is realistic today; pin exact iceberg-rust version; choose single-writer-per-table OR validate concurrent-commit conflict handling (undocumented); cold-tier catalog = REST/S3-Tables, not Glue. [IRust-writer][IRust-450]
10. **DataFusion time-travel (`AS OF`)** SQL surfacing is incomplete in iceberg-datafusion — if backtesting/model-audit (ADR-PROP) needs it, prism may need to wire it. [INCONCLUSIVE]
11. **Audit/observability:** lake-read pushdown decisions, injected-window disclosure, egress estimate, residency-denied, and OCSF-version-skew events likely need new Canonical Structured Event Catalog rows in BC-2.16.002 (CLAUDE.md SAP-1) — downstream spec dependency, NOT actioned here.

## Honest Costs & Caveats

- **The "Security Lake IS Iceberg" premise is false** and is the most important finding to surface. The architectural conclusion ("one DataFusion engine") survives, but only if reworded to two-providers-one-engine. Building prism on the literal "one TableProvider" assumption would produce a wrong design — Security Lake needs a Glue/Hive-Parquet (or raw-S3-Parquet) provider, distinct from `IcebergTableProvider`. [SecLake-store][S3Tables-CWL]
- **iceberg-rust is pre-1.0.** Read path is production-usable (icepick proves it on S3 Tables + R2); but API churn across 0.x is a real maintenance cost, the Glue catalog is incomplete, DataFusion write-DML and SQL time-travel are partial, and concurrent-writer commit-conflict behavior is undocumented. None are blockers for an append-mostly cold cache reading/writing via REST or S3 Tables catalogs; all are risks to pin-and-validate. [IRust-crates][IRust-2186][IRust-450]
- **No SIEM (Splunk/Elastic/OpenSearch/Sentinel/SecOps) is bulk-readable.** Their object stores are proprietary and vendor-documented as off-limits. Federation against them is pushdown-query-API ONLY. Only general-purpose lakes (Snowflake/Databricks) and OCSF lakes (Security Lake; maybe future Sentinel-lake) support bulk reads. [Splunk-smartstore][Elastic-frozen][Sentinel-lake]
- **OCSF version skew is a genuine correctness hazard**, not a theoretical one — 1.4.0 deprecated ~12 attributes, Security Lake lags at 1.1/1.3, upstream is at 1.6.0. Mixing versions without a normalization/mapping layer mis-binds queries. [OCSF-14][OCSF-repo][SecLake-ocsf]
- **Cross-region/cross-account lake reads carry egress + Lake-Formation/RAM grant complexity** that a residency-bound prism deployment must gate at plan-time, before any S3 GET. The Security Lake per-Region subscriber model is the enforcement seam, but it is an AWS-specific seam — other lakes (Snowflake/Databricks) have their own region/sharing models prism must map separately. [SecLake-subs][SecLake-xacct]
- **Sentinel-lake and SecOps/Chronicle bulk-read paths are UNCONFIRMED** in 2025–2026 docs. Treat as pushdown-only until a vendor-documented external OCSF/Parquet (Sentinel) or BigQuery (SecOps) export exists; do not design against undocumented ADLS/BigQuery formats. [Sentinel-lake][GSO-udm]
- **Vision-file not directly read** (see header honesty note). §3.3/§3.5/§17.5 anchors trace to the ADR-PROP capture (which I DID read and which restates §3.3-addendum verbatim) + the brief paraphrase. A reviewer with the vision file open should reconcile.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 3 | (1) Amazon Security Lake federated read model — storage shape, OCSF table-per-class, partition layout, subscriber models, cross-account/Lake-Formation/RAM, external-engine read paths, and the load-bearing Iceberg-vs-Hive-Parquet question (`reasoning_effort=high`). (2) SIEM-as-queryable-store federation across Splunk/Elastic/OpenSearch/Sentinel/SecOps/Snowflake/Databricks — pushdown-vs-bulk + predicate classes (`reasoning_effort=high`). (3) iceberg-rust + DataFusion Iceberg maturity — version state, catalog support, predicate/partition pushdown, schema-evolution/time-travel, and the WRITE path (`reasoning_effort=high`). All three returned 70–86KB; read in full via file (chunks 1–2 directly + targeted Grep over the single-line JSON for the remainder). |
| Perplexity perplexity_reason | 0 | — |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_ask | 1 | OCSF version state + minor-version backward-compatibility / skew (SQ4) — returned current GA = 1.6.0, ~12 deprecations in 1.4.0, no minor-version compat guarantee. |
| Context7 | 2 | `resolve-library-id` (found BOTH `/apache/iceberg-rust` and `/jankaul/iceberg-rust` — the two-stack caveat) + `query-docs` on `/apache/iceberg-rust` confirming `IcebergTableProvider` exact method signatures (`scan`/`supports_filters_pushdown`/`insert_into`), Glue+OpenDAL-S3 catalog example, and the catalog enum (Rest/Glue/Memory/HMS/S3Tables/SQL) with the "not all complete" caveat. |
| Tavily (all) | 0 | — |
| WebFetch | 0 | — |
| WebSearch | 0 | — |
| Read | 5 | C3 prior research + ADR-PROP storage taxonomy (non-contradiction); 3× attempts to read saved Perplexity result files (single-line JSON; pivoted to Grep). |
| Grep | 6 | Extract substantive claims + citations from the two oversized single-line research JSON files (write-path maturity, pushdown, catalog support, Security-Lake-vs-Iceberg, version numbers, citation URLs). |
| Training data | ~2 areas | OCSF↔native-SIEM mapping being prism-internal (no external prior art); the C3/C2 cross-references (read from the C3 file directly, not training data). Both flagged inline. |

**Total MCP tool calls:** 6 (3× `perplexity_research` high-effort + 1× `perplexity_ask` + 2× Context7).
**Training data reliance:** low — every load-bearing claim (Security Lake = Hive Parquet not Iceberg; iceberg-rust version + read/write maturity; SIEM pushdown-vs-bulk; OCSF 1.6.0 + version skew) is web-sourced and citation-backed; Context7 independently corroborated the iceberg-rust DataFusion TableProvider surface against the live public-api dump. Pre-1.0 version numbers (0.9.1/0.10.0) are from crates.io via the research pass (Context7's mirror lagged at v0.7.0 — flagged).

### Citation key (sources from MCP web findings + Context7)

**Amazon Security Lake / AWS:**
- **[SecLake-store]** AWS Security Lake docs — OCSF Parquet on S3, Glue Data Catalog + Lake Formation, NOT Iceberg.
- **[SecLake-custom]** AWS Security Lake docs (custom sources) — `region/accountId/eventDay` partition path, OCSF ≤1.3, object-size/time-sort requirements, IAM/Glue/Lake-Formation actions.
- **[SecLake-subs]** AWS Security Lake docs — subscriber access types (`S3` data-access via HTTPS/SQS; `LAKEFORMATION` query-access), per-Region scoping, 10-source cap, rollup Region.
- **[SecLake-query]** AWS Security Lake docs — query-access subscribers query Lake Formation tables via Athena/Redshift Spectrum/Spark SQL.
- **[SecLake-xacct]** AWS Lake Formation docs (https://docs.aws.amazon.com/lake-formation/latest/dg/what-is-lake-formation.html) — cross-account via RAM, LF-TBAC vs named-resource grants, resource links.
- **[SecLake-ocsf]** AWS Security Lake docs (https://docs.aws.amazon.com/security-lake/latest/userguide/open-cybersecurity-schema-framework.html) — OCSF as schema, `metadata.version = 1.1.0` for native sources, versioning criteria.
- **[SecLake-reinvent]** AWS re:Invent 2024 security recap — OpenSearch zero-ETL integration; no Iceberg/S3-Tables migration for Security Lake.
- **[S3Tables-CWL]** AWS S3 Tables / CloudWatch Logs S3 Tables Integration docs — Iceberg format + Glue Iceberg REST endpoint (the SEPARATE Iceberg context, distinct from Security Lake).
- **[Zscaler-seclake]** Zscaler help portal — Security Lake integration folder structure `region=/accountid=/eventDay=` (third-party corroboration).

**SIEM / lake query APIs:**
- **[Splunk-rest]** Splunk REST API tutorial (help.splunk.com .../creating-searches-using-the-rest-api) — `search/jobs` + `/export`, SPL, time modifiers, output modes.
- **[Splunk-smartstore]** Splunk SmartStore S3 config docs — buckets are SmartStore-exclusive, do not share with other tools.
- **[Splunk-fss3]** Splunk Cloud "Federated Search for Amazon S3" docs — Splunk-as-consumer-of-S3.
- **[Splunk-seclake]** Splunk blog — Splunk Add-On for Amazon Security Lake (Splunk as OCSF-lake consumer).
- **[Elastic-esql]** Elastic ES|QL reference (elastic.co/docs/reference/query-languages/esql) — piped query language, `_query` endpoint.
- **[Elastic-sql]** Elastic SQL docs (elastic.co/docs/explore-analyze/query-filter/languages/sql).
- **[Elastic-sqlrest]** Elastic SQL REST API (elastic.co/docs/reference/query-languages/sql/sql-rest) — cursor pagination, async, DSL filters.
- **[Elastic-frozen]** Elastic blog — frozen tier / searchable snapshots on S3 (internal snapshot format, not Parquet).
- **[OS-sqlppl]** OpenSearch SQL & PPL API docs (docs.opensearch.org/.../sql-and-ppl-api) — `_sql`/`_ppl`/`_explain`, `fetch_size`.
- **[OS-ppl]** OpenSearch SQL GitHub issue #4392 — PPL usage beyond cluster.
- **[Sentinel-lake]** Microsoft Sentinel data lake overview (learn.microsoft.com/.../datalake/sentinel-lake-overview) — ADLS, ASIM/OCSF, open-format messaging.
- **[Sentinel-blog]** Microsoft Security blog 2025-07-22 — Sentinel data lake (350+ connectors, agentic AI).
- **[Sentinel-soc]** "Future of the SOC" (softwareanalyst.substack.com) — Sentinel ASIM + converging OCSF.
- **[Sentinel-migrate]** Azure Sentinel migration docs (docs.azure.cn/.../sentinel/migration-export-ingest) — Blob Storage/AzCopy ingest path.
- **[GSO-udm]** Google SecOps/Chronicle UDM search docs (docs.cloud.google.com/chronicle/docs/investigation/udm-search) — time-range (absolute/relative), result-limit, sampling, UDM-field aggregation.
- **[Snowflake-jdbc]** Snowflake JDBC docs (docs.snowflake.com/.../jdbc/jdbc).
- **[Snowflake-cyber]** Snowflake "AI Data Cloud for Cybersecurity" (snowflake.com/.../cybersecurity).
- **[Databricks-sqlexec]** Databricks SQL Statement Execution API tutorial (docs.databricks.com/aws/en/dev-tools/sql-execution-tutorial).

**iceberg-rust / DataFusion:**
- **[IRust-crates]** crates.io/crates/iceberg-rust + crates.io/crates/iceberg-datafusion — version state (0.9.1 / 0.10.0 family, pre-1.0).
- **[IRust-jankaul]** github.com/JanKaul/iceberg-rust — the unofficial "batteries-included" stack (direct Glue, DataFusion, equality-deletes).
- **[IRust-writer]** docs.rs/iceberg/latest/iceberg/writer/index.html — writer module (append data files, delete files, partition writer, delta writer).
- **[IRust-450]** apache/iceberg-rust issue #450 — "can append data files but no row-level mutations" (COW/MOR incomplete).
- **[IRust-2186]** apache/iceberg-rust issue #2186 — no built-in direct Glue client; REST-shim-over-Glue pattern.
- **[IRust-468]** apache/iceberg-rust discussion #468 — REST catalog demo, static-predicate plumbing, DataFusion-integration-not-complete.
- **[IRust-icepick]** lib.rs/crates/icepick — CLI/wasm using iceberg-rust against AWS S3 Tables + Cloudflare R2; partition pruning, column stats, commit, compaction, snapshot cleanup.
- **[IRust-sqlcat]** crates.io/crates/iceberg-sql-catalog — SQL/JDBC catalog (Postgres/MySQL).
- **[IRust-timetravel]** conduktor.io Iceberg time-travel glossary + iceberg-rust API (snapshot selection at API level; DataFusion `AS OF` surfacing incomplete).
- **[DF-iceberg]** apache/iceberg-rust crates/integrations/datafusion (README + public-api.txt) — `IcebergTableProvider`.
- **[DF-runtimefilter]** apache/datafusion issue #16959 — runtime/dynamic predicate pushdown, "Trino model fits iceberg-rust's reader cleanly (already has static-predicate plumbing)."
- **[Iceberg-spec]** github.com/apache/iceberg — Iceberg spec (schema evolution via field IDs, time travel, partition transforms `days()`/`hours()`).
- **[Ctx7-DF]** Context7 /apache/iceberg-rust — `IcebergTableProvider` method signatures (scan/supports_filters_pushdown/insert_into/schema/table_type).
- **[Ctx7-cat]** Context7 /apache/iceberg-rust — catalog enum (Rest/Glue/Memory/HMS/S3Tables/SQL) + "not all complete" caveat + Glue/OpenDAL-S3 example.

**OCSF:**
- **[OCSF-repo]** github.com/ocsf/ocsf-schema — current GA 1.6.0 (Aug 1, 2025), semantic versioning, `version.json`.
- **[OCSF-14]** query.ai "What's new in OCSF 1.4.0" — ~140 net changes + ~12 deprecations (minor-version breaking-change evidence).

**Cross-references (read from prism artifacts, not web):**
- **[C3-§4]** capability-descriptor-pushdown-2026-06-26.md Topic 4 (mandatory time-bound + default/max limit).
- **[C2-residency]** referenced C2 satellite-mesh residency-by-construction decision (per brief; vision §17.5).
- **[ADR-PROP §Decision]** ADR-PROP-storage-engine-taxonomy.md — Iceberg cold-tier workload-fit, `(source-class, schema, schema-version)` table key, the "one mechanism" claim (lines 102-106).
