---
document_type: proposed-adr
status: capture
do_not_execute: true
decided: "2026-06-27 (human)"
candidate_adr_slots:
  - "ADR-PROP-C8-1: PrismQL piped surface — ship in day-2, desugars to DataFusion logical plan"
  - "ADR-PROP-C8-2: Entity-resolution AS OF reproducibility — RESOLVED 2026-06-27: BITEMPORALITY (valid-time + transaction-time; AS OF KNOWN <T> unified knob; fresh-by-default)"
  - "ADR-PROP-C8-3: OCSF version-binding model — RESOLVED 2026-06-27: pinnable immutable catalog versions unified under same AS OF KNOWN <T> knob as D-C8-2; data-snapshot pinning deferred (OQ-C8-DATASNAPSHOT)"
  - "ADR-PROP-C8-LEAN-1: Entity-pivot grammar — FIND keyword + entity() predicate + AS OF clause"
  - "ADR-PROP-C8-LEAN-2: Multi-hop pivot — SQL/PGQ GRAPH_TABLE as forward-compat target only (NOT day-2)"
  - "ADR-PROP-C8-LEAN-3: Authoring intelligence — single LSP server reused by Monaco + CLI + NL→PrismQL agent"
  - "ADR-PROP-C8-LEAN-4: NL→PrismQL — reuse parser/planner diagnostics as validate-repair signal (NL2KQL pattern)"
  - "ADR-PROP-C8-LEAN-5: Recipe / detection-as-code format — Sigma-aligned metadata + semver + CI harness"
produced_by: architect
timestamp: "2026-06-27"
provenance: >
  Side-analysis C8 capture; human-confirmed decisions 2026-06-27 session.
  Research basis: research/prismql-deliverables-depth-2026-06-27.md — four
  perplexity_research (sonar-deep-research, reasoning_effort=high) covering Q1 entity/observable
  pivot grammar prior art (SPL/KQL-graph/Chronicle/EQL/Sigma/STIX/Cypher/GQL/SQL-PGQ), Q2
  temporal/point-in-time entity resolution (Flink/SQL:2011/SCD2/kdb+ aj/pandas merge_asof/
  Chronicle DHCP aliasing), Q3+Q3b ergonomics/learnability (PRQL/pql/KQL/SPL/EQL + NL2KQL
  guardrails), Q4+Q5 multi-schema field referencing (ASIM/CIM/UDM/OCSF) + autocomplete/LSP/Monaco
  /ariadne; plus 1 perplexity_reason (Q6 portability/Sigma/detection-as-code) and 3 Context7
  calls (Chumsky Rich error API + DataFusion LogicalPlanBuilder). Does NOT modify live ADR files,
  ARCH-INDEX.md, STATE.md, SESSION-HANDOFF.md, or any live factory artifact.
traces_to:
  - matured-vision-day2-requirements.md §12 (PrismQL grammar + ergonomics)
  - matured-vision-day2-requirements.md §12.1 (entity/observable pivot grammar sketch)
  - matured-vision-day2-requirements.md §12.3 (FSQL Ergonomics Parity Ledger — CONFIRMED 2026-06-25)
  - matured-vision-day2-requirements.md §12.4 (SEQUENCE…THEN sugar + MATCH_RECOGNIZE)
  - matured-vision-day2-requirements.md §14.2.1 (SEQUENCE…THEN desugaring — SETTLED)
  - matured-vision-day2-requirements.md §14.7 (recipe library; detection-as-code; Sigma import)
  - matured-vision-day2-requirements.md §16.4 (C8 decisions log entry)
  - matured-vision-day2-requirements.md §17.15-A5 (prism-native entity registry — SETTLED)
  - day2-design-decisions/ADR-PROP-capability-descriptor-pushdown.md (C3 join-guard; cost-based-degrade; NOT reopened)
  - day2-design-decisions/ADR-PROP-detection-engine-depth.md (C6 SEQUENCE…THEN desugaring; recipe library)
  - day2-design-decisions/ADR-PROP-siem-lake-federation.md (C5 OCSF schema-version axis)
  - epics E-PRISMQL-GRAMMAR-001 (PrismQL grammar / piped surface / LSP)
  - epics E-RULE-XLATE-001 (Sigma translator — deferred; recipe library shipped now)
  - research/prismql-deliverables-depth-2026-06-27.md (primary research basis — all six Q1–Q6 depth questions)
  - research/prismql-asof-version-resolution-2026-06-27.md (C8 FOLD research — bitemporality / AS OF KNOWN resolution for D-C8-2 + D-C8-3)
  - CLAUDE.md (#[non_exhaustive] discipline; SAP-1 structured event catalog; error taxonomy E-QUERY-NNN)
---

# ADR-PROP — PrismQL Deliverables (C8)

> **STATUS: FULLY DECIDED 2026-06-27 (human). All five decisions (D-C8-1 through D-C8-3 + leans
> L-C8-1..5) are DECIDED. D-C8-2 (entity-resolution AS OF reproducibility) and D-C8-3 (OCSF
> version-binding model) were previously DEFERRED; both resolved 2026-06-27 via C8 FOLD using
> research `research/prismql-asof-version-resolution-2026-06-27.md`. Resolution: BITEMPORALITY
> (valid-time + transaction-time; single `AS OF KNOWN <T>` knob unifying both axes; fresh-by-
> default; data-snapshot pinning deferred as OQ-C8-DATASNAPSHOT).** This is a CAPTURE artifact
> for the side-analysis C8 program. `do_not_execute: true`. Real ADR numbers and formal
> ARCH-INDEX.md rows are deferred to the morph execution (post-demo, post-T14, gated on
> brief-reframe sign-off §5.1).

> **Research basis:** `research/prismql-deliverables-depth-2026-06-27.md` — four
> `perplexity_research` (sonar-deep-research, `reasoning_effort=high`) calls; one
> `perplexity_reason` (Q6 portability); three Context7 calls (Chumsky `Rich` error API, verified
> against upstream docs; DataFusion `LogicalPlanBuilder`, verified against upstream docs).
> All load-bearing claims are source-grounded. Claims tagged `[model-knowledge]` (two areas:
> ariadne↔Chumsky pairing; DataFusion GRAPH_TABLE support today) are explicitly bounded and
> non-load-bearing.

> **Settled decisions NOT relitigated in this capture.** The following items are HUMAN-CONFIRMED
> in earlier sessions and are treated as immutable inputs that C8 grammar/ergonomics decisions
> CONSUME:
> - Join-guard posture: cost-based degrade, NOT hard-reject (C3 D-C3-1; NOT reopened here)
> - SEQUENCE…THEN…WITHIN sugar desugaring to MATCH_RECOGNIZE (§14.2.1 / §12.4; NOT reopened)
> - Entity registry model: prism-native config-driven, strong/weak tiers, temporal validity (§17.15-A5; NOT reopened)
> - §12.3 FSQL Ergonomics Parity Ledger dispositions (CONFIRMED 2026-06-25; NOT reopened)

---

## Context

Section 12 of the matured vision establishes the PrismQL grammar and ergonomics requirements at
the requirements level, including the entity/observable pivot (§12.1), the ergonomics parity
ledger (§12.3), and the SEQUENCE…THEN sugar (§12.4). This C8 ADR-PROP captures DEPTH and
IMPLEMENTATION decisions — the grammar deliverables, surface design, tooling architecture, and
portability format — that §12 and the Q1–Q6 open questions in the depth research pass
(`research/prismql-deliverables-depth-2026-06-27.md`) left unresolved or explicit as discussion
input.

Six open depth questions drove the research pass:

1. **Q1 / §12.1:** Grammar for the entity/observable pivot (`FIND` / `entity()`) — prior art
   comparison and the settled entity registry's interaction with temporal resolution.
2. **Q2:** Entity resolution semantics — temporal binding model (AS OF, interval-containment vs
   last-value, reproducibility caveat).
3. **Q3 / §12.3:** Piped sugar surface over the SQL core — proven viable? Ship in day-2?
4. **Q4 / §13.6:** Multi-schema field referencing — OCSF canonical names + native namespace +
   per-source OCSF version binding.
5. **Q5:** Authoring intelligence — LSP server, Monaco, ariadne, NL→PrismQL validate-repair.
6. **Q6 / §14.7:** Portability / recipe format — detection-as-code, Sigma import.

C8 settles Q1/Q3/Q5/Q6 and ALL of Q2/Q4. Q2's live-vs-frozen reproducibility choice and Q4's
version-agnostic-vs-explicit-pin choice were DEFERRED to the targeted research pass
`prismql-asof-version-resolution-2026-06-27.md` (OQ-C8-ASOF / OQ-C8-OCSFVER); both are now
RESOLVED via C8 FOLD 2026-06-27: BITEMPORALITY, unified `AS OF KNOWN <T>` knob (see D-C8-2 and
D-C8-3). Remaining open item: OQ-C8-DATASNAPSHOT (cold-tier data-snapshot pinning, deferred).

---

## Decision Ledger

### D-C8-1 — Piped Surface: SHIP IN DAY-2, Desugars to the SAME DataFusion Logical Plan

**DECIDED 2026-06-27 (human).**

Prism ships a **KQL/PRQL-style piped sugar surface** in day-2 that DESUGARS to the IDENTICAL
DataFusion logical plan the SQL surface produces. This is NOT a second engine.

**What is shipped:**

A `|`-piped query syntax (`source | where … | summarize … by … | order … | limit …`) that
DataFusion's `LogicalPlanBuilder` stage-by-stage construction accepts (`scan → filter →
aggregate → sort → limit → build`). The pipe surface is an ergonomic on-ramp, not a competing
language. The SQL surface (canonical semantics, full DataFusion/MATCH_RECOGNIZE support) remains
and is not removed.

**Core pipe operators** (mirroring KQL/PRQL/pql — confirmed prior art):
- `where` (filter predicate)
- `summarize` / `stats` (aggregation; `summarize` is already §12.3 E2 ADOPT)
- `project` (column selection)
- `extend` (computed columns)
- `join` (inner/outer; cross-source join guard C3 D-C3-1 still applies)
- `order` (sort)
- `limit` (row limit)

**Security-domain operators** (piped form of settled operators):
- `FIND` / `entity()` (entity pivot — see D-C8-1 lean below, confirmed §12.1 two-surface design)
- `SEQUENCE…THEN` (settled §14.2.1; pipe-composable as a terminal operator in a pipe chain)

**Why SHIP in day-2 (not defer):**

The research returns a clear verdict: the pipe-surface-over-SQL-relational-core pattern is
proven viable and Rust-native:

- **PRQL** ("a functional, pipelined query language … that compiles to SQL") — implemented in
  Rust, dbt plugin, production-deployed. The canonical proof. [PRQL-talk][PRQL-HN]
- **RunReveal `pql`** — "inspired by Microsoft's KQL," pipe syntax, compiles to SQL, built
  explicitly to avoid vendor lock-in of proprietary security query languages. [RunReveal-pql]
- **KQL / SPL / EQL** — all mature `|`-piped languages; left-to-right data-flow reading is
  their core ergonomic.

A worked DataFusion mapping (research-verified [Context7:datafusion]):
```
events
| where status == "error"
| summarize count() by service
| order by count desc
```
≡ `SELECT service, COUNT(*) AS count FROM events WHERE status = 'error' GROUP BY service ORDER BY count DESC`

The desugar-to-plan step uses DataFusion's existing `LogicalPlanBuilder` — no second planner.

**MANDATORY: expose "show desugared SQL / EXPLAIN."**

Every piped query MUST be inspectable. The DSL-debugging caveat (PRQL/pql both mitigate by
showing the generated SQL or intermediate SQL) is a real UX cost. Analysts will need to
understand what plan their pipe expression produces. The `EXPLAIN` + "show desugared SQL" escape
hatch is not optional. This is baked into the C8 decision, not left to implementation.

**Honest cost — learnability claims:**

The research is explicit that learnability claims in the vendor literature rest on design-
rationale and anecdote, NOT controlled studies. KQL's approachability is attributed "at least
as much" to IntelliSense, table browser, filter-from-grid, and example-query panes as to the
pipe syntax itself. A piped surface is NECESSARY but NOT SUFFICIENT for KQL-grade approachability.
The LSP server (D-C8-LEAN-3 below) carries equal weight. If learnability is a hard success
metric for prism, user research is required — no shortcut exists in the literature.
[INCONCLUSIVE — evidence quality: convergent practice, not proven causation]

[research/prismql-deliverables-depth-2026-06-27.md §Q3, §3.2–3.3 LEAN]

---

### D-C8-2 — Entity-Resolution AS OF Reproducibility: RESOLVED 2026-06-27 (BITEMPORALITY)

**RESOLVED 2026-06-27. Previously DEFERRED (OQ-C8-ASOF). Research basis:
`research/prismql-asof-version-resolution-2026-06-27.md`.**

**DECISION: ADOPT THE BITEMPORAL REGISTRY — valid-time interval-containment (settled) PLUS a
transaction-time axis — as the entity-resolution reproducibility model.**

The research pass confirms that the live-vs-frozen fork is a false choice at the storage level:
to get "frozen-registry-version" reproducibility at all, you must store a transaction-time
(system-versioned) axis on the registry anyway. Once you pay for that second axis, bitemporality
(exposing it as a per-query knob) is the near-free generalization that delivers BOTH the
human's original options from one model. [SQL:2011; Snodgrass; research §2.1]

**The unified reproducibility primitive:**

- **`AS OF <expr>`** (valid-time / event-time axis — SETTLED): binds weak-tier observables
  as-of the row's event-time. Default = EVENT TIME. This axis was already settled in §17.15-A5.
- **`AS OF KNOWN <T>`** (transaction-time / decision-time axis — RESOLVED HERE): when set,
  pins the entity-registry transaction-time to `T`, so "the exact registry state prism knew at
  decision-time T" governs all entity-resolution in the query. Absent this clause, queries use
  the LATEST (fresh) registry state — the empirically-dominant default per XTDB telemetry.
  [XTDB-docs; research §1.2]

**Fresh-by-default posture:** Queries resolve against the current/latest registry state unless
`AS OF KNOWN <T>` is explicitly specified. This matches the prior Q2 LEAN (live-snapshot) and
the XTDB "as-of-now" default, while making the non-reproducibility caveat *resolvable on demand*
by pinning the transaction-time axis. [research §5]

**Forensics / saved-findings use case:** When a finding is created (§14.5 replay-link), stamp
it with the decision-time `T` at creation time. Replaying the finding invokes `AS OF KNOWN T`
automatically — the analyst sees exactly what triggered the finding, even after later registry
corrections. This mirrors the C6 backtesting posture (snapshot-id + rule-version pin) and the
C7 per-update audit-trail decision (human accepted the storage cost of full historization where
audit-grade replay matters). [research §5; posture-consistent with C6/C7]

**Prism-novel differentiator:** No surveyed commercial security tool (Chronicle, Sentinel,
Splunk ES, ServiceNow CMDB) implements true Snodgrass bitemporality for entity resolution.
All use event-time + ingestion-time only; transaction-time-as-of for enrichment is essentially
absent. This is a DFIR prior-art gap that prism closes systematically. [research §1.3]

**HONEST COSTS (must not be glossed):**

- **`AS OF KNOWN <T>` entity-resolution half:** requires adding a transaction-time (system-
  versioned) second axis to the registry. Cost is bounded to the registry + schema catalog —
  NOT the high-volume event stream. Storage magnitude is real but unquantified (no surveyed
  source provides amplification ratios; must be measured on real registry churn).
  [research §1.4 — INCONCLUSIVE on magnitude]
- **`AS OF KNOWN <T>` data-snapshot half (C5 cold tier):** pinning the DATA snapshot for full
  "as-of-T" forensic reproducibility is a SEPARATE, COSTED item. DataFusion + `iceberg-rust`
  does NOT yet expose native Iceberg time-travel by snapshot-id/timestamp as of 2026.
  [research §3.3 — verified constraint]. The entity-resolution + OCSF-catalog-version halves
  of `AS OF KNOWN <T>` are achievable without native Iceberg time-travel; the data-snapshot
  half requires custom integration or upstream contribution. **Record data-snapshot pinning as
  a phased/cost-gated sub-item: entity-registry + catalog-version pinning ships in day-2 scope;
  cold-tier data-snapshot pinning is DEFERRED to a post-day-2 Iceberg time-travel milestone
  (new open item OQ-C8-DATASNAPSHOT).**
- **Two-axis cognitive cost for analysts.** Both axes present. Mitigation: the common
  interactive-hunt case stays single-axis-simple (omit `AS OF KNOWN` → fresh semantics).
  The transaction-time axis is "ubiquitous but opt-in" [XTDB-docs].
- **Composite identity key is unchanged.** Simultaneous IP→multi-asset mappings (NAT/overlap)
  still require `(observable, namespace/site)` regardless of bitemporality.

**SETTLED parts (unchanged from pre-fold, still hold):**

1. **Temporal binding model:** weak-tier observables resolve by **interval-containment as-of
   EVENT-TIME (default)**, closed-open `[valid_from, valid_to)` (SQL:2011). "Current" rows use
   far-future `valid_to = 9999-12-31`.
2. **`AS OF <expr>` (valid-time) clause** exists, defaults to EVENT TIME.
3. **`AS OF KNOWN <T>` (transaction-time) clause** ADDED by this resolution — pins registry
   txn-time (and, per D-C8-3 below, also the OCSF schema-catalog version) to a single `T`.
4. **Composite identity key:** `(observable, namespace/site)`.
5. **Strong-tier IDs bind exactly** (no temporal interval needed; §17.15-A5).
6. **Tier policy in registry;** optional query-level `USING STRONG` override.
7. **EXPLAIN / result metadata** MUST disclose: which axes are pinned vs fresh; the registry
   txn-time snapshot used; the live-federated-tier caveat (upstream API data not under prism
   version control → reproducibility is interpretation-only for sensor-live tier).

**PIV-C8-1 — Bitemporality storage axiom:** At implementation, verify that the registry table
carries BOTH a `[valid_from, valid_to)` application-time period AND a `[db_from, db_to)` /
`system_time_start` system-versioned period. A registry implementation that stores only valid-
time (SCD2-only) is a **P1 violation** of D-C8-2 as resolved. The absence of the transaction-
time axis makes `AS OF KNOWN <T>` semantically unimplementable.

**PIV-C8-2 — AS OF KNOWN default (fresh-by-default axiom):** At implementation and in every
adversary pass on stories touching entity resolution: confirm that queries WITHOUT `AS OF KNOWN
<T>` resolve against the LATEST transaction-time (fresh), NOT against a pinned snapshot.
Defaulting to a stale pin silently breaks live-hunt queries and violates the fresh-by-default
posture.

**PIV-C8-3 — `AS OF KNOWN <T>` scope discipline:** `AS OF KNOWN <T>` pins (a) entity-registry
txn-time and (b) OCSF schema-catalog version atomically as one decision-time coordinate (per
D-C8-3 unified model below). It does NOT automatically pin the C5 cold-tier data snapshot
(that is the deferred OQ-C8-DATASNAPSHOT item). Result metadata MUST distinguish: "entity
resolution and schema interpretation as of T" (achievable day-2) vs "data as of T" (deferred).

[research/prismql-asof-version-resolution-2026-06-27.md §1, §2, §4, §5]
[research/prismql-deliverables-depth-2026-06-27.md §Q2, §2.3–2.4 LEAN (generalized)]

---

### D-C8-3 — OCSF Version-Binding Model: RESOLVED 2026-06-27 (BITEMPORALITY + PINNABLE CATALOG)

**RESOLVED 2026-06-27. Previously DEFERRED (OQ-C8-OCSFVER). Research basis:
`research/prismql-asof-version-resolution-2026-06-27.md`. Unified with D-C8-2 under a single
`AS OF KNOWN <T>` decision-time primitive.**

**DECISION: Keep version-agnostic canonical OCSF names as the ergonomic default, but make the
SCHEMA-CATALOG VERSION an IMMUTABLE, PINNABLE artifact (Confluent schema-id lineage), bound by
the same `AS OF KNOWN <T>` decision-time knob as entity-resolution (D-C8-2).**

**The unified model — one decision-time knob, two interpretation layers:**

Both OQ-C8-ASOF (entity-resolution reproducibility) and OQ-C8-OCSFVER (OCSF version-binding
reproducibility) are structurally the same problem: a **mutable interpretation layer** between
raw data and query logic, where an update to that layer changes results for an unchanged query
over unchanged data. The transaction-time / "as-known-when" axis governs both:
- Fork A's txn-time axis = "when the registry learned this IP↔asset mapping."
- Fork B's txn-time axis = "which catalog version was in effect" (catalog-version IS a
  transaction-time stamp for schema interpretation).

Therefore **a single `AS OF KNOWN <T>` clause** pins BOTH the entity-registry transaction-time
(D-C8-2) AND the schema-catalog version to `T`. A forensic re-query with `AS OF KNOWN T`
reproduces "the world as prism interpreted it at decision-time T" — entity identity AND schema
semantics. [research §4]

**Schema-catalog versioning mechanics (Confluent schema-id lineage):**

- Each published catalog revision receives a **stable, immutable catalog-version ID** (analogous
  to Confluent schema-id). Once published, a catalog version is append-only; no in-place mutation.
- Version-agnostic canonical OCSF names remain the default query identifiers (`event.file.path`,
  etc.) — ergonomic, no per-query annotation required.
- The optional **`@ocsf:<ver>`** per-field pin is retained for version-sensitive fields where
  an analyst needs explicit control.
- When `AS OF KNOWN <T>` is set, the active catalog version is the one in effect at `T` — this
  binds both per-field mappings AND the compatibility-tier classification.
- When no `AS OF KNOWN <T>` is set, the LATEST published catalog version is used (fresh default).

**Compatibility tiers (prism-native, derived from OCSF version diffs):**

OCSF publishes NO compatibility tiers (confirmed by research: "not fully backward compatible at
minor versions"). Prism MUST DERIVE these from concrete OCSF 1.1→1.3→1.6 version diffs:
- **Stable fields:** safe for implicit cross-version mapping (additive-only, name/type/semantics
  unchanged across all supported OCSF versions). Map transparently; no pin required.
- **Version-sensitive fields:** renamed, type-changed, or semantically-shifted across versions.
  Flagged with diagnostic in the LSP server; require explicit `@ocsf:<ver>` pin or emit a
  compatibility warning. Value-level catalog lookup tables cover enum drift.
- **Tier derivation is prism-novel work** — no OCSF-supplied shortcut exists. Build from real
  OCSF 1.x version diffs before implementation. [research §3.4 — INCONCLUSIVE on OCSF tiers]

**Catalog-pin ≠ full data reproducibility (honest disclosure, mandatory):**

A schema-catalog-version pin reproduces *interpretation* (how fields are normalized) but does
NOT reproduce *raw-data drift* unless the underlying data is also versioned/snapshotted. For the
live federated sensor tier (upstream APIs not under prism version control), reproducibility is
fundamentally interpretation-only — the data may have changed in the upstream system. This MUST
be disclosed in EXPLAIN / result metadata. For the C5 cold tier (Iceberg), full reproducibility
requires data-snapshot pinning (the deferred OQ-C8-DATASNAPSHOT item). [research §3.3]

**Prism-novel differentiator:**

No surveyed tool documents per-source-version OCSF field binding with a pinnable catalog +
compatibility-tier classification. The unified `AS OF KNOWN <T>` knob spanning entity-registry
+ schema-catalog is also prism-novel as a combined surface (each half is literature-settled
individually). These together constitute "what did we know, and what was true, as of T" —
genuine bitemporality applied to DFIR reproducibility. [research §1.3, §4]

**HONEST COSTS:**

- **Catalog versioning machinery:** immutable catalog-version IDs + append-only cadence +
  LSP schema-catalog lookup against pinned version = non-trivial catalog management.
- **OCSF tier-derivation is new work.** No OCSF-supplied stable-vs-version-sensitive table.
  prism must build + maintain it from version diffs (OCSF 1.1→1.3→1.6). Ongoing maintenance
  cost as OCSF evolves.
- **Live-federated-tier honesty:** for sensor-API data the interpretation pins but the data
  does not. Result metadata must surface this distinction clearly to avoid analyst over-trust.
- **Data-snapshot half (OQ-C8-DATASNAPSHOT):** same as D-C8-2 — deferred, requires Iceberg
  time-travel work or upstream contribution.

**SETTLED parts (unchanged from pre-fold, still hold):**

1. **Canonical OCSF field names** are the query identifiers.
2. **`native.<source>.<field>` namespace** for raw source values (retain-originals, ASIM lineage).
3. **Compatibility tiers** (stable vs version-sensitive) in catalog — OCSF-derived by prism.
4. **Value-level mapping** for enum drift via catalog lookup tables.
5. **`@ocsf:<ver>` per-field pin** is retained for version-sensitive fields.
6. **`AS OF KNOWN <T>` also pins the catalog version** (unified with D-C8-2).
7. **Fresh-by-default:** absent `AS OF KNOWN <T>`, latest catalog version is active.

**PIV-C8-4 — Catalog immutability invariant:** At implementation, verify that the schema-
catalog storage model makes published catalog versions append-only / immutable. An implementation
that mutates an existing catalog version in place breaks `AS OF KNOWN <T>` reproducibility
for all previously-issued queries pinned to that version. This is a **P1 violation**.

**PIV-C8-5 — Version-sensitive field diagnostic coverage:** At implementation, the LSP server
MUST flag version-sensitive fields (per the derived OCSF compatibility-tier table) with a
diagnostic when no `@ocsf:<ver>` pin is present. Silent cross-version mapping of a
version-sensitive field is a **P1 finding** (may silently produce wrong results across
mixed-version sources).

**PIV-C8-6 — Honest result-metadata disclosure:** Every query result envelope MUST include:
(a) whether `AS OF KNOWN <T>` was set or fresh; (b) the catalog-version-id used; (c) for
the live-sensor tier, the explicit label "interpretation-only reproducibility — upstream data
not version-controlled." Omitting the live-tier caveat is a **P1 finding** under the
production-grade principle (analysts must not over-trust "reproducible").

[research/prismql-asof-version-resolution-2026-06-27.md §3, §4, §5]
[research/prismql-deliverables-depth-2026-06-27.md §Q4, §4.3–4.4 LEAN (extended)]

---

## Confirmed Leans

These leans were presented in the research, confirmed by the human (non-objection) on 2026-06-27,
and are captured as decided implementation directions for morph-time ADR authorship.

### L-C8-1 — Entity-Pivot Grammar: FIND Keyword + entity() Predicate + AS OF Clause

**Confirmed lean (research Q1 + §12.1 two-surface design). Resolves the §12.1 open "FIND vs
PIVOT vs SEARCH" question toward FIND.**

**KEYWORD = `FIND`** (resolves §12.1 open): `PIVOT` collides with SQL PIVOT; `SEARCH` is
overloaded (ambiguous with full-text search). `FIND` has SPL/Chronicle-search-bar lineage and
unambiguously signals "pivot from this observable across all related data."

**Two surfaces are the right design (confirmed):**

1. **Standalone `FIND` statement** (ergonomic shorthand; SPL/Chronicle-search lineage):
   ```
   FIND ip '10.0.0.5' SINCE 24h ACROSS crowdstrike, splunk AS OF EVENT TIME
   FIND user 'alice@example.com' BETWEEN '2026-06-01' AND '2026-06-25'
   ```
   `FIND` maps to exploratory pivot — type a value, see everything related to it.

2. **`entity()` predicate composable in any WHERE** (KQL/SQL lineage):
   ```
   SELECT * FROM federated
   WHERE entity('ip', '10.0.0.5', as_of => event_time)
     AND severity >= 'high'
   SINCE 24h
   ```
   `entity()` maps to embedded filter — anchor an entity constraint in a broader query.

**Extended EBNF grammar sketch** (extends §12.1 DRAFT with Q2 AS OF hook and Q4 native-namespace):
```ebnf
(* ---- Standalone FIND ---- *)
entity_pivot  ::= "FIND" entity_type entity_value as_of? time_bound? source_scope? tier_hint?
entity_type   ::= "ip" | "user" | "host" | "domain" | "email" | "hash" | "cve" | IDENT
entity_value  ::= STRING
as_of         ::= "AS" "OF" ( "EVENT" "TIME" | timestamp | expr )  (* default: EVENT TIME *)
time_bound    ::= "SINCE" duration | "BETWEEN" timestamp "AND" timestamp
source_scope  ::= "ACROSS" ident ( "," ident )*
tier_hint     ::= "USING" ( "STRONG" | "STRONG" "," "WEAK" )       (* default: registry policy *)
duration      ::= INT ( "s" | "m" | "h" | "d" | "w" | "mo" )      (* §12.3 E1 *)

(* ---- entity() predicate (composable inside any WHERE) ---- *)
entity_pred   ::= "entity" "(" STRING "," STRING ( "," as_of_arg )? ")"
as_of_arg     ::= "as_of" "=>" ( "event_time" | expr )

(* ---- Field references: canonical OCSF (optional @ver pin) OR native namespace ---- *)
field_ref     ::= ocsf_path ( "@" "ocsf" ":" version )?            (* canonical OCSF field *)
              | native_ref                                          (* raw source value *)
native_ref    ::= "native" "." ident "." ident
```

**Planner contract** (extends §12.1 — all clauses honor the settled decisions):
1. **Registry expansion** (settled) — `entity('ip', X)` ⇒ disjunction over the registry's
   ordered IP attribute paths: `src_endpoint.ip = X OR dst_endpoint.ip = X OR device.ip = X …`
2. **As-of binding** (D-C8-2 settled parts) — weak-tier observables resolve via
   interval-containment as-of the as-of expr (default = EVENT TIME) against `[valid_from,
   valid_to)` intervals. Strong-tier IDs bind exactly. Tier policy = registry; `USING` override
   = optional query-level narrowing.
3. **Pushdown** (settled C3) — per the capability descriptor, push attribute predicates that
   sources support natively; evaluate the rest centrally post-OCSF-normalization. Cross-source
   join cost = C3 cost-based-degrade stack (NOT reopened).
4. **Mandatory time-bound** (settled §5.3) — absent `SINCE`/`BETWEEN` ⇒ inject default window.
5. **Fan-out / normalize / merge / partial-result metadata** (settled §3.6).
6. **RETAIN interaction** — a `FIND` result may be retained as `FROM cache.<name>` (confirm in
   morph per §12.1 open note "likely yes").

**Piped form illustration** (same logical plan as SQL form — D-C8-1 desugar):
```
FIND ip '10.0.0.5' SINCE 24h ACROSS crowdstrike, splunk AS OF EVENT TIME
| where severity >= 'high'
| summarize count() by device.hostname
```

[research/prismql-deliverables-depth-2026-06-27.md §Q1, §1.3 LEAN; §"Recommended grammar sketch"]

---

### L-C8-2 — Multi-Hop Pivot: SQL/PGQ GRAPH_TABLE as Forward-Compat Target Only (NOT Day-2)

**Confirmed lean (research Q1 §"Multi-hop is the gap").**

The day-2 `FIND` / `entity()` surfaces cover **single-hop observable fan-out** (one disjunction
across the registry's attribute paths for that observable type). Real investigative pivots want
multi-hop: `IP → asset → user → other-assets-of-that-user`.

**Day-2 does NOT ship multi-hop.** The forward-compatibility commitment is:

The day-2 `FIND` / `entity()` grammar MUST NOT foreclose multi-hop pivot syntax. Reserve a
path-pattern surface that would lower to **SQL/PGQ `GRAPH_TABLE` semantics** (SQL:2023 part 16:
`GRAPH_TABLE(g MATCH (a)-[r]->(b) WHERE … COLUMNS(…))` over graph *views* defined on relational
tables) — the only SQL-compatible multi-hop precedent that fits a tabular DataFusion core.

**Implementation status of GRAPH_TABLE in DataFusion: [model-knowledge — not version-verified].**
DataFusion does not implement SQL/PGQ GRAPH_TABLE today. This is the correct reason to defer
multi-hop to post-day-2, not a design objection. The forward-compat reservation prevents the
day-2 grammar from blocking the multi-hop extension when DataFusion support improves.

[research/prismql-deliverables-depth-2026-06-27.md §Q1 §"What prior art adds" #2, §Caveats]

---

### L-C8-3 — Authoring Intelligence: Single LSP Server Reused by Three Consumers

**Confirmed lean (research Q5).**

Build PrismQL's authoring intelligence as a single **LSP server** reused by THREE consumers:
1. The **S2 Monaco console** (browser editor; `monaco-languageclient` + WebSocket → LSP)
2. The **CLI** (terminal; Chumsky `Rich` errors rendered via ariadne color-coded spans)
3. The **NL→PrismQL agent's validate-repair loop** (the NL2KQL two-stage judge: generate →
   validate against schema + parser feedback → repair; see L-C8-4)

**Implementation:**

- **Parser errors:** Chumsky `Rich` error type throughout the grammar — `span()`, `expected()`,
  `contexts()` (labelled contexts), `try_map` / `try_map_with` for semantic validation (unknown
  field detection), `labelled` / `labelled_with` for human-meaningful context on every rule.
  All Context7-verified against upstream Chumsky docs.
- **CLI rendering:** ariadne (color-coded source-mapped diagnostics with labeled spans + notes).
  The standard Chumsky companion; community-standard pairing [model-knowledge — bounded].
- **Monaco / LSP:** translate Chumsky `Rich` span + `expected()` iterator to LSP
  `textDocument/publishDiagnostics`. Completions (`textDocument/completion`) and quick-fixes
  (`textDocument/codeAction`) are LSP server logic; schema-aware logic lives IN THE SERVER, not
  Monaco.

**Four catalogs exposed to the LSP server** (the schema-awareness source):
1. **Grammar / operator metadata** — operator/keyword set + per-position expectation
   (Chumsky `expected()` / `RichPattern` already yields "what was valid here")
2. **OCSF-per-version schema** — canonical OCSF field list per version (C8 §settled/D-C8-3);
   completion and unknown-field diagnostics
3. **Native-field schema** — per-source `native.<source>.<field>` field list
4. **Entity registry** — registered entity types + observable types

**Implemented diagnostics** (KQL IntelliSense lineage):
- **Unknown field → nearest-match suggestion** (reverse IntelliSense: "unknown field
  `event.fil.path`; did you mean `event.file.path`?")
- **Missing time-bound → quick-fix** (inject `SINCE 24h`; the §5.3 mandatory-time-bound NFR
  surfaced as an editor action, not just a runtime error)
- **Planner-derived semantic diagnostics** — DataFusion `LogicalPlanBuilder` type errors
  (group-by a field not in the pipeline, type mismatch) mapped back to source spans via
  `LogicalPlanBuilder` stage construction [Context7:datafusion]

**Honest cost:** the LSP server + schema catalogs + Monaco integration is a substantial,
separate engineering investment. The piped surface alone (D-C8-1) will NOT deliver
"KQL-grade approachable." The tooling carries as much learnability weight as the syntax.
[research Q3/Q5]

[research/prismql-deliverables-depth-2026-06-27.md §Q5, §5.4 LEAN; §Q3 §3.2 #1]

---

### L-C8-4 — NL→PrismQL: Reuse Parser/Planner Diagnostics as Validate-Repair Signal

**Confirmed lean (research Q3b + §12.3 E8).**

NL→PrismQL is agent-native (already §12.3 E8 ADOPT; embedded agent S3, §11.3). The C8 lean is:
reuse the SAME parser/planner diagnostics (L-C8-3) as the NL→PrismQL agent's validate-repair
signal — the **NL2KQL two-stage judge pattern** [NL2KQL paper]:

1. Agent generates a candidate PrismQL expression.
2. The LSP server (or parser-direct API) validates + type-checks + plans the expression WITHOUT
   executing it and returns span-precise diagnostics.
3. Diagnostics (unknown field, type mismatch, missing time-bound, parse error) are fed back to
   the agent as structured repair signals.
4. Agent repairs and re-submits until the expression passes validation.

**Guardrails that are already the safety net (no new agent-specific cost machinery needed):**
- **Schema grounding:** the LSP server's catalog constrains completions and flags unknown fields;
  hallucinated fields are rejected by the parser/planner.
- **Validation-before-execution:** the validation step (step 2 above) runs before any sensor
  fan-out. No execution cost for an invalid query.
- **Hallucinated-field defend + repair:** unknown-field → nearest-match (L-C8-3) doubles as the
  agent's repair signal.
- **Mandatory-time-bound NFR** (§5.3): an agent that omits a window gets the injected default
  (D-C3-2) + disclosure in the result envelope — cost safety net, not silent runaway.
- **C3 cost-based-degrade** (ADR-PROP-capability-descriptor-pushdown): an agent that writes an
  expensive cross-source join gets cap + flag + cost-disclosure in the result envelope.

**Prompt-injection defense for the agent harness is out of scope here** (see project memory
`project_agent_harness_design.md`).

[research/prismql-deliverables-depth-2026-06-27.md §Q3b, §3.4 LEAN]

---

### L-C8-5 — Recipe / Detection-as-Code Format: Sigma-Aligned Metadata + Semver + CI Harness

**Confirmed lean (research Q6 + §14.7 recipe library).**

A PrismQL rule/recipe is: **query text + Sigma-aligned metadata block**.

**Recipe metadata schema** (Sigma lineage [Sigma-rules-doc]):
```yaml
rule_id: "8e3f4a2b-..."          # stable UUID; preserved on Sigma import as external_id
version: "1.0.0"                 # semver; major = breaking entity/schema/logic change
title: "..."
description: "..."
author: "..."
date: "2026-06-27"
modified: "2026-06-27"
status: experimental | stable | deprecated
severity: informational | low | medium | high | critical
required_entities:
  - ip
  - user
required_data_sources:
  - crowdstrike
  - splunk
tags:
  - attack.t1078                  # ATT&CK; Sigma `tags` format preserved 1:1
false_positives:
  - "..."
references:
  - "..."
```

**Semver policy:**
- **Major bump** = breaking change to entity types, OCSF field paths, or detection logic.
- **Minor bump** = new fields in EMIT / MEASURES; widened time window; new data source.
- **Patch bump** = doc update, label change, no behavioral change.

**Git storage:** recipes are stored in Git under the same repo as detection rules. Subject to
peer review and CI validation. Same lifecycle as detection rules (Elastic `detection-rules` +
Splunk Security Content lineage).

**CI harness:**
Ship a CI harness that runs recipes against Arrow/Parquet fixtures via DataFusion with expected
match/non-match assertions. Fixture format: `<rule_id>/fixture-match.parquet` +
`<rule_id>/fixture-no-match.parquet` + expected output rows. This gives recipes the same
test-gate discipline that code gets — a recipe that passes CI is validated against known data.

**Sigma import mapping** (metadata 1:1; logic via entity-registry/canonical-fields):
- `id` → `rule_id` (preserved as canonical external ID + source origin annotation for lineage)
- `title` / `description` / `author` / `date` / `status` / `level` → PrismQL metadata 1:1
- `tags` (ATT&CK) → preserved 1:1
- `logsource` → entity-registry + schema-mapping selection
- `detection` block → PrismQL `WHERE` / `JOIN` / `MATCH_RECOGNIZE` (`SEQUENCE…THEN`) for
  `temporal` / `temporal_ordered` Sigma correlations

**Day-2 scope = docs-guided import** (§12.3 E9). The **automated translator is deferred** to
E-RULE-XLATE-001 (§12.3 D1). The recipe library ships NOW with hand-authored Sigma→PrismQL
translation examples that validate the mapping surface and serve as test vectors for the eventual
pySigma backend (per C6 L-C6-3: "Ship Sigma→PrismQL examples in the recipe library NOW").

[research/prismql-deliverables-depth-2026-06-27.md §Q6, §6.3 LEAN]

---

## Open Questions

| # | Question | Status | Dependency |
|---|---------|--------|------------|
| **OQ-C8-ASOF** | Entity-resolution AS OF reproducibility: live-registry snapshot vs frozen-registry-version vs bitemporality | **RESOLVED 2026-06-27** — BITEMPORAL registry (valid-time + transaction-time). `AS OF KNOWN <T>` pins registry txn-time. Fresh-by-default. See D-C8-2. PIV-C8-1/2/3 added. | D-C8-2 resolved |
| **OQ-C8-OCSFVER** | OCSF version-binding: version-agnostic canonical names + catalog reconciliation vs explicit `@ocsf:<ver>` pin vs catalog-version-pinning for reproducibility | **RESOLVED 2026-06-27** — Pinnable immutable catalog versions (Confluent lineage). `AS OF KNOWN <T>` also pins catalog-version (unified with D-C8-2). Fresh-by-default. See D-C8-3. PIV-C8-4/5/6 added. | D-C8-3 resolved |
| **OQ-C8-DATASNAPSHOT** | Cold-tier data-snapshot pinning for full `AS OF KNOWN <T>` reproducibility: DataFusion + `iceberg-rust` lacks native time-travel as of 2026. Custom integration or upstream contribution required to make the data-snapshot half of `AS OF KNOWN <T>` work for the C5 Iceberg cold tier. | **OPEN** — cost-gated; deferred post-day-2. Entity-registry + catalog-version pinning ships day-2; data-snapshot pinning = future Iceberg time-travel milestone. **CRITICAL DISTINCTION (ADS conformance 2026-06-27; P-ADS-04; CONFLICT-6 resolution):** The Option 3 Central CMEK result cache (PAT-ADS-02 Tenant-Keyed-Central-Cache) is NOT a substitute for OQ-C8-DATASNAPSHOT. The Central cache stores QUERY OUTPUTS (result rows) at a point in time. Forensic replay of the same query against the same INPUT data requires INPUT-level snapshots (Iceberg time-travel). These are structurally different: output caching enables "re-view this result later"; input-level snapshots enable "re-run this exact query against the data as it was." Story-writers MUST NOT treat the Central cache as closing OQ-C8-DATASNAPSHOT — they are separate concerns. PIV-C8-3 governs: `AS OF KNOWN <T>` pins interpretation (entity resolution + schema version, day-2); it does NOT pin the C5 cold-tier data snapshot (deferred). | New; added by C8 FOLD 2026-06-27 |
| **OQ-C8-NATIVE-RESIDENCY** | The `native.<source>.<field>` namespace and its interaction with §13.6 multi-schema descriptor + the A7 per-field residency tag model. A `native.*` ref may carry `raw` residency class — interacts with the residency policy artifact (whether a raw-residency native field ref is pushable only if the source is in a permitted residency zone). | Open architect decision at morph | Before implementing the native field namespace |
| **OQ-C8-RECIPE-SCHEMA** | Exact recipe-format Sigma-metadata schema (field types, required vs optional, validation rules) + CI fixture shape (Parquet fixture format, match/no-match assertion schema) | Open design at morph | Before implementing the recipe library or CI harness |
| **OQ-C8-GRAPHTABLE-GRAMMAR** | Whether the day-2 `FIND` / `entity()` grammar needs a concrete grammar reservation (placeholder syntax) for the SQL/PGQ GRAPH_TABLE multi-hop forward-compat target (L-C8-2), or whether "do not foreclose" is sufficient without a syntax placeholder | Open architect decision at morph | Before finalizing §12.1 grammar ADR |

---

## Downstream SAP-1 Obligations (Note — Not Actioned Here)

Several new event types implied by C8 decisions will need BC-2.16.002 Canonical Structured
Event Catalog rows at morph time. Flagged here; NOT actioned (SAP-1 probe scope is per-story
at implementation time, not at ADR-PROP capture time).

- **`event_type = "query.pipe.desugar"` (or equivalent)** — emitted when a piped query is
  desugared to a SQL/DataFusion logical plan; fields include the pipe expression, the desugared
  SQL, and query_id; audit role = DSL-debugging transparency (ties D-C8-1 "show desugared SQL").
- **`event_type = "query.entity_resolution.as_of"` (or equivalent)** — emitted when an
  `entity()` / `FIND` resolution binds to a registry interval; fields include entity_type,
  observable, as_of_time, registry_snapshot_version, tier_used, bound_entity_count; audit role =
  entity-resolution audit trail.
- **`event_type = "query.lsp.unknown_field_suggestion"` (or equivalent)** — emitted when the
  LSP server flags an unknown field and produces a nearest-match suggestion; fields include
  field_ref, nearest_match, similarity_score; audit role = tooling observability (optional;
  morph-time decision whether to catalog this or treat as a client-side diagnostic).
- SAP-1 obligations for piped-operator desugar-decision, pushdown-disclosure, and injected-window
  events may also need BC-2.16.002 catalog rows — these tie C3/C6 SAP-1 obligations already
  flagged in `ADR-PROP-capability-descriptor-pushdown.md` and `ADR-PROP-detection-engine-depth.md`.

All flagged above; BC-2.16.002 amendment is morph-time work.

---

## Honest Costs

| Cost | Description |
|------|-------------|
| **Piped surface is a desugar layer, not a query planner.** | The pipe → DataFusion `LogicalPlanBuilder` translation must be correct for every operator combination. Each new operator (join across pipe stages, entity() in a pipe, SEQUENCE…THEN as terminal pipe stage) is a new desugar case. Incorrect desugar is a semantics bug, not a style issue. |
| **DSL debugging is a real UX cost.** | Analysts debugging a pipe query that produces unexpected results must inspect the desugared SQL / EXPLAIN. Shipping "show desugared SQL" as optional is NOT acceptable. It is a day-1 requirement. |
| **Learnability claims are unproven.** | PRQL, pql, KQL all assert ergonomic benefits; none provide controlled studies. The convergent practice is "piped + tooling" — not "piped alone." The LSP server is as load-bearing as the pipe syntax for the approachability thesis. |
| **Version-aware OCSF binding is ahead of prior art.** | No surveyed tool documents per-source-version OCSF field binding with a compatibility-tier catalog. The design is analogically grounded but must be validated against real OCSF 1.x version diffs before implementation. Cost: building + maintaining a per-version OCSF schema catalog with compatibility tiers and value-mapping tables. |
| **Multi-hop deferred but grammar must not foreclose it.** | SQL/PGQ GRAPH_TABLE is the only SQL-compatible multi-hop precedent. DataFusion does not implement it today [model-knowledge — not version-verified]. The day-2 grammar reservation is cheap; implementing it is expensive. Do not let the forward-compat reservation become scope-creep. |
| **Bitemporal registry adds a second storage axis (resolved D-C8-2/D-C8-3).** | The transaction-time axis is required to make `AS OF KNOWN <T>` work. Storage cost is bounded to the registry + catalog (not the event stream), but magnitude must be measured on real registry churn. Data-snapshot pinning for the C5 cold tier requires additional Iceberg time-travel work (OQ-C8-DATASNAPSHOT, deferred). EXPLAIN / result-metadata design must distinguish interpretation-pinned (day-2) from data-pinned (deferred). |
| **LSP server is a substantial separate investment.** | Language Server Protocol + Monaco integration + ariadne CLI + four schema catalogs + planner-derived diagnostics is a multi-week engineering deliverable. Do not underestimate it relative to the grammar itself. |

---

## Alternatives Considered and Rejected

### Alternative A: SQL-Only Surface (No Piped Sugar)

Ship only the SQL surface (DataFusion + Chumsky grammar) with no piped ergonomic layer.

**Rejected (D-C8-1) because:**
- The §12.3 FSQL Ergonomics Parity Ledger (CONFIRMED 2026-06-25) explicitly places a piped
  ergonomic on-ramp as a desirable goal. The Ledger states "adopt" for E2 SUMMARIZE and E3
  entity pivot — both of which are most naturally expressed in a piped context.
- PRQL and RunReveal pql prove the piped-over-SQL pattern is Rust-native and viable; there is
  no technical risk argument for deferral.
- SQL-only would not achieve the "approachable for SPL/KQL-trained analysts" goal even with
  excellent tooling; the syntax-expectation mismatch is real.
- Deferring the pipe surface to post-day-2 means building the GRAMMAR TWICE (SQL-only grammar
  now, then adding pipe syntax later) — more costly than building it in-scope.

### Alternative B: Piped Surface as a Separate Compiler (Not Desugar-to-SQL)

Build the piped surface as a separate query compiler that generates DataFusion physical plans
directly, bypassing the SQL layer entirely.

**Rejected (D-C8-1) because:**
- This is a second query engine — exactly the architecture explicitly refused. PRQL and pql
  both prove that desugar-to-SQL is the correct architecture.
- A separate compiler would not inherit the SQL surface's existing DataFusion optimizer,
  MATCH_RECOGNIZE operator, or EXPLAIN tooling automatically.
- Two compilers = double the correctness surface; two semantic models = divergence risk.

### Alternative C: LSP Server Per Consumer (Monaco-Specific; CLI-Specific; Agent-Specific)

Build three separate diagnostic implementations — one for Monaco, one for CLI, one for the
NL→PrismQL agent.

**Rejected (L-C8-3) because:**
- Schema-awareness (the load-bearing part of autocomplete quality) must be consistent across
  all three consumers; duplicating it creates divergence bugs.
- The NL→PrismQL validate-repair loop and the analyst's Monaco diagnostics must agree on what
  constitutes a valid expression; a shared server guarantees this.
- Three implementations of Chumsky `Rich` → LSP / ariadne / structured-repair-signal translation
  is unnecessary engineering cost.

### Alternative D: Automated Sigma Translator in Day-2

Build the automated Sigma → PrismQL translator in day-2 scope rather than deferring to
E-RULE-XLATE-001.

**Rejected (L-C8-5 / §12.3 D1 / L-C6-3) because:**
- The deferral is HUMAN-CONFIRMED (§12.3 D1, CONFIRMED 2026-06-25): "day-2 ships only the docs
  guides (E9); the automated translator is a candidate follow-up epic (E-RULE-XLATE-001)."
- The `pySigma`-style backend (a full ProcessingPipeline targeting OCSF taxonomy) is non-trivial
  engineering. The fidelity-report-for-lossy-edges (L-C6-3) adds additional scope.
- The recipe library can ship hand-authored Sigma→PrismQL examples now (C6 L-C6-3 directive).
  These examples serve as documentation AND as test vectors for the eventual pySigma backend,
  so the deferred E-RULE-XLATE-001 implementation does not start from zero.

---

## Ripple Effects (Must Be Picked Up at Morph Time)

| Affected area | Ripple |
|---------------|--------|
| **§12.1 PrismQL entity/observable pivot** | The EBNF grammar in §12.1 DRAFT should be updated at morph to adopt the L-C8-1 `AS OF` clause, the `tier_hint` modifier, and the `native.<source>.<field>` field_ref. The `FIND` keyword choice is RESOLVED (§12.1 open question "FIND vs PIVOT vs SEARCH" → FIND). |
| **§12.3 FSQL Ergonomics Parity Ledger** | The pipe surface (D-C8-1) is the concrete deliverable that makes the §12.3 ADOPT dispositions real. At morph, confirm the pipe-operator set matches the §12.3 E2/E3/E7/E8 items. |
| **§14.7 recipe library** | L-C8-5 defines the recipe format (Sigma-aligned metadata schema + semver policy + CI harness). §14.7 should be amended at morph to incorporate this metadata schema. Sigma→PrismQL examples ship with the recipe library. |
| **E-PRISMQL-GRAMMAR-001** | Primary proposed epic. Covers: piped surface desugar layer; `FIND` / `entity()` grammar extension with `AS OF` + `tier_hint`; LSP server (four catalogs, diagnostics, quick-fixes); Monaco integration; ariadne CLI rendering; NL→PrismQL validate-repair loop. |
| **E-RULE-XLATE-001** | Deferred automated Sigma translator. L-C8-5 defines the pySigma-style backend architecture and fidelity-report requirement (inherited from C6 L-C6-3). Not shipped in day-2. |
| **BC-2.16.002 §Postconditions** | SAP-1 obligations listed in §Downstream SAP-1 Obligations above (morph-time BC work). Key: piped-desugar event, entity-resolution AS OF audit event, possibly LSP-unknown-field-suggestion event. |
| **ADR-TBD: PrismQL piped surface + LSP server** | This ADR-PROP covers D-C8-1 + L-C8-1..5. The real ADRs (allocated at morph, ADR-NNN+) formalize: the desugar-to-DataFusion contract; the LSP server catalog schema; the bitemporal entity-resolution model + `AS OF KNOWN <T>` grammar (D-C8-2 resolved); the OCSF version-binding + pinnable catalog model (D-C8-3 resolved); PIV-C8-1..6 implementation invariants. |
| **ADR-TBD: Recipe format + CI harness** | L-C8-5 formalized as a separate ADR covering the metadata schema, semver policy, Parquet fixture format, and Sigma import contract. |
| **matured-vision §16.4** | C8 decision block appended (2026-06-27). |
