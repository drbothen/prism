---
document_type: research
produced_by: research-agent
status: capture
do_not_execute: true
timestamp: "2026-06-27"
program: day2-vision-side-analysis
scope: C8 — OPEN PrismQL-deliverable design questions (depth pass)
boundary: OUT-OF-BAND — SEPARATE from the live VSDD factory pipeline
non_contradiction_basis:
  - "matured-vision-day2-requirements.md §12 / §12.1 (entity-pivot grammar + join-guard NFR)"
  - "matured-vision-day2-requirements.md §12.3 (FSQL Ergonomics Parity Ledger — CONFIRMED)"
  - "matured-vision-day2-requirements.md §14.2.1 / §12.4 (SEQUENCE…THEN sugar → MATCH_RECOGNIZE)"
  - "matured-vision-day2-requirements.md §17.15 lean #5 / A5 (prism-native config-driven entity registry, strong/weak tiers, temporal validity)"
  - "ADR-PROP-capability-descriptor-pushdown.md (C3 join-guard: cost-based-degrade — NOT reopened here)"
settled_items_NOT_relitigated:
  - "Join-guard posture (C3 cost-based-degrade, outer/non-equi allowed, audited override hint)"
  - "SEQUENCE…THEN…WITHIN sugar + MATCH_RECOGNIZE desugaring (§14.2.1 + C6 detection pass)"
  - "Entity REGISTRY model (config-driven entity_type→attribute-paths, strong/weak tiers, temporal validity)"
---

# PrismQL Deliverables — DEPTH Research Pass (C8 open grammar/ergonomics questions)

> **CAPTURE artifact. `do_not_execute: true`.** This is research input for a later
> architect/PO morph cycle of `matured-vision-day2-requirements.md §12`. It does NOT
> amend any live spec, BC, ADR, story, STATE.md, SESSION-HANDOFF.md, or RESEARCH-INDEX.md,
> and is NOT git-added. "Leans" below are discussion input only — not decisions.
>
> **Non-contradiction confirmed.** Read against the five settled artifacts in frontmatter.
> The settled pieces (cost-based-degrade join-guard; SEQUENCE…THEN desugaring; the entity
> registry model) are treated as FIXED inputs that the open grammar/ergonomics deliverables
> *consume*. Nothing below reopens them.

---

## How to read this document

Six open depth questions (Q1–Q6, with Q3b on NL→query). Each carries: **cited prior art**,
explicit **[model-knowledge]** / **[INCONCLUSIVE]** flags where evidence was thin, and a
**LEAN** (discussion input). The document closes with **Consolidated Open Design Questions**,
a **Recommended entity-pivot grammar sketch**, and **Honest Costs & Caveats**. A
**Research Methods** table ends the file (MCP-call audit).

Citation convention: bracketed source-tags below (e.g. `[Confluent-Flink-temporal]`,
`[Sigma-rules-doc]`) refer to the live-web sources returned by the Perplexity deep-research
and reason calls logged in Research Methods. Where a claim rests on the model rather than a
retrieved source, it is tagged `[model-knowledge]`. Library-API claims for Chumsky and
DataFusion were verified against Context7-served upstream docs (tagged `[Context7:chumsky]`
/ `[Context7:datafusion]`) and are NOT model-knowledge.

---

## Q1 — Entity / Observable Pivot Grammar (the `FIND` / `entity()` construct, §12.1)

### Prior-art landscape — how security/analytics languages express "pivot from this observable to everything related"

The deep-research pass found a clean **three-pattern spectrum**, ordered by how much entity
semantics the *language* (vs the analyst) carries:

**Pattern A — explicit join-key binding (event-centric; field list lives in the query).**
- **Splunk SPL `transaction`** groups events sharing explicit field values within `maxspan`
  (`| transaction clientip maxspan=30m`). The pivot axis is positional/explicit; SPL infers
  *no* entity semantics from the field. Documented to scale poorly and discouraged for
  wide correlation. [Splunk-transaction-ref]
- **Elastic EQL `sequence by` / `sample by`** — `by` declares join keys shared across events
  (`sequence by host.name, user.name`). Response surfaces `hits.sequences.join_keys`. Field
  list is always explicit; EQL forbids field-to-field comparison and has no entity registry.
  [Elastic-EQL-syntax]
- **KQL `join … on Computer, TargetUserName`** with explicit temporal `where` constraints —
  the dominant Sentinel correlation idiom; no first-class entity object. [MS-Sentinel-correlation]
- **Sigma correlations** (`event_count` / `value_count` / `temporal` / `temporal_ordered`)
  with `group-by` + `aliases` (the alias map reconciles differently-named fields across
  sources — `user.name` ↔ `AccountName`). Note: `group-by` is hyphenated; `group_by` is
  *silently ignored*. [Sigma-rules-doc]

**Pattern B — schema/registry-driven entity fields (the pivot field-set is metadata, not query text).**
- **Splunk data models + Pivot UI** expand a chosen attribute to underlying SPL field paths;
  schema-driven, but aggregation-oriented (pivot-to-events, not graph traversal). [Splunk-datamodel]
- **Google Chronicle/SecOps UDM "Entity Context in Search"** — `graph.entity.user.email_addresses = "…"`,
  `graph.entity.ip = "8.8.8.8" and graph.metadata.entity_type = "ASSET"`. The `graph.entity.*`
  namespace IS a schema-driven registry of where entity attributes live; the engine generates
  YARA-L behind the scenes. First/Last-Seen aggregations attach time to entities. **This is the
  closest mature analog to PrismQL's settled registry** — but it is *type-specific* (you query
  `graph.entity.ip` vs `graph.entity.hostname` separately; there is no generic `entity()`
  predicate that fans across all paths for a type). [Chronicle-entity-context]

**Pattern C — embedded graph pattern sublanguage (entities are first-class nodes).**
- **KQL graph operators** — `make-graph SrcIp --> DstIp with Nodes on Ip | graph-match
  (s)-[e]->(d) where s.Ip == "10.0.0.5" project …`. Pattern matching anchored at the
  observable; a `cycles` option (`all`/`none`/`unique_edges`) guards traversal. Requires an
  explicit graph-construction step; **no built-in temporal semantics** (time must be modeled
  as edge properties and filtered manually). [KQL-make-graph][KQL-graph-match]
- **openCypher `MATCH (ip:IP {value:'10.0.0.5'})-[:ASSIGNED_TO]->(a:Asset)`** — pivot = graph
  traversal; pivot field-set embodied in the graph schema (labels + rel types). Assumes a
  native property-graph runtime — *incompatible with a tabular DataFusion core*. [Neo4j-cypher-match]
- **ISO/IEC 39075 GQL** generalizes Cypher with **Quantified Path Patterns** (`MATCH ((a)-[r]->(b)){1,5}`)
  — formal "fan-out across N hops with bounded length." [GQL-overview]
- **SQL/PGQ (SQL:2023 part 16) `GRAPH_TABLE(g MATCH (…)-[…]->{1,2}(…) WHERE … COLUMNS(…))`** —
  the critical precedent: **property-graph pattern matching INSIDE a SQL `FROM` clause**, over
  graph *views* defined on relational tables, joinable with ordinary tables in one query.
  Oracle 23ai implements it. This is the only Pattern-C member that fits a SQL-grounded engine.
  [Oracle-SQLPGQ]

**STIX/CTI (OpenCTI, MISP, Maltego)** are entity-graph/transform-driven but encapsulate pivot
logic in API workflows and "transforms," not a user-facing declarative query syntax; they
contribute the *observable vs entity vs infrastructure* conceptual distinction and valid-from/
valid-until on indicators, not query grammar. [Filigran-observables][Maltego-transforms]

### Synthesis for PrismQL

PrismQL's settled registry is squarely **Pattern B** (schema-driven `entity_type → ordered
attribute paths`). The §12.1 planner contract — expand `entity('ip', X)` into a disjunction
of equality predicates over the registry's attribute paths — is exactly the Splunk-data-model /
Chronicle-`graph.entity.*` expansion mechanism, but *generalized across the type* (Chronicle
makes you pick the field; PrismQL's registry fans across all IP-bearing paths). The §12.1
sketch is therefore well-grounded and **more ergonomic than every Pattern-A/B prior-art
system for the single-observable-fan-out case**.

What prior art adds beyond §12.1:
1. **Two surfaces is the right call, confirmed.** Standalone `FIND` (ergonomic shorthand,
   SPL/Chronicle-search-bar lineage) + an `entity()` predicate composable in `WHERE` (KQL/SQL
   lineage) maps to the two documented usage modes (exploratory pivot vs embedded filter).
2. **Multi-hop is the gap.** §12.1 covers single-observable fan-out (one disjunction). Real
   investigative pivots want "IP → asset → user → other-assets-of-that-user" (multi-hop).
   The only SQL-compatible multi-hop precedent is **SQL/PGQ `GRAPH_TABLE`** with quantified
   path patterns. This is a *future* lever, not day-2 — but the day-2 grammar should not
   foreclose it (see Recommended sketch §"Forward-compat").

### How strong/weak tiers + temporal validity surface in the grammar

The registry already carries strong/weak tiers and temporal IP↔asset validity (§17.15 A5).
Prior art for surfacing them *in the query* (detail in Q2):
- **As-of-time resolution** is the unifying need. The clean grammar hook is an **`AS OF`
  clause** (SQL:2011 / Flink `FOR SYSTEM_TIME AS OF` lineage) that defaults to **event-time**.
- **Strong/weak tier** surfaces as a resolution-confidence knob — prior art is thin at the
  *grammar* level (Chronicle/Sentinel handle it in the resolution engine, not the query). LEAN:
  expose it as an optional modifier, default = deterministic-first (strong IDs exact, weak IDs
  temporal-window-bounded per A5), with the tier policy living in the registry, not the query.

> **LEAN (Q1):** Keep §12.1's two-surface design. Adopt SQL/PGQ `GRAPH_TABLE` as the *named
> forward-compatibility target* for multi-hop pivots (do not ship multi-hop in day-2). Default
> `entity()`/`FIND` resolution to **event-time as-of** semantics (Q2). Keep tier policy in the
> registry; expose only an optional query-level override. Keyword: prefer **`FIND`** (SPL/
> search-bar familiarity; `PIVOT` collides with SQL PIVOT, `SEARCH` is overloaded) — this also
> resolves the §12.1 open "FIND vs PIVOT vs SEARCH" question toward FIND.

---

## Q2 — Entity Resolution Semantics in Query (point-in-time / temporal binding)

### The problem, and the two unifying patterns

When an observable's identity changes over time (DHCP: IP→laptop-A at T1, →printer-B at T2),
the query must bind the observable to the entity that held it **at event time**, not at query
time. The deep-research pass found this is a well-trodden problem with **two equivalent
matching semantics**:

1. **Last-value-as-of** — pick the last identity-change ≤ event time. Implementations:
   **kdb+/q `aj`** ("most recent records prior to the times in the first" table) and
   **pandas `merge_asof` (backward)**. [kdb-aj][pandas-merge-asof]
2. **Interval-containment** — store explicit `[valid_from, valid_to)` per identity row; bind
   to the row whose interval contains event time. Implementations: **SQL:2011 application-time
   period tables** (`PERIOD FOR`, closed-open semantics), **SQL Server `FOR SYSTEM_TIME AS OF`**,
   **SCD Type-2** dimension joins, and **Chronicle/SecOps** which uses explicit
   `metadata.interval.start_time` / `end_time` and documents that the window MUST cover the
   event's `event_time` for enrichment to apply. [SQL2011-temporal][MSSQL-systime][Chronicle-dhcp-aliasing]

The two are equivalent if each change starts a new interval extending to the next change.
The choice matters for gaps, explicit expirations, and identifier reuse. [research-synthesis]

### Streaming / event-time precedent — and its pitfalls

- **Apache Flink temporal joins** use `FOR SYSTEM_TIME AS OF table1.rowtime` against a
  **versioned table** reconstructed from a changelog. Hard requirements: the dimension side
  needs a **declared primary key** and an **event-time attribute**; otherwise Flink refuses
  ("Temporal table join requires primary key in versioned table"). [Confluent-Flink-temporal][Ibis-flink-issue][dbt-flink-issue]
- **RisingWave** adds `FOR SYSTEM_TIME AS OF PROCTIME()` (processing-time variant). [RisingWave-temporal]
- **Pitfalls flagged explicitly** [research-synthesis]:
  - **Late-arriving dimension updates** past the watermark may be *silently dropped* — if the
    registry learns of a past alias mapping after the fact, already-processed events are NOT
    retroactively corrected without backfill.
  - **Primary-key constraint** breaks when an IP legitimately maps to multiple assets
    simultaneously (NAT, overlapping address space) — needs a composite key (IP + namespace/site).
  - **Boundary ambiguity** — solved by closed-open `[start, end)` intervals (SQL:2011), so an
    event exactly at a lease boundary belongs to the new interval, never both.
  - **Open intervals** — current rows use far-future end (`9999-12-31`) to mean "still valid."

### Synthesis for PrismQL

PrismQL is **federated and largely retrospective** (ephemeral query over sensor APIs + the
RetentionCache), not a long-running Flink dataflow. So the *streaming* machinery (watermarks,
changelog versioned tables) is the wrong import; the *relational* machinery (SQL:2011
interval-containment + `AS OF`, SCD2 join, Chronicle's `interval.start/end_time`) is the right
import. The registry's temporal IP↔asset validity (A5) is structurally an SCD2/application-time
table.

> **LEAN (Q2):** Model the registry's temporal validity as **closed-open `[valid_from, valid_to)`
> intervals** (SQL:2011 application-time semantics; avoids lease-boundary double-binding). Bind
> observables via **interval-containment as-of-EVENT-TIME by default**, exposed as an optional
> **`AS OF <expr>`** clause (lineage: `FOR SYSTEM_TIME AS OF`). Default `AS OF` = the row's
> event-time; allow `AS OF <timestamp literal>` for "resolve as the world was at T" and document
> that omitting it means event-time (NOT query-time). Use a **composite identity key**
> (observable + namespace/site) to handle simultaneous multi-asset mappings. Document the
> **late-update caveat**: because PrismQL resolves at query time against the current registry
> snapshot (not a frozen changelog), late-learned mappings ARE reflected on re-query — which is
> actually *better* than Flink's drop-late behavior for retrospective investigation, at the cost
> of non-reproducibility across registry edits (disclose this in EXPLAIN/result metadata).

---

## Q3 — PrismQL Ergonomics / Learnability (piped surface over a SQL core?)

### Is a piped surface over a SQL/relational core proven? — YES.

The deep-research pass returns a clear verdict with direct precedents:

- **PRQL (Pipelined Relational Query Language)** — "a functional, pipelined query language
  with modern ergonomics … that compiles to SQL, making it usable on any relational database."
  **Implemented in Rust.** Each pipeline stage = a relational-algebra operation on the prior
  stage's output; compiles to dialect-specific SQL; has a dbt plugin. This is the canonical
  proof that pipe-surface-over-relational-core is viable and Rust-native. [PRQL-talk][PRQL-HN]
- **RunReveal `pql`** — syntax "inspired by Microsoft's KQL," begins with a table name then
  pipes `where`/`summarize`/`project`, **compiles to SQL**. Built explicitly to avoid vendor
  lock-in of proprietary security query languages. [RunReveal-pql]
- **KQL / SPL / EQL** — all mature, widely-deployed `|`-piped languages; left-to-right data-flow
  reading is their core ergonomic. [MS-KQL-overview][Splunk-cheatsheet][Elastic-EQL-syntax]

A worked precedent maps a PrismQL pipe query directly to a DataFusion plan [research-synthesis]:
```
events
| where status == "error"
| summarize count() by service
| order by count desc
```
≡ `SELECT service, COUNT(*) AS count FROM events WHERE status='error' GROUP BY service ORDER BY count DESC`.
DataFusion's `LogicalPlanBuilder` exposes exactly this stage-by-stage construction
(`scan → filter → aggregate → sort → build`), verified upstream. [Context7:datafusion]

### What actually drives learnability (documented findings, with honesty about evidence)

**[INCONCLUSIVE — evidence quality]** The research flags that NONE of the vendor docs present
*controlled* learnability studies; ergonomic claims rest on design-rationale + anecdote. Treat
the following as "documented intent + convergent practice," not proven causation.

1. **Tooling ≥ syntax.** KQL's approachability is attributed "at least as much" to IntelliSense,
   table browser, filter-from-grid, and example-query panes (UI gestures that materialize as
   KQL) as to the pipe syntax itself. **Conclusion: a piped surface is necessary but NOT
   sufficient** — the S2 console tooling (Q5) carries equal weight. [MS-KQL-tutorial]
2. **Progressive disclosure / learnability ladder.** KQL: bare table name → add `where` → add
   `summarize` → chart. SPL: search-first (implied `search`, type a keyword, see events) → add
   `stats`/`timechart`. This matches §14.2.1's own ladder (single-event SQL → `SEQUENCE…THEN`
   sugar → raw `MATCH_RECOGNIZE`). [MS-KQL-tutorial][Splunk-cheatsheet]
3. **Task-vocabulary alignment.** KQL operators mirror investigation phrasing ("filter to last
   12h," "summarize by URL"). [MS-KQL-tutorial]
4. **Error-message quality is an ergonomic feature** (Rust/Elm precedent; see Q5).

### Reconciling SQL-grounding with KQL/SPL approachability

This is the crux question, and the research answers it cleanly: **PRQL and pql prove you do not
have to choose.** PrismQL can keep its SQL/DataFusion core *as the canonical semantics* and add
a **piped surface as desugaring sugar** — exactly the disposition already CONFIRMED in §12.3
("'adopt' means add as ergonomic sugar over SQL"). The pipe surface is not a competing language;
it lowers to the same DataFusion logical plan the SQL surface produces.

**Documented tradeoffs / cautions** [PRQL-HN][research-synthesis]:
- Users may misread pipelines as *imperative* scripts rather than declarative descriptions —
  mitigate in docs.
- **Debugging generated SQL/plans is harder** through a DSL layer — PRQL mitigates by showing
  generated SQL; pql shows intermediate SQL. PrismQL should expose the desugared SQL/`EXPLAIN`.
- Some SQL features create **impedance mismatch** with a clean pipe surface — keep an SQL escape
  hatch (PrismQL is already SQL-grounded, so this is free).

> **LEAN (Q3):** Ship **both surfaces over one DataFusion core**: SQL-shaped (canonical, power
> users, full `MATCH_RECOGNIZE`) AND a **piped sugar surface** (KQL/PRQL-style:
> `source | where … | summarize … by … | order …`) for the approachable on-ramp. The pipe
> surface desugars to the identical logical plan — NOT a second engine. Core pipe operators
> (mirroring KQL/pql): `where`, `summarize`/`stats` (already §12.3 E2), `project`, `extend`,
> `join`, `order`, `limit`, plus `FIND`/`entity()` (Q1) and `SEQUENCE…THEN` (settled §14.2.1)
> as security-domain operators. **Treat S2-console tooling as co-equal to syntax** (Q5).
> Always offer "show desugared SQL / EXPLAIN" to defuse the DSL-debugging caveat. This is
> consistent with §12.3 and adds the *piped-surface* decision §12.3 did not explicitly settle.

### Q3b — NL→PrismQL via the embedded agent (S3): prior art + guardrails [BRIEF]

Prior art: **NL2KQL** ("Towards Small Language Models for Security Query Generation in KQL")
uses a two-stage architecture — a small model generates candidate KQL, then a lightweight LLM
judge **validates/refines using schema info + parser feedback** — plus **error-aware prompting**
keyed off common KQL parser failures. Broader NL2SQL practice converges on the same guardrails.
[NL2KQL][NL2SQL-commentary]

Guardrails the research identifies as **necessary but not sufficient**:
- **Schema grounding** — feed the agent the OCSF/native schema catalog (Q4); IntelliSense-style
  constraint to *existing* fields is itself a hallucinated-field defense. [NL2KQL][MS-KQL-tutorial]
- **Validation-before-execution** — parse + type-check + plan the generated query and feed
  diagnostics back to the agent before any execution (the NL2KQL judge loop).
- **Hallucinated-field defense** — reject/repair field refs not in the catalog; suggest nearest
  schema matches (Q5 diagnostics double as the agent's repair signal).
- **Cost-guard / time-bound interplay** — the §5.3 mandatory-time-bound NFR and the C3
  cost-based-degrade guardrails (ADR-PROP, NOT reopened) are the safety net for agent-authored
  queries: an agent that omits a window gets the injected default; an agent that writes an
  expensive cross-source join gets cap+flag+cost-disclosure, not silent runaway.

> **LEAN (Q3b):** NL→PrismQL is agent-native (already §12.3 E8). Reuse the SAME parser/planner
> diagnostics (Q5) as the agent's validate-then-repair signal (NL2KQL judge pattern). The
> existing mandatory-time-bound NFR + C3 cost guardrails are the cost/time safety net — no new
> agent-specific cost machinery needed. Ties to the agent-harness (prompt-injection defense is
> out of scope here; see project_agent_harness_design memory).

---

## Q4 — Multi-Schema Field Referencing (OCSF vs native; OCSF-version-aware) (§13.6)

### Prior art — normalize-on-read vs -write, raw+normalized coexistence, versioning

The deep-research pass produced a direct **ASIM / CIM / UDM / OCSF comparison**:

| Framework | Normalization phase | Raw+normalized coexistence | Versioning mechanism |
|---|---|---|---|
| **Sentinel ASIM** | normalize-on-read via KQL parser functions over native tables | parsers retain originals in dedicated fields (`EventOriginalResultDetails`); native tables stay queryable | explicit `EventSchema` + `EventSchemaVersion` fields (e.g. FileEvent `0.2.1`); mandatory/recommended/optional field tiers [MS-ASIM][MS-ASIM-fileevent] |
| **Splunk CIM** | normalize-on-read (search-time) via tags/aliases/extractions/lookups | raw fields preserved in indexed events; `_raw` + native reachable alongside CIM fields | CIM versions exist but low-surfaced; managed via add-on updates + alias config [Splunk-CIM] |
| **Chronicle UDM** | normalize-on-write at ingest, **but raw log retained alongside UDM record** | dual storage — `$event.*`/`$entity.*` UDM paths + retained raw log | explicit UDM version numbers **not** clearly documented (inferred internal versioning) [Chronicle-UDM] |
| **OCSF** | typically normalize-on-write (e.g. AWS Security Lake transform lib) | depends on platform; mapping libs preserve raw in separate tables/columns | **semantic versioning** 1.0.0→1.3.0+ documented; new event classes/attributes, core fields kept stable [OCSF-versioning] |

Key documented patterns relevant to PrismQL:
- **ASIM "robustness principle"** — "Be strict in what you send, be flexible in what you accept."
  Parsers enforce strict normalized output while tolerating source variation. [MS-ASIM]
- **Retain-originals** — ASIM's `EventOriginalResultDetails` is the canonical "keep the raw value
  next to the normalized one" pattern. [MS-ASIM-parsers]
- **Stable nested paths** — UDM's `$event.metadata.event_type` path convention lets new fields be
  added under a node without breaking existing queries. [Chronicle-UDM]
- **[INCONCLUSIVE]** Tooling that does *version-aware OCSF field binding* across mixed-version
  sources is **not documented** in the corpus — the research is explicit that PrismQL's
  per-source-version binding would be ahead of documented prior art, grounded by analogy to
  ASIM's version fields + CIM's per-source alias mappings.

### Synthesis for PrismQL (OCSF-grounded + native schema-on-read + per-source OCSF version)

The research lands on a coherent design analogous to CIM aliases + ASIM version fields:
1. **Canonical OCSF field names as version-agnostic identifiers**, bound per-source-version by a
   **schema catalog** derived from the OCSF repo/version-history. Analyst writes
   `event.file.path`; the planner maps it to `file.path` in a v1.1 source and `file.full_path`
   in a v1.3 source. [research-synthesis][OCSF-versioning]
2. **Native fields via a reserved namespace** — `native.<source>.<field>` (analogous to Splunk
   `_raw`/sourcetype fields and ASIM original-value fields), reachable in the SAME query as
   canonical OCSF fields. [research-synthesis]
3. **Optional explicit version scope** — `event.file.path@ocsf:1.6` to pin semantic expectation,
   OR version-agnostic-by-default with the catalog reconciling (mirrors §13.6 / C5 schema-version
   axis). [research-synthesis]
4. **Compatibility tiers** — stable-across-versions fields safe for implicit cross-version
   mapping; semantically-changed fields flagged version-sensitive needing explicit handling
   (mirrors ASIM mandatory/optional tiers). [research-synthesis]
5. **Value-level mapping** for enum drift (e.g. `network.direction` `ingress`/`egress` →
   `inbound`/`outbound`) via lookups, with missing fields → null-with-diagnostic or source-excluded.

> **LEAN (Q4):** Analyst writes **canonical OCSF field names against a single reference version
> (e.g. OCSF 1.6)**; the planner binds per-source-version via a schema catalog (CIM-alias +
> ASIM-version-field lineage). Native fields reachable via a reserved **`native.<source>.<field>`**
> namespace in the same query. Default = version-agnostic canonical names; allow an optional
> **`@ocsf:<ver>`** pin for version-sensitive fields. Adopt **compatibility tiers** (stable vs
> version-sensitive) in the catalog. Adopt the **retain-originals** pattern (ASIM
> `EventOriginalResultDetails`) so a normalized field never loses its raw source value. This
> aligns with §13.6 and the C5 schema-version axis. **Flag clearly:** cross-mixed-version OCSF
> binding is ahead of documented prior art — validate the catalog mechanism with concrete OCSF
> 1.x diff tests, do not assume it falls out of an existing tool.

---

## Q5 — Autocomplete / Schema-Awareness + Formal Error Reporting (from a Chumsky grammar)

### Prior art — LSP, Monaco, KQL IntelliSense

- **LSP (Language Server Protocol)** decouples editor from language logic via JSON-RPC. The
  relevant handlers for PrismQL: `textDocument/completion` (autocomplete), `textDocument/hover`
  (inline docs), `textDocument/publishDiagnostics` (errors/warnings), `textDocument/codeAction`
  (quick fixes). On a completion request the server parses the doc, determines syntactic+semantic
  context, consults the **schema catalog**, and returns field/operator/keyword suggestions (e.g.
  after `event.network.` → `src_ip`, `dest_ip`, `direction`, `bytes`). [LSP-spec]
- **Monaco** = browser editor (VS Code core). `monaco-languageclient` + `vscode-ws-jsonrpc`
  connect Monaco to an LSP server over WebSocket; `monaco-editor-react` for React. **Schema-aware
  logic lives in the language server, NOT the editor** — Monaco is a presentation layer. A
  documented Monaco gap: loading external JSON schemas for completion needs custom integration.
  [monaco-languageclient][Monaco-json-issue]
- **Azure Data Explorer KQL IntelliSense** — contextual suggestions for entities/operators/
  functions, autocompletes tables/columns/functions, underlines errors with hover detail, and
  offers **quick-fix code actions**. The case-study target for PrismQL's S2 console. [ADX-KQL-editor]

### Chumsky → diagnostics (VERIFIED against upstream docs, not model-knowledge)

Context7-served Chumsky docs confirm the exact facilities PrismQL needs [Context7:chumsky]:
- **`Rich` error type** — tracks `span()` (a `SimpleSpan`), a `RichReason` (incl.
  `ExpectedFound`), `found()`, `expected()` (iterator of `RichPattern`), and **`contexts()`** —
  "labelled contexts … parser patterns the parser was in the process of parsing when the error
  occurred." `Rich::custom(span, msg)` for custom messages; `merge()` combines co-located errors;
  `map_token` bridges lexer/parser token types.
- **Labels** — `Parser::labelled(label)` / `labelled_with(...)` attach human-meaningful context;
  `LabelError::expected_found(expected, found, span)` constructs the expected-vs-found conflict.
- **`try_map` / `try_map_with`** — post-parse fallible transforms that surface as parse errors
  (the hook for *semantic* validation like unknown-field detection, with `MapExtra` giving span).
- Error-type spectrum: `Cheap` (span only) / `Simple` / `Rich` (full) / `EmptyErr` — pick `Rich`
  for the authoring path.

These map straight onto **ariadne** (color-coded source-mapped diagnostics with labeled spans +
notes), the standard Chumsky companion — confirmed as the rendering layer. [Context7:chumsky][model-knowledge: ariadne↔chumsky pairing is community-standard]

### What PrismQL must expose to power the S2 Monaco editor

Synthesizing LSP + KQL + Chumsky/DataFusion [research-synthesis][Context7:datafusion]:
1. **Grammar metadata** — operator/keyword set + per-position expectation (Chumsky `expected()`/
   `RichPattern` already yields "what was valid here").
2. **Span-precise diagnostics** — exact error span + expected-vs-found (Chumsky `Rich.span()` +
   `expected()`), rendered via ariadne for terminal and surfaced via LSP `publishDiagnostics` for
   Monaco.
3. **Schema catalog access** — OCSF (per-version) + native field lists + entity registry types,
   so completion suggests valid fields and diagnostics can do **"unknown field → nearest-match
   suggestion"** (the reverse-IntelliSense the research recommends).
4. **Planner-derived semantic diagnostics** — DataFusion logical-plan/type errors (group-by a
   field not in the pipeline, type mismatch) mapped back to source spans; DataFusion's
   `LogicalPlanBuilder` + typed schema make this tractable. [Context7:datafusion]
5. **Time-bound + cost warnings** — surface the §5.3 mandatory-time-bound default injection and
   C3 cost-disclosure as editor warnings/code-actions ("insert SINCE 24h?").

> **LEAN (Q5):** Build PrismQL's authoring intelligence as an **LSP server** (reused by Monaco
> in S2, by terminal via ariadne, AND by the NL→PrismQL agent's validate-repair loop, Q3b). Use
> Chumsky **`Rich`** errors with **`labelled`** contexts throughout the grammar; render via
> **ariadne** (CLI) and translate to LSP diagnostics (Monaco). Expose four catalogs to the
> server: grammar/operator metadata, OCSF-per-version schema, native-field schema, entity
> registry. Implement **unknown-field → nearest-match** and **missing-time-bound → quick-fix**
> (KQL quick-fix lineage). Schema-aware logic lives in the server, not Monaco. This is the
> concrete deliverable that makes §12.3's "approachable" claim real.

---

## Q6 — PrismQL Portability / Shareability (detection-as-code, Sigma import) [BRIEF] (§14.7)

### Prior art — what makes detection content portable/shareable

From the reasoning pass [Sigma-rules-doc][Elastic-detection-rules][Splunk-ES-versioning][GitLab-DaC]:
- **Vendor-neutral + declarative core** convertible to multiple backends.
- **Logic decoupled from data-source specifics** via explicit logsource/schema metadata (Sigma
  `logsource`; the alias layer maps source fields to normalized fields).
- **Managed as code in Git** with semantic versioning, peer review, CI tests. Elastic
  `detection-rules` repo and Splunk Security Content/ESCU (with detection versioning metadata:
  id, hash, version, publishing time, parent version) are the production templates.
- **Rich, standardized rule metadata**: stable `id` (UUID), `version`, `title`/`description`,
  `status` (experimental/stable/deprecated), `level`/severity, `references`, ATT&CK `tags`,
  `falsepositives`/tuning notes, `logsource`/data requirements, author + dates.
- **CI-testable** — replay sample logs / fixtures, validate matches before merge.

### Synthesis for PrismQL

PrismQL's settled pieces map cleanly onto detection-as-code:
- **Backend-neutral SQL core + DataFusion** = the portable substrate (vendor-specific UDFs become
  optional capability-flagged features).
- **Entity registry + canonical OCSF fields (Q1/Q4)** = the Sigma-`logsource`/`aliases` analog:
  a recipe references entities + canonical fields; each deployment supplies the schema mappings.
- **`SEQUENCE…THEN` (settled §14.2.1)** = portable pattern recipe; compiler lowers to
  `MATCH_RECOGNIZE` — recipe stays stable even if the engine changes.

**Sigma import mapping** (ties C6, defer the *automated translator* per §12.3 D1):
- Sigma `id`→`rule_id` (preserve as canonical external ID + origin/commit for lineage);
  `title`/`description`/`author`/`date`/`status`/`level`/`references`/`falsepositives`→PrismQL
  metadata; `tags` (ATT&CK) preserved.
- Sigma `logsource`→entity-registry + schema-mapping selection.
- Sigma `detection`→PrismQL `WHERE`/`JOIN` (+ `SEQUENCE…THEN` for temporal/`temporal_ordered`).

> **LEAN (Q6):** Define a **PrismQL rule/recipe format** = query text + Sigma-aligned metadata
> block (stable `rule_id`, semver `version`, `title`/`description`, `status`, `severity`,
> required `entities`+`data_sources`, ATT&CK `tags`, `false_positives`, `references`, author/dates).
> Store recipes in Git with **semver** (major = breaking entity/schema/logic change). Ship a
> **CI harness** that runs recipes against Arrow/Parquet fixtures via DataFusion with expected
> match/non-match assertions. **Sigma import** maps metadata 1:1 and logic via entity-registry/
> canonical fields; day-2 = docs-guided import per §12.3 E9, automated translator stays deferred
> (§12.3 D1, E-RULE-XLATE-001). This realizes the §14.7 recipe library.

---

## Consolidated Open Design Questions (for architect/PO morph of §12)

1. **`FIND` keyword choice** — research leans `FIND` over `PIVOT`/`SEARCH` (SQL `PIVOT`
   collision; SPL/search-bar familiarity). Resolves the §12.1 open question toward `FIND`.
   *Decision owner: architect/PO.*
2. **Piped surface — ship in day-2 or defer?** Research says the *piped-over-SQL* pattern is
   proven (PRQL/pql) and that tooling matters as much as syntax. §12.3 confirmed "sugar over SQL"
   but did not explicitly decide a *piped surface syntax*. **Open: commit to a pipe surface for
   day-2, or SQL-shaped + sugar only?**
3. **`AS OF` semantics** — adopt event-time interval-containment as default? Expose `AS OF <expr>`?
   How to disclose the late-update non-reproducibility caveat (Q2) in result metadata/EXPLAIN?
4. **OCSF version-binding model** — version-agnostic canonical names + catalog reconciliation
   (default) vs explicit `@ocsf:<ver>` pins (opt-in)? Compatibility-tier source (OCSF version-diff).
   **Flagged ahead-of-prior-art — needs concrete OCSF 1.x diff validation.**
5. **Native field namespace** — confirm `native.<source>.<field>`? Reconcile with §13.6
   multi-schema descriptor + A7 per-field residency tags (a `native.*` ref may carry `raw`
   residency class — interacts with the residency policy artifact).
6. **Authoring-intelligence locus** — single LSP server reused by S2 Monaco + CLI ariadne + the
   NL→PrismQL agent? Confirms a shared-diagnostics investment.
7. **Multi-hop pivot** — name SQL/PGQ `GRAPH_TABLE` as the forward-compat target now (so day-2
   grammar doesn't foreclose it), or leave entirely future?
8. **Rule/recipe format** — Sigma-aligned metadata schema for the §14.7 recipe library; semver
   policy; CI fixture harness shape.

---

## Recommended PrismQL entity-pivot grammar sketch (concrete; builds on the SETTLED registry)

> Discussion input for the architect/PO morph. Builds ONLY on settled pieces (registry,
> mandatory-time-bound NFR, C3 cost-degrade, SEQUENCE…THEN). Extends §12.1 with the Q2 `AS OF`
> hook and a Q4 native-namespace note. **Not a ratified grammar.**

```ebnf
(* ---- Standalone FIND (ergonomic shorthand; SPL/Chronicle-search lineage) ---- *)
entity_pivot   ::= "FIND" entity_type entity_value as_of? time_bound? source_scope? tier_hint?
entity_type    ::= "ip" | "user" | "host" | "domain" | "email" | "hash" | "cve" | IDENT
entity_value   ::= STRING
as_of          ::= "AS" "OF" ( "EVENT" "TIME" | timestamp | expr )   (* default: EVENT TIME *)
time_bound     ::= "SINCE" duration | "BETWEEN" timestamp "AND" timestamp
source_scope   ::= "ACROSS" ident ( "," ident )*
tier_hint      ::= "USING" ( "STRONG" | "STRONG" "," "WEAK" )        (* default: registry policy *)
duration       ::= INT ( "s" | "m" | "h" | "d" | "w" | "mo" )         (* §12.3 E1 *)

(* ---- entity() predicate (composable inside any PrismQL WHERE; KQL/SQL lineage) ---- *)
entity_pred    ::= "entity" "(" STRING "," STRING ( "," as_of_arg )? ")"
as_of_arg      ::= "as_of" "=>" ( "event_time" | expr )

(* ---- Field references (Q4): canonical OCSF (optional @ver pin) OR native namespace ---- *)
field_ref      ::= ocsf_path ( "@" "ocsf" ":" version )? | native_ref
native_ref     ::= "native" "." ident "." ident
```

**Planner contract (extends §12.1, all clauses honor settled decisions):**
1. **Registry expansion** (settled) — `entity('ip', X)` ⇒ disjunction over the registry's ordered
   IP attribute paths: `src_endpoint.ip = X OR dst_endpoint.ip = X OR device.ip = X …`.
2. **As-of binding (Q2)** — for weak-tier observables with temporal validity, resolve the
   observable→entity binding by **interval-containment as of the as-of time (default EVENT TIME)**
   against the registry's `[valid_from, valid_to)` intervals; strong-tier IDs bind exactly. Tier
   policy lives in the registry; `USING`/`tier_hint` is an optional override only.
3. **Pushdown (settled, C3)** — per the capability descriptor, push attribute predicates that
   sources support natively; evaluate the rest centrally post-OCSF-normalization. Cross-source
   join cost handled by the C3 **cost-based-degrade** stack (ADR-PROP; NOT reopened here).
4. **Mandatory time-bound (settled §5.3)** — absent `SINCE`/`BETWEEN` ⇒ inject default window.
5. **Fan-out / normalize / merge / partial-result metadata** (settled §3.6).
6. **RETAIN interaction** — a `FIND` result may be retained as `FROM cache.<name>` (likely yes,
   per §12.1 open note) — confirm in morph.

**Forward-compat (do NOT ship in day-2):** reserve a multi-hop path-pattern surface that lowers
to **SQL/PGQ `GRAPH_TABLE`** semantics over an entity-graph view of the registry, so the day-2
single-hop grammar does not foreclose `IP → asset → user → assets` multi-hop pivots later.

**Piped-surface illustration (Q3 sugar; same plan as the SQL form):**
```
FIND ip '10.0.0.5' SINCE 24h ACROSS crowdstrike, splunk AS OF EVENT TIME
| where severity >= 'high'
| summarize count() by device.hostname
```

---

## Honest Costs & Caveats

- **[INCONCLUSIVE — learnability evidence]** Every "language X is learnable" claim rests on
  vendor design-rationale + anecdote, NOT controlled studies. The research is explicit about
  this. The piped-surface and progressive-disclosure leans are *convergent industry practice*,
  not empirically proven causation. If learnability is a hard success metric, PrismQL would need
  its own user research — no shortcut exists in the literature.
- **[Ahead-of-prior-art — Q4]** Version-aware OCSF field binding across mixed-version sources is
  NOT documented in any surveyed tool. The design is grounded by analogy (ASIM version fields +
  CIM per-source aliases), but the mechanism must be validated against real OCSF 1.x version
  diffs. Do not assume it falls out of an existing library. Cost: building + maintaining a
  per-version OCSF schema catalog with compatibility tiers and value-mapping tables.
- **[Caveat — Q2 reproducibility]** Resolving as-of against the *current* registry snapshot (vs a
  frozen changelog) means a re-query after registry edits can return different bindings. This is
  better than Flink's drop-late behavior for retrospective investigation, but it is
  non-reproducible across registry mutation — MUST be disclosed in EXPLAIN/result metadata.
- **[Caveat — Q2 multi-asset]** Simultaneous IP→multi-asset mappings (NAT/overlap) break a naive
  single-key registry; needs a composite (observable + namespace/site) key. Flink's PK constraint
  is the documented warning.
- **[Caveat — Q3 DSL debugging]** A piped sugar surface adds a layer between analyst and plan;
  PRQL/pql both mitigate by exposing generated SQL. PrismQL must ship "show desugared SQL /
  EXPLAIN" or analysts will struggle to debug. Cost: the desugaring must be inspectable, not
  a black box.
- **[Caveat — multi-hop]** Multi-hop entity pivots have only ONE SQL-compatible precedent
  (SQL/PGQ `GRAPH_TABLE`), and that is recent (SQL:2023) with limited engine support; DataFusion
  does not implement it today [model-knowledge — not verified against current DataFusion]. Treating
  it as forward-compat (not day-2) is the honest posture.
- **[Caveat — tooling cost]** The research's strongest finding is that *tooling carries as much
  learnability weight as syntax*. The LSP server + schema catalogs + Monaco integration (Q5) is a
  substantial, separate engineering investment — the piped surface alone will not deliver
  "KQL-grade approachable."
- **[Scope honesty]** Q3b (NL→PrismQL) was kept brief per instructions; prompt-injection defense
  for the agent is explicitly out of scope (see project_agent_harness_design memory).

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 4 | Q1 entity/observable pivot grammar prior art (SPL/KQL-graph/Chronicle/EQL/Sigma/STIX/Cypher/GQL/SQL-PGQ); Q2 temporal/point-in-time entity resolution (Flink/SQL:2011/SCD2/kdb+ aj/pandas merge_asof/Chronicle DHCP aliasing); Q3+Q3b ergonomics/learnability (PRQL/pql/KQL/SPL/EQL + NL2KQL guardrails); Q4+Q5 multi-schema field referencing (ASIM/CIM/UDM/OCSF) + autocomplete/LSP/Monaco/ariadne. All at reasoning_effort=high. |
| Perplexity perplexity_reason | 1 | Q6 portability/shareability (Sigma metadata, Elastic/Splunk detection-as-code, semver, CI) synthesized into PrismQL recipe-format + Sigma-import leans. search_context_size=medium. |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_ask | 0 | — |
| Context7 | 3 | Verified Chumsky parser API (`Rich` error type, `span()`/`expected()`/`contexts()`, `labelled`/`labelled_with`, `try_map`, error-type spectrum) — resolve + query-docs; verified DataFusion API (`ScalarUDFImpl`/`create_udf`, `UserDefinedLogicalNodeCore`/extension nodes, `LogicalPlanBuilder` stage construction) — resolve + query-docs. Library IDs: /websites/rs_chumsky_chumsky, /apache/datafusion. |
| Tavily (any) | 0 | — |
| WebFetch | 0 | — |
| WebSearch | 0 | — |
| Training data | 2 areas (flagged) | (a) ariadne↔Chumsky pairing as community-standard rendering layer [model-knowledge — but Chumsky `Rich` API itself is Context7-verified]; (b) "DataFusion does not implement SQL/PGQ GRAPH_TABLE today" [model-knowledge — NOT version-verified, flagged in Caveats]. All other library claims are Context7-verified, not training data. |

**Total MCP tool calls:** 8 (4 perplexity_research at effort=high + 1 perplexity_reason + 3 Context7).
**Training data reliance:** low — all substantive findings are web-sourced (Perplexity deep-research/reason) or upstream-doc-verified (Context7). The two training-data areas are explicitly flagged and bounded; neither is load-bearing for a settled-piece contradiction. Version numbers (OCSF 1.x, Chumsky/DataFusion APIs) were taken from retrieved sources/Context7, not training data, per the registry-verification rule.

**MCP availability:** All MCP tooling available and exercised; no MCP-UNAVAILABLE escalation required.
**Note on large outputs:** the four `perplexity_research` responses exceeded the inline token cap and were persisted to tool-result files; all four were read in full via chunked Read/Grep before synthesis (entity-pivot read through all 7 prior-art systems + per-system implications; temporal-resolution read through Flink + SQL:2011 + synthesis; ergonomics read through PRQL/pql + NL2KQL + PrismQL synthesis; multi-schema read through the ASIM/CIM/UDM/OCSF table + OCSF version-binding + LSP/Monaco/Chumsky sections).
