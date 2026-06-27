---
document_type: research
produced_by: research-agent
status: capture
do_not_execute: true
timestamp: "2026-06-27"
program: day-2-vision-side-analysis
relation: OUT-OF-BAND — SEPARATE from the live VSDD factory pipeline
scope: DEPTH pass on OPEN implementation questions for the Detection Engine (matured-vision §14, §17.7, §17.14)
non_contradiction_basis:
  - "matured-vision-day2-requirements.md §14 (Detection Engine & Rule Editor) — HUMAN-CONFIRMED 2026-06-25"
  - "matured-vision-day2-requirements.md §17.7 (continuous-operator) — DECIDED 2026-06-26 (human)"
  - "matured-vision-day2-requirements.md §17.14 (synthesis) — PROPOSED, do_not_execute"
settled_decisions_NOT_relitigated:
  - "detection-as-query (PrismQL, not a separate DSL)"
  - "custom MATCH_RECOGNIZE operator built Phase-A-from-start (full NFA)"
  - "SEQUENCE…THEN…WITHIN sugar desugaring to MATCH_RECOGNIZE"
  - "correlation state on RocksDB/RetentionCache (no new datastore)"
  - "prism-native continuous windowed operator on RocksDB state backend (NOT embedded Flink)"
  - "detection spec carries EXPLICIT temporal semantics (lateness/accumulation); planner picks engine, never semantics"
  - "rule editor on S2-console + MCP + CLI (no TUI)"
  - "OT detection in scope; alert model + routing + destinations"
related_prior_research:
  - "match-recognize-rpr-feasibility-2026-06-25.md (the settled feasibility pass — this DEPTH pass extends it, does not supersede it)"
  - "detection-reshape-protocol-dissectors-2026-06-26.md (R1-R4 anchors for §17.14)"
  - "axiathon-detection-engine-analysis-2026-06-25.md"
caveat: >
  CAPTURE artifact. Leans are discussion input only — NOT decisions. Numbers/epics/ADRs
  remain the architect's at morph. This file does not modify STATE.md, SESSION-HANDOFF.md,
  the live ADR registry, any live spec/BC/story, RESEARCH-INDEX.md, or any prior research
  file, and was not git-added or committed.
---

# Detection Engine — DEPTH Research Pass (Day-2 Vision Side-Analysis)

**Date:** 2026-06-27 · **Reasoning effort:** `perplexity_research` at `high` (all 6 primary calls succeeded on first attempt; no overload fallback was needed this run — the prior run's overload abort is not reflected here). **Library APIs:** Context7 `/apache/datafusion`.

This pass researches **HOW to build the OPEN pieces** of the already-confirmed detection engine. It does not re-open settled decisions (see frontmatter). Six open depth questions, each with cited prior art and a discussion **LEAN**. Citations to web sources are flagged inline; `[model-knowledge]` marks model-supplied reasoning; `[INCONCLUSIVE]` marks where sources fell short. Citation indices in brackets (e.g. `[Flink-NFA]`) map to the **Sources** appendix.

> **Cross-tool reading note.** All six Perplexity deep-research responses returned source-grounded prose with numbered citations. The detailed numbered-citation maps live in the saved tool-result transcripts (paths in the Research Methods table); the **Sources** appendix below names the primary source families each finding rests on.

---

## Q1 — The MATCH_RECOGNIZE custom operator on DataFusion (gap G-18, the keystone)

### 1.1 What the prior art actually does (cited)

**The whole field converges on an NFA simulation, not DFA determinization.** Trino, Flink CEP, Oracle, the SASE/SASE+ lineage, Esper and Siddhi all compile the `PATTERN` regex-over-pattern-variables into a nondeterministic finite automaton and **simulate it directly** (maintaining a set of live configurations), because determinizing to a DFA risks exponential blow-up and, more importantly, the `DEFINE` predicates are data-dependent guards that cannot be folded into a static alphabet. [Trino-RPR][Flink-NFA][SASE]

- **Pattern → NFA construction is Thompson-style.** Each pattern variable becomes a fragment (start → guarded-transition → end). Concatenation links fragments with ε-transitions; alternation (`A|B`) adds a new start/end with ε-branches; `B+` adds a back ε-edge end→start; `B*` is `B+` plus a bypass ε-edge (zero-match); `B?` is `B|ε`; bounded `{m,n}` compiles to concatenation + optional tails. [Thompson][model-knowledge for the {m,n} desugar — standard regex theory, corroborated by Trino/Oracle docs]
- **Trino's `RowPatternMatcher`** is the closest reference to "build it as an operator." Trino compiles the pattern into an **instruction program** (a Thompson-NFA-as-bytecode: `MATCH`/`SPLIT`/`JUMP`/`CHECK`-style ops) and runs a thread-pool simulation — each thread has a program counter + match state; `SPLIT` spawns alternation threads; `MATCH` consumes a row and binds it to a variable after the `DEFINE` predicate `CHECK` passes; reaching `accept` records a match. [Trino-RPR]
- **Flink CEP** uses an `NFACompiler` that emits a graph of `State` objects with conditional/ε transitions, plus a **`SharedBuffer`** that stores each event once and lets multiple partial matches share it (with versioning to distinguish runs that share a prefix). The runtime keeps `NFAState` (the live computation states) per key. [Flink-NFA]
- **SASE/SASE+ (Agrawal/Diao/Gyllstrom) "NFAb" model** = NFA **+ a match buffer**; it is the academic origin of `skip-till-next-match` / `skip-till-any-match` and of windowed pruning of partial matches. Microsoft's "RPR Using Joins" paper sits on this lineage and is the validation for the §14.2 Phase-B join-rewrite fast-path (the 5.4× figure). [SASE][MS-RPR]
- **Esper / Siddhi** are higher-level CEP references: Esper's `every` operator ≈ overlapping-match spawning (analogous to `ALL ROWS PER MATCH` + skip), and both confirm the timer-based absence handling reused in Q2. [Esper][Siddhi]

**DEFINE predicate binding (the load-bearing detail).** Each NFA transition labeled with variable `R` carries a guard = the compiled `DEFINE` predicate for `R`. When the engine tries to consume a row via that transition, it evaluates the predicate against **(current row + the match context so far)**. The match context holds, per pattern variable, the ordered list of rows bound to it. `DEFINE` evaluates under **RUNNING** semantics (only the prefix matched so far is visible); navigation funcs `PREV`/`NEXT` look at the partition row stream, `FIRST`/`LAST` look at the variable's binding list; running aggregates over a variable include rows matched so far (count→0 / others→null if unmatched). [Oracle-RPR][Trino-RPR]

**MEASURES.** Evaluated when the match completes, default **FINAL** in Oracle (whole match visible); Trino lets you write `RUNNING`/`FINAL` explicitly and computes per-row measures under `ALL ROWS PER MATCH`. `CLASSIFIER()` returns the variable label bound to each row; `MATCH_NUMBER()` is the per-partition sequential match index. The operator therefore needs **two evaluation modes**: prefix-only (DEFINE + RUNNING measures) and whole-match (FINAL measures). [Oracle-RPR][Trino-RPR]

**AFTER MATCH SKIP (overlap control).**
- `PAST LAST ROW` (default) → non-overlapping: resume at last-row+1; discard all live runs that started ≤ last row. [Oracle-Skip][Trino-RPR]
- `TO NEXT ROW` → resume at first-row+1 (overlapping matches sharing interior rows). [Oracle-Skip]
- `TO FIRST <var>` / `TO LAST <var>` → resume at the first/last row bound to that variable (fine-grained overlap; adjacent matches can share a row). Requires per-match metadata recording the first/last index of each variable. [Oracle-Skip]
- **Error case (must reject at compile time):** `SKIP TO FIRST <var>` where the pattern can produce an **empty match** at the resume position causes an infinite loop; SQL:2016 prohibits it. The operator must statically analyze "can this pattern match empty?" × "can the SKIP-target variable bind the start row?" and reject the query if both hold. Also: a SKIP-target variable that may be **unbound** in some match (alternation/optional) makes SKIP undefined → reject. [Oracle-Skip][Trino-RPR]

**Greedy vs reluctant.** Greedy `B+` = try the loop-back ε-edge before the exit ε-edge; reluctant `B+?` = try exit first, only loop on backtrack. In an instruction program this is just the **branch ordering in `SPLIT`**. [Thompson][Trino-RPR]

**Empty-match handling.** Empty matches are valid under SQL:2016 (`ONE ROW PER MATCH` still emits a summary row, MEASURES over the empty set, partition key from the anchoring row). The hard invariant: **after ANY match (even empty), the start position must advance ≥ 1 row**, or the operator loops. [Oracle-RPR]

### 1.2 Verified DataFusion extension API surface (Context7, `/apache/datafusion`, 2026-06-27)

Context7 confirms the current extension points for building this as a logical + physical operator:

- **Two documented strategies for custom FROM-clause elements** (DataFusion `extending-sql.md`, `RelationPlanner`): (a) **rewrite** the custom syntax into standard relational ops DataFusion already executes (this is the §14.2 Phase-B join-rewrite fast-path), or (b) **end-to-end custom node** — a `UserDefinedLogicalNode` + a corresponding custom `ExecutionPlan`. Prism needs **both**: (b) for the full NFA operator, (a) as the optimizer fast-path for simple fixed-step patterns. [C7-DF-extending]
- **Custom `ExecutionPlan` trait** is the physical operator surface: implement `name()`, `properties() -> &Arc<PlanProperties>`, `children()`, `with_new_children()`, and `execute(partition, TaskContext) -> Result<SendableRecordBatchStream>`. The heavy work (the NFA scan) goes **inside the returned stream**, not in `execute()` itself. [C7-DF-execplan][C7-DF-counting]
- **`PlanProperties`** carries `EquivalenceProperties`, `Partitioning`, **`EmissionType`** (use `Incremental` for a streaming-shaped operator), and **`Boundedness`** (`Bounded` for batch-over-window; `Unbounded` is the lever the Q2 continuous operator flips). The MATCH_RECOGNIZE operator can stream output incrementally as matches complete. [C7-DF-counting]
- **`RecordBatchStreamAdapter`** wraps a `futures::Stream` of `Result<RecordBatch>` into a `SendableRecordBatchStream` — the practical way to emit match rows lazily. [C7-DF-counting]
- **Partition-by / order-by are physical-plan concerns.** DataFusion's physical optimizer inserts `RepartitionExec` (distribution) and `SortExec` (ordering) automatically. The operator should declare its required input distribution (hash-partition on `PARTITION BY` keys) and required input ordering (sorted on `ORDER BY` = `event_time`), and the planner enforces them — analogous to how window/aggregate operators get their input shaped. [C7-DF-physical]
- The logical→physical bridge is `SessionState::create_physical_plan`; a custom `ExtensionPlanner`/`QueryPlanner` converts the `UserDefinedLogicalNode` to the custom `ExecutionPlan`. [C7-DF-buildlogical][model-knowledge: ExtensionPlanner is the standard hook — Context7 surfaced the `RelationPlanner` + custom-node strategy and the `create_physical_plan` entry point; the exact `ExtensionPlanner` trait method wiring should be re-verified against the pinned DataFusion version at build time. [INCONCLUSIVE on exact method signatures for the pinned version]]

### 1.3 LEAN (discussion input only)

Build MATCH_RECOGNIZE as a **`UserDefinedLogicalNode` (`MatchRecognizeNode`) + a custom `ExecutionPlan` (`MatchRecognizeExec`)** that wraps a **Thompson-NFA-as-instruction-program matcher** (Trino's model — easier to make reluctant-quantifier ordering correct, easier to checkpoint as a flat program-counter + bindings than a pointer-graph). Inside `execute()`, return a `RecordBatchStreamAdapter` that drives the matcher per partition over `event_time`-sorted input. Declare hash-partition-on-`PARTITION BY` + sort-on-`ORDER BY` so DataFusion inserts the repartition/sort. Keep the §14.2 join-rewrite as a `RelationPlanner` rewrite for simple fixed-step `SEQUENCE` cases (the Phase-B fast-path). This single matcher core is **the exact thing §17.14 says the continuous operator reuses** — so design the match-context + bindings to be serializable from day one (Q2).

---

## Q2 — Streaming / incremental RPR over the RetentionCache window + the §17.7 continuous operator

### 2.1 Batch-over-window vs incremental (cited)

The decisive distinction, confirmed across the Flink/Realtime-Compute corpus: **batch-over-window** materializes a window then runs the matcher episodically (Splunk real-time / Sentinel NRT lineage = §17.7 Phase 1 NRT-over-cache); **incremental CEP** advances the NFA on each arriving event, keeping in-flight partial matches as durable state, and uses **watermarks + timers** to close/prune windows (= §17.7 Phase 2). [Flink-State][Aliyun-CEP][model-knowledge for the NRT-vs-continuous mapping, which matches §17.7 verbatim]

### 2.2 How Flink keeps & bounds partial-match state (the design Prism reuses)

- **`NFAState` + `SharedBuffer`.** `NFAState` = the live computation states per key; `SharedBuffer` stores each event once with predecessor pointers and **reference-counts** entries so a partial run that dies decrements counts and frees buffer entries at zero. This is how memory stays bounded under nondeterministic spawning. [Flink-NFA][Flink-State]
- **Per-key keyed state on the RocksDB state backend.** In-flight state lives in RocksDB off-heap/native memory, spills to local disk past a threshold, supports state larger than heap. **Incremental checkpoints** persist only changed SSTables since the last checkpoint (delta upload, shared handles for unchanged files) — exactly the "fast operator window-state vs slow durable correlation state" separation §17.14 calls for. [Flink-RocksDB][Flink-State]
- **Pruning levers (three, layered):** (1) **window pruning** via `.within(W)` / `WITHIN INTERVAL` / Ververica `Times{from,to,windowTime}` and graph windows `FIRST_AND_LAST` / `PREVIOUS_AND_CURRENT` — when the watermark passes a partial match's start+W it is purged; (2) **state TTL** as a global safety net (set TTL slightly > max window) so a misconfigured pattern can't leak state forever; (3) **skip-policy GC** — emitting a match prunes overlapping runs per the skip policy, decrementing SharedBuffer refs. [Flink-State][Ververica-CEP][Aliyun-CEP]
- **Checkpoint integrity.** NFAState, SharedBuffer, **and timers** are all keyed state and ride the Chandy-Lamport barrier snapshot; on restart timers re-fire when watermarks/clock reach them — so absence patterns survive failover. [Flink-State][Flink-Exactly-Once]

### 2.3 Absence / non-event / timeout — why a TIMER, not an anti-join (cited, decisive for §17.14)

The corpus is explicit and unanimous: **absence over an unbounded stream is undecidable without a deadline.** A relational anti-join over a finite window can say "no B in THIS finite relation," but it cannot know whether a future B will arrive. CEP engines solve it with **timers**: when `A` arrives for an "A not-followed-by B within W" pattern, register a per-partial-match timer at `t_A + W`; if `B` arrives first, cancel + drop the partial match; if the timer fires (watermark passes `t_A+W` for event-time, or wall-clock for processing-time) with no `B`, **emit the absence match**. Flink's `notFollowedBy()` at end-of-pattern *requires* a `WITHIN` clause for exactly this reason; Esper `timer:within` + `not`, Siddhi `not ... for` confirm. Realtime Compute's `ONE ROW PER MATCH SHOW TIMEOUT MATCHES` exposes timeouts as first-class result rows. [Aliyun-CEP][Esper][Flink-NFA] This is **R1 in detection-reshape-protocol-dissectors-2026-06-26.md** and matches §17.14's ruling that `WATCH…UNLESS` needs TWO physical impls: relational anti-join `AbsenceWindowNode` (polled/batch) **and** a CEP-style per-partition TIMER (continuous).

### 2.4 LEAN (discussion input only)

For Prism's **prism-native** continuous operator (§17.14, NOT embedded Flink): take the Q1 matcher core and add a thin **watermark + timer + checkpoint layer** over RocksDB column families (per §17.14 state-unification DECIDED-LEAN: window state + `detection_state` in **distinct CFs** within the existing 19-CF engine; ML `ModelState` logically separable). Concretely: (a) keep `NFAState`-equivalent (live program-counters + variable bindings) and a SharedBuffer-equivalent with **reference counting** for the dedup-and-free invariant; (b) drive window closure off the §3.3 **`event_time` TTL** (data-intrinsic freshness already neutralizes cross-tier eviction skew, §17.8 F4); (c) implement `WATCH…UNLESS` continuous-path as a **per-partition event-time timer** registered on the triggering event; (d) give window-state its **own incremental-checkpoint cadence** distinct from durable `detection_state` (this is §17.14 open question #1 — checkpoint cadence — left to architect, but the Flink incremental-SSTable model is the validated template). **Honest cost:** this is §17.7's "single most expensive item" — watermark/event-time/late-arrival + fault-tolerant checkpointing is the real build, and the matcher core is the cheaper part.

---

## Q3 — Backtesting over federated + Iceberg-cold-tier sources (gap G-19)

### 3.1 What the platforms actually do (cited) — and the gap they all share

Every surveyed platform backtests by **replaying a LOCAL store**, not federated remote re-query:
- **Elastic** — *rule preview* (run query over a historical range, results to a `.preview.alerts-*` index, no real alerts; recommend 7–14 days) + *manual runs* (real alerts over a past window, run in a dedicated Kibana space to isolate). Look-back-time is the lever; preview is the documented backtest surface. [Elastic-Preview][Elastic-Validate]
- **Google SecOps / Chronicle** — *retrohunt* over up to a year of UDM telemetry; the UI **displays the available date/time range** (a coarse coverage signal); multi-event rules require retrohunt range ≥ match window; `create_retrohunt(start, end, rule_id)` SDK for detection-as-code. Reference-list edits require a **new rule version** to change retrohunt results (snapshotting-by-version). [Chronicle-Retrohunt][Chronicle-SDK]
- **Panther** — *Data Replay* with hard cost/coverage envelope: window within last **15 days**, older than 24h, ≤ **20 GB**, must finish < **1 hour**; cache + network + enrichment **blocked** during replay; live progress + alert summary, recommends stopping replay if match-rate is high. [Panther-Replay]
- **Splunk** — no first-class "backtest," but `fill_summary_index.py` backfills coverage gaps and time-bounded searches re-run correlation searches retrospectively. [Splunk-Backfill]
- **Sigma/pySigma** — portable rule → per-backend query is the detection-as-code substrate; backtesting across multiple targets is an emergent practice, not a documented feature. [Sigma-Spec][pySigma]

**The gap (explicitly flagged by the source):** *no surveyed security platform documents backtesting over purely federated remote sources without first ingesting locally.* For a federated/ephemeral engine this is **genuinely novel** — Prism is ahead of the documented prior art here, which means it owns the hard parts. [model-knowledge synthesis grounded in the absence-of-vendor-coverage finding]

### 3.2 Determinism + cost-bounding over remote / Iceberg cold tier (cited primitives)

- **Iceberg time-travel is the determinism primitive.** `TIMESTAMP AS OF` / `VERSION AS OF`, DataFrame read options `snapshot-id` / `as-of-timestamp`, named `branch`/`tag`, incremental `start-snapshot-id`/`end-snapshot-id`. A backtest pins a **snapshot-id + rule-version** pair → reproducible point-in-time reads even after later appends (as long as the snapshot is retained). This directly avoids "look-ahead bias" (the finance-backtesting analog the source draws). [Iceberg-TimeTravel]
- **Partition pruning + time-bounded scan** = cost bound: partition the cold tier by event-time day; a 7-day backtest scans 7 partitions, parallelizable. For remote APIs, push `createdDate >= start && < end` filters into the connector; respect per-source max-lookback and per-call volume caps. [Iceberg-TimeTravel][model-knowledge for the connector-pushdown pattern]
- **Cost controls validated from Panther's envelope:** dry-run/estimate, sampling, time-slicing a long backtest into bounded chunks, a mandatory time-bound, and a hard wall-clock + volume ceiling per run. [Panther-Replay]

### 3.3 Coverage gaps — the "clean backtest ≠ no historical hits" hazard (cited)

Sources of gaps: **source retention expired** (SaaS like Salesforce keeps limited history — re-query returns nothing), **schema changed** (rule field absent in old data → silent miss or error), **source not onboarded** during the window, **ingestion outage**. Chronicle's "available date range" display is the closest existing mitigation; Elastic only advises analysts to *check ingestion* when preview returns nothing. **No platform offers a unified coverage map.** The hazard is reading "0 hits" as "low FP / no threat" when it is really "no data." [Arcserve-Retention][Chronicle-Retrohunt][Elastic-Validate]

### 3.4 LEAN (discussion input only)

A Prism federated backtest must emit a **coverage map** alongside hits — per source × time-slice, label {full / partial / none} derived from (source retention window) ∩ (connector onboarding date) ∩ (query-error log) ∩ (schema-version availability). For the Iceberg cold tier, pin **`snapshot-id` (or `as-of-timestamp`) + rule-version** for deterministic, reproducible, auditable backtests (records the exact data view). For remote-API sources lacking snapshots, treat the backtest as **best-effort current-state** and **say so explicitly** in the coverage map (non-deterministic, retention-bounded). Enforce a **mandatory time-bound + volume ceiling + dry-run estimate** (Panther's envelope, generalized). The "clean backtest" UI must distinguish **"evaluated, no match"** from **"no data to evaluate"** per slice — this is the single most important correctness affordance and the one the entire prior art is missing. Ties to §14.5 source-coverage-record + replay-link (ADOPT-4) — reuse that machinery for backtest coverage.

---

## Q4 — Staged rollout + auto-rollback (shadow → canary → production)

### 4.1 What exists vs what must be assembled (cited)

The honest finding the source leads with: **no major SIEM ships a Flagger-style fully-automated canary+rollback for detection rules out of the box.** What exists are *building blocks*; the progressive-delivery orchestration is assembled by the detection-engineering team. [Elastic-Validate][Panther-DaC][Flagger]

- **Shadow / run-but-don't-alert:** Elastic *rule preview* (logic runs, results never hit the alerts index) + *manual runs in a dedicated space* (real alerts, isolated). Panther *unit tests* + console test harness (positive & negative cases) + CI runs over historical corpora. Chronicle: run the same YARA-L as a *search* before promoting to a *rule*. Splunk: macro-gated searches (`should_run` macro = `| noop` vs an unsatisfiable filter), app-space toggling, condition-search maintenance windows. Sigma `status: experimental|test|stable` is the **formal lifecycle encoding** that CI maps to deploy targets. [Elastic-Preview][Panther-DaC][Chronicle-YARAL][Splunk-Suppress][Sigma-Spec]
- **FP-rate estimation & quality metrics:** Elastic *Rule Monitoring* + *Execution results* (alert volume per run, gaps, durations) + alert-suppression counts (`kibana.alert.suppression.docs_count`). Splunk UBA **False Positive Suppression Model** (offline ML batch that suppresses likely-FP alerts). Precision / MTTD / signal-to-noise are computable from alert+disposition data but are **organizational metrics, not standardized product features**. [Elastic-Monitoring][Elastic-Suppress][Splunk-UBA]
- **Canary scoping:** Elastic *spaces* + index patterns (tenant/asset subset); Splunk app-spaces + macros; Panther per-environment code deploy + log-source binding; Chronicle `events` filters + match keys. Feature-flag / tenant-scoped rollout is built from these. [Elastic-Validate][Splunk-Suppress][Panther-DaC]
- **The progressive-delivery analog (the one place auto-rollback IS documented):** **Flagger / Argo Rollouts** — a `Canary` CR defines `interval`, `maxWeight`, `stepWeight`, `progressDeadlineSeconds`, and **metric thresholds (success-rate / latency) checked against Prometheus**; **automatic rollback fires when failed checks hit a threshold.** The ML shadow-vs-canary pattern (shadow = 100% mirrored traffic, no decisions, paired by correlation-ID; canary = 1%→20%→50%→100% with daily metric review) maps cleanly onto detection rules. [Flagger][ML-Canary]

**Caveat the source flags:** the user-requested Anvilogic / SnapAttack detection-as-code blogs **did not surface** in the retrieved sources; the staged-rollout findings rest on Elastic / Panther / Chronicle / Splunk / Sigma + Flagger/Argo. [INCONCLUSIVE on Anvilogic/SnapAttack specifics]

### 4.2 LEAN (discussion input only)

Model the §14.4 staged rollout as **detection-as-code orchestration over Prism primitives**, borrowing Flagger's control loop: (1) **shadow** = run the rule (NRT or continuous) writing findings to a non-routed shadow stream keyed by rule-version, paired to baseline by event/finding-ID (the ML correlation-ID pattern) — reuses §14.5 alert model with a `routed=false` flag; (2) **canary** = enable routing for a **scoped subset** (one tenant / one asset group — Prism is already multi-tenant, so tenant-scope is the natural canary unit) with a metric-evaluation interval; (3) **gate** on explicit thresholds carried in the §14.1 `quality` block (`fp_rate`, alert-volume/hr, precision via analyst disposition, MTTD); (4) **auto-rollback trigger** = FP-spike / alert-storm circuit-breaker that **demotes the rule to shadow (never silently deletes findings)** and requires human sign-off to re-promote. Map Sigma-style `status` to Prism lifecycle states (§14.1 already has `draft→…→shadow→canary→production→deprecated`). **Guardrail (carry from Q6):** auto-rollback may DISABLE *routing*, but the rule keeps *evaluating* in shadow so coverage is never silently zeroed — auto-disable of a detection's evaluation requires human approval.

---

## Q5 — Sigma → PrismQL translation (deferred E-RULE-XLATE-001; feasibility researched now)

### 5.1 Sigma structure + pySigma compilation (cited)

- **Rule structure:** `logsource` (category / product / service) selects the OCSF/native table; `detection` block = named selection maps + lists + a `condition` expression (`and`/`or`/`not`, `1 of`/`all of`, `( )`); `count()/value_count() by <field> > N` aggregations; **value modifiers** `contains`, `startswith`, `endswith`, `re`, `base64`, `base64offset`, `cidr`, `all`, `gt/gte/lt/lte`, `windash`, `expand`. [Sigma-Spec]
- **pySigma architecture:** `SigmaRule` → `SigmaDetection`/`SigmaCondition` AST (`ConditionAND`/`OR`/`NOT`) → a **`Backend`** emits the target query, transformed by a **`ProcessingPipeline`** (field mappings, value transforms, conditional rewrites). Existing backends: Elasticsearch (Lucene / ES|QL / EQL), Splunk SPL, others. `sigma-cli` drives conversion. There **is** a Sigma→OCSF pipeline that uses `logsource` category to pick the target OCSF class — but it is a **practical, partial mapping, not a complete field-mapping spec**. [pySigma][Sigma-OCSF]

### 5.2 The hard / lossy cases (cited)

- **Sigma correlation rules** (`event_count` / `value_count` / `temporal` / `temporal_ordered`) are a **distinct, newer class of Sigma objects** and are **only supported by a few backends (Splunk SPL, Elasticsearch)** today. A custom SQL-like target that lacks temporal pattern matching can't express them — **but Prism's MATCH_RECOGNIZE operator (Q1) is precisely the temporal-pattern primitive that makes `temporal`/`temporal_ordered` expressible**, which is a notable strategic alignment. [Sigma-Correlation][pySigma]
- **Value modifiers without a target equivalent** — `base64offset` and `windash` are the classic lossy ones (they encode multiple literal expansions / dash variants); `re` runs into **regex-dialect divergence** (lookaround, backreferences, Unicode, escaping, length limits). [pySigma]
- **Taxonomy / field-mapping mismatch** — Sigma's own taxonomy vs OCSF classes/attributes; a field may map to different OCSF attributes per class. Case sensitivity, wildcard semantics, null handling, and `collect` vs `count` differences are all enumerated lossy points. [pySigma][Sigma-OCSF]
- **OCSF storage shape** — OCSF events often stored per-class in separate tables/partitions, so cross-class Sigma correlations need careful join strategies (lossy/expensive if naive). [Sigma-OCSF]

### 5.3 LEAN (discussion input only)

Sigma→PrismQL is **feasible and worth a pySigma-style backend** with a `ProcessingPipeline` that targets the **OCSF taxonomy** (reuse the existing community Sigma→OCSF pipeline as the starting field map, extend it to a fuller spec). Single-event + selection + threshold/distinct-count Sigma maps cleanly to PrismQL `WHERE`/`GROUP BY…HAVING`. **Sigma correlation rules map onto Prism's MATCH_RECOGNIZE / SEQUENCE operator** — Prism is unusually well-positioned because it owns the temporal primitive most backends lack. Treat `base64offset`, `windash`, exotic `re` dialects, and class-spanning correlations as **explicitly lossy** — translate with a **fidelity report** that flags every modifier/condition that could not be losslessly expressed (never silently drop). This fidelity report is the Q5 analog of the Q3 coverage map. Keep it deferred (E-RULE-XLATE-001) but the recipe library (§14.7) should ship Sigma→PrismQL **examples** now to validate the mapping surface.

---

## Q6 — False-positive auto-tune + exception/suppression (no silent masking)

### 6.1 What platforms do (cited)

- **Auto-tune suggestions:** the honest finding — **no surveyed platform auto-proposes specific exceptions from analyst dispositions out of the box.** Elastic provides the *mechanisms* (exceptions, suppression, duplicated rules for layered coverage, risk scores) and the tuning *guidance*; Splunk UBA's FP-suppression model is offline-batch ML; **Risk-Based Alerting (RBA)** in Splunk ES is the standout pattern — it **does not suppress, it re-aggregates**: individual noisy events accrue risk to an entity, and alerts fire on aggregated risk, so visibility into underlying events is retained even as alert volume drops. [Elastic-Suppress][Splunk-UBA][Splunk-RBA]
- **Suppression-as-code & the taxonomy:** distinguish (a) **rule-level suppression / dedup** (group within a window — Elastic alert suppression by field; Splunk notable suppression by `src_ip`); (b) **exception lists** (do-not-alert conditions — Elastic exception containers, version-controllable); (c) **allowlists**. Scope can be per-rule or shared/global; audit via version control. [Elastic-Suppress][Splunk-Suppress]
- **The silent-masking danger + mitigations (cited):** over-broad exceptions create **detection blind spots** that silently erode coverage. Mitigations the corpus names: **narrow scoping**, **explicit justification workflows**, **time-boxed suppressions that auto-expire and force re-review**, **risk-scoring instead of outright suppression**, **suppression dashboards** that surface how often each suppression fires (to catch over-broad / stale ones), and strong version-control + audit. **But the source is explicit: none of these can give an absolute "never mask a true positive" guarantee.** [model-knowledge-corroborated-by-source][Elastic-Suppress][Splunk-RBA]
- **Exception expiry / review:** time-boxed automation rules (short-lived suppressions minimize long-term blind spots and force re-evaluation toward a permanent rule change or watchlist); expiration dates attached for documentation; periodic review; fire-frequency metrics to detect stale/dead suppressions. [Sentinel-Tuning][Elastic-Suppress]
- **Closed-loop + guardrail:** disposition (TP/FP) labels feed rule quality scores and tuning; the named guardrail — **auto-tuning must not disable a detection entirely without human sign-off.** [Splunk-UBA][model-knowledge for the guardrail framing, corroborated by the "no absolute assurance" finding]

### 6.2 LEAN (discussion input only)

Adopt the **RBA philosophy as the default over hard suppression**: prefer re-prioritization/aggregation (retain visibility) to silent drop. Make **exception/suppression-as-code**: every suppression is a versioned object in the detection repo with **mandatory justification + mandatory expiry (time-box)**; expiry forces re-review (no immortal exceptions). Ship a **suppression dashboard** surfacing per-suppression fire-frequency + scope breadth to catch over-broad/stale ones (the source's "metrics on how often a suppression fires" mitigation). Auto-tune emits **suggestions only** (threshold deltas, candidate exclusions derived from disposition history) — never auto-applies, and **never auto-disables a detection's evaluation without human sign-off** (the Q4 rollback guardrail is the same guardrail). **Honest caveat:** "never silently mask a true positive" is genuinely **unachievable as an absolute guarantee** — the production-grade posture is *transparency + time-boxing + audit + narrow scope*, not a proof. State this plainly in the spec rather than implying a guarantee.

---

## Consolidated Open Design Questions

Open questions this DEPTH pass surfaces for the architect (NOT decided here):

1. **MATCH_RECOGNIZE matcher representation** — instruction-program (Trino model, easier checkpoint/reluctant-ordering) vs explicit state-graph (Flink model). LEAN: instruction-program. Confirm against the pinned DataFusion version's planner hooks.
2. **Exact DataFusion `ExtensionPlanner`/`QueryPlanner` wiring** for the pinned version — Context7 confirmed the *strategy* (`UserDefinedLogicalNode` + custom `ExecutionPlan`, `RelationPlanner` for the rewrite fast-path) but exact trait-method signatures must be re-verified at build time. [INCONCLUSIVE for pinned version]
3. **Continuous-operator checkpoint cadence** — window-state CF cadence vs durable `detection_state` CF cadence vs ML `ModelState` (this is §17.14 open #1; the Flink incremental-SSTable model is the template, not a decision).
4. **Whether window-state ↔ detection_state CF boundary is sufficient isolation** under a shared checkpoint stream (§17.14 honest-cost #5 — fast operator state coupled to slow campaign state). Open.
5. **Backtest coverage-map data model** — how source-retention ∩ onboarding-date ∩ schema-version ∩ query-error compose into per-slice {full/partial/none}; reuse §14.5 ADOPT-4 source-coverage-record.
6. **Iceberg snapshot retention policy for reproducible backtests** — how long snapshots must be pinned/retained for audit re-runs vs cold-tier `expire_snapshots` cost (ties coldtier-iceberg research 2026-06-27).
7. **Sigma fidelity-report schema** — how lossy modifiers/correlations are reported (the Q5 analog of the Q3 coverage map).
8. **Canary unit** — tenant vs asset-group vs traffic-sample as the default canary scope (LEAN: tenant, since Prism is already multi-tenant).
9. **Auto-rollback trigger thresholds** — what FP-spike/alert-storm signal + window defines the circuit-breaker; where the thresholds live (§14.1 `quality` block vs per-deployment config).

---

## Recommended build approach for the MATCH_RECOGNIZE operator (concrete)

Discussion input only — the keystone (G-18). A concrete, buildable shape consistent with all settled decisions:

1. **Grammar/desugar (already specified §12.4/§14.2.1):** `SEQUENCE…THEN…WITHIN` (Chumsky) → `MATCH_RECOGNIZE` AST. Power users may write raw `MATCH_RECOGNIZE`.
2. **Pattern compiler:** `MATCH_RECOGNIZE` AST → Thompson-NFA **instruction program** (`MATCH`/`SPLIT`/`JUMP`/`CHECK`/`ACCEPT`). Greedy/reluctant = `SPLIT` branch ordering. Anchors = zero-consumption position checks. `{m,n}` = concat + optional tails. Compile-time rejection of empty-match × `SKIP TO FIRST` infinite-loop and unbound-SKIP-target cases.
3. **Logical node:** `MatchRecognizeNode : UserDefinedLogicalNode` carrying partition-by exprs, order-by (`event_time`), pattern program, compiled `DEFINE` predicate evaluators, `MEASURES` exprs (RUNNING/FINAL tagged), `rows_per_match`, `AFTER MATCH SKIP` mode. [C7-DF-extending]
4. **Physical node:** `MatchRecognizeExec : ExecutionPlan`. Declare required input distribution = hash-partition on PARTITION BY, required input ordering = sort on `event_time` (planner inserts `RepartitionExec`/`SortExec`). `properties()` → `PlanProperties { EmissionType::Incremental, Boundedness::Bounded }` for batch-over-window. `execute(partition, ctx)` returns a `RecordBatchStreamAdapter` driving the NFA simulation per partition; emit match rows incrementally. [C7-DF-execplan][C7-DF-counting][C7-DF-physical]
5. **Match-context data structure (designed for Q2 reuse):** per live run = program-counter + per-variable ordered binding lists + running aggregates + first/last index per variable (for SKIP). Make it **serializable from day one** so the Q2 continuous operator can checkpoint it to RocksDB CFs unchanged.
6. **Optimizer fast-path (§14.2 Phase B):** a `RelationPlanner` rewrite that detects simple fixed-step `SEQUENCE` patterns and rewrites to self-joins + window (MS "RPR Using Joins," 5.4×) — bypasses the NFA for the common case. [MS-RPR][C7-DF-extending]
7. **Continuous extension (Q2 / §17.7 Phase 2, ordered later):** wrap the SAME matcher core with watermark + per-partition timer + incremental-checkpoint layer on RocksDB CFs; flip `Boundedness::Unbounded`; add the SharedBuffer-equivalent reference-counting + `event_time`-TTL pruning. `WATCH…UNLESS` continuous path = per-partition event-time timer.

**Why this shape:** one matcher core serves both the batch RPR operator (now) and the continuous operator (later) exactly as §17.14 mandates ("reuses the MATCH_RECOGNIZE NFA operator prism already owns"); the instruction-program is the easiest to make correct (reluctant ordering) and to serialize (flat PC + bindings).

---

## Honest Costs & Caveats

1. **MATCH_RECOGNIZE is a real engine operator, not a query rewrite.** DataFusion parses but does not execute it and the core team has low appetite to add it — Prism owns the full lifecycle (compiler, NFA simulation, SKIP/empty-match correctness, MEASURES RUNNING/FINAL). The SQL:2016 edge cases (empty-match infinite-loop prohibition, SKIP-to-unbound-var, greedy/reluctant interaction with overlap) are where correctness bugs hide. [Trino-RPR][Oracle-Skip]
2. **The continuous operator is the single most expensive item** (§17.7 self-identifies this). Watermarks, event-time, late-arrival, fault-tolerant incremental checkpointing on RocksDB — the matcher core is the *cheap* part; the temporal/state-fault-tolerance layer is the build.
3. **Federated backtesting is genuinely novel** — no surveyed security platform documents it. Iceberg time-travel gives determinism on the cold tier, but remote-API sources are best-effort/non-deterministic and retention-bounded. The coverage-map ("evaluated-no-match" vs "no-data") is mandatory and unprecedented in the prior art — Prism builds it from scratch. [Iceberg-TimeTravel][Arcserve-Retention]
4. **Auto-rollback is not a shipped SIEM feature** — it must be assembled from Prism primitives borrowing Flagger's control loop. The Anvilogic/SnapAttack detection-as-code specifics requested did not surface in sources. [INCONCLUSIVE][Flagger]
5. **Sigma→PrismQL is lossy at the edges** — `base64offset`, `windash`, exotic regex dialects, class-spanning correlations. A fidelity report (never silent drop) is required. Sigma correlation rules are new and thinly supported across backends; Prism's MATCH_RECOGNIZE is the asset that makes them expressible. [pySigma][Sigma-Correlation]
6. **"Never silently mask a true positive" is not an achievable absolute guarantee.** The production-grade posture is transparency + narrow scope + mandatory justification + time-boxed expiry + fire-frequency dashboards + RBA-over-suppression — not a proof. The spec must say this plainly. [Splunk-RBA]
7. **Citation depth caveat.** All six deep-research responses returned numbered-citation prose; this report names source *families* in the Sources appendix and the inline tags. The exhaustive per-claim numbered-citation maps are in the saved tool transcripts (Research Methods table). Where the model supplied reasoning beyond the sources it is tagged `[model-knowledge]`; where sources fell short, `[INCONCLUSIVE]`.

---

## Sources (primary source families surfaced by the deep-research passes)

> These are the source families the Perplexity `sonar-deep-research` responses grounded their numbered citations in. Exact URLs/numbered maps are in the saved transcripts (see Research Methods).

- **[Trino-RPR]** Trino MATCH_RECOGNIZE / RowPatternMatcher (Trino blog + docs + community broadcast on the Thompson-like instruction-program matcher).
- **[Flink-NFA]** Apache Flink CEP — `NFACompiler`, `SharedBuffer`, `NFAState`, CepOperator (Flink docs + source + Towards Data Science Flink CEP article).
- **[Flink-State][Flink-RocksDB][Flink-Exactly-Once]** Flink state model + RocksDBStateBackend + incremental checkpoints + Chandy-Lamport exactly-once (Flink blog/docs + video).
- **[SASE]** Agrawal/Diao/Gyllstrom SASE / SASE+ "Efficient Pattern Matching over Event Streams" (NFAb model).
- **[MS-RPR]** Microsoft "Row Pattern Recognition Using Joins" (join-rewrite, 5.4× speedup).
- **[Oracle-RPR][Oracle-Skip]** Oracle SQL/CQL pattern-matching docs + AFTER MATCH SKIP semantics blog.
- **[Esper][Siddhi]** EsperTech EPL reference (`timer:within`, `every`, `not`); Siddhi streaming-SQL sequence/`not...for` patterns.
- **[Thompson]** Thompson NFA construction (regex→NFA, ε-transitions for concat/alt/quantifiers).
- **[Aliyun-CEP][Ververica-CEP]** Alibaba Realtime Compute for Apache Flink + Ververica dynamic-CEP rule-graph docs (`WITHIN`, `notFollowedBy`, `SHOW TIMEOUT MATCHES`, graph windows, skip strategies).
- **[Iceberg-TimeTravel]** Apache Iceberg time-travel (`TIMESTAMP/VERSION AS OF`, `snapshot-id`, `as-of-timestamp`, branches/tags, incremental scans).
- **[Elastic-Preview][Elastic-Validate][Elastic-Monitoring][Elastic-Suppress]** Elastic Security rule preview, validation & testing, rule monitoring/execution results, alert suppression + exceptions docs + DaC-Reference.
- **[Chronicle-Retrohunt][Chronicle-SDK][Chronicle-YARAL]** Google SecOps / Chronicle retrohunt docs, SecOps SDK `create_retrohunt`, YARA-L 2.0 reference.
- **[Panther-Replay][Panther-DaC]** Panther Data Replay (beta) limits + code-based detections / panther-analysis CI workflow.
- **[Splunk-Backfill][Splunk-Suppress][Splunk-UBA][Splunk-RBA]** Splunk `fill_summary_index.py`, notable suppression, UBA False-Positive-Suppression-Model, Risk-Based Alerting.
- **[Sigma-Spec][pySigma][Sigma-Correlation][Sigma-OCSF]** SigmaHQ specification (`status`, logsource, modifiers), pySigma backends/ProcessingPipeline, Sigma correlation rules, Sigma→OCSF community pipeline.
- **[Flagger][ML-Canary]** AWS App Mesh + Flagger progressive-delivery reference (Canary CR, metric-gated auto-rollback); shadow-vs-canary ML deployment blog.
- **[Arcserve-Retention]** Arcserve SaaS data-retention-policy challenges (retention gaps → backtest coverage gaps).
- **[C7-DF-extending][C7-DF-execplan][C7-DF-counting][C7-DF-physical][C7-DF-buildlogical]** Context7 `/apache/datafusion` docs: `extending-sql.md` (RelationPlanner + UserDefinedLogicalNode/ExecutionPlan strategy), `custom-table-providers.md` (ExecutionPlan trait, CountingExec example, PlanProperties/EmissionType/Boundedness, RecordBatchStreamAdapter, physical-planning/distribution/sort enforcement), `building-logical-plans.md` (`create_physical_plan`).

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 6 | All 6 at `reasoning_effort=high`, `strip_thinking=true`: (Q1) MATCH_RECOGNIZE NFA/automaton implementation, SKIP semantics, MEASURES; (Q2) incremental/streaming CEP, RocksDB state, timers/absence; (Q3) federated + Iceberg backtesting + coverage gaps; (Q4) staged rollout/shadow/canary/auto-rollback; (Q5) Sigma structure + pySigma + OCSF mapping + lossy cases; (Q6) FP auto-tune + suppression/exception expiry + no-silent-masking. |
| Perplexity perplexity_reason | 0 | — |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_ask | 0 | — |
| Context7 | 2 | `resolve-library-id` (Apache DataFusion → `/apache/datafusion`) + `query-docs` (custom logical node, ExtensionPlanner, custom ExecutionPlan, PlanProperties/distribution/ordering, RecordBatchStream). |
| Tavily tavily_search | 0 | — |
| Tavily tavily_research | 0 | — |
| Tavily tavily_extract | 0 | — |
| WebFetch | 0 | — |
| WebSearch | 0 | — |
| Training data | ~4 areas (flagged inline) | Thompson `{m,n}` desugar (standard regex theory, corroborated by docs); NRT-vs-continuous §17.7 mapping (matches vision verbatim); ExtensionPlanner method-wiring synthesis (strategy confirmed by Context7, exact signatures `[INCONCLUSIVE]` for pinned version); auto-rollback/no-silent-masking guardrail framing (corroborated by source "no absolute assurance" finding). All tagged `[model-knowledge]`/`[INCONCLUSIVE]` inline. |

**Total MCP tool calls:** 8 (6 `perplexity_research` @ high + 2 Context7).
**Training data reliance:** **low** — every substantive technical claim is grounded in a cited web source family; model knowledge is confined to standard CS theory (Thompson construction), explicit vision-alignment notes, and clearly-tagged synthesis where vendor docs fell short. All such cases are flagged `[model-knowledge]` or `[INCONCLUSIVE]` inline.

**Overload-resilience note:** This run hit no API overload — all 6 high-effort deep-research calls succeeded on first attempt (the prior aborted attempt referenced in the task is not reflected here). The medium-effort fallback was therefore not exercised.

**Saved deep-research transcripts (full numbered-citation prose):**
- Q1: `tool-results/mcp-perplexity-perplexity_research-1782545828451.txt`
- Q2: `tool-results/mcp-perplexity-perplexity_research-1782546114512.txt`
- Q3: `tool-results/mcp-perplexity-perplexity_research-1782546410221.txt`
- Q4: `tool-results/mcp-perplexity-perplexity_research-1782546350038.txt`
- Q5 (Sigma): `tool-results/mcp-perplexity-perplexity_research-1782546705276.txt`
- Q6 (auto-tune): `tool-results/mcp-perplexity-perplexity_research-1782546654227.txt`
(under `/Users/jmagady/.claude/projects/-Users-jmagady-Dev-prism/1cbcd55e-1092-4bcc-ab2e-65460a5c2bee/`)
