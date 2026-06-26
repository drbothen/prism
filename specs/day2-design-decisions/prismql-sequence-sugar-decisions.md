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
  - HD-1: NOT/WITHOUT primary semantics (exclusion vs timeout — see §4)
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

> **THIS IS THE KEY HUMAN DECISION.** See "Open Decisions for Human" section below.

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

**Interpretation B — Timeout (absence-within-window):**
"If B does not occur within time t after A, fire immediately." The sequence fires based
on A alone if no B arrives within the window; C is not required.

```
A ----t expires, no B seen → FIRE ALERT
A ----B occurs before t   → no alert
```

### Recommendation: ADOPT Interpretation A (exclusion-between-anchors) as primary

**Rationale:**

1. **Analyst mental model alignment.** The most common SOC use case for negation in a
   sequence is "A happened, then C happened, but B (the expected approval / containment /
   rollback) was absent between them." This is an exclusion pattern, not a timer. Analysts
   writing `DETECT unapproved_account: STEP c: create THEN NOT a: approve` expect the
   alert to fire when an account is subsequently observed in a state consistent with
   missing approval — i.e., when some downstream activity C occurs without A having
   intervened. Pure timeout ("fire if approve never comes") requires different syntax.

2. **SQL:2016 `MATCH_RECOGNIZE` exclusion pattern exists.** The standard defines the
   `{- B -}` exclusion syntax ("match B but do not include it in the matched sequence").
   Combined with a final anchor, this expresses "A appears, then C appears, with no B
   between them." This is the idiomatic RPR lowering.

3. **Timeout is a distinct construct.** Pure event-absence-with-timeout ("alert if B does
   not arrive within t seconds of A") is semantically a **standing alert**, not a sequence.
   It requires persistent per-key state tracking with a wall-clock timer. This is:
   - Hard to implement correctly over a federated/ephemeral cache (RetentionCache does
     not guarantee flush on window expiry).
   - Semantically a different primitive (watchdog/alerter, not pattern matcher).
   - Already expressible via the threshold correlation type (`GROUP BY key HAVING
     COUNT(*) = 0 within_window(t)` over the approval event stream).
   Mixing timeout semantics into the `NOT` keyword would make the same syntax mean two
   different things depending on whether a trailing anchor is present.

4. **FSQL's precedent.** Query.io (FSQL) uses exclusion semantics for negation in
   sequence-like constructs; their recipe library examples use the "B absent between A
   and C" model.

### Lowering: Exclusion (Interpretation A)

**Without a trailing anchor (degenerate: `THEN NOT b: P` as final step):**

This is only meaningful if another `THEN` follows. A bare terminal `NOT` step with no
anchor has undefined semantics — the grammar SHOULD NOT allow `NOT` as the last step.
Validation: parser rejects `NOT` on the final `THEN` with a syntax error pointing to
the missing trailing anchor.

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

**Sugar → lowering table:**

| Sugar | MATCH_RECOGNIZE target | Semantics |
|---|---|---|
| `THEN NOT b: P THEN c: Q` | `PATTERN (A {- B -} C)` + `DEFINE B AS P, C AS Q` | C must occur with no B between A and C |
| `THEN WITHOUT b: P THEN c: Q` | Same as NOT | `WITHOUT` is an alias; identical lowering |
| `THEN NOT b: P THEN c: Q WITHIN 10m` (per-step) | `PATTERN (A {- B -} C)` + `DEFINE C AS C.t <= PREV(C.t) + 10m` | C within 10m of A's end; no B allowed |

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

### Timeout construct (Interpretation B) — proposed separate syntax

If pure timeout semantics are needed (fire if event B does NOT arrive within t of A),
the proposed surface is a distinct `WATCH` construct, not `SEQUENCE…THEN NOT`:

```
DETECT missing_approval_timer
  WATCH account.uid FOR 1h AFTER c: activity = 'account.create'
  UNLESS a: activity = 'account.approve'
  EMIT account.uid, c.time AS created
```

This is NOT defined here — it is flagged as a potential Phase-B construct and
a follow-up ADR. The key decision is: `NOT`/`WITHOUT` in `SEQUENCE…THEN` means
**exclusion-between-anchors only**.

---

## Open Decisions for Human

### HD-1 (REQUIRED before ADR is authored) — NOT/WITHOUT primary semantics

**The question:** Should `THEN NOT b: P THEN c: Q` mean:
- **(A — RECOMMENDED)** "C occurred AND B was absent between the two anchors" (exclusion),
  or
- **(B)** "B did not occur within the window" (pure timeout, no anchor required)?

**Why this is hard:**
- Exclusion (A) is safer, maps cleanly to SQL:2016 `{- B -}`, and keeps the `SEQUENCE`
  surface deterministic over a federated cache. Its UX limitation: analysts who want a
  "fire after N minutes of silence" pattern cannot use `NOT` — they need the separate
  `WATCH … UNLESS` construct (Phase-B).
- Timeout (B) is more intuitive for some SOC use cases (e.g., "alert if a host does not
  check in within 24h") but requires persistent wall-clock state that is architecturally
  at odds with the ephemeral/federated model. It also introduces ambiguity: what does
  `THEN NOT b: P THEN c: Q` mean when B is absent — did C occur first, or did the timer
  fire?

**Recommended resolution:** Adopt A (exclusion) for the `NOT`/`WITHOUT` keyword in
Phase A. Define `WATCH … UNLESS` as a distinct Phase-B construct. Flag in the ADR that
pure-timeout absence detection is planned but syntactically distinct.

**What happens if you choose B:** The engine requires a wall-clock alarm subsystem
(outside the NFA match loop), additional RocksDB state per watch rule, and a more complex
interaction with the RetentionCache eviction policy. The Phase-A MATCH_RECOGNIZE
implementation budget should be scoped assuming A.

### HD-2 (lower urgency) — Raw MATCH_RECOGNIZE as power-user escape hatch

**The question:** Should the PrismQL parser expose raw `MATCH_RECOGNIZE` syntax (the full
SQL:2016 form including `PATTERN`/`DEFINE`/`MEASURES`/`PARTITION BY`/`ORDER BY`/
`AFTER MATCH SKIP`) directly to users?

**Recommendation:** YES — expose as documented power-user syntax. The sugar desugars to
it; letting power users write it directly costs nothing extra and prevents "I need to do
X that the sugar can't express" hitting a dead end. Gate behind a doc note: "raw
MATCH_RECOGNIZE is advanced syntax; most analysts should use SEQUENCE…THEN."

**Tradeoff:** raw `MATCH_RECOGNIZE` in user queries is harder to lint for the join-guard
NFR (§12.2) — the linter must understand both syntactic forms. This is a known cost,
acceptable at Phase A.

---

## Summary of Decisions

| # | Question | Recommendation | Status |
|---|----------|----------------|--------|
| 1 | Keyword finalization | Adopt DETECT / SEQUENCE BY / STEP / THEN / NOT|WITHOUT / ANY OF / EMIT / OVERLAP as specified; case-insensitive; WITHIN reused at two levels by position | PROPOSED |
| 2 | Overall-WITHIN semantics | Bounds first-to-last span of matched sequence; expressed as post-match WHERE predicate; NOT a hard row filter | PROPOSED |
| 3 | Cross-step running semantics | Pattern var references read the last matched row for that var; aggregates (`count(f)`) available for quantified Kleene vars; OCSF dot-notation flattened at desugar time | PROPOSED |
| 4 | NOT/WITHOUT desugaring | Exclusion-between-anchors (SQL:2016 `{- B -}`) as primary; bare terminal NOT forbidden; pure timeout is a distinct Phase-B `WATCH…UNLESS` construct | PROPOSED — **HD-1 required from human** |
| 5 | Raw MATCH_RECOGNIZE escape hatch | YES — expose as documented advanced syntax | PROPOSED — **HD-2 lower urgency** |
