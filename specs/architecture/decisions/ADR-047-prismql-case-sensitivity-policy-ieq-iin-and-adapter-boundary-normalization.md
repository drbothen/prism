---
document_type: adr
adr_id: "ADR-047"
title: "PrismQL Case-Sensitivity Policy — Case-Sensitive Default, IEQ/IIN Opt-In, and Adapter-Boundary OCSF Enum-Label Normalization"
status: accepted
date: "2026-06-27"
version: "1.1"
modified: "2026-07-06"
producer: architect
subsystems_affected: [SS-11, SS-02]
supersedes: null
superseded_by: null
amends: null
anchor_stories: [S-PRISMQL-CASE-INSENSITIVE-001]
related_adrs: [ADR-041, ADR-043, ADR-044, ADR-046, ADR-024]
related_bcs: [BC-2.11.002, BC-2.11.004, BC-2.11.018, BC-2.02.002, BC-2.02.010]
locked_decisions: [OD-1, OD-2, OD-3, OD-4]
wiring_deferred_to: null
open_decisions: []
resolved_decisions:
  - OD-1: "ALL OCSF enum-label fields (architect recommendation adopted). Demo-minimum guaranteed set: severity + status. Human sign-off D-1398, 2026-06-27."
  - OD-2: "Case-sensitive default confirmed for =, !=, IN. Human sign-off D-1398, 2026-06-27."
  - OD-3: "IEQ/IIN/INE spelling confirmed (I-prefix family, collision-free with =~ regex). Human sign-off D-1398, 2026-06-27."
  - OD-4: "Zero-rows near-miss pedagogical hint DEFERRED to follow-up story. NOT included in S-PRISMQL-CASE-INSENSITIVE-001. Human sign-off D-1398, 2026-06-27."
---

# ADR-047: PrismQL Case-Sensitivity Policy — Case-Sensitive Default, IEQ/IIN Opt-In, and Adapter-Boundary OCSF Enum-Label Normalization

## Status

ACCEPTED v1.1 (2026-07-06). All four open decisions (OD-1 through OD-4) ratified per human
sign-off D-1398 (2026-06-27):

| ID | Resolution |
|----|------------|
| OD-1 | ALL OCSF enum-label fields normalized at adapter boundary. Demo-minimum guaranteed: severity + status. |
| OD-2 | Case-sensitive default for `=`/`!=`/`IN` confirmed. |
| OD-3 | IEQ/IIN/INE operator spelling confirmed. |
| OD-4 | Zero-rows near-miss pedagogical hint DEFERRED to follow-up story. NOT in S-PRISMQL-CASE-INSENSITIVE-001. |

Product-owner has authored the following artifacts per §Amendment Obligations:
- BC-2.11.024 (new): PrismQL IEQ/IIN/INE case-insensitive operators
- BC-2.02.013 (new): Adapter-boundary OCSF enum-label canonical-case normalization
- BC-2.11.002 v1.5, BC-2.11.004 v1.13, BC-2.11.018 v1.3, BC-2.02.002 v1.5, BC-2.02.010 v1.5 (amended)

Original PROPOSED record: v1.0 (2026-06-27). Architect decision following research artifact
`prismql-case-sensitivity-2026-06-27.md`. Addresses the demo-critical defect where
LLM-agent queries like `WHERE severity IN ('HIGH','CRITICAL')` return zero rows
because prism stores OCSF Title-case labels (`'High'`, `'Critical'`).

## Context

### The Demo Defect

LLM agents authoring PrismQL queries have consistent priors from SIEM query languages
where severity/status values are uppercase (`'HIGH'`, `'CRITICAL'`, `'open'`). Prism's
OCSF normalization pipeline produces Title-case labels (`'High'`, `'Critical'`) per
`crates/prism-ocsf/src/enum_map.rs`. PrismQL `=` and `IN` are exact case-sensitive
matches today (consistent with DataFusion's default). The result: zero rows on demo
queries, failing the T13 demo scenario silently.

### Three Facts from the Codebase

1. **PrismQL already has a case-insensitivity convention** — but only for string-pattern
   operators. `StringOp` carries a `case_insensitive: bool` flag surfaced as `ICONTAINS`/
   `ISTARTSWITH`/`IENDSWITH` in the grammar (`filter_parser.rs:985-993`), lowered to
   `lower(field) LIKE lower('%pat%')` in `predicate_to_datafusion_sql`
   (`pipe_sql_emitter.rs:541-564`). The `=`/`!=` (`Predicate::Compare`) and `IN`
   (`Predicate::In`) paths have NO such flag (`ast.rs:1284-1306`, `1550-1554`). This is
   the exact gap this ADR closes.

2. **Prism already encodes canonical OCSF casing.** `enum_map.rs` maps `severity_id: 4
   → "High"`, `5 → "Critical"`, `1 → "Informational"`. The canonical string label form
   for OCSF severity in prism is Title-case. The data is internally consistent; the
   agent's guesses are inconsistent with an undocumented convention.

3. **DataFusion emitter is the single lowering point.** `predicate_to_datafusion_sql`
   (`pipe_sql_emitter.rs:506`) is where all filter-mode predicates become DataFusion SQL,
   and it already uses the `lower(...)` idiom for `ICONTAINS`. Extending this to
   `=`/`IN` is a near-verbatim pattern reuse with zero new DataFusion version dependencies.

### Why This ADR Governs Two Mechanisms

Adapter-boundary normalization (§D.3) and query-time CI operators (§D.2) are
complementary, not alternatives. Normalization makes the stored data trustworthy and
aggregations correct across all sensors. CI operators provide an ergonomic safety net for
fields prism has not yet normalized, free-form non-enum fields (hostnames, usernames,
file paths), and forgiving the agent that types wrong casing. Both are needed; the ADR
establishes the contract for both.

## Decision

### D.1 — Default Policy: Case-Sensitive `=`, `!=`, `IN`

The default semantics of `=`, `!=`, and `IN` in PrismQL remain case-sensitive exact
matches. Rationale:

- **SIEM execution-language consensus:** KQL `==`, EQL `==`, Elastic `term` are all
  case-sensitive for structured field equality. PrismQL is an execution language in this
  category, not a portable rule abstraction (Sigma). The LLM agent's learned priors for
  structured field equality (`==`) point to case-sensitive behavior.
- **DataFusion parity:** DataFusion's SQL engine is case-sensitive by default. The
  PrismQL semantic model matches the underlying engine — no surprise for analysts who
  drop down to SQL mode.
- **Existing-query preservation:** flipping the default to case-insensitive would
  silently alter precision for username, file path, process name, and registry key
  filters where exact case is security-meaningful (masquerading detection). This is a
  correctness regression for existing saved rules.
- **Discoverability via grammar:** the case-sensitive default is named behavior (it must
  be documented with a canonical example), not a silent fallthrough. The `IEQ`/`IIN`
  grammar makes the insensitive opt-in equally discoverable.

### D.2 — Opt-In Case-Insensitive Operators: IEQ / IIN

Extend PrismQL's existing `I`-prefix convention (`ICONTAINS`/`ISTARTSWITH`/`IENDSWITH`)
to equality and membership:

| Operator | Semantics | DataFusion Lowering |
|----------|-----------|---------------------|
| `IEQ` | Case-insensitive equality: `field IEQ 'val'` | `lower(field) = lower('val')` |
| `IIN` | Case-insensitive membership: `field IIN ('a','b')` | `lower(field) IN (lower('a'), lower('b'))` |
| `INE` | Case-insensitive inequality: `field INE 'val'` | `lower(field) != lower('val')` |

The `INE` operator is included for completeness of the family (additive, no extra cost).
Operators are parsed case-insensitively in the grammar (`kw(...)` combinator) — `ieq` and
`IEQ` both parse.

**Why `I`-prefix and not a symbolic operator:**
PrismQL uses `=~` for MATCHES (regex) (`filter_parser.rs:887`). Adopting `=~` for
case-insensitive equality (KQL convention) would collide with the existing regex operator
and create a grammar ambiguity. The `I`-prefix is collision-free, already established in
PrismQL, and a single, consistent mental model for operators that carry a
case-insensitivity modifier.

**DataFusion lowering rationale:**
`lower()`/`upper()` normalization is the DataFusion-idiomatic, version-stable,
unambiguously-supported mechanism. `ILIKE` is supported but is a pattern operator (its
`%`/`_` metacharacters are semantically wrong for exact `=`/`IN` matching). `COLLATE`
is not supported in DataFusion for case-insensitive matching. The `lower(...)` idiom is
already exercised in production for `ICONTAINS` — reusing it adds zero new DataFusion
version dependency.

### D.3 — Adapter-Boundary Canonical-Case Normalization

Independently of D.2, normalize OCSF enum-label string fields to their canonical
OCSF casing at the adapter/normalizer boundary in `prism-ocsf`, reusing the canonical
captions already in `enum_map.rs`. This is the OCSF-blessed architecture: the OCSF
ecosystem (e.g., Datadog's OCSF processor workflow) normalizes vendor values — including
case — at ingest, not at query time.

**Scope (pending OD-1 human decision):**
The scope of which OCSF enum-label fields are normalized is an open product decision
(OD-1). The architect recommends normalizing all OCSF enum-label string fields (severity,
status, activity, disposition, category, and their sensor-specific mappings) to achieve
full cross-sensor consistency. The minimum viable subset for the T13 demo is the two
demo-critical fields: `severity` and `status`.

**Current inconsistency by sensor (verified in codebase):**
- CrowdStrike adapter: emits Title-case severity (consistent with `enum_map.rs`)
- Armis adapter: emits UPPER-case (`'UNHANDLED'`)
- Claroty adapter: emits as-received from API (`'Unresolved'`)

**Consequences of D.3:**
- `GROUP BY severity` produces correct aggregation across sensors (no `'HIGH'`/`'High'`
  fragmentation)
- Normalization cost is paid once at ingest, not per query
- Column semantics in sensor TOML specs change: `severity` column value is now
  contractually Title-case per OCSF. BC-2.02.002 and BC-2.02.010 require amendment to
  make this normalization explicit in postconditions (product-owner must amend).
- The `prism-ocsf` `enum_map.rs` is the canonical casing authority; any sensor-specific
  override must be justified against the OCSF schema.

### D.4 — Discoverability

`IEQ`/`IIN` must be reflected in:
1. The PrismQL grammar reference resource (governed by BC-2.11.022 / ADR-045 parity gate)
2. The `prism describe` / tool-schema pedagogical examples (ADR-041 L1/L2 teaching surface)
3. An OCSF casing note in agent-facing docs: *"OCSF severity is Title-case (`'High'`).
   Use `severity IEQ 'high'` to match regardless of case, or `severity = 'High'` for
   canonical exact match."*
4. (Pending OD-4 human decision) A zero-rows near-miss hint: when `=`/`IN` returns zero
   rows but `IEQ`/`IIN` would match, emit a pedagogical diagnostic. This is the
   highest-leverage agent-ergonomics win and directly prevents the demo failure mode from
   recurring silently.

## Alternatives Considered

### Alt-1: Case-Insensitive Default (`=`/`IN` CI by default)

Rejected. Would silently change precision for username, file path, process name, registry
key, and any other case-meaningful field in existing saved rules and detection rules.
Diverges from DataFusion's own default, creating a semantic surprise for analysts who
understand the underlying SQL engine. Contradicts the SIEM execution-language consensus
(KQL/EQL). The ONLY scenario where this is unambiguously correct is if prism were a
portable rule-abstraction language like Sigma — it is not.

### Alt-2: `ILIKE` for IEQ Lowering

Considered but rejected for exact equality. `ILIKE` is a pattern-matching operator; its
`%` and `_` metacharacters are semantically wrong for a non-pattern match. A literal
value passed to `ILIKE` without escaping will misbehave on values containing `%` or `_`
(e.g., `status ILIKE 'in_progress'` matches `'in progress'`). The `lower()` idiom has
none of these failure modes. `ILIKE` remains appropriate for pattern operators.

### Alt-3: COLLATE

Rejected. DataFusion does not implement ANSI `COLLATE` for case-insensitive matching.
The DataFusion project guidance is to extend the parser/planner rather than use
`COLLATE`. This mechanism is unavailable in the current DataFusion version.

### Alt-4: `=~` Symbolic Operator for Case-Insensitive Equality

Rejected. PrismQL uses `=~` for MATCHES (regex), grammar source `filter_parser.rs:887`.
Adding `=~` as a CI equality operator creates a direct grammar collision — an analyst
seeing `severity =~ 'high'` cannot determine without documentation whether this is a
regex match or a CI equality. The `I`-prefix convention is collision-free and
already-established.

### Alt-5: Sigma-style `|ci` Modifier

Considered. Would allow `severity = 'high' |ci`. Less consistent with PrismQL's existing
`I`-prefix family (`ICONTAINS` etc.) and requires a modifier-argument grammar extension.
No advantage over `IEQ`/`IIN` that outweighs the added grammar complexity. The `I`-prefix
is simpler and already established.

### Alt-6: Query-Time Normalization Only (No Adapter Boundary Fix)

Rejected as insufficient. Query-time CI via `IEQ`/`IIN` solves case mismatches in
filter predicates but does NOT fix `GROUP BY severity` fragmentation (where `'HIGH'` and
`'High'` produce separate buckets), does NOT achieve cross-sensor consistency, and does
not align with the OCSF design intent (normalize at ingest). D.2 and D.3 are both
required; D.3 is the primary fix for data quality, D.2 is the ergonomic safety net.

## Consequences

### Positive

- Demo defect resolved: `severity IEQ 'high'` matches `'High'` stored by CrowdStrike
  adapter; `status IEQ 'open'` matches Claroty's `'Unresolved'` once D.3 lands.
- Cross-sensor aggregation correctness: `GROUP BY severity` produces 5 buckets
  (Informational/Low/Medium/High/Critical), not 8+ fragmented variants.
- Backward compatible: all existing queries preserve exact semantics. Additive change only.
- Consistent mental model: one `I`-prefix convention for all case-insensitive operators.
- `normalized_pql` (BC-2.11.018) reflects `IEQ`/`IIN` in the round-trip output,
  reinforcing the grammar in the agent's in-session learning loop (ADR-041 OPD-1).

### Negative / Tradeoffs

- `lower(field) = lower('val')` sacrifices index sargability — the full column is
  lowercased before comparison. Prism's MemTable query size is bounded at 10,000 rows
  (BC-2.11.006), so this cost is negligible in practice. Analysts running `IEQ` on
  large-scale cold-query engines outside prism cannot assume the same performance profile.
- Adapter-boundary normalization (D.3) changes the column semantics in sensor TOML specs
  (severity, status values become contractually Title-case). Any downstream consumer of
  raw OCSF label strings must be audited for the new canonical casing. DTU test vectors
  that assert uppercase severity values will need updating.
- Adding `case_insensitive: bool` to `Predicate::Compare` and `Predicate::In` adds two
  public struct fields. These types are `#[non_exhaustive]`, so existing match arms
  compile unchanged, but the `EXPECTED=87` non-exhaustive gate count is NOT incremented
  (these are new fields on existing structs, not new `#[non_exhaustive]`-annotated types;
  the gate counts annotated types, not field additions). Verify at story implementation
  time.

## Open Decisions

All decisions resolved. See §Status for resolution table.

~~These require human sign-off before this ADR advances from PROPOSED to ACCEPTED:~~

| ID | Decision | Resolution | Sign-Off |
|----|----------|------------|----------|
| OD-1 | Adapter-boundary normalization scope | ALL OCSF enum-label fields; demo-minimum: severity + status | D-1398, 2026-06-27 |
| OD-2 | Case-sensitive default for `=`/`IN` | Case-sensitive confirmed | D-1398, 2026-06-27 |
| OD-3 | Operator spelling: IEQ/IIN/INE | IEQ/IIN/INE confirmed | D-1398, 2026-06-27 |
| OD-4 | Zero-rows near-miss pedagogical hint | DEFERRED to follow-up story | D-1398, 2026-06-27 |

## Amendment Obligations (for Product Owner)

This ADR, once ACCEPTED, triggers the following BC amendments (product-owner must author):

1. **BC-2.11.002 (PrismQL Filter Mode Parsing)** — amend §Postconditions to add `IEQ`,
   `IIN`, `INE` to the supported operator table; amend §Canonical Test Vectors to add
   IEQ/IIN round-trip vectors.
2. **BC-2.11.004 (PrismQL Pipe Mode)** — same operator table extension as BC-2.11.002
   (filter predicates are shared grammar per ADR-046 D7 invariant in BC-2.11.023).
3. **BC-2.11.018 (normalized_pql Echo)** — amend §Postconditions / EC-11-055 to cover
   that IEQ/IIN predicates are reflected correctly in the normalized PQL round-trip.
4. **BC-2.02.002 (DynamicMessage Creation)** — amend §Postconditions to make
   adapter-boundary canonical-case normalization explicit: "OCSF enum-label string fields
   are normalized to canonical OCSF casing per `enum_map.rs` before the DynamicMessage
   is emitted." Scope conditioned on OD-1 resolution.
5. **BC-2.02.010 (OCSF Enum Value Map)** — amend §Postconditions and §Invariants to
   explicitly state that the enum-value-map defines the canonical casing for string labels
   at the adapter boundary (not only for MCP display enrichment).

Additionally, NEW BCs are needed for IEQ/IIN semantics (product-owner authors):
- NEW: `BC-2.11.024` — PrismQL IEQ/IIN/INE case-insensitive equality and membership
  operators: grammar, AST flag, emitter lowering, round-trip property.
- NEW: `BC-2.02.013` — Adapter-boundary OCSF enum-label canonical-case normalization
  (conditioned on OD-1 resolution; scope defined by human decision).

## Verification Obligations (for Formal Verifier)

1. **proptest** — verify that `IEQ(field, 'VAL')` and `IEQ(field, 'val')` produce
   identical DataFusion plans for arbitrary mixed-case string literals.
2. **proptest (round-trip)** — verify `normalized_pql` for queries containing `IEQ`/`IIN`
   parses back to the same AST (BC-2.11.018 round-trip invariant extended to new ops).
3. **Integration** — verify `severity IEQ 'high'` returns rows stored as `'High'` from
   CrowdStrike DTU; `severity IEQ 'HIGH'` returns the same rows.

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.1 | S-PRISMQL-CASE-INSENSITIVE-001-bc-burst | 2026-07-06 | product-owner | **PROPOSED → ACCEPTED.** All four ODs resolved per human sign-off D-1398 (2026-06-27). Frontmatter: `status: proposed → accepted`, `version: 1.0 → 1.1`, `modified: 2026-07-06`, `locked_decisions: [OD-1..OD-4]`, `open_decisions: []`, `resolved_decisions` recorded. §Status updated with resolution table and PO artifact delivery record. §Open Decisions table updated to show resolutions. ARCH-INDEX ADR-047 row update (PROPOSED v1.0 → ACCEPTED v1.1) to be applied by state-manager. |
| 1.0 | S-PRISMQL-CASE-INSENSITIVE-001-design | 2026-06-27 | architect | Initial PROPOSED decision. Two-pronged design: case-sensitive default + IEQ/IIN opt-in + adapter-boundary normalization. Research basis: prismql-case-sensitivity-2026-06-27.md. |
