---
document_type: proposed-adr
status: capture
do_not_execute: true
decided: "2026-06-27 (human)"
candidate_adr_slots:
  - "ADR-PROP-C5-1: Security Lake storage format correction (Hive-Parquet, NOT Iceberg)"
  - "ADR-PROP-C5-1b: Cold-tier = Apache Iceberg, REAFFIRMED after head-to-head"
  - "ADR-PROP-C5-2: Security Lake binding = S3 data-access subscriber DEFAULT"
  - "ADR-PROP-C5-3: Residency = REJECT at plan-time (uniform with C2 D-C2-12)"
  - "ADR-PROP-C5-4: iceberg-rust lineage = defer to morph-time bake-off"
  - "ADR-PROP-C5-5: Federation sources = connector PLUGINS (HUMAN DIRECTIVE)"
produced_by: architect
timestamp: "2026-06-27"
provenance: >
  Side-analysis C5 capture; human-confirmed decisions 2026-06-27 session.
  Research basis: research/siem-lake-federation-2026-06-27.md (C5 flagship — Security Lake
  storage format, iceberg-rust maturity, SIEM pushdown-vs-bulk, OCSF version skew,
  cost/egress/residency guards) + research/coldtier-iceberg-vs-hive-parquet-2026-06-27.md
  (head-to-head; Iceberg reaffirmed). Does NOT modify live ADR files, ARCH-INDEX.md,
  STATE.md, SESSION-HANDOFF.md, or any live factory artifact.
traces_to:
  - matured-vision-day2-requirements.md §3.3 (cold tier; the "one mechanism" claim corrected)
  - matured-vision-day2-requirements.md §3.5 (SIEM / Security Lake / Data Lake federation)
  - matured-vision-day2-requirements.md §16.4 (C5 decisions log entry)
  - matured-vision-day2-requirements.md §17.5 (residency-by-construction reference)
  - day2-design-decisions/ADR-PROP-storage-engine-taxonomy.md (cold tier R4 corrected; Iceberg reaffirmed)
  - day2-design-decisions/ADR-PROP-dynamic-schema-connectors.md (C4 — connector decisions inherited by C5 plugins)
  - day2-design-decisions/ADR-PROP-capability-descriptor-pushdown.md (C3 — pushdown descriptor + cost guards)
  - day2-design-decisions/ADR-PROP-satellite-mesh.md (C2 — residency-by-construction D-C2-12)
  - proposed epic E-LAKE-CONNECTOR-001
  - CLAUDE.md (AD-017 AI-opaque credentials; SAP-1 structured event catalog; ADR-024 ColumnType enums)
---

# ADR-PROP — SIEM / Security-Lake Federation (C5)

> **STATUS: DECIDED 2026-06-27 (human).** This is a CAPTURE artifact for the side-analysis C5
> program. `do_not_execute: true`. Real ADR numbers and formal ARCH-INDEX.md rows are deferred to
> the morph execution (post-demo, post-T14, gated on brief-reframe sign-off §5.1).

> **Research basis:** `research/siem-lake-federation-2026-06-27.md` — primary C5 research
> (three `perplexity_research` sonar-deep-research calls at `reasoning_effort=high` covering
> Amazon Security Lake federated read, iceberg-rust + DataFusion Iceberg maturity, SIEM
> pushdown-vs-bulk matrix, OCSF version axis, cost/egress/residency guards, and write-path
> symmetry; plus one `perplexity_ask` + two Context7 calls). Load-bearing factual claims are
> web-sourced and citation-backed in that research document.
> `research/coldtier-iceberg-vs-hive-parquet-2026-06-27.md` — the decision-gating head-to-head
> for the cold-tier format choice (two `perplexity_research` + four Context7 + two WebSearch calls).
> The head-to-head leaned SWITCH-to-Hive-Parquet; the human REAFFIRMED Iceberg (D-C5-1b).

> **Scope fence vs C2/C3/C4.** Residency-by-construction is OWNED by C2
> (`ADR-PROP-satellite-mesh.md` D-C2-12) and extended here to lake federation. The
> pushdown/descriptor/exactness/cost-guard mechanics are OWNED by C3
> (`ADR-PROP-capability-descriptor-pushdown.md`) and applied here. The connector-boundary
> normalization, schema drift, and WASM sandbox decisions are OWNED by C4
> (`ADR-PROP-dynamic-schema-connectors.md`) and inherited here without re-derivation.
> C5 covers the lake/SIEM-federation-SPECIFIC concerns: source storage shapes, Iceberg
> read/write path reality, the two-provider architecture, OCSF-as-lingua-franca, connector-plugin
> mandate, and lake-specific cost/egress/residency guards.

---

## Context

Prism's §3.5 dual stance positions SIEMs, security lakes, and data lakes as both competitors
(replace-by-capability) and source types in the adapter model (federate-into). Amazon Security
Lake is the highest-value first lake connector: it stores OCSF-normalized data and therefore
the adapter transform stage collapses to near-zero semantic normalization work.

The matured vision's original §3.3 addendum contained a load-bearing factual claim:
*"Security Lake IS OCSF-as-Iceberg … the SAME DataFusion + Iceberg TableProvider — one mechanism,
not two."* The C5 research (2026-06-27) established this claim is **incorrect on storage format**:
Amazon Security Lake stores OCSF Parquet in Hive-style partitions cataloged in AWS Glue Data
Catalog + Lake Formation — NOT Apache Iceberg. Iceberg appears only in the separate S3 Tables
service. This correction is load-bearing for the architecture: the cold-tier and Security Lake
read paths require TWO TableProvider implementations, not one.

The correction prompted a head-to-head analysis of whether to switch the cold tier to
Hive-Parquet (which would recover the one-provider simplification). The human reaffirmed
Iceberg after reviewing the head-to-head, on schema-evolution + row-level-mutation-headroom
grounds. The "one provider" unification simplification is real but is outweighed by durability +
future-proofing.

Beyond the format correction, C5 establishes the architectural model for federating against the
broader SIEM/lake ecosystem: two adapter archetypes, the connector-plugin mandate, residency
enforcement, OCSF version normalization, and cost guardrails.

---

## Decision Ledger

### D-C5-1 — Security Lake Storage Format Correction (LOAD-BEARING)

**CORRECTED 2026-06-27 (human). Corrects the §3.3 addendum + storage-taxonomy ADR-PROP R4.**

Amazon Security Lake stores OCSF-normalized **Apache Parquet in Hive-style partitioned layout**
(`region=/accountId=/eventDay=`) cataloged in **AWS Glue Data Catalog** and governed by **AWS
Lake Formation** — NOT Apache Iceberg. AWS documentation consistently describes Parquet +
Hive-style partitioning; Iceberg appears only in the *separate* S3 Tables / CloudWatch Logs
lakehouse context. No AWS material confirms Security Lake migrating to Iceberg or integrating
S3 Tables. [SecLake-store][S3Tables-CWL]

**Impact:**
- The "one DataFusion + Iceberg TableProvider" framing in §3.3 addendum is INCORRECT.
- The accurate framing: **one DataFusion ENGINE, two TableProviders** — the self-managed cold
  tier reads via `IcebergTableProvider`; Amazon Security Lake reads via a distinct
  Glue/Hive-Parquet (`ListingTable`) provider.
- The engine-level unification (no second query engine needed) still holds.
- The storage-format equivalence does NOT hold.
- This correction stands regardless of the cold-tier format choice. Security Lake is Hive-Parquet
  whether Prism's cold tier is Iceberg or also Hive-Parquet.

**Corrected prose has been applied to:**
1. `matured-vision-day2-requirements.md` §3.3 addendum sentence (surgical reword, in-place).
2. `ADR-PROP-storage-engine-taxonomy.md` R4 bullet (corrected + false-premise note added, in-place).

**Research citation:** [SecLake-store][SecLake-custom][S3Tables-CWL] from
`research/siem-lake-federation-2026-06-27.md` §SQ1.

---

### D-C5-1b — Cold Tier = Apache Iceberg, REAFFIRMED 2026-06-27 (human)

**REAFFIRMED 2026-06-27 (human) after the head-to-head head-to-head
(`research/coldtier-iceberg-vs-hive-parquet-2026-06-27.md`) leaned SWITCH-to-Hive-Parquet.**

The row-2 engine in the four-engine taxonomy (`ADR-PROP-storage-engine-taxonomy.md` §Decision)
— Apache Iceberg as the cold analytic tier — is UNCHANGED. The Iceberg format commitment stands.

**Rationale for reaffirmation (human-confirmed):**

- **ACID + field-ID schema-evolution (R2/R3 intact):** Multi-year OCSF drift (1.1 → 1.3 → 1.4 →
  1.6+; OCSF 1.4.0 carried ~12 deprecations and ~140 net changes; Security Lake is pinned to 1.1/1.3
  while upstream is at 1.6.0) is a real, observed problem. Iceberg's stable integer field-ID schema
  authority — add/rename/drop/reorder with field-ID tracking — is purpose-built for exactly this
  multi-year-drift workload. Hive-Parquet's workaround (version-homogeneous tables enforced
  out-of-band via version-keyed table naming) is viable but places the enforcement burden on Prism
  rather than the storage layer.

- **Row-level-mutation HEADROOM (the deciding factor):** The head-to-head's "flip conditions" for
  keeping Iceberg include:
  - Condition #1: Row-level corrections/deletes become a real workload (GDPR erasure of specific
    records, customer offboarding, event-correction/supersession — not whole-partition deletes).
  - Condition #2: True SQL `AS OF` snapshot time-travel becomes a hard requirement AND
    iceberg-datafusion surfaces it.
  The human's explicit rationale is that condition #1 (GDPR erasure / customer offboarding /
  event correction) represents real, foreseeable workloads in an MSSP product with multi-tenant
  isolation requirements. Iceberg's copy-on-write / merge-on-read row-level delete preserves this
  path. Switching to Hive-Parquet forfeits it. **Choosing durability + future-proofing over the
  one-provider simplification is the explicit trade-off.**

- **R4 false-premise note:** ADR-PROP R4's original justification ("Security Lake IS Iceberg → one
  mechanism") was a false premise now corrected by D-C5-1. The reaffirmation is therefore NOT
  based on R4. It is grounded in R2/R3 (schema evolution) + the row-level-mutation headroom above.

**iceberg-rust 0.9.0 currency update (supersedes C5 research's version snapshot):**
The `research/coldtier-iceberg-vs-hive-parquet-2026-06-27.md` head-to-head found a load-bearing
currency note: iceberg-rust reached **0.9.0 on 2026-03-10** with significantly expanded DataFusion
integration — DDL via SQL (create/drop table), limit pushdown, expanded predicate pushdown
(Boolean/IsNaN/Timestamp/Binary/string-pattern matching), and insert with automatic sort-based
clustering for partitioned writes. This materially narrows the gap vs. DataFusion's `ListingTable`
maturity (the head-to-head revised its maturity axis from "strong lean Hive-Parquet" to "moderate
lean"). The crates.io version-string discrepancy (C5's `0.9.1/0.10.0` vs the site's `0.9.0` release
post) is flagged as INCONCLUSIVE on the precise current patch; the capability set is confirmed.
iceberg-rust remains pre-1.0 — pin exact version and budget for 0.x API churn.

**Write path (cold tier, Iceberg — append-only RETAIN→Iceberg, realistic today):**
- Single-writer-per-table (sidesteps undocumented concurrent-commit conflict handling in iceberg-rust).
- Catalog: REST or S3 Tables (icepick-proven); NOT Glue (Glue write-path maturity unproven).
- Record `ingest_time` per row (cheap; supports append-only as-of approximation + audit).
- Iceberg snapshot/partition expiry driven by a prism-managed retention process.

**The ADR-PROP storage-taxonomy row 2 is UNCHANGED.** The REAFFIRMATION NOTE has been appended
to `ADR-PROP-storage-engine-taxonomy.md` in-place (2026-06-27).

---

### D-C5-2 — Security Lake Binding: S3 Data-Access Subscriber DEFAULT

**DECIDED 2026-06-27 (human).**

The default Security Lake binding for prism is the **`S3` data-access subscriber model**: Prism
reads raw OCSF Parquet objects directly from S3 with IAM credentials, applies its own partition
projection on `region/accountId/eventDay`, and performs OCSF interpretation. Prism receives
new-object notifications via HTTPS endpoint or SQS polling.

The **`LAKEFORMATION` query-access model** (Glue/Athena, Lake-Formation-governed) is an
OPT-IN for deployments that mandate it (e.g., organizations requiring Lake Formation column-level
row-filtering governance on the Security Lake data).

**Rationale:**
- The `S3` model sidesteps the iceberg-rust Glue catalog gap entirely. Prism reads raw Parquet
  from S3 using DataFusion's `ListingTable`-style Parquet provider with partition projection —
  no Glue catalog dependency, no iceberg-rust Glue client involved.
- Partition projection on `region/accountId/eventDay` maps directly to the Hive layout.
  The `eventDay` partition is the cost-guard anchor: a time-bound query prunes to a handful
  of `eventDay=` prefixes before any S3 GET.
- Prism already owns its OCSF normalization boundary (CLAUDE.md §Conventions). Doing OCSF
  interpretation at the `S3` layer is dogfood-consistent; relying on Athena to do it via the
  `LAKEFORMATION` path is not.
- The `S3` model is compatible with the connector-plugin architecture (D-C5-5): the Security
  Lake connector plugin uses `object_store` + IAM + DataFusion `ListingTable`, no Athena
  dependency.

**OCSF version handling:** Security Lake native sources annotate `metadata.version = 1.1.0`;
custom sources may be up to 1.3. Prism normalizes inbound OCSF UP to its configured target
version at the adapter boundary and carries `metadata.version` through for audit/disclosure.
See D-C5-5 (connector-plugin OCSF version handling) for the full normalization contract.

**Research citation:** [SecLake-subs][SecLake-query] from `research/siem-lake-federation-2026-06-27.md` §SQ1.

---

### D-C5-3 — Residency = REJECT AT PLAN-TIME (Uniform with D-C2-12)

**DECIDED 2026-06-27 (human). REVERSES an initial "degrade" lean.**

A PrismQL query targeting a lake table in a residency-disallowed region or tenant is **REJECTED
at plan-time, before any S3 GET**, via:

1. **Fail-closed descriptor binding:** out-of-region lake tables are not bound into the query
   plan. A query referencing an unbound table fails descriptor resolution — no partial execution.
2. **Explicit plan-time policy check:** emits a structured residency-denied audit event
   (`event_type = "query.residency_denied"` — BC-2.16.002 Canonical Structured Event Catalog
   row required at morph time, SAP-1 obligation, NOT actioned here).

**This is UNIFORM with the C2 satellite-mesh residency decision (D-C2-12):** the satellite mesh
enforces residency-by-construction (raw data never crosses a satellite boundary; Satellite-local
credential resolution is a hard invariant). Lake federation extends the same posture: a query
cannot reach a lake in a disallowed region, period.

**ASYMMETRY with D-C3-1 (cost-based-degrade for cross-source joins):** Residency enforcement
and join-cost enforcement are intentionally asymmetric:
- **Residency = hard REJECT.** A residency violation is a compliance/legal failure mode. There
  is no "degrade" path for data sovereignty. Fail-closed is the only acceptable default.
- **Join cost guard = DEGRADE (D-C3-1).** A join that is expensive but within residency is a
  performance problem, not a compliance problem. Cost-based degrade with disclosure is the right
  posture.
This asymmetry is intentional and explicit. Do not conflate the two enforcement postures.

**Enforcement seam:** Prism only configures Security Lake subscriber bindings for
residency-permitted regions. Out-of-region tables are not bound; queries referencing them
fail plan-time descriptor resolution. This is the same fail-closed binding used throughout
the C3 capability-descriptor model (undeclared = Unsupported → central/reject).

**Research citation:** [SecLake-subs][SecLake-xacct][C2-residency] from
`research/siem-lake-federation-2026-06-27.md` §SQ5.

---

### D-C5-4 — iceberg-rust Lineage: DEFER to Morph-Time Prototype Bake-Off

**DECIDED 2026-06-27 (human). Mirrors the C2 transport deferral (D-C2-1 prototype bake-off).**

Two distinct Rust Iceberg stacks exist. The binding choice is DEFERRED to a morph-time
prototype bake-off, not decided here:

| Stack | Crates | Strengths | Weaknesses |
|-------|--------|-----------|------------|
| **ASF `apache/iceberg-rust`** (primary candidate) | `iceberg` + `iceberg-datafusion` | ASF project; REST + S3 Tables catalogs proven (icepick); `IcebergTableProvider` with `scan/supports_filters_pushdown/insert_into`; 0.9.0 (2026-03-10) expanded DataFusion integration | Glue catalog incomplete ("not all catalog implementations are fully complete"); no equality-deletes yet; pre-1.0 |
| **`JanKaul/iceberg-rust`** (secondary candidate) | `iceberg-rust` + `datafusion_iceberg` | Direct Glue catalog support; equality-deletes advertised; "batteries-included" experience | Unofficial project (not ASF); maintenance/longevity less certain; `#[non_exhaustive]` compatibility with Prism's pub-type discipline unverified |

**Lean recorded (not binding until bake-off):** ASF `apache/iceberg-rust` + REST/S3-Tables catalog
(icepick-proven) as the probable default. `JanKaul/iceberg-rust` considered only if direct-Glue
catalog support or equality-deletes become hard requirements at morph time.

**Why deferred:** The Security Lake `S3` data-access binding (D-C5-2) sidesteps the Glue catalog
gap entirely — the primary argument for JanKaul's direct-Glue advantage disappears. The bake-off
at morph time will confirm whether the ASF stack's REST/S3-Tables catalog is sufficient or whether
a direct-Glue path becomes necessary for any other Security Lake path.

**Research citation:** [IRust-crates][IRust-jankaul][IRust-icepick] from
`research/siem-lake-federation-2026-06-27.md` §SQ2.

---

### D-C5-5 — Federation Sources ARE Connector PLUGINS (Human Directive, Load-Bearing)

**DECIDED 2026-06-27 (human). HUMAN DIRECTIVE — non-negotiable.**

Amazon Security Lake AND every SIEM/lake federation source (Splunk, Elastic/OpenSearch, Microsoft
Sentinel, Google SecOps/Chronicle, Snowflake, Databricks — the full federation surface from §3.5)
is a **Connector** per the §3.4 source/connector taxonomy. **Connectors are REQUIRED to be
PLUGINS** per the connector-plugin model and Prism's existing plugin SDK
(`crates/prism-spec-engine/plugins/`). Federation sources are therefore NOT core-engine built-ins.

**CRITICAL ASYMMETRY in the two-provider architecture:**
- **Self-managed Iceberg cold-tier provider** (`IcebergTableProvider`) = **Prism-INTERNAL / CORE**.
  This is Prism's OWN storage; it is managed by the core engine, not a plugin.
- **Security Lake (Glue/Hive-Parquet ListingTable) provider AND all SIEM adapters** =
  **CONNECTOR PLUGINS**. These are external federation sources and must live in the plugin layer.

This asymmetry is architecturally sound: prism owns and directly manages its own cold tier;
everything external (lakes, SIEMs, data warehouses) is a plugin with the associated
sandboxing, versioning, and upgrade-decoupling benefits.

**C4 decisions inherited by all federation connector-plugins:**

| C4 Decision | Inheritance |
|-------------|-------------|
| D-C4-1 Mandatory boundary-normalization chokepoint | Every federation plugin passes through the identifier + value sanitization tiers. No trusted-source exemption. |
| D-C4-3 Capability-sandboxed WASM plugin host (Wasmtime WASI-P2, no ambient authority) | Federation connector-plugins that cannot be expressed as declarative TOML use the WASM escape-hatch (D-C4-3). Lake Parquet read + IAM + partition projection may need WASM — reconcile against existing plugin SDK at morph (PIV-C4-3). |
| Opaque credentials resolved satellite-local (AD-017 / C2) | No federation connector credential ever transits AI context or central Prism; resolved at the Satellite (or locally on central for central-deployed lakes). |
| Discover-then-pin static-TOML-default with confirm-or-narrow probes | TOML is the default schema declaration; introspection/inference is opt-in confirm-or-narrow-only. |
| C3 capability-descriptor per-(table, schema-class, schema-VERSION) | The OCSF-version axis is a REQUIRED third key for all federation connectors (see OCSF Version section below). |

**Two adapter archetypes (both delivered as connector plugins):**

**Archetype 1 — PUSHDOWN-API ("fetch-then-residual"):**
Applies to: Splunk (SPL), Elastic/OpenSearch (ES|QL / SQL+PPL), Microsoft Sentinel (KQL),
Google SecOps/Chronicle (UDM search).

- `pushdown_target = source` in the capability descriptor.
- Predicate classes that the native query language evaluates are `exact`: time-range, equality,
  IN, full-text (where the language supports it). Aggregation/group-by `exact` ONLY if the
  adapter issues a native aggregate and presents pre-reduced rows (DataFusion `TableProvider`
  does not negotiate aggregation — the adapter must handle it).
- **JOIN is ALWAYS central** (no SIEM can join across to Prism's other sources). Matches C3's
  cross-source join guard posture.
- Fetch post-pushdown + re-check residuals centrally.
- OCSF-normalize result rows at the adapter boundary (per CLAUDE.md OCSF normalization rule).

**Archetype 2 — LAKE-BULK-READ ("prune-then-scan"):**
Applies to: Amazon Security Lake (Glue/Hive-Parquet), Snowflake (external tables + Iceberg),
Databricks (Delta + Unity Catalog).

- DataFusion scans Parquet/Iceberg/Delta directly using the relevant TableProvider.
- Partition + column-stat pruning before data transfer.
- Time-range predicate pushes as partition prune: `exact` (eventDay/ Iceberg `days()` transform).
- Equality/IN on stats-bearing columns push as file-prune: often `inexact` → central re-check.
- JOIN always central (same cross-source posture as Archetype 1).

**NO SIEM IS BULK-READABLE:**
Splunk (SmartStore S3 buckets — proprietary index format; docs explicitly say do not share with
other tools), Elastic/OpenSearch (frozen tier / searchable-snapshots — internal snapshot format,
not Parquet), Sentinel (ADLS — "open-format" messaging but no documented external Parquet/Delta
export endpoint confirmed), SecOps/Chronicle (BigQuery backing assumed but no documented external
table export confirmed in cited docs). **Do NOT design against undocumented ADLS/BigQuery
formats.** Treat Sentinel-lake and SecOps as pushdown-only until a vendor-documented external
OCSF/Parquet export exists. [Splunk-smartstore][Elastic-frozen][Sentinel-lake][GSO-udm]

---

## Confirmed Leans

These leans were confirmed (no objection) by the human on 2026-06-27.

### L-C5-1 — OCSF Version Axis: per-(table, schema-class, schema-VERSION)

The C3 capability-descriptor key is extended: `per-(table, schema-class)` → `per-(table,
schema-class, schema-VERSION)` for all federation connectors. This matches the cold-tier table
keying in `ADR-PROP-storage-engine-taxonomy.md` R3 (`(source-class, schema, schema-version)`).

**Why the version axis is load-bearing:**
- OCSF upstream GA is 1.6.0 (2025-08-01). Security Lake native sources are pinned to
  `metadata.version = 1.1.0`; custom sources to ≤1.3.
- OCSF minor bumps are NOT guaranteed backward-compatible: 1.4.0 alone carried ~12 deprecations
  + ~140 net changes. A query authored against 1.5/1.6 field shapes can mis-bind against 1.1 data.
- An OCSF predicate on a field absent from the source's stamped version = `unsupported` (central
  compute, returns null/empty — the agent-harness MUST be disclosed, per C3 exactness rules).
- An OCSF predicate on a field present in both but with changed semantics across versions =
  `inexact` (push widened, re-check centrally — C3 transform-exactness rule applied to version skew).

**Target OCSF version per deployment:** Prism adopts a configured target version. All inbound
sources (including 1.1/1.3 Security Lake data) are normalized UP to the target at the adapter
boundary. `metadata.version` is carried through for audit/disclosure. Prism does not silently
drop version metadata.

### L-C5-2 — Cost: Time-Bound PRIMARY; Egress Ceiling + Result-Limit SECONDARY

The mandatory time-bound (C3 Topic 4 / D-C3-2) is the **primary cost control on lakes**.
- On Iceberg cold tier: prunes via `days()`/`hours()` partition transforms before any S3 GET.
- On Security Lake Hive-Parquet: prunes `eventDay=` prefixes before any S3 GET.
- Without a time-bound, a query can scan years of Parquet at S3-GET cost. This is
  **catastrophically expensive, not merely slow**, for a multi-year lake. The default-window
  injection should be **TIGHTER for lake sources than for live sensor APIs.**

Additional cost controls (both enforced at PrismQL plan-time):
- **Egress ceiling:** cross-region/cross-account S3 GET incurs egress. Surface estimated
  bytes-scanned/egress in the query plan disclosure envelope. Enforce a configurable egress
  ceiling (default per-deployment + per-tenant override + audited per-query escape hatch).
- **Result-limit:** default + max rows, pushed as Iceberg/Parquet `scan(limit)` where supported,
  residual-capped centrally. On pushdown-API SIEMs, map to native result-limit.

### L-C5-3 — Write Path (Cold Tier, Iceberg): Append-Only RETAIN, Single-Writer, REST/S3-Tables Catalog

The `RETAIN <dur>` → Iceberg write path is realistic today for append-only workloads:
- `IcebergTableProvider::insert_into` maps `INSERT INTO` → append via the iceberg-rust writer.
- Single-writer-per-Iceberg-table (sidesteps undocumented concurrent-commit conflict handling).
- Catalog: REST or S3 Tables (icepick-proven write path for both); **NOT Glue** (write-path
  Glue maturity is even less certain than read-path Glue maturity).
- Record `ingest_time` per row (cheap; supports append-only as-of approximation for
  backtesting + audit trail).
- Retention expiry = Iceberg snapshot/partition expiry driven by a prism-managed retention
  process (icepick-style snapshot cleanup by retention policy).

**The two real write-path risks to flag:**
1. **Pre-1.0 API churn** in the writer/commit API across 0.x bumps — pin exact version, budget
   for breaking changes.
2. **Concurrent-commit handling is undocumented in iceberg-rust.** For a single-writer-per-table
   cold-cache design this risk is minimal; for concurrent multi-writer it must be validated
   (a dtu-validator-style test against the chosen catalog) before relying on it.

---

## Open Design Questions

The following questions are genuinely open. Answers require morph-time implementation
or prototype work.

1. **DataFusion SQL `AS OF` time-travel surfacing.** iceberg-datafusion exposes snapshot
   selection at the API level only — no SQL `AS OF` syntax in the DataFusion Iceberg
   integration today. If backtesting/model-audit (ADR-PROP §R5, §14.3) needs SQL-level
   time-travel, Prism may need to wire it. [INCONCLUSIVE — API-level selection exists; SQL
   surfacing is incomplete as of 2026-03-10 iceberg-rust 0.9.0.]

2. **Pre-scan bytes-scanned estimation precision for the disclosure envelope.** Iceberg
   manifests carry per-file row/byte counts — rich input for a pre-scan estimate. Security
   Lake Hive-Parquet needs partition-count × avg-object-size heuristic — coarser. Iceberg's
   manifests are richer; the Security Lake estimate is an approximation. [INCONCLUSIVE on
   whether the approximation is acceptable for the egress-ceiling enforcement use case — measure
   at morph time.]

3. **Federation connector-plugin archetype: TOML declarative or WASM escape-hatch?** Lake
   Parquet read + IAM credential resolution + partition projection may need WASM (imperative
   auth, pagination over object-store list, custom partition logic). Must reconcile against
   the existing plugin SDK (`crates/prism-spec-engine/plugins/`) per PIV-C4-3. The TOML
   declarative path is the default (D-C4-5); only use WASM where the declarative model cannot
   express the connector. [MORPH-TIME resolution required.]

4. **Security Lake subscriber model: one connector exposing many class-tables, or one
   connector per OCSF class?** Lean: one connector, many `[[tables]]` blocks (one per OCSF
   class Prism cares about), each with its own pushdown descriptor and OCSF version key. This
   is consistent with C3 Topic 5 per-(table, schema-class, schema-VERSION). [CONFIRM at morph.]

5. **iceberg-rust lineage bake-off (D-C5-4).** ASF vs JanKaul prototype at morph time. Lean
   recorded (ASF primary, JanKaul only if direct-Glue / equality-deletes become hard
   requirements). [MORPH-TIME bake-off.]

---

## Downstream Spec Dependencies (Note — Not Actioned Here)

SAP-1 obligations (BC-2.16.002 Canonical Structured Event Catalog new rows needed at morph):
- `event_type = "query.residency_denied"` — emitted when a query targets a lake in a
  residency-disallowed region/tenant. Fields: query_id, region, tenant_id, lake_source, reason;
  audit role = compliance audit; recurrence = per plan-time rejection.
- `event_type = "query.lake_pushdown_decision"` — emitted when a lake scan predicate is
  classified exact/inexact/unsupported (analogous to `query.pushdown.decision` for live sensors).
  Fields: query_id, source_id, predicate_class, exactness, pushdown_target; audit role = planner
  observability; recurrence = per lake scan in each query.
- `event_type = "query.injected_default_window.lake"` — emitted when the time-bound default
  window is injected for a lake query (tighter window than live-sensor default). Fields: query_id,
  source_id, injected_window_secs, reason; audit role = cost-guard transparency; recurrence = per
  injection.
- `event_type = "query.egress_estimate"` — emitted with the pre-scan bytes-scanned/egress
  estimate in the plan disclosure envelope. Fields: query_id, source_id, estimated_bytes,
  estimated_egress_dollars_hint, partition_count; audit role = cost observability; recurrence =
  per lake scan.
- `event_type = "connector.ocsf_version_skew"` — emitted when the source's `metadata.version`
  does not match the deployment target OCSF version. Fields: connector_id, source_version,
  target_version, affected_columns_count; audit role = schema fidelity audit; recurrence =
  per fetch where version skew is detected.

All five events are flagged here; BC-2.16.002 amendment is morph-time work.

---

## Honest Costs and Risks

| Item | Cost / Risk |
|------|-------------|
| **Two TableProviders, not one** | The `IcebergTableProvider` (cold tier) and the `ListingTable`/Glue-Hive-Parquet provider (Security Lake) are two distinct DataFusion integration code paths. Two things to `#[non_exhaustive]`-audit, security-review, test, and maintain. This is the honest engineering cost of the D-C5-1 correction. |
| **iceberg-rust pre-1.0 maintenance tax** | Pin exact version; budget for breaking changes across 0.x bumps. The 0.9.0 release improved DataFusion integration substantially but pre-1.0 API stability remains a genuine operational cost. |
| **Glue catalog gap (ASF iceberg-rust)** | The recommended S3 data-access binding (D-C5-2) sidesteps the Glue gap entirely. If `LAKEFORMATION` opt-in becomes widespread, a REST-shim-over-Glue or JanKaul pivot may become necessary. |
| **OCSF version skew is a correctness risk** | Security Lake data at 1.1/1.3 vs upstream 1.6.0 means queries on newer OCSF fields silently return null/empty without the normalization layer. The version-axis descriptor key (L-C5-1) is the mechanism; the normalization must be implemented before any Security Lake query is served to an agent. |
| **No bulk read for Splunk/Elastic/OpenSearch/Sentinel/SecOps today** | These are pushdown-API only. Prism cannot read their proprietary object stores. Sentinel-lake and SecOps bulk-read paths are UNCONFIRMED. Do not bet architectural choices on undocumented formats. |
| **Cross-region egress cost** | Security Lake is per-Region. Cross-region reads incur S3 egress. The egress ceiling (L-C5-2) and the residency reject (D-C5-3) together cap this risk; neither alone is sufficient. |
| **SAP-1 event catalog debt** | Five new event types are flagged above. Until these are registered in BC-2.16.002 and the connector-plugin emits them, the lake federation path operates without the full audit trail. This is a morph-time gap, not an implementation blocker, but it must close before the first production deployment of a lake connector. |

---

## Alternatives Considered and Rejected

### Alternative A: One TableProvider (Switch Cold Tier to Hive-Parquet)

Use Hive-partitioned OCSF Parquet for Prism's cold tier (matching Security Lake's layout),
yielding ONE `ListingTable` provider for both.

**Result of head-to-head analysis:** Leaned toward this option (`research/coldtier-iceberg-vs-hive-parquet-2026-06-27.md` overall lean: SWITCH, conditional on version-homogeneous tables). The unification win (drop pre-1.0 iceberg-rust entirely; one OCSF-on-read path; one `#[non_exhaustive]`-audit surface) is real.

**Rejected (D-C5-1b, human reaffirmation) because:** Row-level-mutation headroom (GDPR erasure /
customer offboarding / event correction — Iceberg's copy-on-write/merge-on-read; Hive-Parquet
has no equivalent) and field-ID schema-evolution durability outweigh the one-provider
simplification for an MSSP product with multi-tenant isolation requirements. The unification
simplification is acknowledged as real but is explicitly traded away for correctness + future-proofing.

### Alternative B: LakeFormation Query-Access as Default

Use the `LAKEFORMATION` path (Glue/Athena-governed) as the primary Security Lake binding,
deferring partition projection and OCSF interpretation to Athena.

**Rejected (D-C5-2) because:** Adds an Athena/Glue dependency; conflicts with the dogfood
principle (Prism owns its OCSF normalization boundary); requires Lake Formation grant plumbing
that the `S3` path avoids; and the Glue catalog gap in ASF iceberg-rust makes this path harder
to implement with the preferred stack. `LAKEFORMATION` is preserved as an OPT-IN for deployments
that mandate it.

### Alternative C: Residency as Cost-Based Degrade (Not Hard Reject)

Allow queries to cross residency boundaries with a degrade posture (warning + disclosure),
consistent with D-C3-1's cost-based-degrade join posture.

**Rejected (D-C5-3) because:** Residency enforcement is a compliance/legal boundary, not a
performance boundary. A residency violation is not a "slow query" — it is a data sovereignty
failure that exposes the customer (and Prism) to regulatory risk. The C3 cost-based-degrade
posture is intentionally asymmetric with the D-C5-3 residency-reject posture. Conflating the
two would be an error.

### Alternative D: Federation Sources as Core-Engine Components (Not Plugins)

Build the Security Lake adapter and SIEM adapters as first-class components inside the prism
core engine, not as plugins.

**Rejected (D-C5-5, human directive) because:** The connector-plugin model and the existing
plugin SDK (`crates/prism-spec-engine/plugins/`) are the correct architectural boundary for
external federation sources. Building them as core-engine components would: (1) couple the
release cycle of external adapters to the core engine; (2) bypass the WASM capability sandbox
(D-C4-3) that provides sandboxing and no-ambient-authority guarantees; (3) violate the
dogfood principle (Prism's own built-in sensors ship as TOML specs that feed the same
plugin model). There is no technical justification for treating Security Lake as a core
component when it is manifestly an external source.

---

## Ripple Effects (Must Be Picked Up at Morph Time)

| Affected area | Ripple |
|---------------|--------|
| **E-LAKE-CONNECTOR-001** | The primary day-2 epic. Amazon Security Lake first (D-C5-2 `S3` binding + `ListingTable` provider + IAM + partition projection + OCSF 1.1/1.3 normalization). Then generic Iceberg/Parquet-on-S3. Then pushdown-API SIEM adapters. Each is a separate connector plugin. |
| **plugin SDK** | D-C5-5 requires reconciliation with `crates/prism-spec-engine/plugins/` — specifically whether the Security Lake plugin can be TOML-declarative or needs WASM (PIV-C4-3). Must reconcile before implementing the Security Lake connector. |
| **BC-2.16.002 §Postconditions** | Five new SAP-1 event types listed in §Downstream Spec Dependencies above (morph-time BC work). |
| **C3 capability-descriptor TOML schema** | Add the `schema_version` axis to the per-table pushdown descriptor block. Security Lake connector TOML must declare `[[tables]]` blocks keyed by `(source, class, version)`. |
| **ADR-PROP-storage-engine-taxonomy.md** | R4 corrected + REAFFIRMATION NOTE added (done). Engine taxonomy row 2 (Iceberg cold tier) UNCHANGED. |
| **matured-vision-day2-requirements.md §3.3** | §3.3 addendum sentence corrected (done). |
| **iceberg-rust version pin** | The exact iceberg-rust version used for the cold tier must be pinned in `Cargo.toml` at morph time. Budget for 0.x API churn. The D-C5-4 bake-off determines ASF vs JanKaul. |
| **ARCH-INDEX.md** | At morph time: new subsystem entry for E-LAKE-CONNECTOR-001 (proposed name: SS-2x Lake Federation Connectors; number assigned at morph). |
