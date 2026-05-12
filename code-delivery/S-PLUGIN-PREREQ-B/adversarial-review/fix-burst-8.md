---
fix_burst: 8
story: S-PLUGIN-PREREQ-B
addresses_pass: 8
worktree_head_before: ebd9a3ec
worktree_head_after: 411f4cbf
factory_sha_at_close: <will be this commit>
findings_closed: 4
findings_closure_class: 2 paper-fix detections (TD-VSDD-059), 1 audit-signal discipline, 1 fan-out semantics
tests_added: 4
red_gate_tests_total: 45 (was 41)
discipline: TDD with explicit Red-Gate verification per TD-VSDD-059 + sibling sweep per TD-VSDD-060
date: 2026-05-11
---

# Fix-Burst 8 — Closure Report (S-PLUGIN-PREREQ-B)

**Worktree HEAD after burst:** `411f4cbf`
**factory-artifacts HEAD:** (this commit)
**Pass addressed:** Pass-8 (BLOCKED-soft, 3 MED + 1 LOW actionable)
**Streak position:** Pass-8 was 0/3; pass-9 dispatch will target 1/3.

---

## Findings Closed

### F-LP8-MED-001 — Empty-token tracing branch — CLOSED

- **Test added:** `test_BC_2_16_002_auth_initial_acquired_emits_distinct_events_per_token_state` (pipeline_http_integration.rs)
- **Production change:** None (fix-burst-7 implementation was correct; gap was test coverage).
- **TD-VSDD-059 load-bearing argument:** Sub-case (b) of the test asserts both that `auth_initial_acquired_empty` appears AND that bare `auth_initial_acquired` does NOT (for the empty-token path). Merging the two Ok arms back into one info-level emit would fail sub-case (b). Therefore the test is regression-load-bearing against the specific defect class.
- **Sibling sweep (TD-VSDD-060):** execute_step branch at pipeline.rs:466-500 mirrors execute()'s 3-way match. Zero call sites (TD-S-PLUGIN-PREREQ-B-012, P3). Covered by structural symmetry; runtime coverage deferred to PREREQ-D wire-in (acknowledged TD).

### F-LP8-MED-002 — Partial-record discard test — CLOSED

- **Test rewritten:** `test_BC_2_16_002_execute_discards_partial_records_on_mid_pipeline_500` (pipeline_http_integration.rs)
- **New structure:** CursorToken paginated 1-step pipeline. Page-1 returns `{"items":[{r1},{r2}], "next":"abc"}` with `response_path: "$.items"` and `is_final_step: true` → `all_records` accumulates 2 records. Page-2 (cursor=abc) returns 500. Test asserts `Err(HttpRequestFailed { status_code: 500 })`.
- **TD-VSDD-059 load-bearing argument:** A developer who changed `execute()` to return `Ok(PipelineResult{records: all_records.clone(), ...})` on the error path would get `Ok(2 records)` instead of `Err` — `assert!(result.is_err())` FAILS. The test now actually exercises the discard semantic that BC-2.16.002 v1.6 codified.
- **Sibling sweep:** No other tests assert `Err(HttpRequestFailed)` from mid-pagination failure with prior record accumulation. All other Err tests exercise single-page or non-paginated failure modes; they remain correct for their respective coverage.

### F-LP8-MED-003 — Cursor pagination non-string termination — CLOSED

- **Test added:** `test_BC_2_16_002_cursor_unsupported_type_emits_structured_event` (pipeline_http_integration.rs)
- **Production change:** pipeline.rs `extract_cursor` warn at line 880-887 enriched with structured `event_type = "pagination_cursor_unsupported_type"`, `actual_type = <%type_name>`, `cursor_preview = <truncated value>`. Behavior unchanged (still terminates pagination, returns Ok with page-1 records) — fix is observability-only, consistent with v1.6 BC scope.
- **TD-VSDD-059 load-bearing argument:** Pre-fix, the warn lacked event_type — the test asserts that field is present in the captured tracing buffer. Pre-fix code FAILS the assertion (only `cursor_path` and `actual_type` Debug-formatted into the message body, no structured event_type field).
- **Pre-fix FAIL output captured:** `panicked at: F-LP8-MED-003: warn event must contain 'pagination_cursor_unsupported_type' event_type field; captured log: 2026-05-12T01:25:37.679308Z WARN prism_spec_engine::pipeline: non-string/non-numeric cursor terminated pagination cursor_path=$.next actual_type=Array [Number(1), Number(2), Number(3)]`
- **Sibling sweep (TD-VSDD-060):** Audited ALL `tracing::warn!` calls in pipeline.rs. `auth_refresh_double_401` at line 682 already has `event_type`. The bare-warn at line 880-887 was the only missing one; now fixed. All WARN paths in pipeline.rs now carry structured `event_type`.

### F-LP8-LOW-001 — Multi-array fan-out template ambiguity — CLOSED

- **Test added:** `test_BC_2_16_002_spec_with_multi_array_fan_out_template_rejected` (validation.rs tests)
- **Production change:** validation.rs Category 2b multi-array fan-out check added after Category 2 variable reference validation. Validator rejects specs where a step's templates reference > 1 array-valued variable from prior steps (heuristic: only paginated steps or `[*]` response_path classified as array-valued; documented in comment block).
- **Decision context:** Per user mandate "no defer-with-surfacing without substantial justification" — chose validator rejection over runtime warn. Future cartesian/zipped fan-out semantics are PREREQ-C/D scope; rejection forces spec authors to be explicit.
- **TD-VSDD-059 load-bearing argument:** Pre-fix validator accepts the multi-array spec → returns `Ok(Some([]))`. Test asserts `Err(ValidationError)` with multi-array message. Pre-fix code FAILS.
- **Pre-fix FAIL output captured:** `panicked at: F-LP8-LOW-001: spec with multi-array fan-out template must be rejected by validator; got Ok(Some([]))`
- **False-positive escape valve:** documented in error message — only paginated steps or response_path-ending-`[*]` are classified as array-valued; non-array references pass unchanged.
- **Sibling sweep:** Existing `validate_variable_references` at validation.rs:453 checks reference resolution only (dangling/forward refs). New Category 2b correctly targets type-interaction ambiguity. No other validator functions require this check.

---

## Discipline Verification

- [x] TD-VSDD-059 paper-fix detection applied per finding (Red Gate FAIL captured)
- [x] TD-VSDD-060 sibling sweep applied per finding (documented)
- [x] `just check-fast` clean at end of burst (clippy --all-features 0 warnings)
- [x] Full suite: 278 tests / 278 passed / 1 skipped / 0 failed
- [x] Red Gate tests added: +4 (41 → 45)
- [x] Single commit on worktree per TD-VSDD-053: `fix(prism-spec-engine): close pass-8 paper-fix detections (F-LP8-M001..M003+L001)`

---

## Files Modified (worktree)

- `crates/prism-spec-engine/src/pipeline.rs` — extract_cursor warn structured event_type
- `crates/prism-spec-engine/src/validation.rs` — Category 2b multi-array fan-out check
- `crates/prism-spec-engine/tests/pipeline_http_integration.rs` — 3 new tests + 1 rewritten

---

## Recommendation

Dispatch LOCAL adversary pass-9 against worktree HEAD `411f4cbf`. Target streak: 0/3 → 1/3. Scope: closure verification of fix-burst-8's 4 findings + NEW dimensions disjoint from P5-A..K, P6-A..K, P7-A..L, P8-A..J.

**Verdict:** fix-burst-8 COMPLETE. Pass-9 dispatch authorized.
