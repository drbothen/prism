---
document_type: adversarial-review-pass
pass_number: 1
pr_number: 130
story_id: S-3.06
branch_sha: 4dff9394
verdict: BLOCKED
convergence_window: 0/3
reviewer: adversary
timestamp: 2026-05-06
producer: adversary
inputs:
  - .factory/stories/S-3.06-prismql-write-parser.md
  - .factory/specs/behavioral-contracts/BC-2.11.003-prismql-sql-mode.md
  - .factory/specs/behavioral-contracts/BC-2.11.004-prismql-pipe-mode.md
  - .factory/specs/behavioral-contracts/BC-2.11.006-query-security-limits.md
input-hash: "[live-adv-review]"
traces_to: PR-130
---

# PR #130 Adversarial Pass-1 — BLOCKED

## Verdict: BLOCKED — 2 HIGH, 3 MEDIUM, 3 LOW, 7 OBS

Convergence window: **0 / 3** clean passes

The 5-commit fix bundle (`80ac978e`, `a06bafde`, `e8620e39`, `236146a1`, `4dff9394`) correctly addresses the 5 originally-named PR review findings. However, fresh-context analysis surfaced 2 NEW HIGH-severity defects introduced by the fix bundle itself.

## High Findings

### F-PR130-P1-HIGH-001 — `parse_with_registry` BYPASSES the SQL denylist (BC-2.11.003 v1.4)
- **Confidence:** HIGH | **Category:** Security / Spec drift
- **Where:** `crates/prism-query/src/filter_parser.rs:115-158` (added by commit `a06bafde`)
- **What:** New public entry point `parse_with_registry` performs only DML/SELECT/pipe branch checks. Falls through to filter mode for everything else. The denylist enforced in `parse_with_limits` (lines 211-264) — covering MERGE, REPLACE, UPSERT, COPY, CREATE, DROP, ALTER, RENAME, TRUNCATE, COMMENT, COMMIT, ROLLBACK, SAVEPOINT, RELEASE, BEGIN, START, GRANT, REVOKE, DENY, EXECUTE, CALL, DO, PERFORM, EXPLAIN, ANALYZE, VACUUM, LOCK, REINDEX, SET, SHOW, USE, PRAGMA, ATTACH, DETACH (~33 keywords) — is NOT applied in `parse_with_registry`.
- **Why fails:** A caller submitting `MERGE INTO foo USING bar` via `parse_with_registry` gets generic E-QUERY-001 instead of canonical E-QUERY-002. Violates BC-2.11.003 v1.4 denylist invariant. Per `lib.rs:110`, "External consumers MUST use `PrismQlParser::parse` or `PrismQlParser::parse_with_registry`" — both must be equivalent public security entry points.
- **Fix:** Factor denylist + mode-detection into a shared helper invoked by both `parse_with_limits` and `parse_with_registry`. OR duplicate the denied_keywords loop into `parse_with_registry` between empty-string check and DML branch. Add regression test: `test_BC_2_11_003_parse_with_registry_rejects_denied_keyword` for MERGE, CREATE, COMMIT, GRANT, EXPLAIN.

### F-PR130-P1-HIGH-002 — `parse_sql_dml_with_limits` missing from BC + perimeter-violation crate
- **Confidence:** HIGH | **Category:** Perimeter / Spec drift
- **Where:** Implementation `sql_parser.rs:864`; lib.rs:101 (listed); BC-2.11.006 v1.13 lines 25-54 (NOT listed); perimeter-violation/src/main.rs (NOT imported)
- **What:** Commit `236146a1` added `pub(crate) fn parse_sql_dml_with_limits` to forward `ParseLimits` through the DML path. Visibility correctly `pub(crate)`. But the 3-way perimeter-symbols-sync invariant is broken: BC frontmatter and perimeter-violation crate do not enumerate this 27th symbol. CI sync silently passes because (a) BC list contains `parse_sql_dml` (passes the check), (b) `parse_sql_dml_with_limits` in lib.rs has no `::`, no leaf in `bc_leaf_to_normalized` mapping, so it's classified as "not a restricted symbol reference" and silently ignored.
- **Why fails:** If a future refactor changes `pub(crate) fn parse_sql_dml_with_limits` to `pub fn parse_sql_dml_with_limits`, NO existing CI gate will catch it. The perimeter-violation crate doesn't import the symbol; cargo check still fails on OTHER 27 imports; sync check passes; lib.rs docstring claims authority from BC but BC doesn't list this symbol. Silent perimeter coverage hole introduced by the fix bundle itself.
- **Fix:** (1) Add `prism_query::sql_parser::parse_sql_dml_with_limits` to BC-2.11.006 frontmatter `restricted_symbols.symbols`. (2) Add `use prism_query::sql_parser::parse_sql_dml_with_limits;` to `tests/external/perimeter-violation/src/main.rs` plus `let _ = parse_sql_dml_with_limits;` in main(). (3) Bump BC-2.11.006 to v1.14 with changelog row. (4) New expected count: 28 E-errors (23 E0603 + 5 E0624).

## Medium Findings

### F-PR130-P1-MED-001 — `parse_pipe_with_write` re-snapshots ParseLimits (F-HIGH-001 race regression)
- **Confidence:** HIGH | **Category:** Concurrency / Spec drift
- **Where:** `pipe_parser.rs:711` and `:725` — both call `let limits = security::ParseLimits::snapshot();` instead of using caller-provided limits.
- **What:** `PrismQlParser::parse_with_registry` snapshots ParseLimits once and installs it as thread-local at filter_parser.rs:119-121. Passes registry but NOT limits to `parse_pipe_with_write`. That function on lines 711/725 calls `ParseLimits::snapshot()` again — re-reading env vars at a SECOND time, breaking F-HIGH-001 invariant (single snapshot per parse call).
- **Why fails:** F-HIGH-001 in BC-2.11.006 (pass-6 remediation) established that all security guards within a single parse call MUST use the same snapshotted ParseLimits. By re-snapshotting twice, this fix bundle reintroduces the original race window for `parse_with_registry`.
- **Fix:** Change `parse_pipe_with_write(input, registry)` → `parse_pipe_with_write(input, registry, limits: &ParseLimits)`. Replace lines 711, 725 with `parse_pipe_with_limits(base_input, limits)` / `parse_pipe_with_limits(input, limits)`. Update unit-test call sites.

### F-PR130-P1-MED-002 — `walk_sql_statement` silently skips SqlStatement::Dml variant
- **Confidence:** HIGH | **Category:** Correctness / Test coverage
- **Where:** `crates/prism-query/src/visit.rs:136-142`
- **What:** The visitor matches only `SqlStatement::Select(sq)` and falls through `_ => {}`. Comment says "S-3.06 will add Dml and Ddl variants here" — but S-3.06 added Dml and visitor was NOT updated. Visitors traversing AST via `walk_ast` → `walk_sql_statement` silently skip DML AST sub-nodes (assignments, filter predicates, source_select).
- **Why fails:** Fuzzers, AST-analyzers, future security tooling depending on visitor would not exercise DML inputs. Defense-in-depth analyses miss predicates inside `DmlNode.filter`.
- **Fix:** Add `fn visit_dml_node(&mut self, _node: &DmlNode)` default to Visitor trait. Add `walk_dml_node` helper visiting assignments, filter, source_select. Update `walk_sql_statement` arm. Add regression test.

### F-PR130-P1-MED-003 — `parse_with_registry` lacks DML/SELECT/empty-input test coverage
- **Confidence:** HIGH | **Category:** Test coverage
- **Where:** `crates/prism-query/src/tests/write_parser_unit_tests.rs:651-728`
- **What:** Only 3 tests for `parse_with_registry` (pipe-write, filter-reject, filter-clean). The function has 5 distinct branches; 3 untested: DML route, SELECT route, empty-input.
- **Why fails:** Without tests, future refactors could break SELECT/DML routing without signal. Combined with HIGH-001, the test gap allowed denylist defect to ship.
- **Fix:** Add 4 unit tests for DML route, SELECT route, empty input, and HIGH-001 denylist regression.

## Low Findings

### F-PR130-P1-LOW-001 — PR description Mermaid cites BC-2.11.010 (wrong) instead of BC-2.11.004
- **Where:** `.factory/code-delivery/S-3.06/pr-description.md:73-75`
- **Fix:** Replace `BC010[BC-2.11.010]` with `BC004[BC-2.11.004]` for AC-3/AC-7/AC-8 traceability.

### F-PR130-P1-LOW-002 — Pipe-stage-count limit (32) doesn't include optional terminal write stage
- **Where:** `pipe_parser.rs:545-728`
- **What:** 32 read stages + write stage = 33 effective stages. Write stage stripped before count. Spec ambiguity.
- **Fix:** Either update BC-2.11.004 v1.5 to clarify, OR include write stage in the count.

### F-PR130-P1-LOW-003 — Bare write verb path unreachable via public API (dead code)
- **Where:** `pipe_parser.rs:702-708`
- **Fix:** Document in code comment that path is reachable only via direct `parse_pipe_with_write` calls (test-only).

## Observations (incl. 2 process-gaps + 3 KUDOs)

- **OBS-1 [process-gap]:** `perimeter-symbols-sync` CI script silently ignores backtick tokens that don't normalize to known BC symbols. Tighten to FAIL when a backtick token in perimeter docstring doesn't normalize. This is what allowed HIGH-002 to ship.
- **OBS-2 [process-gap]:** Fix bundle introduced new `pub` API (`PrismQlParser::parse_with_registry`) but did NOT update `crates/prism-query/tests/api_surface.rs`. Per BC-2.11.006 §Enforcement layer 3, api_surface.rs should confirm "only sanctioned APIs are callable externally."
- **OBS-3:** TD-S306-001 deferral for `sensor_verbs` is correctly scoped. No action needed.
- **OBS-4:** `clippy::large_enum_variant` allow on SqlStatement follows Ast pattern. Acceptable.
- **KUDO OBS-5:** F-PR130-SEC-001 case-insensitive guard is clean — single `to_ascii_lowercase().starts_with("prism_")` with 4 dedicated regression tests.
- **KUDO OBS-6:** F-PR130-CR-003 cleanly added `columns: Option<Vec<String>>` to DmlNode with comprehensive tests.
- **KUDO OBS-7:** F-PR130-SEC-003 changed `DmlNode.filter` from sentinel `Bool(true)` to real `Option<Predicate>` with substantive regression tests.

## 7-Lens Verification Matrix

| Lens | Status |
|------|--------|
| 1. Fix-bundle validation | PASS with caveats (HIGH-001 + MED-001 are NEW regressions from the fix bundle) |
| 2. BC-2.11.004 v1.4 invariants | PASS |
| 3. BC-2.11.006 v1.13 perimeter integrity | **FAIL** (HIGH-002) |
| 4. AST evolution safety | PASS |
| 5. Cross-cutting concerns | PASS |
| 6. Story↔AC↔Test traceability | PASS |
| 7. Process-gap | 2 process-gaps flagged (OBS-1, OBS-2) |

## Summary

- **Verdict:** BLOCKED
- **Convergence:** 0/3 (window reset by HIGH findings)
- **Required fixes before pass-2:** HIGH-001 (denylist propagation), HIGH-002 (BC v1.14 + perimeter-violation update)
- **Recommended in same window:** MED-001/002/003, LOW-001/002/003, OBS-2 fix
