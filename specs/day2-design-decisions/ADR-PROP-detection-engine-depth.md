---
document_type: proposed-adr
status: capture
do_not_execute: true
decided: "2026-06-27 (human)"
candidate_adr_slots:
  - "ADR-PROP-C6-1: Backtesting posture — BOTH cold-tier deterministic AND remote best-effort, ALWAYS with coverage map"
  - "ADR-PROP-C6-2: False-positive handling — RBA as default over hard suppression; suppression-as-code with mandatory justification + time-box expiry"
  - "ADR-PROP-C6-3: Auto-rollback — DEFERRED pending deep-research; OPEN ITEM OQ-C6-AUTOROLLBACK"
  - "ADR-PROP-C6-LEAN-1: MATCH_RECOGNIZE operator — UserDefinedLogicalNode + MatchRecognizeExec wrapping Thompson-NFA instruction-program"
  - "ADR-PROP-C6-LEAN-2: Continuous/incremental RPR — SAME matcher core + watermark/timer/checkpoint layer on RocksDB CFs"
  - "ADR-PROP-C6-LEAN-3: Sigma → PrismQL — pySigma-style Backend + ProcessingPipeline targeting OCSF taxonomy; fidelity report for lossy edges"
produced_by: architect
timestamp: "2026-06-27"
provenance: >
  Side-analysis C6 capture; human-confirmed decisions 2026-06-27 session.
  Research basis: research/detection-engine-depth-2026-06-27.md (six perplexity_research
  sonar-deep-research calls at reasoning_effort=high covering Q1 MATCH_RECOGNIZE NFA/operator,
  Q2 incremental/streaming RPR, Q3 federated backtesting, Q4 staged rollout/auto-rollback,
  Q5 Sigma→PrismQL, Q6 FP auto-tune/suppression; plus 2 Context7 calls for DataFusion API).
  Does NOT modify live ADR files, ARCH-INDEX.md, STATE.md, SESSION-HANDOFF.md, or any live
  factory artifact.
traces_to:
  - matured-vision-day2-requirements.md §14 (Detection Engine — HUMAN-CONFIRMED 2026-06-25)
  - matured-vision-day2-requirements.md §14.2 (correlation type coverage; G-18 MATCH_RECOGNIZE)
  - matured-vision-day2-requirements.md §14.3 (federated/ephemeral adaptation; backtesting over cold tier)
  - matured-vision-day2-requirements.md §14.4 (rule editor — staged rollout; backtest panel; exception manager; auto-tune)
  - matured-vision-day2-requirements.md §14.5 (source-coverage-record + replay-link; ADOPT-4)
  - matured-vision-day2-requirements.md §14.7 (content library; Sigma import; recipe library)
  - matured-vision-day2-requirements.md §14.8 (epics G-18/G-19; E-DETECT-ENGINE-001 / E-DETECT-SEQUENCE-001)
  - matured-vision-day2-requirements.md §16.4 (C6 decisions log entry)
  - matured-vision-day2-requirements.md §17.7 (continuous-operator capability, phased)
  - matured-vision-day2-requirements.md §17.14 (synthesis — reuse MATCH_RECOGNIZE NFA; dual impl for WATCH…UNLESS)
  - day2-design-decisions/ADR-PROP-capability-descriptor-pushdown.md (C3 — Q3 backtest query path inherits join-guard + time-bound)
  - day2-design-decisions/ADR-PROP-siem-lake-federation.md (C5 — Iceberg cold tier read + snapshot-id time-travel for deterministic backtests)
  - epics E-DETECT-ENGINE-001, E-DETECT-SEQUENCE-001, E-DETECT-EDITOR-001, E-ALERT-ROUTING-001, E-RULE-XLATE-001
  - research/detection-engine-depth-2026-06-27.md (primary research basis — all six Q1–Q6 depth questions)
  - CLAUDE.md (#[non_exhaustive] discipline; SAP-1 structured event catalog; error taxonomy E-QUERY-NNN; RocksDB CFs)
---

# ADR-PROP — Detection Engine Depth (C6)

> **STATUS: DECIDED 2026-06-27 (human) — EXCEPT D-C6-3 (auto-rollback) which is DEFERRED pending
> deep-research pass.** This is a CAPTURE artifact for the side-analysis C6 program.
> `do_not_execute: true`. Real ADR numbers and formal ARCH-INDEX.md rows are deferred to the morph
> execution (post-demo, post-T14, gated on brief-reframe sign-off §5.1).

> **Research basis:** `research/detection-engine-depth-2026-06-27.md` — six
> `perplexity_research` (sonar-deep-research, `reasoning_effort=high`) calls covering:
> Q1 MATCH_RECOGNIZE/NFA custom operator on DataFusion; Q2 streaming/incremental RPR over
> RetentionCache + §17.7 continuous operator; Q3 backtesting over federated + Iceberg cold-tier
> sources (gap G-19); Q4 staged rollout + auto-rollback; Q5 Sigma→PrismQL translation
> (deferred E-RULE-XLATE-001, feasibility confirmed); Q6 FP auto-tune + suppression (no silent
> masking). Plus 2 Context7 calls for DataFusion API surface. All load-bearing claims are
> source-grounded in that document.

> **Settled decisions NOT relitigated in this capture.** The following §14 decisions are
> HUMAN-CONFIRMED 2026-06-25 and are treated as the immutable foundation for C6:
> detection-as-query (PrismQL, not a separate DSL); custom MATCH_RECOGNIZE operator built
> Phase-A-from-start (full NFA); `SEQUENCE…THEN…WITHIN` sugar desugaring to `MATCH_RECOGNIZE`;
> correlation state on RocksDB/RetentionCache (no new datastore); detection spec carries EXPLICIT
> temporal semantics (planner picks engine, never semantics); rule editor on S2-console + MCP +
> CLI (no TUI); OT detection in scope; alert model + routing + destinations.

---

## Context

Section 14 of the matured-vision establishes the detection engine model at the requirements level.
This C6 ADR-PROP captures DEPTH and IMPLEMENTATION decisions — HOW to build the open pieces that §14
left unresolved. Six open implementation questions drove the research pass (research/detection-engine-depth-2026-06-27.md):

1. **G-18 / Q1:** How to build the `MATCH_RECOGNIZE` custom operator on DataFusion specifically (the keystone).
2. **Q2 / §17.7:** How to extend the NFA matcher core to the continuous/incremental streaming operator.
3. **G-19 / Q3:** What backtesting posture to adopt for federated + Iceberg cold-tier sources.
4. **§14.4 / Q4:** What staged-rollout and auto-rollback design to adopt.
5. **E-RULE-XLATE-001 / Q5:** Whether Sigma→PrismQL translation is feasible and how to handle lossy edges.
6. **§14.4 / Q6:** How to design false-positive auto-tune and suppression without silent masking.

C6 resolves Q3, Q6, and the key implementation leans for Q1/Q2/Q5. Q4 auto-rollback is
deferred (D-C6-3 / OQ-C6-AUTOROLLBACK) pending a focused deep-research pass.

---

## Decision Ledger

### D-C6-1 — Backtesting Posture: BOTH Cold-Tier Deterministic AND Remote Best-Effort, ALWAYS With Coverage Map

**DECIDED 2026-06-27 (human). Resolves gap G-19.**

Prism backtests via **two sources in a single run**, with a mandatory coverage map in every output:

**Tier A — Iceberg cold tier (deterministic, reproducible):**
- Pin `snapshot-id` (or `as-of-timestamp`) + `rule-version` as the backtest key pair.
  A pinned `snapshot-id` gives reproducible, point-in-time reads even after later appends — the
  Iceberg time-travel primitive designed for exactly this use case. [Iceberg-TimeTravel]
- Ensures auditable re-runs: the same `(snapshot-id, rule-version)` pair always produces the same
  result. Avoids look-ahead bias (the data the rule "sees" cannot include events that arrived after
  the snapshot).
- Relies on C5 cold-tier decisions: Apache Iceberg + iceberg-rust (ASF primary, JanKaul fallback);
  write path = append-only RETAIN (single-writer, REST or S3 Tables catalog); `ingest_time` per row
  (supports append-only as-of approximation for audit).

**Tier B — Remote API sources (best-effort, non-deterministic):**
- Re-query the live connector using a time-bounded scan (C3 mandatory time-bound, D-C3-2).
- Explicitly NON-DETERMINISTIC: retention limits, schema evolution, and connector onboarding dates
  mean the same time range may return different rows in a different run.
- **The coverage map says so plainly.** Results from Tier B must be labeled non-deterministic in the
  output. Do not imply reproducibility.

**The mandatory coverage map:**
Every backtest run emits a **coverage map** as a first-class output alongside the hit count. The
coverage map records, per (source × time-slice), one of:
- `full` — source was onboarded at this time, retention covers this slice, schema version at this
  time matches the rule's field set, no query errors.
- `partial` — some but not all of the above hold (e.g., retention covers the slice but the
  connector was onboarded mid-window, or schema-version drift causes some field lookups to be absent).
- `none` — source had no data to evaluate for this slice (not onboarded, retention expired, or a
  query error prevented evaluation entirely).

The coverage map derivation: `(source retention window) ∩ (connector onboarding date) ∩ (query
error log) ∩ (schema-version availability)`.

**The single most important correctness affordance — distinguish "evaluated, no match" from
"no data to evaluate."** A clean backtest (zero hits) MUST NOT be displayed as "low FP rate" or
"no threat" when it is actually "no data to evaluate for several time slices." Every prior-art
platform surveyed (Elastic, Chronicle/SecOps, Panther, Splunk) fails this check — none offer a
unified coverage map. This is genuinely novel; Prism builds it from scratch. [Q3 research: §3.3
"the gap the entire prior art is missing"; Panther-Replay, Chronicle-Retrohunt, Elastic-Validate]

**Cost controls (mandatory, generalized from Panther envelope):**
- Dry-run / estimate mode before full execution (compute coverage map + row-count estimate without
  returning result rows).
- Mandatory time-bound per run (C3 D-C3-2 — no unbounded backtest scans).
- Per-run volume ceiling (maximum rows scanned / maximum wall-clock time). Operator-configurable.
- Guidance: stop / sample when preliminary match rate is high (pathological rules create noise, not
  signal, during backtest).

**Reuse §14.5 ADOPT-4 machinery:** the source-coverage-record + replay-link (already specified in
§14.5) is the substrate the coverage map builds on. The backtest coverage map is a structured
extension of the per-run `source_coverage_record` — same data model, additional `full/partial/none`
labeling per slice.

**Why NOT cold-tier-only:** federated remote sources are the primary data path for many customers;
backtesting only against the Iceberg tier would miss most of the corpora these rules would fire
against. The non-determinism of remote sources is a reality to manage honestly, not a reason to
exclude them. The coverage map is the honesty mechanism.

[research/detection-engine-depth-2026-06-27.md §Q3, §3.4 LEAN]

---

### D-C6-2 — False-Positive Handling: RBA as Default; Suppression-as-Code With Mandatory Justification + Time-Box Expiry

**DECIDED 2026-06-27 (human).**

**Risk-Based Alerting (RBA) as the DEFAULT posture over hard suppression:**
Adopt the Splunk RBA philosophy: noisy events accrue **risk** to an entity rather than being
suppressed. Alert on aggregated entity risk exceeding a threshold — retaining visibility into the
underlying events while reducing alert volume. [Splunk-RBA]

The RBA default means:
- An analyst tuning for noise does NOT configure "suppress this event type globally." They configure
  "accrue these events as risk points to the source entity; alert when the entity's rolling risk
  score exceeds N."
- The underlying events remain queryable, auditable, and visible in investigation workflows.
- Volume reduction is achieved by changing the *alert surface* (risk aggregation), not by dropping
  the *event evaluation*.

**Hard suppression is the exception, not the default. When allowed: suppression-as-code.**
There are legitimate cases for hard suppression (known-benign automated activity, test traffic,
monitoring self-signals). Every suppression object MUST satisfy all of:
1. **Versioned** — suppression is a typed object in the detection repo (YAML/TOML), subject to
   the same git history, review, and CI validation as detection rules.
2. **Mandatory justification** — every suppression carries a human-authored `justification:` field
   explaining why this suppression is safe. An empty or boilerplate justification fails CI.
3. **Mandatory time-box expiry** — every suppression carries an `expires_at:` field. No immortal
   suppressions. Expiry forces re-review: when a suppression expires, CI flags it for re-evaluation
   (confirm the suppression is still appropriate, narrow the scope, or remove it).
4. **Scope as narrow as possible** — suppression must target specific entity/source/rule
   combinations, not broad categories. Overly-broad scope is a finding at review.

**Auto-tune emits suggestions only:**
The auto-tune subsystem (driven by analyst disposition history — TP/FP labels from §14.5) may
emit suggestions: threshold delta proposals, candidate exclusion candidates, frequency anomalies.
Auto-tune suggestions MUST be presented to a human for acceptance. The system MUST NOT:
- Auto-apply any suppression or threshold change.
- Auto-disable any detection's evaluation pipeline.
- Act on a suggestion without human sign-off.
Suggestions are a UI/dashboard affordance, not an autonomous tuning loop.

**Suppression fire-frequency dashboard (mandatory):**
A per-suppression dashboard surfacing:
- How often this suppression fires (events suppressed per time period).
- The scope breadth (how many distinct sources/entities are being suppressed).
- Time until expiry (and a flag when expired/due for review).

This catches: over-broad suppressions (firing thousands of times/day → likely papering over a
detection problem, not a legitimate suppression), stale suppressions (never fires anymore →
the underlying source probably changed; safe to remove), and scope creep (suppression that was
once narrow is now hitting many more entities than intended). [Elastic-Suppress, Sentinel-Tuning]

**HONEST CAVEAT — stated plainly, not implied away:**
"Never silently mask a true positive" is **not achievable as an absolute guarantee.** Any
suppression, however narrow and well-justified, carries risk of masking a true positive. The
production-grade posture is: transparency + mandatory justification + time-boxed expiry +
fire-frequency-dashboard + RBA-over-suppression. This is the best achievable posture; it is NOT
a proof. The spec must say this plainly rather than implying a guarantee. [research Q6 §6.2 honest
caveat; Splunk-RBA, Elastic-Suppress — none claim an absolute guarantee either]

[research/detection-engine-depth-2026-06-27.md §Q6, §6.2 LEAN]

---

### D-C6-3 — Auto-Rollback (Staged Rollout FP-Spike Circuit-Breaker): DEFERRED

**DEFERRED 2026-06-27 (human-directed). OPEN ITEM: OQ-C6-AUTOROLLBACK.**

A focused deep-research pass is in flight (file will be at
`/Users/jmagady/Dev/prism/.factory/research/detection-auto-rollback-depth-2026-06-27.md`).
This decision is recorded as OPEN and will be resolved when that research returns.

**What the deferred research pass must cover:**
- Automated canary analysis (Kayenta/Flagger approach) applied to detection rules.
- Bad-rollout detection without ground-truth labels (change-point / CUSUM / Page-Hinkley on alert
  volume as a proxy signal for FP spikes; FP-spike detection from alert volume alone is inherently
  ambiguous with legitimate-spike-vs-noise discrimination).
- Circuit-breaker design: what signal triggers rollback + hysteresis (how long must the signal
  persist before rollback fires; how long must the signal clear before re-promotion is allowed).
- The rollback-action fork — the single most important unresolved question:
  a. **Demote-to-shadow (keep evaluating, stop routing):** the detection keeps running; findings
     write to a shadow stream; no alert routing. Coverage is preserved. The analyst can inspect
     shadow findings to diagnose whether it was a real spike or a false alarm.
  b. **Full disable (stop evaluating entirely):** faster cost savings; harder to re-enable safely;
     loses coverage entirely during the disabled window.
  c. **Revert-version:** roll back to the prior rule version if available; evaluating against old
     semantics may also generate noise if the data changed.
- Promotion gates: what criteria must a rule satisfy before graduating from shadow → canary →
  production; where thresholds live (§14.1 `quality` block vs per-deployment config).
- Canary unit choice: tenant (Prism is already multi-tenant → natural canary scope) vs asset-group
  vs traffic sample.

**GUARDRAIL THAT HOLDS REGARDLESS OF THE DEFERRED MECHANISM:**
Even before the rollback-action fork is decided, one invariant is settled and must be respected
by whichever mechanism is chosen:

> **Auto-rollback MAY disable routing. It MUST NOT auto-disable evaluation without human sign-off.**
>
> The production-grade default: a detection that is auto-rolled-back continues EVALUATING and
> writing findings to a shadow or monitoring stream. Coverage is never silently zeroed by an
> automated circuit-breaker. Auto-disabling a detection's evaluation pipeline — not just routing,
> but the evaluation itself — requires explicit human approval.

This guardrail is identical to D-C6-2's auto-tune guardrail: the automation may surface a problem
and take protective routing action; it may not decide the detection is no longer valid.

**Canary unit lean (recorded, not binding):** tenant is the natural scope for Prism's multi-tenant
architecture. Confirm at morph-time alongside the deferred rollback-action fork resolution.

**Relationship to D-C6-2:** The staged-rollout shadow phase (§14.4) directly feeds the
suppression fire-frequency dashboard (D-C6-2): shadow mode alerts write to a non-routed stream
from which fire-frequency and precision signals can be computed before promotion. The two
mechanisms reinforce each other.

**Resolution instruction:** "Fold on return" — when `research/detection-auto-rollback-depth-2026-06-27.md`
returns, resolve OQ-C6-AUTOROLLBACK by appending a `### D-C6-3-RESOLVED` section to this document
with the decided rollback-action fork, promotion-gate thresholds, and circuit-breaker design.

[research/detection-engine-depth-2026-06-27.md §Q4, §4.2 LEAN; Flagger; ML-Canary]

---

## Implementation Leans (Confirmed 2026-06-27, Human Non-Objection)

These leans were presented in the research, not objected to by the human, and are captured as
decided implementation directions for morph-time ADR authorship. They are NOT binding decisions
at the ADR-PROP level — the architect confirms against the pinned DataFusion version at morph.

### L-C6-1 — MATCH_RECOGNIZE Operator: Thompson-NFA Instruction-Program Build Shape

**Confirmed lean (research Q1). Resolves G-18 implementation shape.**

Build the MATCH_RECOGNIZE operator as a custom DataFusion logical/physical operator pair:

**Step 1 — Grammar / desugar (specified in §12.4/§14.2.1):**
`SEQUENCE…THEN…WITHIN` (Chumsky grammar) → `MATCH_RECOGNIZE` AST. Power users may write raw
`MATCH_RECOGNIZE` directly.

**Step 2 — Pattern compiler:**
`MATCH_RECOGNIZE` AST → Thompson-NFA **instruction program** (`MATCH` / `SPLIT` / `JUMP` /
`CHECK` / `ACCEPT` ops).
- Greedy / reluctant quantifiers = SPLIT branch ordering (greedy: loop-back first; reluctant:
  exit first). This is the Trino `RowPatternMatcher` model. [Trino-RPR]
- Anchors = zero-consumption position checks.
- `{m,n}` = concatenation + optional tails (standard Thompson desugaring).
- **Compile-time rejection** of two malformed cases:
  1. Empty-match × `SKIP TO FIRST <var>` where the skip target can bind the start row →
     infinite-loop; SQL:2016 prohibits this; the compiler must detect and reject.
  2. SKIP-target variable that may be unbound in some match (optional/alternation paths) →
     undefined SKIP behavior; compiler rejects.
- DEFINE predicate evaluation uses RUNNING semantics (only the prefix matched so far is visible);
  MEASURES output uses FINAL semantics by default (whole match visible), with RUNNING/FINAL
  tagable per measure expression.
- `AFTER MATCH SKIP` modes: `PAST LAST ROW` (non-overlapping, default), `TO NEXT ROW`,
  `TO FIRST/LAST <var>` (requires per-match first/last index metadata per variable). [Oracle-Skip][Trino-RPR]

**Step 3 — Logical node:**
`MatchRecognizeNode : UserDefinedLogicalNode` carrying:
- `PARTITION BY` expressions (for required input distribution).
- `ORDER BY` expression (= `event_time`; for required input ordering).
- Compiled instruction program + DEFINE predicate evaluators.
- MEASURES expressions (tagged RUNNING or FINAL).
- `rows_per_match` mode (ONE ROW / ALL ROWS PER MATCH).
- `AFTER MATCH SKIP` mode.

DataFusion extension point: `RelationPlanner` custom-node strategy. [C7-DF-extending]

**Step 4 — Physical node:**
`MatchRecognizeExec : ExecutionPlan`.
- Declare required input distribution = hash-partition on `PARTITION BY` keys → planner inserts
  `RepartitionExec`.
- Declare required input ordering = sort on `event_time` → planner inserts `SortExec`.
- `properties()` → `PlanProperties { EmissionType::Incremental, Boundedness::Bounded }` for the
  initial batch-over-window mode.
- `execute(partition, ctx)` returns a `RecordBatchStreamAdapter` driving the NFA simulation per
  partition. Emit match rows incrementally as matches complete.
[C7-DF-execplan][C7-DF-counting][C7-DF-physical]

**Step 5 — Match-context data structure (designed for continuous reuse from day one):**
Per live NFA run: `program_counter` + per-variable ordered binding lists + running aggregates +
first/last index per variable (for SKIP). **Must be serializable to a flat byte representation
from day one** — the continuous operator (L-C6-2 / §17.7 Phase 2) checkpoints this to RocksDB CFs
unchanged. Designing for serializability at the batch stage avoids a redesign at the streaming stage.

**Step 6 — Optimizer fast-path (§14.2 Phase B):**
A `RelationPlanner` rewrite detects simple fixed-step `SEQUENCE` patterns (no alternation, no
Kleene quantifiers, fixed N steps) and rewrites to self-joins + window (Microsoft "RPR Using
Joins" pattern, confirmed 5.4× speedup). [MS-RPR][C7-DF-extending] This Phase B rewrite is the
optimizer fast-path only; the full NFA operator (Phase A) is the general case.

**Why the Trino instruction-program (not the Flink pointer-graph):**
- Reluctant-quantifier ordering is simpler to implement correctly as SPLIT branch ordering than as
  graph edge reordering.
- A flat program-counter + binding list is easier to serialize to RocksDB CFs for checkpointing
  than a pointer-graph with object-identity semantics.
- Trino's production deployment validates the correctness of the model. [Trino-RPR]

**INCONCLUSIVE at this stage:**
The exact DataFusion `ExtensionPlanner`/`QueryPlanner` trait method signatures for the pinned
DataFusion version must be re-verified at build time. Context7 confirmed the *strategy* (`UserDefinedLogicalNode` + custom `ExecutionPlan`, `RelationPlanner` rewrite for the fast-path) but exact
method signatures are pinned-version-dependent. Flag as **PIV-C6-1: re-verify ExtensionPlanner
wiring against the pinned DataFusion version before implementation.**

[research/detection-engine-depth-2026-06-27.md §Q1, §1.3 LEAN, §Recommended build approach]

---

### L-C6-2 — Continuous / Incremental RPR (§17.7 Phase 2): SAME Matcher Core + Checkpoint Layer

**Confirmed lean (research Q2 / §17.7 / §17.14).**

The continuous/incremental operator (§17.7 Phase 2 — "prism-NATIVE windowed operator on RocksDB
state backend") is NOT a redesign of the matching logic. It is the L-C6-1 matcher core wrapped
with a thin checkpoint/state-management layer:

**What is added on top of the Phase A batch matcher:**

1. **Watermark + event-time TTL pruning.** Drive window closure and partial-match expiry off the
   §17.8 `event_time` TTL (data-intrinsic freshness, already part of the retention design). When
   the event-time watermark passes a partial match's `start_time + WITHIN_window`, the partial match
   is expired and the SharedBuffer entry reference-count is decremented.

2. **SharedBuffer-equivalent with reference counting.** Each event entering the window is stored
   once; multiple partial-match runs that share a prefix reference the same event via a counted
   handle. When a partial-match run dies (timeout, match completion, skip-policy GC), it decrements
   the ref-counts for its events; events with zero references are freed. This bounds memory under
   nondeterministic NFA spawning. [Flink-NFA][Flink-State]

3. **Per-partition timer for absence / non-event detection (`WATCH…UNLESS`).**
   Absence over an unbounded stream is undecidable without a deadline; a relational anti-join
   cannot know whether a future event will arrive. The continuous-path `WATCH…UNLESS` / `not
   followed-by within W` implementation uses a per-partition event-time timer registered when the
   triggering event `A` arrives (timer fires at `t_A + W`). If the absence-breaking event `B`
   arrives before the timer fires, cancel the timer and drop the partial match. If the timer fires
   with no `B`, emit the absence match. This is R1 in the §17.14 dual-implementation ruling:
   `AbsenceWindowNode` for polled/batch (relational anti-join) vs per-partition CEP timer for
   continuous. [Aliyun-CEP][Esper][Flink-NFA; the "two modes" match §17.14 verbatim]

4. **Incremental checkpoint cadence on RocksDB CFs (§17.14 open question #1 — partially deferred).**
   The Flink incremental-SSTable model (checkpoint only changed SSTables since the last checkpoint;
   delta upload, shared handles for unchanged files) is the validated template. The concrete cadence
   split — window-state CF cadence vs durable `detection_state` CF cadence vs ML `ModelState`
   cadence — is §17.14's own "open question #1" and is left to architect at morph. This lean
   records the template, not the cadence values.
   - **Window-state CF** (fast-changing, short-lived NFA states + SharedBuffer entries) and
     **`detection_state` CF** (durable alert/correlation state) must be distinct CFs within the
     existing 19-CF RocksDB engine (§17.14 state-unification DECIDED-LEAN). `ModelState` for ML
     (§15) is logically separable.
   - Whether the window-state and detection_state CF boundary provides sufficient isolation under a
     shared checkpoint stream (§17.14 honest-cost #5: fast operator state coupled to slow campaign
     state) is an open question. Flag as **PIV-C6-2: validate CF boundary isolation sufficiency
     under shared-checkpoint semantics at morph.**

5. **`Boundedness` flip.** The Phase A batch operator declares `Boundedness::Bounded`. The
   continuous operator flips to `Boundedness::Unbounded`. This is the one physical-plan property
   change between the two modes; the matcher core is otherwise unchanged.

**HONEST COST (from §17.7 self-assessment):**
The continuous/incremental operator is the single most expensive item in the detection engine.
The matcher core (L-C6-1) is the cheaper part. The real build is: watermarks, event-time, late-
arrival handling, fault-tolerant incremental checkpointing, per-partition timers, SharedBuffer
reference-counting, and the CF-boundary isolation. Every one of these is novel infrastructure.

[research/detection-engine-depth-2026-06-27.md §Q2, §2.4 LEAN; Flink-State; Aliyun-CEP; §17.7]

---

### L-C6-3 — Sigma → PrismQL: pySigma-Style Backend + OCSF ProcessingPipeline; Fidelity Report for Lossy Edges

**Confirmed lean (research Q5). Remains deferred (E-RULE-XLATE-001) but feasibility is
confirmed. Sigma→PrismQL examples in the recipe library (§14.7) are NOT deferred.**

**Feasibility is confirmed.** Single-event + selection + threshold/distinct-count Sigma rules
map cleanly to PrismQL `WHERE` / `GROUP BY … HAVING`. [pySigma, Sigma-Spec]

**Strategic alignment — Sigma correlation rules:** Sigma correlation rules (`event_count`,
`value_count`, `temporal`, `temporal_ordered` types) are a newer, thinly-supported Sigma class.
Across surveyed backends (Splunk SPL, Elasticsearch), they receive only partial support. Prism's
`MATCH_RECOGNIZE` / `SEQUENCE…THEN` operator (L-C6-1) is precisely the temporal-pattern primitive
that makes `temporal` and `temporal_ordered` Sigma correlations expressible as first-class
queries, not approximations. Prism is unusually well-positioned here. [Sigma-Correlation]

**Build approach (at morph, when E-RULE-XLATE-001 is executed):**
- Implement a pySigma-style `Backend` targeting PrismQL, paired with a `ProcessingPipeline` that
  maps Sigma field names to the **OCSF taxonomy** (extend the existing community Sigma→OCSF pipeline
  as the starting field map, then extend/correct to a fuller spec).
- `logsource` → OCSF class/table selection.
- `detection` block → PrismQL `WHERE` / `GROUP BY … HAVING`.
- Sigma correlation types → PrismQL `MATCH_RECOGNIZE` / `SEQUENCE…THEN` for `temporal` and
  `temporal_ordered`; `GROUP BY … HAVING COUNT(*) > N` for `event_count`/`value_count`.
- Sigma `status: experimental|test|stable` → Prism lifecycle states (§14.1 `status` field).

**Lossy edges — fidelity report (NEVER silent drop):**
These Sigma constructs cannot be losslessly translated to PrismQL and MUST be flagged in the
fidelity report:
- `base64offset`: multiple literal expansions; no PrismQL equivalent.
- `windash`: dash variants; no native wildcard.
- Exotic `re` regex dialects: lookaround, backreferences, Unicode-specific escaping not supported
  in PrismQL regex.
- Class-spanning correlations: a Sigma correlation joining events from OCSF classes stored in
  separate tables requires a cross-source join (legal under C3 D-C3-1, but may be expensive and
  needs the join-guard stack).

The fidelity report emits, per translated rule, a list of every modifier/condition that could not
be losslessly expressed, with the original Sigma expression and the approximate PrismQL
approximation (if one exists) or a `NOT_EXPRESSIBLE` flag. The analyst can decide whether to
accept the approximate translation or translate manually. **The fidelity report is the Q5 analog
of the D-C6-1 coverage map** — both are honesty mechanisms that never silently drop information.

**Ship Sigma→PrismQL examples in the recipe library NOW (§14.7), regardless of deferral:**
The recipe library (§14.7 content) should include Sigma→PrismQL translation examples validating
the mapping surface. These examples can be hand-authored and serve as both documentation and as
the first test vectors for the eventual pySigma backend.

[research/detection-engine-depth-2026-06-27.md §Q5, §5.3 LEAN; pySigma; Sigma-Correlation]

---

## Open Questions (Architect / Morph-Time)

| # | Question | Status | Dependency |
|---|---------|--------|------------|
| **OQ-C6-AUTOROLLBACK** | Auto-rollback: rollback-action fork (demote-to-shadow vs full-disable vs revert-version), circuit-breaker design + hysteresis, promotion-gate thresholds, legitimate-spike-vs-noise discrimination, canary unit | **DEFERRED — pending deep-research pass** at `research/detection-auto-rollback-depth-2026-06-27.md`. Fold on return; amend D-C6-3 with a `D-C6-3-RESOLVED` section. | Deep-research pass (in flight) |
| **PIV-C6-1** | DataFusion `ExtensionPlanner`/`QueryPlanner` exact method signatures for the pinned DataFusion version; re-verify `UserDefinedLogicalNode` + custom `ExecutionPlan` + `RelationPlanner` rewrite wiring against that version's source | Pre-implementation verification — read DataFusion source at Prism's pinned version + integration test with `EXPLAIN VERBOSE` | Morph-time, before MATCH_RECOGNIZE implementation starts |
| **PIV-C6-2** | Window-state CF ↔ `detection_state` CF boundary isolation under a shared checkpoint stream (§17.14 honest-cost #5) | Pre-implementation verification — validate that fast operator state and slow campaign state can checkpoint independently without coupling. Empirical test. | Morph-time, before continuous operator work starts |
| **OQ-C6-3** | Continuous-operator checkpoint cadence — window-state CF cadence vs durable `detection_state` CF cadence vs ML `ModelState`. Flink incremental-SSTable model is the template; concrete cadence values are left to architect. (§17.14 open question #1) | Open architect decision at morph | Morph-time |
| **OQ-C6-4** | Backtest coverage-map data model — how `(source retention window) ∩ (connector onboarding date) ∩ (query error log) ∩ (schema-version availability)` compose into per-slice `{full/partial/none}`. Reuse §14.5 ADOPT-4 source-coverage-record schema or extend it? | Open design decision — ties C5 cold-tier + §14.5 machinery | Morph-time, before E-DETECT-ENGINE-001 implementation |
| **OQ-C6-5** | Iceberg snapshot-retention policy for reproducible backtests — how long snapshots must be pinned/retained for audit re-runs vs cold-tier `expire_snapshots` cost. Ties to C5 Iceberg write path. | Open — ties to `iceberg_snapshot_retention_policy` morph-time decision | Morph-time |
| **OQ-C6-6** | Sigma fidelity-report schema — exact field set for the per-rule lossy translation report (fidelity report for Q5). | Open design decision at morph | Morph-time, before E-RULE-XLATE-001 |

---

## Downstream SAP-1 Obligations (Not Actioned Here)

Several new event types implied by C6 decisions will need BC-2.16.002 Canonical Structured Event
Catalog rows at morph time. Flagged here; NOT actioned (SAP-1 probe scope is per-story at
implementation time, not at ADR-PROP capture time).

- `event_type = "detection.backtest.run"` — emitted when a backtest run completes; fields include
  `rule_id`, `rule_version`, `snapshot_id` (if cold-tier), `time_range`, `tier_a_result` (cold,
  deterministic), `tier_b_result` (remote, best-effort), `coverage_map` (per-source per-slice).
  Audit role = backtest audit; recurrence = per run.
- `event_type = "detection.backtest.coverage_gap"` — emitted per (source × time-slice) classified
  as `none` or `partial`; fields include rule_id, source_id, time_slice, gap_reason
  (retention_expired / not_onboarded / query_error / schema_version_absent).
  Audit role = backtest correctness audit; recurrence = per gap slice in a run.
- `event_type = "detection.suppression.applied"` — emitted when a suppression object fires; fields
  include suppression_id, suppression_version, rule_id, source_entity, match_count.
  Audit role = suppression audit (enables fire-frequency dashboard); recurrence = per event batch
  suppressed.
- `event_type = "detection.suppression.expired"` — emitted when a suppression reaches its
  `expires_at` date; fields include suppression_id, last_fire_time, scope.
  Audit role = suppression lifecycle audit.
- `event_type = "detection.autotune.suggestion"` — emitted when auto-tune generates a threshold
  delta or exclusion suggestion; fields include rule_id, suggestion_type, proposed_value, basis
  (disposition history window). NOT an action — human sign-off required.
- Rollout-transition events (shadow→canary promotion, canary→production, rollback trigger) — schema
  deferred to OQ-C6-AUTOROLLBACK resolution.

All six categories above are flagged; BC-2.16.002 catalog rows are morph-time work.

---

## Honest Costs

| Item | Cost / Risk |
|------|-------------|
| **MATCH_RECOGNIZE is a real engine operator, not a query rewrite.** | DataFusion parses it but does not execute it; the core team has low appetite to add it. Prism owns the full lifecycle: pattern compiler, NFA simulation, SKIP/empty-match correctness, MEASURES RUNNING/FINAL, SQL:2016 edge cases. The correctness bugs hide in empty-match × SKIP interactions and greedy/reluctant nesting. [Trino-RPR][Oracle-Skip] |
| **The continuous operator is the single most expensive item in §14.** | Watermarks, event-time, late-arrival, fault-tolerant incremental checkpointing on RocksDB, per-partition timers, SharedBuffer reference-counting — the matcher core (L-C6-1) is the cheap part; this temporal/fault-tolerance layer is the real build. Ordered later as a whole feature (§17.7 self-identifies this). |
| **Federated backtesting is genuinely novel.** | No surveyed security platform (Elastic, Chronicle, Panther, Splunk, Sigma/pySigma) documents backtesting over federated remote sources without first ingesting locally. Iceberg time-travel gives determinism on the cold tier; remote-API sources are best-effort/non-deterministic and retention-bounded. The coverage map ("evaluated-no-match" vs "no-data") is unprecedented in the prior art — Prism builds it from scratch. [Q3 survey findings] |
| **Auto-rollback is not a shipped SIEM feature.** | Must be assembled from Prism primitives borrowing Flagger's control-loop model. The deferred deep-research pass (D-C6-3 / OQ-C6-AUTOROLLBACK) must resolve the rollback-action fork before this is implementable. Anvilogic/SnapAttack detection-as-code specific practices were INCONCLUSIVE in the research. [Flagger, INCONCLUSIVE on Anvilogic/SnapAttack] |
| **Sigma→PrismQL is lossy at the edges.** | `base64offset`, `windash`, exotic regex dialects, class-spanning correlations. A fidelity report (never silent drop) is required. Sigma correlation rules are thinly supported across all backends; Prism's MATCH_RECOGNIZE is the advantage. [pySigma][Sigma-Correlation] |
| **"Never silently mask a true positive" is not achievable as an absolute guarantee.** | State this plainly. The production-grade posture (RBA + transparency + time-boxing + fire-frequency dashboards) is the best achievable, not a proof. [D-C6-2 honest caveat] |

---

## Alternatives Considered and Rejected

### Alternative A: Cold-Tier-Only Backtesting (Reject Remote Sources)

Restrict backtesting to the Iceberg cold tier, which gives fully deterministic, reproducible runs.
Excludes remote sources due to their non-determinism.

**Rejected (D-C6-1) because:**
- Most customers' relevant data history for detection tuning lives in remote sources (sensors with
  limited `RETAIN` windows, no existing cold-tier population). Cold-tier-only backtesting would be
  near-useless for most day-2 deployments.
- Non-determinism in remote sources is a reality to manage honestly, not a reason to exclude them.
- The coverage map (D-C6-1) is the honesty mechanism: it explicitly labels non-deterministic slices.
  Excluding remote sources would be less honest (the analyst would not know why coverage was absent)
  than including them with explicit labeling.

### Alternative B: Hard Suppression as the Default FP-Handling Posture (Reject RBA)

Configure suppressions as the primary FP-reduction mechanism, with risk aggregation as an optional
add-on.

**Rejected (D-C6-2) because:**
- Hard suppression silently discards events that match the suppression predicate, permanently
  reducing visibility. Over-broad suppressions create detection blind spots.
- RBA retains the underlying events as queryable data while reducing alert volume via aggregated
  risk scores. It is strictly more transparent.
- The research shows no surveyed platform can guarantee "never silently mask a true positive" with
  any suppression mechanism. The RBA-as-default posture reduces this risk surface.
- Hard suppression is still available as an exception (suppression-as-code with mandatory
  justification + expiry), not forbidden.

### Alternative C: Auto-Tune Autonomous Application (No Human Sign-Off Required)

Allow auto-tune to apply threshold adjustments and suppression additions autonomously when
the disposition data exceeds a confidence threshold.

**Rejected (D-C6-2) because:**
- Autonomous application is an autonomous suppression: even if "confident," the system is making
  a decision to reduce or eliminate alert visibility without human review. Under the production-grade
  default principle, this is an unacceptable default.
- Detection-as-code (git-versioned, CI-validated) makes the human review step cheap — a suggestion
  shows up as a diff the analyst approves. The cost savings from bypassing human review are minimal.
- A confident but wrong autonomous suppression silently degrades coverage in a way that is hard to
  detect and attribute. The honest-caveat (D-C6-2) implies this risk.

### Alternative D: Single-Matcher Design (Batch and Continuous Use Different Codepaths)

Design the batch MATCH_RECOGNIZE operator (L-C6-1) and the continuous streaming operator (§17.7)
as entirely separate implementations with different matcher internals.

**Rejected (L-C6-2 / §17.14) because:**
- §17.14 explicitly decided that the continuous operator "reuses the MATCH_RECOGNIZE NFA operator
  prism already owns." This is an architectural constraint from the settled decisions, not a lean.
- Separate implementations would double the correctness surface: two NFA simulators to validate,
  two sets of SQL:2016 edge cases to handle, two serialization schemas. All of the hardest
  correctness bugs (empty-match infinite loops, SKIP interactions, DEFINE predicate evaluation under
  RUNNING semantics) would need to be solved twice.
- Shared matcher core + thin checkpoint layer is the Flink CEP lesson: the matcher NFA is the
  well-understood part; the state-management layer is where the work is. Keep the matcher correct
  once.

---

## Ripple Effects (Must Be Picked Up at Morph Time)

| Affected area | Ripple |
|---------------|--------|
| **E-DETECT-ENGINE-001** | D-C6-1 defines the backtest coverage-map requirement + cost-control envelope. The epic's acceptance criteria for backtesting must include the coverage map as a mandatory output, the deterministic cold-tier path (snapshot-id + rule-version), the best-effort remote path (explicitly labeled non-deterministic), and the "evaluated-no-match" vs "no-data" UI distinction. |
| **E-DETECT-SEQUENCE-001** | L-C6-1 concrete build shape (UserDefinedLogicalNode + MatchRecognizeExec + Thompson-NFA instruction-program + serializable match-context). PIV-C6-1 (DataFusion ExtensionPlanner wiring) must be confirmed before implementation starts. L-C6-2 continuous extension is ordered later; the serializable match-context must be built into Phase A. |
| **E-DETECT-EDITOR-001** | D-C6-2 defines the suppression-as-code requirements (versioned, justified, time-boxed, fire-frequency dashboard). D-C6-3 OQ-C6-AUTOROLLBACK defines the staged-rollout automation; once resolved, the circuit-breaker + promotion-gate design must flow into this epic's AC. |
| **E-RULE-XLATE-001** | L-C6-3 confirms feasibility; defines the pySigma-style backend + OCSF ProcessingPipeline + fidelity report requirement. Sigma→PrismQL examples ship in §14.7 recipe library NOW (before E-RULE-XLATE-001 is formally executed). |
| **§14.5 source-coverage-record** | D-C6-1 extends the §14.5 ADOPT-4 source-coverage-record schema with the per-slice `{full/partial/none}` coverage-map field. PO must amend §14.5 at morph time to add the coverage-map fields to the Alert/run record. |
| **§14.7 recipe library** | Sigma→PrismQL translation examples (L-C6-3) should be included in the first recipe library release, regardless of E-RULE-XLATE-001 schedule. |
| **BC-2.16.002 §Postconditions** | Six SAP-1 event type categories listed in §Downstream SAP-1 Obligations above (morph-time BC work; backtest run/gap events, suppression applied/expired events, auto-tune suggestion event, rollout-transition events pending OQ-C6-AUTOROLLBACK resolution). |
| **ADR-TBD: MATCH_RECOGNIZE operator implementation** | This ADR-PROP covers the implementation lean; the real ADR (allocated at morph, ADR-NNN) formalizes the DataFusion extension API choice, the Thompson-NFA instruction-program design, and the match-context serialization contract. |
| **ADR-TBD: Backtesting posture** | D-C6-1 formalized as a separate ADR covering the dual-tier architecture, coverage-map data model, snapshot-pinning protocol, cost-control envelope, and "evaluated-no-match" vs "no-data" UI contract. |
| **ADR-TBD: FP handling + suppression-as-code** | D-C6-2 formalized as a separate ADR covering RBA-as-default, suppression-as-code schema, mandatory justification/expiry fields, fire-frequency dashboard, and honest-caveat language. |
| **matured-vision §16.4** | C6 decision block appended in-place (2026-06-27). |
