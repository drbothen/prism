---
document_type: design-decision
status: capture
do_not_execute: true
provenance: "2026-06-26 side-analysis — day-2 PrismQL design decision; PROPOSED, gated on brief-reframe sign-off; separate from live factory."
author: architect
created: "2026-06-26"
related_sections:
  - "matured-vision-day2-requirements §12.4"
  - "matured-vision-day2-requirements §14.2.1"
  - "matured-vision-day2-requirements §16.2 #4"
related_bcs:
  - BC-2.13.004
related_adrs: []
human_decisions_required:
  - HD-1: NOT/WITHOUT primary semantics (exclusion vs timeout — see §4) — RESOLVED 2026-06-26 (human): BOTH in Phase A
  - HD-2: Raw MATCH_RECOGNIZE power-user escape hatch — ACCEPTED 2026-06-27 (human): YES, expose it, lower priority / later phase
---

# PrismQL SEQUENCE Sugar — Design Decisions (PROPOSED)

> **GUARDRAIL:** This is a PROPOSED capture artifact. Status `capture`, `do_not_execute: true`.
> It does NOT modify any live spec, grammar, ADR, or factory file. Execution is gated on
> brief-reframe sign-off (matured-vision §16.5).

---

## Background and Settled Decisions

The following are CONFIRMED per matured-vision §16.2 #4 and are NOT re-litigated here:

- Full `MATCH_RECOGNIZE` operator is Phase A (in scope now), built as a custom
  logical/physical operator on top of DataFusion (which parses but does not execute RPR).
- The `SEQUENCE…THEN…WITHIN` sugar is a readable analyst surface that desugars to
  `MATCH_RECOGNIZE`. Raw `MATCH_RECOGNIZE` remains available as a power-user escape hatch.
- Phase-B join/window rewrite is an optimizer fast-path only; it does not change surface
  semantics.
- Correlation state persists to RocksDB / RetentionCache (not PostgreSQL).

The four open questions listed in §12.4 are resolved below.

---

## Decision 1 — Keyword Finalization

### Recommendation: Adopt the keyword set as drafted in §12.4, with two minor clarifications

**Final surface keyword set:**

```
DETECT  SEQUENCE  BY  WITHIN  STEP  THEN  NOT  WITHOUT  ANY OF
EMIT    AS        OVERLAP  ALLOWED  NONE
```

**Concrete sugar grammar (EBNF, normative for the ADR):**

```ebnf
detection      ::= "DETECT" ident sequence_block emit_clause? overlap_clause?
                 | sequence_block                       (* ad-hoc, no rule name *)

sequence_block ::= "SEQUENCE" "BY" field_list within_clause? step then_step+

within_clause  ::= "WITHIN" duration                   (* overall maxspan *)

step           ::= "STEP" quant_var ":" predicate

then_step      ::= "THEN" gap_clause? negation? quant_var ":" predicate
               |   "THEN" gap_clause? "ANY" "OF" "[" alt_step ("," alt_step)* "]"

gap_clause     ::= "WITHIN" duration                   (* max gap from previous step *)

negation       ::= "NOT" | "WITHOUT"                   (* absence / non-event *)

quant_var      ::= pattern_var quantifier?

quantifier     ::= "+" | "*" | "?" | "{" int ("," int?)? "}"

alt_step       ::= pattern_var ":" predicate

emit_clause    ::= "EMIT" emit_item ("," emit_item)*

emit_item      ::= expr ("AS" ident)?

overlap_clause ::= "OVERLAP" ("ALLOWED" | "NONE")

predicate      ::= (* PrismQL boolean expr; MAY reference earlier pattern vars,
                      e.g.  host = b.host *)

duration       ::= int ("s"|"m"|"h"|"d"|"w"|"mo")     (* shared with SINCE sugar, §12.3 E1 *)

pattern_var    ::= ident                               (* a, b, c, … *)

field_list     ::= field ("," field)*
```

**Keyword design rationale:**

- `DETECT … SEQUENCE` mirrors the detection-as-query model: `DETECT` names the rule, `SEQUENCE`
  declares the correlation type. An anonymous `SEQUENCE` (no `DETECT` wrapper) is valid for
  ad-hoc investigation queries — same pattern as SQL's unnamed subqueries.
- `STEP` / `THEN` carry the ordered-time intuition: step A, _then_ step B, _then_ step C.
  Symmetric with spoken language for kill-chain narration.
- `WITHIN` is reused at two levels (overall span on `SEQUENCE BY`, per-step gap on `THEN`) —
  position disambiguates. Ambiguity does not arise because `SEQUENCE BY … WITHIN t` appears
  once at the block level, while `THEN WITHIN t` appears inline before the pattern var.
- `NOT` / `WITHOUT` are synonyms for the absence operator (see Decision 4 for semantics). Both
  are reserved to match analyst intuition; the grammar accepts either form; the ADR specifies
  one canonical lowering.
- `ANY OF [...]` provides alternation without exposing `|` pipe-character conflicts (pipe is
  already the PrismQL pipe-mode delimiter).
- `OVERLAP ALLOWED` / `OVERLAP NONE` map directly to `AFTER MATCH SKIP TO NEXT ROW` /
  `AFTER MATCH SKIP PAST LAST ROW` in `MATCH_RECOGNIZE`. `NONE` is the safer default for
  detection rules (no double-alert on the same anchor event).
- Keywords are case-insensitive (consistent with all existing PrismQL keywords).
- `EMIT … AS` maps to `MEASURES … AS` + `ONE ROW PER MATCH`. If `EMIT` is absent, the
  engine emits all `MEASURES` variables as named by their pattern-var prefix.

**Worked Example 1 — Fixed-step kill-chain (the §14.2.1 canonical example):**

```
DETECT credential_theft
  SEQUENCE BY user.name WITHIN 30m
    STEP a: process.name = 'mimikatz.exe'
    THEN b: access.type = 'dump' AND resource = 'lsass'
    THEN c: file.path ENDS WITH '.kdbx'
  EMIT user.name, a.time AS started, c.time AS completed
```

Lowers to:

```sql
SELECT user_name, A.event_time AS started, C.event_time AS completed
FROM events
MATCH_RECOGNIZE (
  PARTITION BY user_name
  ORDER BY event_time
  MEASURES A.event_time AS started, C.event_time AS completed
  ONE ROW PER MATCH
  AFTER MATCH SKIP PAST LAST ROW   -- OVERLAP NONE (default)
  PATTERN (A B C)
  DEFINE
    A AS A.process_name = 'mimikatz.exe',
    B AS B.access_type = 'dump' AND B.resource = 'lsass',
    C AS C.file_path LIKE '%.kdbx'
)
WHERE LAST(event_time) - FIRST(event_time) <= INTERVAL '30' MINUTE
```

**Worked Example 2 — Kleene quantifier + capture (brute-then-success):**

```
DETECT brute_then_success
  SEQUENCE BY user.name, src.ip WITHIN 5m
    STEP f+: auth.outcome = 'failure'
    THEN s:  auth.outcome = 'success'
  EMIT user.name, src.ip, count(f) AS failures, s.time AS broke_in
```

Lowers to:

```sql
SELECT user_name, src_ip, COUNT(F) AS failures, S.event_time AS broke_in
FROM events
MATCH_RECOGNIZE (
  PARTITION BY user_name, src_ip
  ORDER BY event_time
  MEASURES COUNT(F.*) AS failures, S.event_time AS broke_in
  ONE ROW PER MATCH
  AFTER MATCH SKIP PAST LAST ROW
  PATTERN (F+ S)
  DEFINE
    F AS F.auth_outcome = 'failure',
    S AS S.auth_outcome = 'success'
)
WHERE LAST(event_time) - FIRST(event_time) <= INTERVAL '5' MINUTE
```

---

## Decision 2 — Overall-WITHIN Semantics

### Recommendation: Overall-WITHIN bounds the first-to-last matched event span; it is NOT a hard filter on the `MEASURES` output

**Semantics:**

`WITHIN t` on `SEQUENCE BY … WITHIN t` means: the elapsed time from the first matched event
(pattern variable with the lowest `ORDER BY` value) to the last matched event must be ≤ t.

This is a **match predicate** — it constrains which complete NFA matches are accepted, not
which rows enter the operator. A sequence that fully matches its pattern but whose
first-to-last span exceeds `WITHIN t` is silently discarded (no match output, no error).

**Mapping to `MATCH_RECOGNIZE`:**

`MATCH_RECOGNIZE` has no standard `WITHIN` keyword (confirmed in §14.2, research-verified).
The span predicate is expressed as a `DEFINE`-level or post-`MEASURES` filter:

```sql
-- Option A: expressed as a post-SELECT WHERE (simplest; works for ONE ROW PER MATCH)
WHERE LAST(event_time) - FIRST(event_time) <= INTERVAL '<t>'

-- Option B: expressed as a DEFINE constraint on the last variable
-- (not idiomatic; harder to read; not recommended)
```

**Recommendation: Option A.** The custom `MATCH_RECOGNIZE` operator implementation wraps the
overall `WITHIN` as a post-match filter on the `(FIRST(t), LAST(t))` interval. This is
equivalent to how Trino and Flink implement maxspan (`WITHIN … INTERVAL`).

**What this means for analysts:**

- `WITHIN 30m` on the `SEQUENCE BY` line means "the entire sequence must complete within a
  rolling 30-minute window."
- It does NOT mean "fetch only 30 minutes of data." The operator scans whatever window the
  RetentionCache holds; it just discards matches whose span exceeds 30m.
- The RetentionCache's actual retention depth is governed by the rule's `WITHIN` value as a
  hint to the cache warm-up scheduler (§3.3) — but that is a cache-management concern, not
  a match-predicate concern.

**Per-step gap (`THEN WITHIN t b: Q`) is separate and additive.** Both constraints apply
independently:

```
SEQUENCE BY user.name WITHIN 60m   -- overall span ≤ 60m
  STEP a: ...
  THEN WITHIN 10m b: ...           -- B must occur within 10m of A
  THEN WITHIN 20m c: ...           -- C must occur within 20m of B
```

A match where A→B takes 9m and B→C takes 19m (total span 28m) satisfies all three bounds.
A match where A→B takes 9m and B→C takes 11m but total span is 20m also satisfies all three.
A match where total span is 61m fails the overall bound regardless of per-step gaps.

---

## Decision 3 — Cross-Step Running-Semantics (Variable References)

### Recommendation: Pattern variable references in DEFINE predicates are single-value reads from the matched row; aggregate semantics are available only for quantified variables

**Running-semantics model:**

In `MATCH_RECOGNIZE`, the `DEFINE` clause evaluates each pattern variable against the
_current candidate row_. References to an earlier pattern variable (e.g., `B.host = A.host`)
read the **last matched row** for that variable unless the variable is quantified (Kleene
`+`/`*`/`?`/`{n,m}`), in which case aggregate functions apply.

PrismQL preserves these semantics unchanged in the sugar lowering:

| Sugar expression | DEFINE lowering | Semantics |
|---|---|---|
| `THEN b: host = a.host` | `DEFINE B AS B.host = A.host` | B's host field must equal the (single) A row's host — row-level equality cross-step |
| `THEN c: score > b.score + 10` | `DEFINE C AS C.score > B.score + 10` | C's score must exceed B's score + 10; B is the last matched B row |
| `THEN b+: auth.outcome = 'failure'` | `PATTERN (B+)` + `DEFINE B AS B.auth_outcome = 'failure'` | each B row evaluated independently; `count(b)` in EMIT uses RPR running count |
| `EMIT count(f) AS failures` | `MEASURES COUNT(F.*) AS failures` | RPR aggregate over all matched F rows |

**Worked Example 3 — Cross-step field reference (lateral movement):**

```
DETECT lateral_movement
  SEQUENCE BY src.ip WITHIN 15m
    STEP a: network.direction = 'outbound' AND network.bytes > 1000000
    THEN b: network.direction = 'inbound'
            AND dst.ip = a.src.ip      -- B's dst must equal A's src (pivot)
            AND b.time > a.time
  EMIT src.ip, a.time AS exfil_started, b.time AS callback_time
```

Lowers to:

```sql
SELECT src_ip, A.event_time AS exfil_started, B.event_time AS callback_time
FROM events
MATCH_RECOGNIZE (
  PARTITION BY src_ip
  ORDER BY event_time
  MEASURES A.event_time AS exfil_started, B.event_time AS callback_time
  ONE ROW PER MATCH
  AFTER MATCH SKIP PAST LAST ROW
  PATTERN (A B)
  DEFINE
    A AS A.network_direction = 'outbound' AND A.network_bytes > 1000000,
    B AS B.network_direction = 'inbound'
       AND B.dst_ip = A.src_ip
       AND B.event_time > A.event_time
)
WHERE LAST(event_time) - FIRST(event_time) <= INTERVAL '15' MINUTE
```

**Field-path flattening note:** OCSF dot-notation paths (`a.src.ip`) are flattened to
snake_case column names in the DataFusion schema (`A.src_ip`). The Chumsky parser handles
this flattening during the sugar→`MATCH_RECOGNIZE` desugaring pass, not at execution time.
This is consistent with how all other PrismQL dot-notation paths are treated (§13.6
multi-schema flattening).

**Multi-schema note:** the `ORDER BY` binding uses the source's mapped time attribute
(OCSF `time`/`event_time`, or the configured timestamp column for native schema-on-read
sources). The sugar never hard-codes `event_time` — the desugarer resolves the time
attribute from the schema registry at plan time.

---

## Decision 4 — NOT / WITHOUT Non-Event Desugaring

> **DECIDED 2026-06-26 (human): BOTH forms are Phase A in-scope.**
> Exclusion-between-anchors (`THEN NOT / WITHOUT`) AND timeout/absence (`WATCH … UNLESS`)
> are both Phase A deliverables. See the full specification below.

### Context

A non-event step like `THEN NOT b: activity = 'account.approve'` has two distinct
interpretations with materially different semantics, lowering complexity, and analyst
mental models.

**Interpretation A — Exclusion (absence-between-anchors):**
"B must NOT occur between A and C." The sequence completes only if the anchor C event
arrives AND no B event occurred between A and C.

```
A ----[no B in here]---- C  → MATCH
A ----[B occurs]---------- → NO MATCH (B intervened, sequence fails)
A ----------------------------------------> timeout, no C → NO MATCH (C never came)
```

**Interpretation B — Timeout / absence-within-window (`WATCH … UNLESS`):**
"If B does not occur within time t after A, fire immediately." The sequence fires based
on A alone if no B arrives within the window; C is not required.

```
A ----t expires, no B seen → FIRE ALERT
A ----B occurs before t   → no alert
```

### Decision: BOTH forms are in Phase A scope

The human directed that both non-event semantics ship in Phase A:

1. **Exclusion (`THEN NOT / WITHOUT`)** — absence-between-two-anchors via SQL:2016
   `MATCH_RECOGNIZE` `{- B -}` exclusion syntax. The analyst writes this inside a
   `SEQUENCE…THEN` block.
2. **Timeout / absence (`WATCH … UNLESS`)** — fired by a wall-clock window expiry when an
   expected event is absent. This is a syntactically distinct, named construct, NOT an
   extension of `SEQUENCE…THEN NOT`. Previously proposed as Phase-B; **promoted to Phase A
   by human decision 2026-06-26.**

The two forms are kept strictly syntactically separate so `NOT` / `WITHOUT` inside
`SEQUENCE…THEN` **always** means exclusion-between-anchors. An analyst who wants timeout
semantics uses `WATCH … UNLESS`. There is no overloading.

---

### Form 1: Exclusion — `THEN NOT b: P THEN c: Q`

**Rationale:**

1. **Analyst mental model alignment.** The most common SOC use case for negation in a
   sequence is "A happened, then C happened, but B (the expected approval / containment /
   rollback) was absent between them." This is an exclusion pattern, not a timer.
2. **SQL:2016 `MATCH_RECOGNIZE` exclusion pattern exists.** The `{- B -}` exclusion
   syntax ("match B but do not include it in the matched sequence") maps directly.
3. **No ambiguity.** `NOT` / `WITHOUT` inside `SEQUENCE…THEN` is defined as exclusion
   with no trailing-anchor-optional semantics.
4. **FSQL's precedent.** Query.io (FSQL) uses exclusion semantics for negation in
   sequence-like constructs.

**Grammar constraint:** A `NOT` / `WITHOUT` step is only valid when at least one
non-negated `THEN` step follows it. A bare terminal `NOT` step with no trailing anchor
is a parse error with a diagnostic message: `"NOT/WITHOUT step requires a subsequent
THEN anchor step"`.

**With a trailing anchor (`THEN NOT b: P THEN c: Q`):**

```sql
MATCH_RECOGNIZE (
  ...
  PATTERN (A {- B -} C)
  DEFINE
    A AS <A predicate>,
    B AS <B predicate>,      -- B rows ARE evaluated but excluded from the output match
    C AS <C predicate>
)
```

The `{- B -}` exclusion operator in SQL:2016 means: match the pattern `A … C` only if
no row between A and C satisfies `B`. Rows matching B disqualify the match.

**Sugar → lowering table (exclusion):**

| Sugar | MATCH_RECOGNIZE target | Semantics |
|---|---|---|
| `THEN NOT b: P THEN c: Q` | `PATTERN (A {- B -} C)` + `DEFINE B AS P, C AS Q` | C must occur with no B between A and C |
| `THEN WITHOUT b: P THEN c: Q` | Same as NOT | `WITHOUT` is an alias; identical lowering |
| `THEN NOT b: P THEN c: Q WITHIN 10m` (per-step) | `PATTERN (A {- B -} C)` + post-match filter on span | C within 10m of A's end; no B allowed |

**Worked Example 4 — Unapproved account creation (the §12.4 example):**

```
DETECT unapproved_account
  SEQUENCE BY account.uid WITHIN 1h
    STEP c:   activity = 'account.create'
    THEN NOT a: activity = 'account.approve'
    THEN d:   activity = 'account.login'
  EMIT account.uid, c.time AS created, d.time AS first_login
```

Lowers to:

```sql
SELECT account_uid, C.event_time AS created, D.event_time AS first_login
FROM events
MATCH_RECOGNIZE (
  PARTITION BY account_uid
  ORDER BY event_time
  MEASURES C.event_time AS created, D.event_time AS first_login
  ONE ROW PER MATCH
  AFTER MATCH SKIP PAST LAST ROW
  PATTERN (C {- A -} D)
  DEFINE
    C AS C.activity = 'account.create',
    A AS A.activity = 'account.approve',
    D AS D.activity = 'account.login'
)
WHERE LAST(event_time) - FIRST(event_time) <= INTERVAL '1' HOUR
```

This fires when an account is created, then logs in, and no approval event occurred
between creation and first login — within a 1-hour window. The approval-absence is a
condition on the match, not a timer.

---

### Form 2: Timeout / Absence — `WATCH … UNLESS` (Phase A, promoted 2026-06-26)

**DECIDED 2026-06-26 (human): the `WATCH … UNLESS` timeout/absence construct is IN Phase A scope.**

This construct fires when an expected event **does not arrive** within a time window
after a trigger event. It is semantically a standing watchdog / absence alert, not a
sequence pattern matcher.

#### Surface Syntax

```ebnf
watch_block    ::= "DETECT" ident watch_stmt emit_clause?
               |   watch_stmt                        (* ad-hoc, no rule name *)

watch_stmt     ::= "WATCH" field_list "FOR" duration "AFTER" anchor_step
                   "UNLESS" suppress_step

anchor_step    ::= pattern_var ":" predicate         (* trigger event: starts the clock *)

suppress_step  ::= pattern_var ":" predicate         (* suppression event: cancels the alert *)

field_list     ::= field ("," field)*                (* partition key(s) *)

duration       ::= int ("s"|"m"|"h"|"d"|"w"|"mo")   (* shared with WITHIN *)

emit_clause    ::= "EMIT" emit_item ("," emit_item)* (* optional; defaults to partition key + anchor time *)
```

**Semantics:**

1. When an event matching `anchor_step` is observed for a given partition key value,
   a per-key timer is started with duration `FOR t`.
2. If a suppression event matching `suppress_step` arrives before the timer expires,
   the alert is **cancelled** (no output).
3. If the timer expires with no suppression event observed, the alert **fires**
   (the `EMIT` clause is evaluated using the anchor event's fields).
4. Multiple overlapping anchor events for the same partition key: each starts an
   independent timer. A suppression event cancels **all** active timers for that key.
5. The timer is a logical window, not a wall-clock alarm — it is evaluated against
   the RetentionCache's event stream. If no event arrives to drive evaluation, the
   window expires on the next query execution that reaches the expiry point in event time.

**Worked Example 5 — Missing approval timer:**

```
DETECT missing_approval_timer
  WATCH account.uid FOR 1h AFTER c: activity = 'account.create'
  UNLESS a: activity = 'account.approve'
  EMIT account.uid, c.time AS created
```

**Natural language:** "For each account.uid, if an `account.create` event occurs and
no `account.approve` event for the same uid is seen within 1 hour, fire an alert."

#### Lowering: `WATCH … UNLESS`

The `WATCH … UNLESS` construct does NOT desugar to SQL:2016 `MATCH_RECOGNIZE`. It
requires a separate engine primitive because:

1. `MATCH_RECOGNIZE` requires an anchor row at both ends; absence-only requires only
   the trigger row.
2. The timer must persist across query executions (the window may span multiple
   RetentionCache refresh cycles).

**Recommended lowering — negative anti-join over a keyed window:**

```sql
-- Conceptual lowering (the physical operator is a custom DataFusion logical node):
SELECT anchor.partition_key, anchor.event_time AS anchor_time
FROM events AS anchor
WHERE anchor.<anchor_predicate>
  AND NOT EXISTS (
    SELECT 1
    FROM events AS suppress
    WHERE suppress.<suppress_predicate>
      AND suppress.partition_key = anchor.partition_key
      AND suppress.event_time > anchor.event_time
      AND suppress.event_time <= anchor.event_time + INTERVAL '<t>'
  )
  AND <now_or_latest_event_time> > anchor.event_time + INTERVAL '<t>'
-- The final condition: only emit when the window has fully elapsed.
-- "Now" is defined as the latest event time in the RetentionCache for this
-- partition key, or the query execution timestamp if using event-time processing.
```

**DataFusion implementation notes:**

- The `WATCH … UNLESS` desugarer emits a custom `AbsenceWindowNode` logical plan node
  (distinct from `MatchRecognizeNode`).
- State management: per-partition-key timer state is held in RocksDB under the
  detection rule's namespace (analogous to how MATCH_RECOGNIZE partial-match state
  is held). On each RetentionCache refresh, the timer evaluator is invoked for active
  rules.
- The negative anti-join is a standard DataFusion `HashJoin` with `NOT IN` semantics
  — this is well-supported and avoids custom CBMC machinery.
- **Event-time vs processing-time:** The window boundary is evaluated in event time
  (the timestamp of the latest event processed for the partition key), not wall-clock
  time. This preserves deterministic replay behavior. A separate "liveness check"
  mechanism (outside the scope of this ADR) is needed for real-time alerting where no
  new events may arrive to trigger evaluation.

**Operator summary:**

| Property | `THEN NOT / WITHOUT` (exclusion) | `WATCH … UNLESS` (timeout) |
|----------|----------------------------------|---------------------------|
| Construct | `SEQUENCE … THEN NOT / WITHOUT` | `WATCH … FOR … AFTER … UNLESS` |
| Requires trailing anchor | YES (mandatory) | NO |
| Output trigger | Downstream anchor event arrives with no B between | Timer expires with no suppression event |
| SQL:2016 lowering | `MATCH_RECOGNIZE` `{- B -}` | `AbsenceWindowNode` (custom, negative anti-join) |
| RocksDB state | MATCH_RECOGNIZE partial-match state | Per-key timer state |
| Phase A | YES | YES (promoted 2026-06-26) |

#### Phase-A scope / effort impact

Promoting `WATCH … UNLESS` from Phase-B to Phase-A increases Phase-A scope. The
estimated additional effort beyond the MATCH_RECOGNIZE engine work:

- `AbsenceWindowNode` DataFusion logical plan node: 1 implementation story
- Parser support for `WATCH … FOR … AFTER … UNLESS` syntax (Chumsky grammar extension): 0.5 stories
- RocksDB per-key timer state schema + TTL eviction: 0.5 stories (likely shared with
  MATCH_RECOGNIZE state management; co-locate in the same story)
- Integration tests: co-located with the MATCH_RECOGNIZE integration test story

Total additional Phase-A scope estimate: ~2 stories added to the SEQUENCE/MATCH_RECOGNIZE epic.

The Phase-A scope reconciliation in §16.2 #4 (MATCH_RECOGNIZE in scope) should be
updated at morph time to include `WATCH … UNLESS` as an in-scope deliverable alongside
the sugar and raw `MATCH_RECOGNIZE`.

---

## Open Decisions for Human

### HD-1 — NOT/WITHOUT primary semantics — RESOLVED 2026-06-26 (human): BOTH in Phase A

**Decision:** BOTH forms ship in Phase A:
- `THEN NOT / WITHOUT` in `SEQUENCE … THEN` = **exclusion-between-anchors** (SQL:2016
  `{- B -}`). Trailing anchor is mandatory. See Decision 4 Form 1 above.
- `WATCH … UNLESS` = **timeout/absence** (custom `AbsenceWindowNode`, negative anti-join).
  No trailing anchor required. Promoted from Phase-B to Phase A. See Decision 4 Form 2 above.

The two forms are syntactically distinct: `NOT`/`WITHOUT` inside `SEQUENCE…THEN` is
always and only exclusion. Analysts who need timeout semantics use `WATCH … UNLESS`.
No overloading of the `NOT` keyword occurs.

**Phase-A scope impact:** ~2 additional stories for `WATCH … UNLESS` (AbsenceWindowNode +
parser + RocksDB timer state). Reconcile with §16.2 #4 at morph time.

### HD-2 (lower urgency) — Raw MATCH_RECOGNIZE as power-user escape hatch

**ACCEPTED 2026-06-27 (human):** YES — expose raw `MATCH_RECOGNIZE` as documented
power-user syntax. Lower priority / later phase within Phase A scope.

The sugar desugars to it; letting power users write it directly costs nothing extra and
prevents "I need to do X that the sugar can't express" hitting a dead end. Gate behind a
doc note: "raw MATCH_RECOGNIZE is advanced syntax; most analysts should use SEQUENCE…THEN."

**Tradeoff (recorded):** raw `MATCH_RECOGNIZE` in user queries is harder to lint for the
join-guard NFR (§12.2) — the linter must understand both syntactic forms. This is a known
cost, accepted at Phase A.

---

## Summary of Decisions

| # | Question | Decision | Status |
|---|----------|----------|--------|
| 1 | Keyword finalization | Adopt DETECT / SEQUENCE BY / STEP / THEN / NOT\|WITHOUT / ANY OF / EMIT / OVERLAP as specified; case-insensitive; WITHIN reused at two levels by position | PROPOSED |
| 2 | Overall-WITHIN semantics | Bounds first-to-last span of matched sequence; expressed as post-match WHERE predicate; NOT a hard row filter | PROPOSED |
| 3 | Cross-step running semantics | Pattern var references read the last matched row for that var; aggregates (`count(f)`) available for quantified Kleene vars; OCSF dot-notation flattened at desugar time | PROPOSED |
| 4 | NOT/WITHOUT desugaring — BOTH forms Phase A | (a) `THEN NOT/WITHOUT … THEN` = exclusion-between-anchors via SQL:2016 `{- B -}`; bare terminal NOT forbidden. (b) `WATCH … FOR … AFTER … UNLESS` = timeout/absence via `AbsenceWindowNode` (negative anti-join); promoted from Phase-B to Phase A 2026-06-26. Two syntactically distinct constructs; no overloading. Phase-A scope +~2 stories. | **DECIDED 2026-06-26 (human): BOTH** |
| 5 | Raw MATCH_RECOGNIZE escape hatch | YES — expose as documented advanced syntax, lower priority / later phase | **ACCEPTED 2026-06-27 (human)** |
