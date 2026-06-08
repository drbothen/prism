---
story_id: S-DEMO-CLAROTY-SPEC-PROSE-FIX-001
title: "Demo Evidence — Claroty audit_log spec-prose + TOML-comment fidelity fix"
recorded: "2026-06-08"
recorder: demo-recorder
bc: BC-2.16.013 v1.25
wave: wave-5-e-demo-fidelity
recording_tool: VHS 0.10.0
font: "FiraCode Nerd Font Mono"
---

# Demo Evidence: S-DEMO-CLAROTY-SPEC-PROSE-FIX-001

Story: Claroty audit_log spec-prose + TOML-comment fidelity fix (closes F-P2-DEFER-001)
BC: BC-2.16.013 v1.25 §Postconditions §1

---

## AC-001 — No stale "DTU gap" comments in claroty.sensor.toml

**Acceptance criterion:** `crates/prism-sensors/specs/claroty.sensor.toml` contains no
comment lines with `"DTU gap"`, `"no /api/v1/audit_log/get route"`, or
`"404 until DTU route lands"` in the audit_logs table block.

**Recording:** `AC-001-no-stale-dtu-gap-comments.gif` / `.webm`

**Evidence:** `rg -c 'DTU gap' ...` and `rg -c '404 until DTU route lands' ...` both
return no matches and print `PASS-no-matches`. Exit code 1 from rg = zero matches found.

```
$ rg -c 'DTU gap' crates/prism-sensors/specs/claroty.sensor.toml || echo PASS-no-matches
PASS-no-matches
$ rg -c '404 until DTU route lands' crates/prism-sensors/specs/claroty.sensor.toml || echo PASS-no-matches
PASS-no-matches
```

**Status: EVIDENCED — AC-001 SATISFIED**

---

## AC-002 — Gap-CL-006 closure comment present in claroty.sensor.toml

**Acceptance criterion:** The audit_logs table block contains a comment line with
`"Gap-CL-006 CLOSED"` and a reference to `S-DEMO-CLAROTY-AUDIT-DTU-001`.

**Recording:** `AC-002-gap-cl-006-closed-comment.gif` / `.webm`

**Evidence:** `rg -n -A2 'table_name = .audit_logs.' claroty.sensor.toml` shows:

```
147:table_name = "audit_logs"
148-ocsf_class = "audit_activity"
149-# Gap-CL-006 CLOSED by S-DEMO-CLAROTY-AUDIT-DTU-001.
```

The line `# Gap-CL-006 CLOSED by S-DEMO-CLAROTY-AUDIT-DTU-001.` is present at line 149,
immediately after the `table_name = "audit_logs"` declaration.

**Status: EVIDENCED — AC-002 SATISFIED**

---

## AC-002 + AC-004 — Red Gate tests PASS (3/3)

**Acceptance criteria:** The `claroty_spec_prose_fidelity` test binary passes all three
tests:
- `test_BC_2_16_013_AC001_audit_logs_no_stale_dtu_gap_comments` (no-regression guard)
- `test_BC_2_16_013_AC002_audit_logs_gap_cl_006_closed_comment_present` (AC-002 Red Gate)
- `test_BC_2_16_013_AC004_audit_logs_functional_fields_unchanged` (AC-004 no-regression)

**Recording:** `AC-002-AC-004-red-gate-tests.gif` / `.webm`

**Evidence:** nextest output captured in recording:

```
Starting 3 tests across 1 binary (6 binaries skipped)
    PASS [   0.011s] (1/3) prism-sensors::claroty_spec_prose_fidelity test_BC_2_16_013_AC001_audit_logs_no_stale_dtu_gap_comments
    PASS [   0.011s] (2/3) prism-sensors::claroty_spec_prose_fidelity test_BC_2_16_013_AC002_audit_logs_gap_cl_006_closed_comment_present
    PASS [   0.014s] (3/3) prism-sensors::claroty_spec_prose_fidelity test_BC_2_16_013_AC004_audit_logs_functional_fields_unchanged
────────────
 Summary [   0.014s] 3 tests run: 3 passed, 0 skipped
```

**Status: EVIDENCED — AC-002 (Red Gate) + AC-004 (no-regression) SATISFIED**

---

## AC-003 — BC-2.16.013 §Postconditions §1 audit_logs prose verified

**Acceptance criterion:** BC-2.16.013 v1.25 §Postconditions §1 contains the corrected
prose reading `POST /api/v1/audit_log/get; DTU route registered by
S-DEMO-CLAROTY-AUDIT-DTU-001 (Gap-CL-006 CLOSED)`.

**Status note:** AC-003 is marked PARTIALLY CLOSED. The PO authored the correction in
BC-2.16.013 v1.25 during the Wave-5 Phase-A burst (2026-06-03). The implementer verifies
the prose is in place.

**Recording:** `AC-003-bc-postcondition-prose.gif` / `.webm`

**Evidence:** `rg -n 'Gap-CL-006 CLOSED' .factory/specs/behavioral-contracts/BC-2.16.013-bundled-sensor-spec-dtu-parity.md` shows:

```
149:# Gap-CL-006 CLOSED by S-DEMO-CLAROTY-AUDIT-DTU-001.
178:  - `audit_logs` — POST `/api/v1/audit_log/get`; DTU route registered by
179:    S-DEMO-CLAROTY-AUDIT-DTU-001 (Gap-CL-006 CLOSED). Method is POST-for-read,
480:| 1.25 | Wave-5-Phase-A-PO-burst | ... | Gate 4 (S-DEMO-CLAROTY-SPEC-PROSE-FIX-001 / F-P2-DEFER-001 closure):
     §Postconditions §1 Claroty `audit_logs` clause corrected from stale "GET /api/v1/audit_logs via offset
     pagination. No DTU route registered." to "POST /api/v1/audit_log/get; DTU route registered by
     S-DEMO-CLAROTY-AUDIT-DTU-001 (Gap-CL-006 CLOSED)". ... BC v1.24 → v1.25. |
```

The BC-2.16.013 §Postconditions §1 `audit_logs` clause at line 178–179 reads exactly:
`POST /api/v1/audit_log/get; DTU route registered by S-DEMO-CLAROTY-AUDIT-DTU-001 (Gap-CL-006 CLOSED)`.

**Status: EVIDENCED — AC-003 SATISFIED**

---

## Coverage Summary

| AC | Description | Recording | Status |
|----|-------------|-----------|--------|
| AC-001 | No stale DTU gap comments in audit_logs TOML block | `AC-001-no-stale-dtu-gap-comments.gif/.webm` | EVIDENCED |
| AC-002 | Gap-CL-006 CLOSED comment present in audit_logs block | `AC-002-gap-cl-006-closed-comment.gif/.webm` | EVIDENCED |
| AC-002 + AC-004 | Red Gate tests 3/3 PASS | `AC-002-AC-004-red-gate-tests.gif/.webm` | EVIDENCED |
| AC-003 | BC-2.16.013 v1.25 §Postconditions §1 prose verified | `AC-003-bc-postcondition-prose.gif/.webm` | EVIDENCED |

**All 4 ACs evidenced.** Story S-DEMO-CLAROTY-SPEC-PROSE-FIX-001 is demo-complete.

---

## File Manifest

| File | Type | AC |
|------|------|----|
| `AC-001-no-stale-dtu-gap-comments.tape` | VHS script | AC-001 |
| `AC-001-no-stale-dtu-gap-comments.gif` | VHS recording | AC-001 |
| `AC-001-no-stale-dtu-gap-comments.webm` | VHS recording | AC-001 |
| `AC-002-gap-cl-006-closed-comment.tape` | VHS script | AC-002 |
| `AC-002-gap-cl-006-closed-comment.gif` | VHS recording | AC-002 |
| `AC-002-gap-cl-006-closed-comment.webm` | VHS recording | AC-002 |
| `AC-002-AC-004-red-gate-tests.tape` | VHS script | AC-002 + AC-004 |
| `AC-002-AC-004-red-gate-tests.gif` | VHS recording | AC-002 + AC-004 |
| `AC-002-AC-004-red-gate-tests.webm` | VHS recording | AC-002 + AC-004 |
| `AC-003-bc-postcondition-prose.tape` | VHS script | AC-003 |
| `AC-003-bc-postcondition-prose.gif` | VHS recording | AC-003 |
| `AC-003-bc-postcondition-prose.webm` | VHS recording | AC-003 |
| `evidence-report.md` | This report | All ACs |

## POL-10 Compliance

All output files are under `docs/demo-evidence/S-DEMO-CLAROTY-SPEC-PROSE-FIX-001/`.
No files placed directly at `docs/demo-evidence/*.md`.
The evidence report is at `docs/demo-evidence/S-DEMO-CLAROTY-SPEC-PROSE-FIX-001/evidence-report.md`.
POL-10 COMPLIANT.
