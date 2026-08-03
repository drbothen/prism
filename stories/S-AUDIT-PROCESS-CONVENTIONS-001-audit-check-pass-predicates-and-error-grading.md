---
document_type: story
story_id: "S-AUDIT-PROCESS-CONVENTIONS-001"
title: "Audit script authoring conventions — PASS-grounding, error-grading checklist, and parse_envelope structural pattern"
wave: maintenance
epic_id: maintenance
priority: P2
status: draft
version: "0.2"
spec_version: "v0.2"
level: ops
producer: story-writer
timestamp: "2026-07-12"
modified: "2026-07-12"
input-hash: ""
inputs:
  - scripts/t13-preflight-audit.py
traces_to: "F-AUD-P1-OBS-002, F-AUD-P3-OBS-003, F-AUD-P21-OBS-003, F-AUD-P21-OBS-005"
origin_finding: "F-AUD-P1-OBS-002 [process-gap] + F-AUD-P3-OBS-003 [process-gap] + F-AUD-P21-OBS-003/OBS-005 [process-gap]"
origin_cascade: "AUDIT-COVERAGE-001 B-hardening; D-1694 (passes 1–20) + D-1695 (pass-21 PO adjudication); LOCAL 3-CLEAN converged D-1713 (2026-07-12)"
cycle: "v1.0.0-greenfield"
phase: 3
tdd_mode: strict
track: "Platform Engineering"
subsystems: []
crates_touched: []
target_module: "scripts/t13-preflight-audit.py"
behavioral_contracts: []
# BC status: pending PO authorship
# These three process-gap items govern audit-script quality conventions.
# No pre-existing BC covers "audit script PASS predicate grounding" or
# "error-layer discrimination in MCP audit checks".
# PO must author a BC covering audit script quality invariants before
# this story can advance to status: ready (S-7.01 gate).
verification_properties: []
depends_on: []
blocks: [S-AUDIT-INVARIANTS-001]
points: 3
estimated_days: 1.0
risk: LOW
acceptance_criteria_count: 4
red_gate_tests: 0
estimated_passes: "1-2"
holdout_scenarios: []
assumption_validations: []
risk_mitigations: []
---

# S-AUDIT-PROCESS-CONVENTIONS-001: Audit script authoring conventions — PASS-grounding, error-grading, and parse_envelope structural pattern

## §Origin — [process-gap] F-AUD-P1-OBS-002 + F-AUD-P3-OBS-003 + F-AUD-P21-OBS-003/OBS-005

**Cascade:** AUDIT-COVERAGE-001 B-hardening; findings surfaced across passes 1, 3, and 21
**Session records:** D-1694 (passes 1–20, items 1–2 queued), D-1695 (pass-21 PO adjudication, item 3 queued)
**Convergence:** LOCAL 3-CLEAN(strict) D-1713 (2026-07-12); S-7.02 codification gate now due

During the AUDIT-COVERAGE-001 44-pass cascade three recurring classes of audit-check defects were
repeatedly surfaced by adversary passes rather than being caught at authoring time:

1. **F-AUD-P1-OBS-002 — Over-permissive success predicates.** Audit check PASS predicates accepted
   empty results, zero-length lists, or trivially true conditions as evidence of compliance, producing
   PASS-on-nothing outcomes. The cascade enforced positively-grounded PASS predicates by hand in many
   individual fix-bursts (most recently B10 at pass 33). There is no written convention that authors
   can consult to avoid this class.

2. **F-AUD-P3-OBS-003 — Error-grading checklist absent.** Audit checks that probe error paths
   inconsistently distinguished among four observable error layers: (a) transport-level error (network /
   process exit), (b) structured `error_code` field in the MCP response JSON, (c) `sensor_errors` list
   in the response body, (d) empty `rows` with no error signal. Without a checklist, each author
   applies their own partial discrimination, producing gaps caught across many fix-bursts.

3. **F-AUD-P21-OBS-003/OBS-005 — parse_envelope structural pattern undocumented.** The cascade
   converged on a specific layered isinstance-guard convention for MCP envelope parsing:
   `envelope → content[0] → text → results → structuredContent.error`. This five-layer guard pattern
   prevents silent swallowing of parse failures and avoids the double-lookup anti-pattern (resolved at
   fix-burst 30 via `_res` single-bind). The pattern exists in the script but is not documented as a
   required convention — future authors have no reference.

This story creates `scripts/audit-conventions.md` codifying all three conventions and adds
docstring references to `t13-preflight-audit.py` pointing at it.

## Narrative

As a Prism developer authoring or reviewing audit check extensions to `scripts/t13-preflight-audit.py`,
I want written conventions documenting (a) positively-grounded PASS predicates, (b) the four-layer
error-discrimination checklist, and (c) the required `parse_envelope` structural pattern, so that new
checks avoid the recurrent defect classes found across the 44-pass AUDIT-COVERAGE-001 cascade without
requiring an adversarial review pass to catch them.

## Authority

No numbered ADR governs audit-script authoring conventions. The governing authorities for this story are:

**Origin findings:** F-AUD-P1-OBS-002, F-AUD-P3-OBS-003, F-AUD-P21-OBS-003/OBS-005 (AUDIT-COVERAGE-001 cascade, passes 1, 3, and 21) are the three process-gap findings that triggered this story. Session records D-1694 (passes 1–20) and D-1695 (pass-21 PO adjudication) contain the authoritative finding texts: over-permissive PASS predicates, absent four-layer error-grading checklist, and undocumented `parse_envelope` five-layer isinstance-guard pattern.

**CLAUDE.md §Standing Adversary Probes & Implementer Disciplines** (SAP-1, SAP-2, SAP-3, SID-1, SID-2) codifies the project-level adversary probes and implementer disciplines. The PASS-grounding convention (AC-001) and error-grading checklist (AC-002) this story documents are the audit-script equivalents of the rigor encoded in SID-1 (no-ignored-test rationalization prohibition) and SAP-1 (tracing emission catalog completeness).

**CLAUDE.md §Operational Discipline TDs — TD-VSDD-091:** The anti-volatile-pin rule applies to the docstring reference added by AC-004 — cite function names (`parse_envelope`, `sensor_errors_gate`), not line numbers. The `##parse_envelope-Pattern` section documents the canonical guard structure by function name, consistent with this rule.

No product BCs govern audit-script authoring quality. The `behavioral_contracts: []` status is intentional per the frontmatter note; PO authorship required before `status: ready` (S-7.01).

---

## Behavioral Contracts

No active BCs govern audit script authoring quality. This story creates the conventions document
from which a product-owner can author a BC. Until a BC is authored, `status` must remain `draft`
(S-7.01).

## Acceptance Criteria

### AC-001 — PASS-grounding convention codified in audit-conventions.md
(pending BC trace — BC authorship required before status=ready)

`scripts/audit-conventions.md` (new file) contains a §PASS-Grounding section stating:

> Every audit check's PASS predicate MUST assert a specific positive property. Acceptable:
> - `len(rows) >= 1` (at least one result row present)
> - `error_code == "E-QUERY-NNN"` (specific error code present)
> - `value == expected_value` (field matches contract)
> Not acceptable (PASS-on-nothing patterns, per F-AUD-P1-OBS-002):
> - PASS when result list is empty and no other assertion was made
> - PASS because an exception was not raised
> - PASS because a required call was not made (absence of evidence is not evidence of absence)
>
> Exception: explicitly documented negative checks (e.g., "this field MUST NOT appear") are
> acceptable when they are combined with at least one positive assertion in the same check.

The section cites F-AUD-P1-OBS-002 as the originating finding and references B10 as the most
recent enforcement example.

### AC-002 — Four-layer error-grading checklist codified in audit-conventions.md
(pending BC trace — BC authorship required before status=ready)

`scripts/audit-conventions.md` contains a §Error-Grading-Checklist section with the four-layer
discrimination table:

| Layer | What to check | How to detect |
|-------|--------------|---------------|
| L1 — Transport | Process exit / connection refused | `subprocess` non-zero exit; `_PrismCrashError` / `AUDIT_INTERNAL_ERROR` synthetic result |
| L2 — Structured error_code | JSON-RPC error code in `result.isError` / `content[0].text.error_code` | `parse_envelope(...)` returns body with `error_code` field |
| L3 — sensor_errors | Partial fan-out failures in sensor_errors list | `body.get("sensor_errors", [])` non-empty; checked AFTER confirming rows present |
| L4 — Empty rows | Successful response with zero result rows | `len(body.get("rows", [])) == 0` after L1–L3 clear |

An audit check probing a success path MUST assert L3 (sensor_errors is empty) in addition to
asserting L4 (rows present). A check that asserts only that rows are present but does not gate on
`sensor_errors` is incomplete per this checklist (F-AUD-P3-OBS-003).

### AC-003 — parse_envelope five-layer isinstance-guard convention codified in audit-conventions.md
(pending BC trace — BC authorship required before status=ready)

`scripts/audit-conventions.md` contains a §parse_envelope-Pattern section documenting the
required guard structure for any function that parses an MCP response envelope:

```python
# Required guard order (F-AUD-P21-OBS-003/OBS-005 convention):
def parse_envelope(resp):
    # Layer 1: envelope must be a dict
    if not isinstance(resp, dict):
        return None, f"envelope not a dict: {type(resp)}"
    # Layer 2: content[0] must exist
    content = resp.get("result", {}).get("content", [])
    if not content:
        return None, "no content in envelope"
    # Layer 3: text field must be a string (single bind to avoid double-lookup)
    _res = content[0]
    if not isinstance(_res, dict):
        return None, f"content[0] not a dict: {type(_res)}"
    text = _res.get("text", "")
    if not isinstance(text, str):
        return None, f"content[0].text not a string: {type(text)}"
    # Layer 4: results dict must be parseable
    try:
        results = json.loads(text)
    except json.JSONDecodeError as e:
        return None, f"failed to parse content[0].text as JSON: {e}"
    # Layer 5: optional structuredContent.error extraction
    if not isinstance(results, dict):
        return None, f"parsed result not a dict: {type(results)}"
    return results, None
```

Any deviation from this guard order (skipping layers, double-lookup via `resp['result']`,
catching exceptions silently) is a convention violation. Cite F-AUD-P21-OBS-003/OBS-005.

The section also documents the anti-pattern of double-lookup (resolved at fix-burst 30):
`resp['result']['content']` called twice — the first successful lookup does not prevent the second
from raising `KeyError` in a different code path; the `_res` single-bind pattern is mandatory.

### AC-004 — Docstring reference added to t13-preflight-audit.py module header
(pending BC trace — BC authorship required before status=ready)

The module-level docstring (or a `# CONVENTIONS` comment block) at the top of
`scripts/t13-preflight-audit.py` gains a reference:

```python
# Authoring conventions: see scripts/audit-conventions.md
# - §PASS-Grounding: every PASS must assert a specific positive property
# - §Error-Grading-Checklist: L1 transport → L2 error_code → L3 sensor_errors → L4 empty rows
# - §parse_envelope-Pattern: five-layer isinstance-guard convention
```

This reference must appear within the first 30 lines of the file so authors encounter it before
reading any check implementations.

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| audit-conventions.md (new) | `scripts/audit-conventions.md` | Pure data (convention document) |
| t13-preflight-audit.py header | `scripts/t13-preflight-audit.py` | Pure (docstring/comment addition) |

Architecture section references:
- N/A — this story touches no Rust crates; it operates entirely within the `scripts/` directory.

**Anchor justifications:**
- No subsystem anchor: `scripts/` tooling is not assigned to a Subsystem in the ARCH-INDEX Subsystem
  Registry. This story is maintenance-track, not product-track.
- No `depends_on` dependencies: the convention document is self-contained and requires no
  pre-existing story to complete.
- `blocks: [S-AUDIT-INVARIANTS-001]`: S-AUDIT-INVARIANTS-001 extracts invariant contracts from the
  script body; having the three conventions document in place first reduces the volume of inline
  citations that need thinning in that story.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | A new check legitimately does not call parse_envelope (e.g., a crash-detection check) | Convention still applies to any function that handles MCP JSON responses; crash-detection checks that only test process exit are exempt from §parse_envelope-Pattern |
| EC-002 | An existing check uses a simplified envelope parser for historical reasons | Convention document notes that all new checks must use the canonical pattern; existing checks are migrated opportunistically (full migration is tracked in S-AUDIT-INVARIANTS-001) |
| EC-003 | The four-layer checklist order does not apply to a negative probe (e.g., "assert that request X returns an error") | §Error-Grading-Checklist notes that negative probes must still check L1 (no crash) and positively assert the expected L2/L3/L4 layer |

## Token Budget Estimate

| Item | Lines | Tokens (est.) |
|------|-------|--------------|
| Story spec (this file) | ~130 | ~1,800 |
| scripts/t13-preflight-audit.py (header + conventions anchor area) | ~50 | ~700 |
| Existing check implementations (reference for convention examples) | ~100 | ~1,400 |
| **Total estimate** | | **~3,900 tokens** |

Fits well within a 100k-token agent context window (<4%). No split required.

## Tasks

- [ ] Review F-AUD-P1-OBS-002 finding description in D-1694 and collect 3 representative PASS-on-nothing examples from the script.
- [ ] Review F-AUD-P3-OBS-003 finding description in D-1694 and confirm the four-layer discrimination table is complete.
- [ ] Review F-AUD-P21-OBS-003/OBS-005 finding descriptions in D-1695 and extract the canonical parse_envelope pattern from the script.
- [ ] Create `scripts/audit-conventions.md` with three sections: §PASS-Grounding, §Error-Grading-Checklist, §parse_envelope-Pattern (AC-001, AC-002, AC-003).
- [ ] Add `# CONVENTIONS` reference block to the module header of `scripts/t13-preflight-audit.py` (AC-004).
- [ ] Verify no Rust build is required (this story is scripts-only; run `py_compile scripts/t13-preflight-audit.py` to confirm no syntax introduced).

## Previous Story Intelligence

N/A — first story in the AUDIT-COVERAGE-001 S-7.02 codification sub-track. Prior context:
- The three conventions were enforced by hand throughout the 44-pass AUDIT-COVERAGE-001 cascade
  (F-AUD-P1-OBS-002: pass 1, multiple fix-bursts through pass 33 B10; F-AUD-P3-OBS-003: pass 3,
  enforced in fix-bursts 3, 30+; F-AUD-P21-OBS-003/OBS-005: pass 21, parse_envelope
  double-lookup fixed fix-burst 30, _res single-bind fix-burst 31).
- `scripts/t13-preflight-audit.py` already uses the canonical `parse_envelope` pattern in its latest
  form (HEAD acf7ded0); the convention document records this as the required pattern.
- `scripts/audit-conventions.md` does not yet exist at HEAD acf7ded0.

## Architecture Compliance Rules

- **TD-VSDD-091:** Cite function names (`parse_envelope`, `sensor_errors_gate`), NOT file/line numbers.
- **No `println!` rule:** N/A — this story is Python-only.
- **py_compile gate:** After adding docstring to `t13-preflight-audit.py`, run `py_compile scripts/t13-preflight-audit.py` to confirm syntactic validity.
- **ruff gate (post-S-AUDIT-LINT-001):** Once S-AUDIT-LINT-001 adds ruff to the Justfile, this file must pass `ruff check --select F821 scripts/`. This story may be implemented before or after the lint gate; either order is valid.

## Library & Framework Requirements

| Library | Version | Notes |
|---------|---------|-------|
| Python | 3.x (workspace standard) | `py_compile` validates no syntax errors |

No new Python dependencies. `scripts/audit-conventions.md` requires only a text editor.

**Forbidden dependencies:** None applicable.

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `scripts/audit-conventions.md` | Create (new) | Three-section conventions doc (AC-001/002/003) |
| `scripts/t13-preflight-audit.py` | Modify | Add `# CONVENTIONS` reference block in module header (AC-004) |

No Rust files modified. No Cargo.toml changes. No new crates.

## Changelog

| Version | Burst | Date | Author | Changes |
|---------|-------|------|--------|---------|
| 0.2 | DRIFT-STORY-AUTHORITY-ABSENT-CORPUS-001-R6 | 2026-08-02 | story-writer | Add §Authority section (D-2084 Round 6 DRIFT-STORY-AUTHORITY-ABSENT-CORPUS-001). No numbered ADR governs; authority is origin findings F-AUD-P1-OBS-002/F-AUD-P3-OBS-003/F-AUD-P21-OBS-003/OBS-005, CLAUDE.md §Standing Adversary Probes, and CLAUDE.md §TD-VSDD-091. |
| 0.1 | — | 2026-07-12 | story-writer | Initial story creation. |
