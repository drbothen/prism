---
document_type: research
produced_by: research-agent
status: capture
do_not_execute: true
timestamp: "2026-06-27"
program: day2-vision-side-analysis
scope: C8 — AS-OF / version-resolution reproducibility forks (Q2 entity as-of + Q4 OCSF version-binding)
boundary: OUT-OF-BAND — SEPARATE from the live VSDD factory pipeline
non_contradiction_basis:
  - "prismql-deliverables-depth-2026-06-27.md Q2 (entity resolution AS OF; interval-containment as-of EVENT-TIME default; the live-snapshot non-reproducibility caveat)"
  - "prismql-deliverables-depth-2026-06-27.md Q4 (OCSF canonical names + per-source-version catalog; optional @ocsf:<ver> pin; compatibility tiers)"
  - "matured-vision-day2-requirements.md §17.15-A5 (prism-native config-driven entity registry; strong/weak tiers; temporal IP↔asset validity) — SETTLED INPUT, not reopened"
  - "C5 schema-version axis + cold-tier (Iceberg snapshot-id) — referenced, not reopened"
  - "C6 backtest determinism (snapshot-id + rule-version pin) — referenced as posture precedent"
  - "C7 ML-replay decision: human chose FULL per-update audit-trail (changelog + materialization) — tested as posture signal"
settled_items_NOT_relitigated:
  - "Entity REGISTRY model (config-driven entity_type→attribute-paths, strong/weak tiers, temporal validity)"
  - "OCSF schema-version axis exists; sources lag (1.1/1.3) vs upstream 1.6"
  - "Interval-containment as-of EVENT-TIME is the resolution mechanism (settled §17.15-A5 / Q2 LEAN)"
---

# PrismQL — AS-OF / Version-Resolution Reproducibility (C8 Q2 + Q4, depth pass)

> **CAPTURE artifact. `do_not_execute: true`.** Research input for a later architect/PO morph of
> the two C8-deferred decisions. Does NOT amend any live spec, BC, ADR, story, STATE.md,
> SESSION-HANDOFF.md, RESEARCH-INDEX.md, or any prior research file. NOT git-added. "Leans"
> below are discussion input only — not decisions.
>
> **Non-contradiction confirmed.** Read against `prismql-deliverables-depth-2026-06-27.md` (C8
> Q2 + Q4). This pass does NOT reopen the entity registry model or the OCSF schema-version axis
> (both SETTLED). It researches the **resolution semantics + reproducibility** the prior pass
> flagged as deferred. Where this pass refines a prior LEAN, it is called out explicitly; nothing
> here contradicts a settled piece.

---

## How to read this document

The human deferred **two coupled forks**, both "resolve-as-of-WHEN, and is it reproducible?":

- **Fork A (Q2 entity as-of):** LIVE registry snapshot (fresh, non-reproducible) vs FROZEN
  registry-version (reproducible, stale until re-pin).
- **Fork B (Q4 OCSF version-binding):** version-agnostic canonical names + updatable catalog
  (ergonomic, mapping can drift → reproducibility hazard) vs explicit `@ocsf:<ver>` pin
  (reproducible, verbose).

This document researches **bitemporality FIRST** (§1) because it may dissolve BOTH forks into a
single model, then characterizes each fork (§2 Fork A, §3 Fork B), the interaction (§4), and the
reproducibility-vs-freshness context question (§5). It closes with per-fork **LEANs**, a concrete
**reproducibility-model recommendation**, **Consolidated Open Design Questions**, and **Honest
Costs & Caveats**, marking each finding **literature-settled** vs **prism-novel**.

Citation convention: bracketed source-tags (e.g. `[Snodgrass]`, `[SQL2011]`, `[Confluent-compat]`,
`[XTDB-docs]`, `[NIST-800-86]`) refer to the live-web sources returned by the Perplexity
deep-research / ask calls logged in **Research Methods**. Claims resting on the model are tagged
`[model-knowledge]`; thin-evidence areas are tagged `[INCONCLUSIVE]`.

---

## §1 — Bitemporality as the candidate unifying answer (researched FIRST)

### 1.1 The model (literature-settled)

Bitemporal data modeling separates two **orthogonal** time axes [Snodgrass][SQL2011]:

- **Valid-time** (a.k.a. application-time, business-time, effective-time): *when a fact is true
  in the modeled world*. For prism: when an IP actually held an asset (the DHCP-lease reality).
- **Transaction-time** (a.k.a. system-time): *when the system stored, modified, or deleted the
  fact*. For prism: when the registry **learned** of that mapping (including late corrections).

The temporal-database literature (Snodgrass's *Developing Time-Oriented Database Applications in
SQL* is the canonical reference) formalizes this separation, and **SQL:2011 is the first SQL
standard to make it native** [SQL2011]:

- **Application-time period tables** (valid-time): declared with a `PERIOD FOR` annotation over
  two timestamp columns, **closed-open `[start, end)` semantics**, with `WITHOUT OVERLAPS`
  constraints and automatic **period splitting** on partial updates (`FOR PORTION OF`).
- **System-versioned tables** (transaction-time): the DBMS automatically maintains each row's
  database-validity period on insert/update/delete; queried via `AS OF SYSTEM TIME` /
  `VERSIONS BETWEEN SYSTEM TIME …`.
- **Bitemporal tables**: application-time + system-versioning **combined** — query either axis or
  both. SQL:2011 also adds Allen-interval temporal predicates (`CONTAINS`, `OVERLAPS`, `PRECEDES`,
  `IMMEDIATELY PRECEDES`, …). [SQL2011]

**The crux — one model, both human options.** A bitemporal registry answers, in a single query
[Snodgrass][SQL2011][XTDB-docs]:

- **"As-of event-time `e`, as the registry knew it at decision-time `d`"** — bind the observable
  to whatever the registry believed at transaction-time `d` for valid-time `e`. This is the
  **reproducible / audit-grade** query: re-running it with the same `d` reproduces the exact
  binding the analyst saw, *even if the mapping was later corrected*.
- **"As-of event-time `e`, as the registry knows it NOW"** — bind using the latest
  transaction-time, valid-time `e`. This is the **fresh / late-corrections-included** query.

This is precisely the two options the human posed in Fork A — and the **same structure underlies
Fork B** (catalog-version = a transaction-time analog for schema interpretation). **Bitemporality
is the literature-settled framework in which the reproducibility–freshness tradeoff is a *knob*,
not a fork.** [Snodgrass][SQL2011]

A practitioner framing worth noting: the "Dolt blog" / bitemporal-historization literature names
the two axes a **state timeline** ("when a value was valid in reality") and an **assertion
timeline** ("when the organization knew about that value"), and explicitly says bitemporal
historization "makes it possible to ensure the correct data is available for every audit" — letting
auditors reconstruct both *what was true* and *what the org believed* at a past time. [Dolt-bitemporal]

### 1.2 Bitemporality in practice (literature-settled, with one verified correction)

| System | Bitemporal? | How the two axes surface | Implementation | Cost notes |
|---|---|---|---|---|
| **XTDB (v2, formerly Crux)** | Yes — "ubiquitous but opt-in" | `FOR VALID_TIME` + `FOR SYSTEM_TIME AS OF`; `FOR PORTION OF VALID_TIME` for retroactive edits; queries specify a "basis" timestamp per axis | **JVM / Clojure** — NOT Rust. **But v2 "Core 2" engine is built on Apache Arrow** for polyglot interop [XTDB-impl] | "partially persistent + fully retroactive" indexes; storage amplification from multi-version docs [XTDB-docs] |
| **Datomic** | Transaction-time primary (not full bitemporal) | `asOf` by transaction-id or instant; immutable assertion/retraction log | JVM/Clojure | indelible log → storage grows with history |
| **Dolt** | Git-style versioned SQL | `AS OF` / `VERSIONS BETWEEN` by timestamp or commit-id; corrections = new commits | Go | commit-graph traversal cost; storage per commit |
| **SQL Server / mainstream RDBMS** | System-versioned tables (transaction-time); app-time periods | `FOR SYSTEM_TIME AS OF`; `PERIOD FOR` | per-vendor | history table storage |

> **VERIFIED CORRECTION to the deep-research output:** the deep-research pass flagged XTDB's
> implementation language as *not answerable from its sources*. A targeted lookup confirms **XTDB
> 2.x is Clojure/JVM, not Rust**, but its **Core-2 query engine uses Apache Arrow** as its columnar
> substrate. [XTDB-impl] → For prism this means XTDB is **not a drop-in Rust dependency**, but its
> Arrow-native design is architecturally adjacent to prism's DataFusion/Arrow core — XTDB is a
> reference *model*, not a reusable *component*. **[prism-relevant]**

XTDB's own usage telemetry is a useful design signal: it documents that **the vast majority of
queries use the system-time default "as best known," and within those most use the valid-time
default "as of now"** — i.e., users default to *fresh on both axes* and reach for the audit/as-known-then
semantics only deliberately. [XTDB-docs] This validates a **fresh-by-default, pin-on-demand** posture
(see §5).

### 1.3 Do commercial security tools / CMDBs implement bitemporality? (the key negative finding)

**No surveyed commercial security tool or CMDB implements true Snodgrass bitemporality for entity
resolution.** [research-synthesis — Fork-A + bitemporal files]

- **Chronicle / Google SecOps** exposes an **Asset view** where an investigator enters a
  hostname / client-IP / MAC **plus a timestamp** (defaulting to current UTC) — i.e., it
  acknowledges the need to specify *event-time* for asset lookup, but the sources do **not** show a
  separate **transaction-time / "as-known-then"** axis. [Chronicle-asset-view]
- **Microsoft Sentinel** (on ADX / Log Analytics) stores logs with **event-time + ingestion-time**
  and can query log ranges, but the corpus shows **no explicit bitemporal entity-resolution
  semantics**. [Sentinel]
- **Splunk Enterprise Security** indexes by **event-time + index-time**, offers a Forensics tab —
  again event-time-oriented, **no transaction-time-as-of for enrichment**. [Splunk-ES]
- **ServiceNow CMDB** records `Valid from` / `Valid to` on *some* discovery patterns (e.g. MS
  Certificate Authority) and `Last discovered` timestamps, but the sources do **NOT** show that
  the CMDB core implements transaction-time / bitemporal tables. [ServiceNow-CMDB]

**Conclusion (prism-novel finding):** event-time + ingestion-time is widespread; **transaction-time
as a queryable "as-known-then" axis for entity resolution is essentially absent from productized
security tooling.** The deep-research pass states this directly: prism's proposed design would be
"a genuinely novel application of bitemporal entity resolution in DFIR." [research-synthesis]
This is the same "ahead-of-prior-art" posture the prior pass flagged for Q4 OCSF binding — the
*mechanism* is literature-settled (SQL:2011/Snodgrass), but its *application to a security entity
registry* is prism-novel and must be validated by prism, not assumed to fall out of a tool.

### 1.4 The cost of bitemporality (literature-settled, quantities INCONCLUSIVE)

- **Storage amplification** — multi-version rows across two axes; valid-time updates trigger
  interval *splitting* which further multiplies rows. The deep-research is explicit that **no
  source gives a quantitative ratio** of historical-to-current data, so any number would be
  speculative. **[INCONCLUSIVE — magnitude]** [bitemporal-file]
- **Query complexity** — analysts must reason over two interval axes; queries must combine
  predicates across both. Mitigated by good defaults (most queries omit both axes → "current
  state"). [XTDB-docs]
- **Cognitive load** — the second axis is a real conceptual tax; bitemporality is "ubiquitous but
  opt-in" precisely so the common case stays simple. [XTDB-docs]
- **Mitigation** — bitemporal metadata (`system_to` / `valid_to`) enables **tiered archival**: cold
  history compresses to cheaper storage without hurting live-query performance. [bitemporal-file]

**When is bitemporality OVERKILL vs right?** [bitemporal-file]
- **Overkill** for atemporal or append-only event data that is never corrected (the raw OCSF event
  stream itself — events don't get re-mapped).
- **Right** for "data that represent evolving states and decisions" — **asset↔observable mappings,
  user roles, configuration changes**. The prism entity registry is *exactly* this class. The
  storage cost "is real but manageable" for a registry (small relative to the event volume it
  enriches).

> **§1 takeaway:** Bitemporality is the literature-settled model that **collapses Fork A into a
> default-plus-knob** rather than a fork, and supplies the conceptual frame for Fork B's
> catalog-versioning. It is **prism-novel as applied to a DFIR entity registry** (no commercial
> precedent). Cost is real but bounded *because it applies to the registry, not the event stream*.

---

## §2 — Fork A: entity AS-OF reproducibility (Q2)

### 2.1 The three options characterized (literature-settled mechanics)

| Option | Semantics | Reproducible? | Fresh? | Cost | Prior art |
|---|---|---|---|---|---|
| **Live snapshot** | resolve event-time `e` against the **current** registry | **No** — re-query after a registry edit can change the binding | **Yes** — late/corrected mappings reflected immediately | cheapest; single-axis (valid-time only) | the prior pass's Q2 LEAN default; matches XTDB "as-of-now" default [XTDB-docs] |
| **Frozen version** | pin the registry to a transaction-time `T`; resolve `e` under `FOR SYSTEM_TIME AS OF T` | **Yes** — same `T` → same binding | **No** — corrections after `T` invisible until re-pin (operational friction, "stale" risk) | store registry versions (system-versioned table); single explicit pin | SCD2 + SQL:2011 system-versioning [SCD2][SQL2011] |
| **Bitemporal (both)** | resolve `e` (valid-time) at knowledge-time `T` (transaction-time); EITHER axis defaultable | **Yes** (pin `T`) **or** **Yes-fresh** (latest `T`) — *both, per query* | **Yes** (latest `T`) | both axes stored; max storage/complexity (§1.4) | XTDB / SQL:2011 bitemporal [XTDB-docs][SQL2011] |

**Critical mechanical clarification (literature-settled):** the prior pass's Q2 LEAN already
adopts **SCD2 / SQL:2011 application-time `[valid_from, valid_to)` interval-containment as-of
EVENT-TIME**. The deep-research confirms an important nuance: **a Type-2 SCD alone is
single-axis** — it can reconstruct *"what we **currently** believe was true at event-time `e`"*,
but it **cannot** reconstruct *"what we believed at `e` **according to knowledge as of** transaction-time
`T`"*. [SCD2] To get the frozen-version (reproducible) option, SCD2's valid-time must be **combined
with transaction-time** — i.e., system-versioning. So:

- **Live-snapshot = SCD2 / app-time only** (the prior pass's current model).
- **Frozen-version = SCD2 app-time + system-versioning** (add the second axis).
- **Bitemporal = the same two axes, with the second axis exposed as a query knob** rather than a
  per-investigation pin.

**This means the frozen-version option and the bitemporal option require the *same underlying
storage* (both axes); they differ only in whether the transaction-time axis is exposed as a
per-query knob (bitemporal) or a coarser per-investigation pin (frozen-version).** That is the
pivotal finding for Fork A: **once you pay for the second axis to get reproducibility at all,
bitemporality is nearly free as a query-surface choice.** **[prism-relevant synthesis]**

### 2.2 How forensic tools actually handle this today (literature-settled negative)

- **Forensic timeline tools (Plaso/log2timeline, Timesketch, Velociraptor)** prioritize
  **event-level reproducibility** — re-analyzing the *same raw events* transparently — **NOT
  versioned enrichment layers.** Timesketch advertises "reproducible and transparent" timeline
  analysis, but the corpus indicates this means *event-level* reproducibility (same timeline
  events, same analysis steps), **not** a versioned IP↔asset enrichment layer. The sources do
  **not** show any of these tools maintaining historical versions of entity-resolution mappings.
  **[INCONCLUSIVE for these specific tools — public docs sparse]** [Plaso/Timesketch/Velociraptor]
- **The standards gap:** **NIST SP 800-86** (forensic guide) and chain-of-custody guidance
  emphasize reproducible forensic *processes* and evidence integrity, but **do not specify how to
  handle mutable enrichment layers** like an asset registry. Chain-of-custody attaches to
  *evidence artifacts* (disk images, logs, memory dumps), not to the *enrichment join* that maps
  an IP to an asset. [NIST-800-86] In practice, analysts capture "what was known when" in **case
  notes / report snapshots** — a human-level, ad-hoc reproducibility mechanism, not a system one.
  **This human-level workaround is exactly the gap a bitemporal registry would close
  systematically.** **[prism-novel opportunity]**

**Synthesis for prism (literature-settled + prism-novel):** the forensic/audit world *needs*
"reproduce the exact finding the analyst acted on" (legal defensibility), but **lacks a productized
mechanism for late-arriving entity corrections** — they fall back to case-note snapshots. The
deep-research is explicit: live-snapshot vs frozen vs bitemporal has "little explicit, productized
precedent in mainstream security tools." prism solving this systematically is the differentiator,
but also means prism is **ahead of prior art** and must validate the design itself.

### 2.3 Fork A LEAN

> **LEAN (Fork A):** Adopt the **bitemporal registry** (valid-time interval-containment + a
> transaction-time axis), because — per §2.1 — **the frozen-version option already forces you to
> store the transaction-time axis, so bitemporality is the near-free generalization** that gives
> *both* the human's options from one model. Default behavior: **fresh on both axes** (matches
> XTDB's empirically-dominant default and the prior pass's live-snapshot LEAN), with an explicit
> **`AS OF KNOWN <T>`** (transaction-time) knob layered on the already-leaned **`AS OF <event-time>`**
> (valid-time) clause. Keep the prior pass's settled mechanics: closed-open `[valid_from, valid_to)`
> intervals (SQL:2011), composite identity key (observable + namespace/site) for NAT/overlap.
> **This does not contradict the prior Q2 LEAN — it generalizes it:** the prior pass's
> "live-snapshot is better for retrospective investigation, at the cost of non-reproducibility"
> becomes "live-snapshot *is the default*, and the non-reproducibility caveat is *resolved on
> demand* by pinning the transaction-time axis."

---

## §3 — Fork B: OCSF version-binding reproducibility (Q4)

### 3.1 The reproducibility hazard, made precise (literature-settled)

The hazard: with version-agnostic canonical names + an **updatable** reconciliation catalog, "the
same query executed at different times over the same underlying data may produce different results,
because the normalization step changes the semantics of fields **before** the query logic is
applied." [research-synthesis-B] This is **structurally identical to Fork A's hazard** — a mutable
interpretation layer between the raw data and the query — but for *schema* rather than *entity
identity*. The catalog-update event is the transaction-time analog of a registry correction.

OCSF specifics (literature-settled, with one ambiguity): OCSF **follows semantic versioning** and
publishes a `version` file; the project provides a self-governance process for producers/consumers.
**But the corpus confirms OCSF minor bumps are "not fully backward compatible in practice"** — OCSF
may treat some minor changes as semantically breaking even when structurally compatible — **and OCSF
does NOT document explicit compatibility tiers** beyond semver adherence. [OCSF-versioning]
**[INCONCLUSIVE — OCSF's own tier guidance]:** there is no OCSF-published "stable vs version-sensitive"
field classification; prism would have to **derive** one (see §3.4).

### 3.2 Confluent Schema Registry — the version-pinning precedent (literature-settled)

Confluent Schema Registry is the mature reference for "does pinning a version make interpretation
reproducible?" — and the answer is **yes, when you pin the exact schema-id** [Confluent-compat]:

- Schemas are versioned per **subject** (a named schema lineage); each registration gets a
  **stable, immutable schema-id + incremented version**.
- The serializer **embeds the schema-id in the message payload**; the consumer fetches *that* schema
  to interpret the bytes → **deterministic, reproducible interpretation** of a given message.
- Compatibility **modes**: `BACKWARD` (default), `FORWARD`, `FULL`, and **transitive** variants
  (`BACKWARD_TRANSITIVE`, etc.). **Crucially, the default `BACKWARD` is *non-transitive*** — it
  checks only against the immediately previous version, NOT the entire history. [Confluent-compat]
- **The reproducibility caveat:** registry governance optimizes for **producer/consumer
  compatibility, NOT automatically for reproducibility of analytical results**. A consumer that
  references the *latest* version (rather than a pinned id) is exposed to drift — **"reinforcing the
  importance of explicit version pinning for reproducible analytics."** [Confluent-compat]

This is the direct precedent for Fork B: **version-agnostic = "track latest" = ergonomic but
drift-exposed; `@ocsf:<ver>` pin = "pin the schema-id" = reproducible.** The two are the same
tradeoff Confluent already documents.

### 3.3 Iceberg snapshot pinning — the stronger analog, and its limit (literature-settled + prism-relevant)

The human's question: does pinning the schema-catalog version give reproducibility "the same way
Iceberg snapshot-id does"? The deep-research answer is **partially, with an important gap**
[Iceberg]:

- **Iceberg snapshots version the *complete table state* — data files **AND** schema** — and
  time-travel queries (`AS OF` snapshot-id / timestamp) reproduce the exact historical state. This
  is **stronger** than a schema-registry version pin because it captures **data evolution + physical
  layout + schema** together. [Iceberg]
- **A schema-catalog version pin is analogous only at the *interpretive layer*:** if the catalog
  version (the source-field → canonical-OCSF mapping logic) is **immutable once published**, then
  re-running the same query over the same raw data with the same catalog version reproduces the
  same normalized data → same results. **But** catalog-version pinning "binds only the mapping
  logic, leaving the underlying data in external systems that may themselves evolve" — so **a
  catalog pin alone does NOT guarantee reproducibility unless the raw data is *also* versioned /
  snapshotted.** [research-synthesis-B]

> **prism tie to C5 (referenced, not reopened):** this is precisely why the C5 cold-tier (Iceberg
> snapshot-id) matters — for *full* forensic reproducibility you need BOTH the catalog-version pin
> (interpretation) AND the data-snapshot pin (the bytes). For the live federated tier (querying
> sensor APIs, where the upstream data is NOT under prism's version control), catalog-version
> pinning reproduces the *interpretation* but cannot reproduce *upstream data that has since
> changed*. **Disclose this honestly** in EXPLAIN/result metadata. **[prism-relevant caveat]**
>
> **VERIFIED constraint:** Apache **DataFusion + `iceberg-rust` does NOT yet expose native Iceberg
> time-travel by snapshot-id/timestamp** as of 2026 (unlike Spark/Trino/Athena). [DataFusion-iceberg
> — flagged uncertain at source]. So the C5 "pin the cold-tier snapshot" mechanism is **not free in
> the current Rust stack** — it would need custom integration or upstream contribution. This is a
> real cost, not a given. **[prism-novel cost]**

### 3.4 Compatibility tiers (literature-settled pattern; prism must derive the OCSF instance)

The "stable vs version-sensitive field" split is well-precedented:
- **ASIM** uses **mandatory / recommended / optional** field tiers — mandatory in every parser,
  recommended normalized when available, optional source-dependent. [ASIM] This maps to "stable
  (safe for implicit cross-version mapping) vs version-sensitive (needs explicit handling)."
- **Avro / Protobuf evolution rules** are the field-level precedent: Avro matches fields **by name**
  (allowing add-with-default, remove-optional, reorder); Protobuf guidance is to **avoid changing
  names/types/order of existing members**, and to **create a new type rather than mutate** when a
  breaking change is needed. [Avro-Protobuf]
- **OCSF itself does not publish tiers** (§3.1) — so **prism must derive a compatibility-tier table
  from concrete OCSF 1.x version diffs** (the prior pass flagged this; this pass confirms there is
  no OCSF-supplied shortcut). **[prism-novel — validate against real OCSF 1.1→1.3→1.6 diffs]**

### 3.5 Fork B LEAN

> **LEAN (Fork B):** Ship **version-agnostic canonical OCSF names as the default** (ergonomic,
> matches the prior Q4 LEAN), but make the **catalog VERSION an immutable, pinnable artifact**
> (Confluent-schema-id lineage) and bind it as part of any reproducible context. Keep the optional
> **`@ocsf:<ver>`** pin for *individual version-sensitive fields*. Derive **compatibility tiers**
> (ASIM mandatory/recommended/optional lineage) from concrete OCSF version diffs — stable fields
> map implicitly across versions; version-sensitive fields require a pin or emit a diagnostic.
> **This does not contradict the prior Q4 LEAN — it adds the missing reproducibility primitive:**
> the prior pass leaned "version-agnostic default + optional `@ocsf` pin"; this pass adds that the
> *catalog itself must be versioned and pinnable* (not just individual fields), because an
> unversioned-but-updatable catalog is the actual reproducibility hazard. Disclose the
> raw-data-not-pinned caveat (§3.3) in result metadata.

---

## §4 — The interaction: ONE as-of knob for both axes?

The human's pivotal question: a single forensic re-query may need BOTH as-of-event-time entity
resolution AND as-of-version schema binding to be reproducible — does **one mechanism** serve both?

**Finding (synthesis; literature-settled mechanics, prism-novel composition):** Forks A and B are
**the same problem in two layers** — a mutable interpretation layer (entity identity / schema
mapping) sitting between raw data and query logic, where an update changes results for an unchanged
query over unchanged data. Both are governed by a **transaction-time / "as-known-when" axis**:
- Fork A's transaction-time axis = "when the registry learned this IP↔asset mapping."
- Fork B's transaction-time axis = "which catalog version was in effect" (the catalog-version *is*
  a transaction-time stamp for schema interpretation).

Therefore a **single "as-of decision-time" knob** can pin **both** the registry transaction-time
**and** the schema-catalog version to the same point `T` — reproducing "the world as prism
interpreted it at decision-time `T`." This is the **bitemporal `decision-time`** the Dolt/practitioner
literature names [Dolt-bitemporal], generalized from entity-only to entity+schema.

> **LEAN (interaction — UNIFY):** Expose **one reproducibility primitive** — a single
> **`AS OF KNOWN <T>`** (decision-time / knowledge-time) clause — that, when set, pins BOTH (a) the
> registry transaction-time axis (Fork A) AND (b) the active schema-catalog version (Fork B) to the
> consistent state as of `T`. The valid-time / event-time axis (`AS OF <event-time>`, already
> settled §17.15-A5 / Q2) remains a **separate, orthogonal** clause — because event-time answers
> "*when in the world*" while decision-time answers "*as prism knew it when*," and conflating them
> would re-break the very separation bitemporality exists to provide. **Net: TWO orthogonal as-of
> axes in the grammar (event-time + decision-time), but ONE decision-time knob spanning both the
> registry and the schema catalog.** This is prism-novel as a *unified* surface; each half is
> literature-settled.

**Tie to C5/C6 (referenced, not reopened):** the decision-time pin `T` should *also* select the C5
cold-tier data snapshot (Iceberg snapshot-id) and the C6 rule-version, so that a single pinned `T`
reproduces interpretation (catalog) + identity (registry) + data (snapshot) + logic (rule). The
C6 backtest-determinism posture (snapshot-id + rule-version pin) is the *same pattern* — this pass
recommends the entity-registry and schema-catalog axes **join** that existing pin rather than
inventing a parallel one.

---

## §5 — Reproducibility vs freshness: when does each matter, and what's the default?

**Literature-settled framing:** security operations have a **dual mandate** — *reproducible,
defensible reconstructions of past decisions* AND *continuous improvement of the picture as new
information arrives.* [research-synthesis] These genuinely conflict; bitemporality is the framework
that lets you serve both *without choosing globally* — you choose **per query / per context**.

**Context-dependent default (prism-relevant recommendation):**

| Context | Need | Recommended default |
|---|---|---|
| **Forensic / IR report / legal-hold / audit** | Reproduce the **exact** finding the analyst acted on; legally defensible | **Reproducible — pin decision-time `T`** (freeze registry txn-time + catalog version + data snapshot + rule version). Ties to finding replay-link §14.5. |
| **Detection / saved finding / scheduled rule** | Deterministic, replayable | **Reproducible — pin at finding-creation time** (mirrors C6 backtest snapshot-id + rule-version pin) |
| **Live hunting / ad-hoc triage / exploration** | Best-known-now mapping; freshest corrections | **Fresh — latest registry txn-time + latest catalog** (matches XTDB's empirically-dominant "as-of-now" default) [XTDB-docs] |

**This context-split is consistent with the C7 signal.** The human chose the **full per-update
audit-trail** (changelog + materialization) for ML-model replay — a posture that *values audit-grade
reproducibility* and is *willing to pay storage for it*. The same posture applies here: prism should
**store the transaction-time axis** (the cost) so that the **reproducible-pinned** option exists for
findings/forensics/audit, while keeping **fresh** as the default for interactive hunting. The C7
decision is a precedent that the human accepts the storage cost of full historization where
audit-grade replay matters — which is the §1.4 cost objection's answer. **[posture-consistent]**

**The finding replay-link (§14.5) is the natural home for the pin:** when a finding is created, stamp
it with the decision-time `T` (and thereby the registry txn-time + catalog version + data snapshot +
rule version). Replaying the finding resolves `AS OF KNOWN T` automatically → the analyst sees
*exactly* what triggered the finding, even after later corrections. Ad-hoc re-runs without the stamp
get fresh semantics. **[prism-relevant; ties §14.5 + C6 + C7]**

---

## Per-fork LEANs (consolidated; discussion input only)

- **Fork A action (entity AS-OF):** Adopt the **bitemporal registry** — valid-time interval-containment
  (settled) + a **transaction-time axis**. Default fresh-on-both; expose an explicit decision-time
  pin. Justification: the *frozen-version* option already forces the second axis, so bitemporality
  is the near-free generalization that delivers BOTH human options. *Generalizes — does not
  contradict — the prior Q2 live-snapshot LEAN.*
- **Fork B action (OCSF version-binding):** Keep version-agnostic canonical names as the default
  (settled), but make the **schema-catalog VERSION an immutable, pinnable artifact** (Confluent
  schema-id lineage), with the optional **`@ocsf:<ver>`** field-level pin and **derived compatibility
  tiers** (ASIM lineage; OCSF publishes none — prism must derive from version diffs). *Adds the
  missing reproducibility primitive to the prior Q4 LEAN.*
- **Does bitemporality unify them? YES.** Both forks are a mutable-interpretation-layer
  reproducibility hazard governed by a transaction-time axis. A **single `AS OF KNOWN <T>`
  decision-time knob** pins both the registry txn-time and the catalog version (and should also
  select the C5 data snapshot + C6 rule version). Event-time (valid-time) stays a separate
  orthogonal axis. **Two as-of axes in the grammar; one decision-time knob spanning registry +
  schema + data + rule.**

---

## The reproducibility model recommendation (concrete)

A **bitemporal interpretation layer** with **two orthogonal as-of axes** and a **single unifying
decision-time pin**:

1. **Valid-time / event-time axis** (settled §17.15-A5 / Q2): `AS OF <event-time>` (default = the
   row's event-time) — *"resolve identity/schema as the world was at event-time."* Closed-open
   `[valid_from, valid_to)` intervals; composite identity key (observable + namespace/site).
2. **Transaction-time / decision-time axis** (this pass): `AS OF KNOWN <T>` (default = latest /
   "now") — *"as prism knew it at decision-time `T`."* When set, pins:
   - the **entity-registry** transaction-time (Fork A),
   - the **schema-catalog version** (Fork B),
   - (recommended tie, not reopened) the **C5 data snapshot-id** and **C6 rule-version**.
3. **Defaults by context** (§5): fresh-on-both for interactive hunt; decision-time-pinned for
   findings / forensic / audit (stamped at finding-creation via §14.5 replay-link, mirroring C6 +
   C7 posture).
4. **Honest disclosure** in EXPLAIN/result metadata: which axes are pinned vs fresh, and the
   raw-data-not-under-prism-control caveat for the live federated tier (§3.3).
5. **Storage scope discipline:** the transaction-time axis applies to the **registry + catalog**
   (evolving-state data) — NOT the raw event stream (append-only, never re-mapped). This bounds the
   §1.4 storage cost to the small enrichment layer, not the high-volume telemetry.

---

## Consolidated Open Design Questions (for architect/PO morph of §12/§13)

1. **Adopt the second (transaction-time) axis on the registry at all?** (Fork A) — buys
   reproducibility; costs the storage/complexity of system-versioning. *Decision owner: architect
   (storage/perf) + PO (audit requirement).*
2. **Grammar surface for decision-time** — `AS OF KNOWN <T>` keyword vs reuse SQL `FOR SYSTEM_TIME
   AS OF`? Keep it **separate** from the event-time `AS OF` (per §4) — confirm two distinct clauses.
3. **Unify or separate the decision-time pin across registry + catalog + C5 snapshot + C6 rule?**
   This pass leans UNIFY (one `T` selects all). Confirm the cross-subsystem coupling is acceptable.
4. **Catalog versioning model** — immutable published catalog versions (Confluent-schema-id
   lineage)? Cadence (OCSF practitioner guidance suggests annual upgrades + unused-column removal)?
5. **Compatibility-tier derivation** — who builds + maintains the OCSF stable-vs-version-sensitive
   tier table from version diffs? (OCSF publishes none — prism-novel work.)
6. **Default-by-context policy** — formalize fresh-for-hunt / pinned-for-findings (§5)? Where is the
   pin stamped (the §14.5 replay-link)?
7. **C5 cold-tier reproducibility gap** — DataFusion+iceberg-rust lacks native time-travel (verified
   §3.3). Custom integration, upstream contribution, or accept that live-tier reproducibility is
   interpretation-only (catalog+registry pinned, data not)? *Architect decision.*
8. **Live-federated-tier honesty** — for sensor-API data NOT under prism version control, a pin
   reproduces *interpretation* but not *upstream data drift*. How to surface this in result metadata
   so analysts don't over-trust "reproducible"?

---

## Honest Costs & Caveats

- **[prism-novel — no commercial precedent]** Bitemporal entity resolution for DFIR is essentially
  absent from productized security tools (Chronicle/Sentinel/Splunk-ES/ServiceNow use event-time +
  ingestion-time, NOT transaction-time-as-of for enrichment). The deep-research calls prism's design
  "a genuinely novel application." The *mechanism* (SQL:2011/Snodgrass) is literature-settled; the
  *application* is prism-novel and must be validated by prism, not assumed.
- **[INCONCLUSIVE — storage magnitude]** No surveyed source quantifies bitemporal storage
  amplification ratios; any number would be speculative. The cost is bounded by applying the second
  axis to the **registry + catalog only** (not the event stream), but the actual ratio must be
  measured on prism's real registry churn.
- **[INCONCLUSIVE — OCSF tiers]** OCSF follows semver but publishes **no compatibility tiers** and
  is "not fully backward compatible" at minor versions. prism must **derive** the stable-vs-version-sensitive
  tier table from concrete OCSF 1.1→1.3→1.6 diffs — there is no OCSF-supplied shortcut. (Confirms the
  prior pass's Q4 "ahead-of-prior-art" flag.)
- **[INCONCLUSIVE — forensic tool internals]** Plaso/Timesketch/Velociraptor public docs are sparse
  on entity-resolution + late-correction handling; the finding that they do event-level (not
  enrichment-layer) reproducibility is the best available reading, not a documented guarantee.
- **[prism-novel cost — verified]** DataFusion + `iceberg-rust` does NOT yet expose native Iceberg
  time-travel (snapshot-id/timestamp) as of 2026. The C5 "pin the data snapshot" half of full
  reproducibility is **not free in the current Rust stack** — needs custom work or upstream
  contribution. (Source flagged uncertain; treat as a risk to confirm, not a settled blocker.)
- **[Caveat — catalog pin ≠ full reproducibility]** A schema-catalog-version pin reproduces
  *interpretation* but NOT *raw-data drift* unless the data is also snapshotted (Iceberg-style). For
  the live federated tier (upstream sensor APIs not under prism version control), reproducibility is
  fundamentally interpretation-only — MUST be disclosed honestly.
- **[Caveat — cognitive cost]** Two as-of axes is a real analyst tax. XTDB's "ubiquitous but opt-in,
  fresh-by-default" posture is the mitigation: the common case stays single-axis-simple; the second
  axis appears only when reproducibility is explicitly invoked.
- **[Caveat — composite key, carried from prior pass]** Simultaneous IP→multi-asset mappings
  (NAT/overlap) still require a composite (observable + namespace/site) identity key regardless of
  the bitemporality decision; the second axis does not remove this.
- **[Scope honesty]** This pass did NOT reopen the entity registry model, the OCSF schema-version
  axis, the C5 cold-tier, C6 backtest, or C7 ML-replay decisions — they are referenced as settled
  inputs / posture precedents only. Leans are discussion input, not decisions.

**Literature-settled vs prism-novel summary:**
- *Literature-settled:* bitemporal model + SQL:2011 semantics; SCD2 single-axis limitation; Confluent
  version-pin reproducibility; Iceberg snapshot model; ASIM/Avro/Protobuf field-evolution tiers;
  reproducibility-vs-freshness dual mandate.
- *prism-novel (must validate):* bitemporal entity resolution applied to a DFIR registry; the
  unified single-`T` decision-time knob spanning registry + schema-catalog (+ C5/C6); the derived
  OCSF compatibility-tier table; the DataFusion+Iceberg time-travel gap as a C5 cost.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 3 | (1) Bitemporality as unifying answer — Snodgrass/SQL:2011 valid-time vs transaction-time, XTDB/Datomic/Dolt, SIEM/CMDB "as-known-then vs now", cost/overkill. (2) Fork A — point-in-time entity resolution reproducibility: SCD2/SQL:2011 dimension reconstruction, DFIR forensic-timeline tools (Plaso/Timesketch/Velociraptor), NIST SP 800-86/chain-of-custody, Chronicle/Sentinel late-correction handling, live-vs-frozen-vs-bitemporal cost. (3) Fork B — schema version-binding reproducibility: Confluent compatibility modes + schema-id pinning, Iceberg snapshot vs catalog-version pinning, OCSF semver specifics + compatibility tiers, ASIM/Avro/Protobuf field-evolution. ALL at reasoning_effort=high. |
| Perplexity perplexity_reason | 0 | — |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_ask | 2 | Verify XTDB 2.x implementation language (Clojure/JVM + Arrow Core-2 engine) — corrected the deep-research INCONCLUSIVE flag; verify DataFusion + iceberg-rust native time-travel support (negative as of 2026, source-flagged-uncertain). |
| Context7 | 0 | — (no library-API documentation claim required beyond the two factual verifications above; SQL:2011/OCSF/Confluent are standards/specs, not Rust libraries) |
| Tavily (any) | 0 | — |
| WebFetch | 0 | — |
| WebSearch | 0 | — |
| Training data | 1 area (flagged) | Snodgrass as the canonical temporal-DB reference is [model-knowledge]; the bitemporal *mechanics* attributed to him are corroborated by the SQL:2011 deep-research output, not training data alone. All version/standard facts (SQL:2011 features, OCSF semver, Confluent modes, Iceberg model, XTDB impl) are web-sourced. |

**Total MCP tool calls:** 5 (3 perplexity_research at effort=high + 2 perplexity_ask for targeted verification).
**Training data reliance:** low — all substantive findings are web-sourced via Perplexity deep-research at reasoning_effort=high; the two `perplexity_ask` calls were used precisely to *replace* a model-knowledge gap (XTDB language) and to verify a registry-state claim (DataFusion/Iceberg) rather than rely on training data.

**MCP availability:** All MCP tooling available and exercised; no MCP-UNAVAILABLE escalation required.
**Resilience note:** All three `perplexity_research` calls completed at reasoning_effort=high on first attempt (no overload fallback to medium required). Each response exceeded the inline token cap and was persisted to a tool-result file; all three were read in full via targeted Grep extraction (regex-windowed passages across every major section — bitemporal fundamentals/SQL:2011/XTDB/Datomic/Dolt/SIEM-CMDB/cost; Fork-A SCD2/forensic-tools/NIST/options-cost; Fork-B Confluent/Iceberg/OCSF/tiers/catalog-pin/interaction) because the single-line JSON exceeded the Read token limit. Extraction coverage spanned every section heading found in each file.
