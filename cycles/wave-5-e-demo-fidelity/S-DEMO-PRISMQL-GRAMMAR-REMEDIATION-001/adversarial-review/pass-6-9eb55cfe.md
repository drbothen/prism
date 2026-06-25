---
document_type: adversarial-review-pass
pass: 6
story: S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001
frozen_head: 9eb55cfe
story_version: v1.6
clean_strict: false
clean_pr_merge: false
finding_count: 1
findings_open: 0
findings_closed: 1
cascade_streak_after: 0
reviewer: vsdd-factory:adversary
timestamp: 2026-06-25T00:00:00Z
---

# LOCAL Adversary Pass 6 — S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001

**Frozen HEAD:** `9eb55cfe`
**Story version read:** v1.6
**Diff reviewed:** `903c8fcb..9eb55cfe`
**CLEAN(strict):** NO
**CLEAN(PR-merge):** NO
**Finding count:** 1 (MED-1 — CLOSED)

---

## Findings

### MED-1 (MEDIUM) — AC-023 IS-NOT-NULL-on-JSON-list semantics note absent from `build_reference_content`

**Status: CLOSED** (implementer fix-burst; code HEAD advanced `9eb55cfe → 64d91111`)

**Location:** `crates/prism-mcp/src/resources.rs` — `build_reference_content` function

**Finding:**

AC-023 (GRAMMAR-006 in the story's AC table, traced to BC-2.11.022) mandates that
`build_reference_content` include a verbatim semantics note explaining IS-NOT-NULL
behavior for JSON-list columns:

> "A JSON array column that is non-empty (including `[]`) materializes as a non-null
> UTF-8 string. `IS NOT NULL` returns `true` for `[]`. Only a JSON `null` value
> maps to Arrow null."

At frozen HEAD `9eb55cfe`, this note was absent from `build_reference_content`.
Additionally, no test asserted the note's presence or the described behavior.

AC-023 acceptance criterion was marked as testable with a load-bearing test;
the absence of the test left the requirement unverifiable.

**Root cause:** The v1.5→v1.6 story edit (TD-VSDD-091 anti-volatile-pin) was a
spec-hygiene pass; it did not add the missing note. The AC-023 implementation gap
predated the v1.6 edit.

**Fix (code HEAD `9eb55cfe → 64d91111`):**

1. `build_reference_content` in `resources.rs` — verbatim semantics note added
   per AC-023 spec.
2. Test `test_bc_2_11_022_ac023_json_list_is_not_null_note` added — non-`#[ignore]`,
   asserts note text present in `build_reference_content` output.
3. Verified actual behavior in `spec_driven_adapter.rs`: `column_type_to_arrow`
   maps `ColumnType::Json` → Arrow `Utf8`; `build_column_array` materializes JSON
   values as Utf8 strings; empty list `[]` → non-null Utf8 → IS NOT NULL = true;
   JSON `null` → Arrow null → IS NOT NULL = false. Note accurately describes
   production behavior.

**`just check` on `9eb55cfe` EXIT=0 confirmed** (pre-pass gate; prior HEADs all EXIT=0).

---

## OBS-1 (not a finding — proactive correction)

**Status: CLOSED by story-writer proactively** (story v1.5→v1.6 TD-VSDD-091 correction)

Story spec contained five stale `engine.rs` file-location hints describing
FORBID-BOTH and NOW()-injection wiring. These violated TD-VSDD-091 (anti-volatile-pin
— narrative must use function-name anchors, not `file.rs:NNN` line numbers).

Corrected in story v1.6 before this pass was gated:
- `inject_now` → `lib.rs`
- `plan_sqlpipe_query` → `lib.rs`
- `execute_against_session` Ast::SqlPipe arm → `materialization.rs`

Not flagged as an adversary finding because the correction was already in place
at story v1.6 when this pass was conducted. Recorded here for audit completeness.

---

## Directed Checks (All PASS except MED-1)

- SAP-1 (tracing catalog): PASS (filter.* + pipe.* rows present; no new emission sites)
- SAP-2 (DTU↔TOML parity): N/A (no TOML/DTU changes in diff)
- SID-1 (no-ignored-test rationalization): PASS
- BC-2.11.023 D1 + D2 mode-bridge: PASS (verified in Pass 5; no regression in diff)
- Anti-tautology CI gate: PASS (EXPECTED=83 unchanged)
- Temporal NOW()/INTERVAL, FORBID-BOTH, filter-mode: PASS (carry-forward from Pass 5)

---

## Cascade State After This Pass

**3-CLEAN streak RESET to 0/3** — MED-1 fix required a code commit
(`9eb55cfe → 64d91111`), advancing the frozen HEAD. Per
DRIFT-ORCH-PRLEVEL-PUSH-001, the streak restarts against the new HEAD `64d91111`.

**NEXT:** confirm `just check` EXIT=0 on `64d91111` → LOCAL adversary
cascade Pass 1/2/3 on UNCHANGED `64d91111` (need 3 consecutive CLEAN(strict)).

---

## Severity Trend Across Cascade

| Pass | HEAD | Finding Count | Worst Severity | Notes |
|------|------|---------------|----------------|-------|
| 1 (e518d96c) | e518d96c | 5 | CRIT | All closed D-1338 |
| 2 (f03679b2) | f03679b2 | 1 | MED | MED-1 BC-pin drift — closed D-1339 |
| 3 (f03679b2) | f03679b2 | 3 | HIGH | 1H+1M+1L — all closed D-1340 |
| 4 (81372a22) | 81372a22 | 4 | HIGH | 2H+2M — all closed D-1341 |
| 5 (9eb55cfe) | 9eb55cfe | 0 | — | CLEAN(strict) — streak 1/3 |
| **6 (9eb55cfe)** | **9eb55cfe** | **1** | **MED** | **MED-1 AC-023 note absent — CLOSED 64d91111** |

Severity trend: 2 CRIT → 3 HIGH → 2 HIGH → 1 MED → CONVERGING.
