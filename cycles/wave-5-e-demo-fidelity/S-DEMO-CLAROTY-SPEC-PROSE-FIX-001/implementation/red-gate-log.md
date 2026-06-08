# Red Gate Log — S-DEMO-CLAROTY-SPEC-PROSE-FIX-001

**Story:** S-DEMO-CLAROTY-SPEC-PROSE-FIX-001 — Claroty audit_log spec-prose + TOML-comment fidelity fix
**Phase:** 3 (TDD Implementation) — Red Gate Step
**Wave:** wave-5-e-demo-fidelity
**Date:** 2026-06-08
**Author:** test-writer
**Worktree:** .worktrees/S-DEMO-CLAROTY-SPEC-PROSE-FIX-001 (branch: feature/S-DEMO-CLAROTY-SPEC-PROSE-FIX-001, based on develop@5c5d240d)

---

## Red Gate Status: MIXED (1 RED, 2 GREEN-BY-DESIGN)

1 of 3 tests FAILS (genuine Red Gate). 2 tests pass pre-implementation (no-regression guards).
Workspace COMPILES. Red Gate discipline satisfied for AC-002 per BC-5.38.001.

---

## Nextest Output (pre-implementation)

```
Starting 3 tests across 7 binaries (179 tests skipped)
    PASS [   0.027s] prism-sensors::integration claroty_spec_prose_fidelity::test_BC_2_16_013_AC001_audit_logs_no_stale_dtu_gap_comments
    PASS [   0.030s] prism-sensors::integration claroty_spec_prose_fidelity::test_BC_2_16_013_AC004_audit_logs_functional_fields_unchanged
    FAIL [   0.027s] prism-sensors::integration claroty_spec_prose_fidelity::test_BC_2_16_013_AC002_audit_logs_gap_cl_006_closed_comment_present

Summary: 3 tests run: 2 passed, 1 failed, 179 skipped
```

**Failure message (AC-002):**
```
thread 'claroty_spec_prose_fidelity::test_BC_2_16_013_AC002_audit_logs_gap_cl_006_closed_comment_present'
panicked at crates/prism-sensors/tests/claroty_spec_prose_fidelity.rs:142:5:
audit_logs block must contain 'Gap-CL-006 CLOSED' comment; not found in:
[[tables]]
table_name = "audit_logs"
...
  [[tables.steps]]
  name = "fetch_audit_logs"
  method = "POST"
  path_template = "/api/v1/audit_log/get/"
  ...
```

---

## Test File

**`crates/prism-sensors/tests/claroty_spec_prose_fidelity.rs`** — new file, 3 tests.

Module declared in `crates/prism-sensors/tests/integration.rs`.

---

## Test Summary

| Test | AC | Result pre-impl | Why |
|------|----|-----------------|-----|
| `test_BC_2_16_013_AC001_audit_logs_no_stale_dtu_gap_comments` | AC-001 | GREEN-BY-DESIGN (no-regression guard) | Stale "DTU gap" comments were already removed by the `docs(S-DEMO-CLAROTY-AUDIT-DTU-001)` commit (9e4e17bf) — before this story was actioned. Test asserts continued absence. |
| `test_BC_2_16_013_AC002_audit_logs_gap_cl_006_closed_comment_present` | AC-002 | **RED (genuine Red Gate)** | `"Gap-CL-006 CLOSED"` comment line not present in the audit_logs block. This is the primary deliverable the implementer must add. |
| `test_BC_2_16_013_AC004_audit_logs_functional_fields_unchanged` | AC-004 | GREEN-BY-DESIGN (no-regression guard) | Parses the full TOML via `SpecLoader::parse`; asserts `path_template = "/api/v1/audit_log/get/"`, `method = "POST"`, `response_path = "$.audit_log"`, and all 5 columns intact. |

---

## AC-001 Red Gate Note

The story spec states "AC-001 MUST fail now (stale comments still present)." Investigation shows:

The stale "DTU gap" / "no /api/v1/audit_log/get route" / "404 until DTU route lands" comments
were already removed by commit `9e4e17bf` (`docs(S-DEMO-CLAROTY-AUDIT-DTU-001): correct stale
audit_log route-gap comments in claroty.sensor.toml (F-PR7-LOW-001)`), which predates this story.

The story was written under the assumption those comments were still present when it was actioned,
but they were cleaned up during the S-DEMO-CLAROTY-AUDIT-DTU-001 cascade before this branch
was cut. Therefore AC-001 is a no-regression guard (ensures they stay absent), not a failing
Red Gate. This is an expected divergence between story expectation and current state.

The genuine Red Gate (AC-002: `"Gap-CL-006 CLOSED"` absent) is intact and correctly RED.

---

## Implementer Instructions

To make all tests pass, add the following comment lines to the `audit_logs` table block
header in `crates/prism-sensors/specs/claroty.sensor.toml` (in the `# Table: audit_logs`
header section):

```toml
# Gap-CL-006 CLOSED by S-DEMO-CLAROTY-AUDIT-DTU-001.
# POST /api/v1/audit_log/get route registered in prism-dtu-claroty.
```

Do NOT change any functional TOML fields (path_template, method, response_path, columns).

After the fix:
- `test_BC_2_16_013_AC002_audit_logs_gap_cl_006_closed_comment_present` → PASS
- All 3 tests PASS
- Run `cargo nextest run -p prism-sensors -E 'test(claroty_spec_prose)'` to verify.
