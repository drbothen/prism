---
document_type: proposed-adr
status: capture
do_not_execute: true
decided: "2026-06-27 (human)"
candidate_adr_slots:
  - "ADR-PROP-C8-1: PrismQL piped surface — ship in day-2, desugars to DataFusion logical plan"
  - "ADR-PROP-C8-2: Entity-resolution AS OF reproducibility — DEFERRED pending targeted research (OQ-C8-ASOF)"
  - "ADR-PROP-C8-3: OCSF version-binding model — DEFERRED pending targeted research (OQ-C8-OCSFVER)"
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
  - CLAUDE.md (#[non_exhaustive] discipline; SAP-1 structured event catalog; error taxonomy E-QUERY-NNN)
---

# ADR-PROP — PrismQL Deliverables (C8)

> **STATUS: PARTIALLY DECIDED 2026-06-27 (human). Two items (D-C8-2 entity-resolution AS OF
> reproducibility; D-C8-3 OCSF version-binding model) are DEFERRED pending a targeted research
> pass (file: `research/prismql-asof-version-resolution-2026-06-27.md`). All other decisions and
> leans are DECIDED 2026-06-27 (human).** This is a CAPTURE artifact for the side-analysis C8
> program. `do_not_execute: true`. Real ADR numbers and formal ARCH-INDEX.md rows are deferred to
> the morph execution (post-demo, post-T14, gated on brief-reframe sign-off §5.1).

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

C8 settles Q1/Q3/Q5/Q6 and the non-reproducibility-mechanism parts of Q2/Q4. Q2's
live-vs-frozen reproducibility choice and Q4's version-agnostic-vs-explicit-pin choice are
DEFERRED to the targeted research pass flagged as OQ-C8-ASOF and OQ-C8-OCSFVER.

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

### D-C8-2 — Entity-Resolution AS OF Reproducibility: DEFERRED (OQ-C8-ASOF)

**DEFERRED 2026-06-27 (human-directed: "targeted research for both options").**

A focused research pass is in flight at
`research/prismql-asof-version-resolution-2026-06-27.md`. The deferred question is:

**Live-registry snapshot** (fresh, non-reproducible: a re-query after registry edits may return
different entity bindings) **vs frozen-registry-version** (reproducible/audit-grade: a fixed
registry snapshot is pinned to the query execution) **vs bitemporality** (valid-time +
transaction-time, giving BOTH from one model — may unify the live and frozen axes).

The research pass is examining whether ONE as-of mechanism unifies both this entity-resolution
axis and the OCSF version-binding axis (OQ-C8-OCSFVER). A forensic re-query may need BOTH
as-of-event-time entity resolution AND as-of-version schema binding to be reproducible. The
deferred choice is whether that reproducibility is explicit (frozen pin) or implicit (live
snapshot with disclosed non-determinism).

**Record as OPEN ITEM OQ-C8-ASOF:** "resolution pending research pass + human decision — fold
on return."

**SETTLED parts (hold regardless of the deferred choice):**

The following are decided and do NOT change based on the live-vs-frozen outcome:

1. **Temporal binding model:** weak-tier observables (e.g., DHCP IP→asset mappings) resolve by
   **interval-containment as-of EVENT-TIME (default)**, NOT query-time. The registry's validity
   periods use **closed-open `[valid_from, valid_to)` intervals** (SQL:2011 application-time
   semantics; avoids lease-boundary double-binding; "current" rows use far-future `valid_to =
   9999-12-31`).
2. **`AS OF <expr>` clause exists** and defaults to EVENT TIME. Explicitly stated in the grammar
   sketch (EBNF, §"Recommended grammar" below). Analytic default: the row's event-time is the
   as-of anchor; override to a literal timestamp for "resolve as the world was at T."
3. **Composite identity key:** `(observable, namespace/site)` — not a bare IP/username — to
   handle simultaneous multi-asset mappings (NAT, address-space overlap). Flink's documented
   primary-key constraint failure mode for multi-asset is the warning reference.
4. **Strong-tier IDs bind exactly** (no temporal interval needed; binding is exact by definition
   per §17.15-A5).
5. **Tier policy lives in the registry** with an optional query-level `USING` override for
   strong-only resolution (`USING STRONG`; default = registry policy).

**Caveat on the live-registry default (must be disclosed regardless of final choice):**

The live-registry default (current snapshot) means a re-query after registry edits can return
different entity bindings than the original query. This is BETTER than Flink's drop-late
behavior for retrospective investigation (late-learned alias mappings ARE reflected on re-query),
at the cost of non-reproducibility across registry mutations. If reproducibility is chosen
(frozen-registry-version), this caveat disappears but the mechanism becomes more complex. Either
way, the EXPLAIN / result metadata MUST disclose the registry snapshot version used.

[research/prismql-deliverables-depth-2026-06-27.md §Q2, §2.3–2.4 LEAN]

---

### D-C8-3 — OCSF Version-Binding Model: DEFERRED (OQ-C8-OCSFVER)

**DEFERRED 2026-06-27 (human-directed: "research this one as well").**

The same targeted research pass (`research/prismql-asof-version-resolution-2026-06-27.md`) is
examining:

**Version-agnostic canonical names + per-source-version schema-catalog reconciliation**
(ergonomic: analyst writes `event.file.path`; catalog maps it to `file.path` in a v1.1 source
and `file.full_path` in a v1.3 source) **vs explicit `@ocsf:<ver>` pin** (predictable /
reproducible: analyst writes `event.file.path@ocsf:1.6`) **vs catalog-version-pinning-for-
reproducibility** (Iceberg-snapshot-id analog: pin the catalog version alongside the query for
audit re-runs).

The research is also examining whether the OCSF version-binding reproduciblity and the entity-
resolution AS OF reproducibility (D-C8-2) can be unified by ONE as-of mechanism — so that a
forensic re-query anchors BOTH the entity registry AND the OCSF schema version at the same point.

**Record as OPEN ITEM OQ-C8-OCSFVER:** "resolution pending research pass + human decision —
fold on return."

**SETTLED parts (hold regardless of the deferred choice):**

1. **Canonical OCSF field names are the query identifiers.** Analysts do not write per-source
   native field names in the canonical path. The OCSF path (e.g. `event.file.path`) is stable
   even as the per-source binding changes across versions.
2. **Native fields reachable via a reserved `native.<source>.<field>` namespace** in the same
   query. This is the ASIM `EventOriginalResultDetails` analog — normalized fields never lose
   their raw source value. A `native.*` ref may carry `raw` residency class (interacts with the
   residency policy artifact — see Open Questions).
3. **Retain-originals pattern adopted** (ASIM `EventOriginalResultDetails` lineage): a
   normalized field NEVER silently loses its raw source value. The raw value is reachable via
   `native.<source>.<field>` in the same query.
4. **Compatibility tiers in the catalog:** stable-across-versions fields (safe for implicit
   cross-version mapping) vs version-sensitive fields (flagged; need explicit handling or an
   `@ocsf:<ver>` pin). The OCSF upstream version history drives the tier classification.
5. **Value-level mapping** for enum drift (e.g., `network.direction` `ingress/egress` →
   `inbound/outbound` across versions) via catalog lookup tables. Fields absent in the source's
   stamped version → null-with-diagnostic or source-excluded (not silent empty).

**NOTE: version-aware OCSF field binding across mixed-version sources is AHEAD OF PRIOR ART.**
The ASIM/CIM/UDM/OCSF comparison in the research is explicit: no surveyed tool documents
per-source-version binding with a compatibility-tier catalog. The mechanism is grounded by
analogy (ASIM version fields + CIM per-source alias mappings), but the catalog must be validated
against real OCSF 1.x version diffs before implementation. Do not assume it falls out of an
existing library.

**INTERACTION WITH D-C8-2:** A forensic re-query may need BOTH as-of-event-time entity
resolution (D-C8-2 settled parts) AND as-of-OCSF-version schema binding to be fully
reproducible. The research pass is examining whether one mechanism serves both axes.

[research/prismql-deliverables-depth-2026-06-27.md §Q4, §4.3–4.4 LEAN]

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
| **OQ-C8-ASOF** | Entity-resolution AS OF reproducibility: live-registry snapshot (non-reproducible) vs frozen-registry-version (audit-grade) vs bitemporality (unifies both) | **OPEN** — resolution pending research pass `prismql-asof-version-resolution-2026-06-27.md` + human decision. "Fold on return." | D-C8-2 deferred |
| **OQ-C8-OCSFVER** | OCSF version-binding: version-agnostic canonical names + catalog reconciliation vs explicit `@ocsf:<ver>` pin vs catalog-version-pinning for reproducibility | **OPEN** — resolution pending the SAME research pass (OQ-C8-ASOF and OQ-C8-OCSFVER may unify) + human decision. "Fold on return." | D-C8-3 deferred |
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
| **Entity-resolution reproducibility choice (OQ-C8-ASOF) is non-trivial.** | The live-vs-frozen fork is a real tradeoff between operational simplicity and forensic auditability. The bitemporality option (valid-time + transaction-time) may unify both but is architecturally heavier. The research pass must return before the EXPLAIN / result-metadata design can be finalized. |
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
| **ADR-TBD: PrismQL piped surface + LSP server** | This ADR-PROP covers D-C8-1 + L-C8-1..5. The real ADRs (allocated at morph, ADR-NNN+) formalize: the desugar-to-DataFusion contract; the LSP server catalog schema; the entity-resolution AS OF semantics (once OQ-C8-ASOF is resolved); the OCSF version-binding model (once OQ-C8-OCSFVER is resolved). |
| **ADR-TBD: Recipe format + CI harness** | L-C8-5 formalized as a separate ADR covering the metadata schema, semver policy, Parquet fixture format, and Sigma import contract. |
| **matured-vision §16.4** | C8 decision block appended (2026-06-27). |
