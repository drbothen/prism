---
document_type: story
story_id: S-MCP-E003-SERIALIZATION-MIGRATION-001
title: "Migrate 18 serialization map_err sites in server.rs from PrismError::Internal to E-MCP-003 McpSerializationError"
wave: unscheduled
# Wave assignment: unscheduled — queue after T13/T14 demo waves are complete (post-demo P3 backlog).
epic_id: maintenance
priority: P3
# P3: post-demo refinement. Current behavior (E-INT-001 from PrismError::Internal) is spec-compliant
# catch-all per error-taxonomy v2.43. Migration surfaces the more specific E-MCP-003 code for
# MCP serialization failures so operators can distinguish serialization failures from generic
# internal errors in audit logs. Does not block any current demo or customer deliverable.
status: draft
# BC status: BC-2.10.007 v1.18 is ACTIVE and governs this story. S-7.01 gate satisfied.
# OBS-002 (pass-7): The McpSerializationError VariantMeta arm fix is DELIVERED BY PR #222
# (DEFECT-MCP-ROWSHAPE-NULLS-001). Verified 2026-07-13 in the worktree at
# .worktrees/DEFECT-MCP-ROWSHAPE-NULLS-001/crates/prism-mcp/src/error_mapping.rs:1737:
#   category: "internal", ec_code_override: Some("E-MCP-003"), suggestion verbatim, load-bearing
#   test test_BC_2_10_007_mcp_serialization_error_category_is_internal present.
# This story's scope is ONLY the 18 construction-site migrations in server.rs — the arm is
# a prerequisite delivered by PR #222, not work to be done here.
version: "0.8"
spec_version: "v0.8"
level: ops
producer: product-owner
timestamp: "2026-07-13"
modified: "2026-07-14"
input-hash: ""
inputs:
  - crates/prism-mcp/src/server.rs
  - crates/prism-mcp/src/error_mapping.rs
  - crates/prism-core/src/error.rs
  - .factory/specs/behavioral-contracts/BC-2.10.007-structured-error-responses.md
  - .factory/specs/prd-supplements/error-taxonomy.md
origin_finding: "DEFECT-MCP-ROWSHAPE-NULLS-001 pass-2: E-MCP-003 18-site migration surfaced; PENDING-HUMAN since D-1719"
origin_cascade: "DEFECT-MCP-ROWSHAPE-NULLS-001; D-1719 PENDING-HUMAN; D-1718 cascade checkpoint"
human_deferral: "2026-07-13 — human decision: 'Defer to follow-up story'; satisfies Canonical Principle Rule 3: explicit human direction + concrete dependency (post-demo queue) + story anchor (this file)"
cycle: "v1.0.0-greenfield"
phase: 3
tdd_mode: strict
track: "Platform Engineering"
subsystems: [SS-10]
# Subsystem anchor justification:
#   SS-10 (MCP Interface) owns this story's scope per ARCH-INDEX Subsystem Registry because
#   the 18 construction sites are in crates/prism-mcp/src/server.rs and the VariantMeta fix
#   is in crates/prism-mcp/src/error_mapping.rs — both files are entirely within the prism-mcp
#   crate which is SS-10's primary crate. No other subsystem boundary is crossed.
crates_touched:
  - prism-mcp
target_module: "crates/prism-mcp"
behavioral_contracts: [BC-2.10.007]
# BC status: BC-2.10.007 v1.18 is ACTIVE (lifecycle_status: active).
# BC-2.10.007 v1.18 §Implementer Code Follow-Up (OBS-002) is a REQUIRED implementer action
# that amends the McpSerializationError VariantMeta arm to:
#   category: "internal" (was "upstream_error")
#   suggestion: "Prism MCP serialization failure. Contact Prism operator; see audit log for details."
#   ec_code_override: Some("E-MCP-003") (was None)
# This story closes OBS-002 and migrates the 18 server.rs construction sites.
verification_properties: []
depends_on:
  - "PR #222 merge (DEFECT-MCP-ROWSHAPE-NULLS-001)"
  # Dependency anchor: PR #222 delivers the McpSerializationError VariantMeta arm fix (OBS-002)
  # and the load-bearing test test_BC_2_10_007_mcp_serialization_error_category_is_internal in
  # error_mapping.rs. This story's AC-001 verification and RGT-PRE only make sense post-merge.
  # The 18 construction-site sweep is technically independent of the arm (the variant already
  # compiled before OBS-002), but operationally this story should target the develop HEAD that
  # includes PR #222 so the full E-MCP-003 round-trip is testable end-to-end.
blocks: []
points: 2
# 2 points breakdown:
#   18 construction-site sweep in server.rs (mechanical but 18 sites): 1.5 pt
#   1 net-new static grep-gate test (RGT-001): 0.5 pt
#   VariantMeta arm fix: NOT in scope — delivered by PR #222.
estimated_days: 0.5
risk: P3
acceptance_criteria_count: 4
red_gate_tests: 2
estimated_passes: "1-2"
holdout_scenarios: []
assumption_validations: []
risk_mitigations: []
---

# S-MCP-E003-SERIALIZATION-MIGRATION-001: Migrate 18 serialization map_err sites from PrismError::Internal to E-MCP-003 McpSerializationError

## §Origin — DEFECT-MCP-ROWSHAPE-NULLS-001 pass-2 discovery; human deferral 2026-07-13

**Cascade:** DEFECT-MCP-ROWSHAPE-NULLS-001 (fix/DEFECT-MCP-ROWSHAPE-NULLS-001)
**Session record:** D-1719 (PENDING-HUMAN decision); D-1718 cascade checkpoint (2026-07-13)
**Human deferral:** 2026-07-13 — "Defer to follow-up story" (this story IS that follow-up)

Canonical Principle Rule 3 is satisfied:
- Explicit human direction: human decision "Defer to follow-up story" (2026-07-13)
- Concrete future dependency: post-demo P3 queue (T13/T14 demo must complete first)
- Story anchor: this file (S-MCP-E003-SERIALIZATION-MIGRATION-001)

### Background

`crates/prism-mcp/src/server.rs` contains 18 `map_err` closures that construct
`to_error_data(PrismError::Internal { detail: format!("...") })` when serde_json or
arrow_json serialization operations fail. These sites were discovered during the
DEFECT-MCP-ROWSHAPE-NULLS-001 pass-2 cascade as F-MCPNULL-P9-OBS-001.

`PrismError::McpSerializationError { detail: String }` already exists in
`prism-core/src/error.rs` (line 822) with `#[error("E-MCP-003: MCP response
serialization error: {detail}")]`.

**State of the VariantMeta arm (important — branch-vs-develop distinction):**

The `prism_error_to_structured_call_result` arm for `McpSerializationError` in
`crates/prism-mcp/src/error_mapping.rs` has TWO states depending on which branch you read:

- **develop branch (main checkout):** arm carries pre-v1.11 values (`category: "upstream_error"`,
  `ec_code_override: None`). This is the state story-writer initially read, producing an
  incorrect "notable discovery" claim that the arm still needed fixing.
- **DEFECT-MCP-ROWSHAPE-NULLS-001 worktree / PR #222:** arm is ALREADY FIXED per OBS-002
  (verified 2026-07-13 at error_mapping.rs:1737): `category: "internal"`,
  `ec_code_override: Some("E-MCP-003")`, suggestion verbatim, with load-bearing test
  `test_BC_2_10_007_mcp_serialization_error_category_is_internal` at line 4129.

**This story targets the develop HEAD AFTER PR #222 merges.** The arm fix is a prerequisite
delivered by PR #222, not scope for this story. This story's entire scope is the 18
construction-site sweep in server.rs.

### Verified site count and grep pattern

Grep pattern used to identify the 18 sites (run against the DEFECT-MCP-ROWSHAPE-NULLS-001
worktree at `/Users/jmagady/Dev/prism/.worktrees/DEFECT-MCP-ROWSHAPE-NULLS-001/crates/prism-mcp/src/server.rs`):

```python
# Reproducible count: PrismError::Internal construction within 11 lines of a
# serde_json:: or arrow_json:: operation
import re
with open("crates/prism-mcp/src/server.rs") as f:
    lines = f.readlines()
results = []
for i, line in enumerate(lines, 1):
    if "PrismError::Internal" in line:
        start = max(0, i - 11)
        window = "".join(lines[start:i])
        if "serde_json" in window or "arrow_json" in window:
            results.append(i)
print(f"Serialization PrismError::Internal sites: {len(results)}")
# Output: Serialization PrismError::Internal sites: 18
```

Equivalent shell grep (approximate — window-based approach more precise):
```bash
rg --line-number 'to_error_data\(PrismError::Internal' crates/prism-mcp/src/server.rs
# Then manually cross-reference with serde_json/arrow_json context in preceding ~10 lines.
```

The 18 sites span:
- Lines ~1958, 1964, 1973: `arrow_json::writer` serialization (WriterBuilder write/finish + serde_json::from_slice parse)
- Lines ~2035, 2179, 2309, 2387, 2499: `serde_json::to_value(&envelope)` envelope serialization
- Lines ~2566, 2582, 3063, 3604: `serde_json::to_value` result/envelope serialization
- Lines ~3961, 3978, 4058, 4144: `serde_json::to_value` envelope serialization
- Lines ~4326, 4433: `serde_json::to_value(&entry)` and final envelope serialization

(Line numbers are in the DEFECT-MCP-ROWSHAPE-NULLS-001 worktree. They will shift on develop
after the worktree merges — use the grep pattern above to re-identify sites.)

## Narrative

As a Prism operator inspecting audit logs after an MCP response serialization failure, I want the
structured error response to carry error code `E-MCP-003` (not the generic `E-INT-001`) so that I
can distinguish MCP layer serialization failures from other internal infrastructure failures without
reading stack traces or raw tracing output.

## Behavioral Contracts

| BC | Title | Version | Relevance |
|----|-------|---------|-----------|
| BC-2.10.007 | Structured Error Responses | v1.13 | Primary anchor. §OBS-002 (pass-7): McpSerializationError VariantMeta arm must carry `category: "internal"`, `ec_code_override: Some("E-MCP-003")`, `suggestion: "Prism MCP serialization failure..."`. Rule 1: `message = "Internal error"` (universal, no exception for McpSerializationError). |

## Acceptance Criteria

### AC-001 — Precondition: PR #222 VariantMeta arm verified as delivered
(traces to BC-2.10.007 v1.18 postcondition Rule 2 — dedicated VariantMeta arm class;
DELIVERED by PR #222 / DEFECT-MCP-ROWSHAPE-NULLS-001, NOT implementation work for this story)

At story start (after PR #222 merges to develop), the implementer verifies by reading
`crates/prism-mcp/src/error_mapping.rs` that `PrismError::McpSerializationError { .. }` arm
in `prism_error_to_structured_call_result` carries ALL of the following correct values:

| Field | Required Value (per BC-2.10.007 v1.18 §OBS-002) |
|-------|--------------------------------------------------|
| `category` | `"internal"` |
| `ec_code_override` | `Some("E-MCP-003")` |
| `suggestion` | `"Prism MCP serialization failure. Contact Prism operator; see audit log for details."` |
| `retryable` | `false` |

AND that `test_BC_2_10_007_mcp_serialization_error_category_is_internal` exists and passes
in `crates/prism-mcp/src/error_mapping.rs` test module.

If these values are NOT present (i.e., PR #222 has not merged), stop and wait for the merge.
Do NOT implement this story against a develop HEAD that still carries the stale arm values.

### AC-002 — All 18 construction sites migrated
(traces to BC-2.10.007 v1.18 postcondition — McpSerializationError variant produces E-MCP-003)

All 18 `to_error_data(PrismError::Internal { detail: format!("...") })` closures in
`crates/prism-mcp/src/server.rs` that wrap serde_json or arrow_json serialization operations
are replaced with `to_error_data(PrismError::McpSerializationError { detail: format!("...") })`.

The `detail` message string content for each site is preserved verbatim (e.g.,
`"Failed to serialize RecordBatch to JSON: {e}"`, `"Failed to serialize explain result: {e}"`,
`"Failed to serialize response: {e}"`, etc.) — only the variant name changes.

### AC-003 — Grep-gate: zero PrismError::Internal in serialization map_err context
(traces to BC-2.10.007 v1.18 invariant — McpSerializationError is the ONLY correct variant
for MCP layer serialization failures)

Running the Python script from §Origin (or its equivalent grep) against
`crates/prism-mcp/src/server.rs` after this story is implemented returns a count of **0**.
The grep pattern:

```bash
python3 -c "
with open('crates/prism-mcp/src/server.rs') as f:
    lines = f.readlines()
count = 0
for i, line in enumerate(lines, 1):
    if 'PrismError::Internal' in line:
        window = ''.join(lines[max(0,i-11):i])
        if 'serde_json' in window or 'arrow_json' in window:
            count += 1
print(f'Remaining PrismError::Internal serialization sites: {count}')
assert count == 0, f'Expected 0, found {count}'
"
```

### AC-004 — E-MCP-003 code + "internal" category + "Internal error" message verified by existing test
(traces to BC-2.10.007 v1.18 postcondition — McpSerializationError structured error fields)

`test_BC_2_10_007_mcp_serialization_error_category_is_internal` (delivered by PR #222, present
in `crates/prism-mcp/src/error_mapping.rs`) passes on the post-PR-#222 develop HEAD and continues
to pass after this story's construction-site sweep. The test verifies the full wire shape produced
by `prism_error_to_structured_call_result(PrismError::McpSerializationError { detail: ... })`:

| Field | Required Value |
|-------|---------------|
| `structuredContent.error.code` | `"E-MCP-003"` |
| `structuredContent.error.category` | `"internal"` |
| `structuredContent.error.message` | `"Internal error"` (Rule 1 — detail MUST NOT appear here) |
| `structuredContent.error.suggestion` | `"Prism MCP serialization failure. Contact Prism operator; see audit log for details."` |
| `structuredContent.error.retryable` | `false` |
| `structuredContent.error.upstream_message` | `null` (present-as-null, not absent) |
| `structuredContent.error.original_params_valid` | `true` |
| `isError` | `true` |

No new test is required for this AC — the construction-site sweep does not change the arm
logic; AC-004 is a regression non-regression assertion (the existing test must not break).

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| 18 construction-site sweep | `crates/prism-mcp/src/server.rs` | Pure (error path construction) |
| Net-new grep-gate test (RGT-001) | `crates/prism-mcp/src/error_mapping.rs` `#[cfg(test)] mod tests` | Pure (in-process static analysis via `include_str!`) |

Architecture section reference: `.factory/specs/architecture/api-surface.md` (SS-10 MCP Interface)

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `McpSerializationError` with empty `detail` string | Valid construction — empty detail is legal; BC-2.10.007 Rule 1 still applies (message = "Internal error") |
| EC-002 | `McpSerializationError::detail` contains characters that could be interpreted as injection payload | `detail` is placed in `detail` field of the variant but NEVER in `message` or `content[0].text` (DI-006); the structured error response does not expose `detail` directly |
| EC-003 | Post-migration: a future developer adds a new serialization site using `PrismError::Internal` | Not caught at compile time — the AC-003 grep-gate test in the test suite catches this at `just check` time |

## Red Gate Tests

| # | Test Name | Location | Status | Target AC | Fails Without |
|---|-----------|----------|--------|-----------|---------------|
| RGT-PRE | `test_BC_2_10_007_mcp_serialization_error_category_is_internal` | `crates/prism-mcp/src/error_mapping.rs` test module | **EXISTS — delivered by PR #222; NOT a new test** | AC-004 | VariantMeta arm regression (category, code, message, suggestion) |
| RGT-001 | `test_zero_PrismError_Internal_in_serde_context_server_rs` | `crates/prism-mcp/src/error_mapping.rs` test module | **NET-NEW** | AC-002, AC-003 | Any of the 18 sites NOT yet migrated |

**RGT-PRE** passes before and after this story (it tests the arm, not the construction sites).
It is listed here as a precondition sanity check; the implementer must confirm it passes on
the post-PR-#222 develop HEAD before beginning the sweep.

**RGT-001** (net-new) uses `include_str!` to read server.rs source at test runtime and asserts
zero `PrismError::Internal` constructions within 11 lines of a `serde_json::` or `arrow_json::`
call. It FAILS before the sweep (18 sites present) and PASSES after (0 sites):

```rust
#[test]
fn test_zero_PrismError_Internal_in_serde_context_server_rs() {
    // S-MCP-E003-SERIALIZATION-MIGRATION-001 AC-002/AC-003 grep-gate.
    // Fails if any serde_json/arrow_json map_err site still uses PrismError::Internal.
    let source = include_str!("server.rs");
    let lines: Vec<&str> = source.lines().collect();
    let mut count = 0usize;
    for (i, line) in lines.iter().enumerate() {
        if line.contains("PrismError::Internal") {
            let start = i.saturating_sub(11);
            let window = lines[start..=i].join("\n");
            if window.contains("serde_json") || window.contains("arrow_json") {
                count += 1;
            }
        }
    }
    assert_eq!(
        count, 0,
        "Found {count} PrismError::Internal construction(s) in serde/arrow serialization \
         context in server.rs; expected 0 after S-MCP-E003-SERIALIZATION-MIGRATION-001 sweep. \
         Migrate remaining sites to PrismError::McpSerializationError {{ detail: ... }}."
    );
}
```

**Red Gate execution:**
```bash
# RGT-PRE (pre-existing, must pass before sweep):
cargo nextest run -p prism-mcp -E 'test(BC_2_10_007_mcp_serialization_error_category_is_internal)'

# RGT-001 (net-new, fails before sweep, passes after):
cargo nextest run -p prism-mcp -E 'test(zero_PrismError_Internal_in_serde_context)'
```

## §Token Budget Estimate

| Source | Estimated Tokens |
|--------|-----------------|
| Story spec (this file) | ~5,000 |
| BC-2.10.007 v1.18 (postconditions, OBS-002 §Implementer Code Follow-Up) | ~8,000 |
| error-taxonomy.md E-MCP-003 row (v2.43) | ~2,000 |
| `crates/prism-mcp/src/server.rs` (18 site context windows, ~15 lines each) | ~5,000 |
| `crates/prism-mcp/src/error_mapping.rs` (McpSerializationError arm + VariantMeta structure + test module) | ~6,000 |
| `crates/prism-core/src/error.rs` (McpSerializationError variant definition) | ~500 |
| **Total** | **~26,500 tokens** |

Context window usage: ~26,500 / 200,000 = **~13%** (well within the 20-30% limit)

## §Tasks

**Step 1 — Verify PR #222 prerequisite (AC-001)**
- [ ] Confirm PR #222 (DEFECT-MCP-ROWSHAPE-NULLS-001) is merged to develop
- [ ] Read `crates/prism-mcp/src/error_mapping.rs` McpSerializationError VariantMeta arm; confirm `category: "internal"`, `ec_code_override: Some("E-MCP-003")`, suggestion verbatim
- [ ] Run `cargo nextest run -p prism-mcp -E 'test(BC_2_10_007_mcp_serialization_error_category_is_internal)'` — must PASS
- [ ] If any check fails, stop and wait for PR #222 merge; do NOT proceed

**Step 2 — Add RGT-001 (net-new grep-gate test)**
- [ ] Add `test_zero_PrismError_Internal_in_serde_context_server_rs` to `crates/prism-mcp/src/error_mapping.rs` test module (use `include_str!("server.rs")` as shown in Red Gate section)
- [ ] Run it: `cargo nextest run -p prism-mcp -E 'test(zero_PrismError_Internal_in_serde_context)'`
- [ ] Confirm it FAILS with count = 18 (Red Gate — sweep not done yet)

**Step 3 — Sweep 18 construction sites in server.rs (AC-002)**
- [ ] Run Python grep script from §Origin §"Verified site count" against `crates/prism-mcp/src/server.rs` to identify the 18 lines
- [ ] For each site: replace `PrismError::Internal { detail: ... }` with `PrismError::McpSerializationError { detail: ... }` (preserve `detail` string verbatim)
- [ ] Run `cargo nextest run -p prism-mcp` (no `--no-fail-fast`) to confirm compile + all tests pass

**Step 4 — Verify AC-003 grep-gate**
- [ ] Run `cargo nextest run -p prism-mcp -E 'test(zero_PrismError_Internal_in_serde_context)'` — must PASS with count = 0
- [ ] Run Python script from AC-003 directly to confirm count = 0 (belt-and-suspenders)

**Step 5 — Pre-push gate**
- [ ] `just check` (full workspace gate: fmt + clippy + nextest + doctests + crate-layout)

## §Previous Story Intelligence

N/A — this is the first story in the MCP error-code migration series. The pattern established
here (McpSerializationError for MCP-layer serialization failures) may inform future migrations
in resources.rs and prism_describe.rs (2 additional sites using `rmcp::model::ErrorData::internal_error()`
with hardcoded "E-MCP-500" prefix — those are out of scope for this story and are tracked
separately if needed).

The DEFECT-MCP-ROWSHAPE-NULLS-001 cascade (BC-5.39.001 3-CLEAN protocol) is the predecessor
context: this story was deferred from that cascade per human decision D-1719 on 2026-07-13.

## §Architecture Compliance Rules

Extracted from `crates/prism-mcp/src/error_mapping.rs` and CLAUDE.md:

1. **No new PrismError variants.** `McpSerializationError` already exists. Do NOT define a new
   variant — use the existing one in `prism-core/src/error.rs` (line 822 on develop post-PR-#222).

2. **Do NOT modify the VariantMeta arm.** The `PrismError::McpSerializationError { .. }` arm in
   `prism_error_to_structured_call_result` is owned by PR #222. This story must not touch it.
   If the arm appears wrong (stale values), PR #222 has not merged — stop and wait.

3. **Error taxonomy compliance.** The `detail` field content is an internal operator diagnostic
   string. It MUST NOT leak into `message`, `content[0].text`, or `suggestion` per DI-006.
   The `to_error_data` helper enforces this via `prism_error_to_structured_call_result`.

4. **No `map_prism_error` changes in scope.** The simpler `map_prism_error` function (line 24)
   produces `(i32, String)` tuples for a different code path. Its McpSerializationError arm
   is out of scope for this story (different function, different caller path). Do not change it.

5. **Forbidden dependency.** Do NOT add a dependency on any crate not already in prism-mcp's
   Cargo.toml. The migration is purely a construction-site change in existing code.

6. **`#[non_exhaustive]` discipline.** No new public types are added by this story; the existing
   `PrismError::McpSerializationError` struct carries no fields (only `detail: String` on the
   variant). No `#[non_exhaustive]` gate changes are needed.

## §Library & Framework Requirements

| Library | Version | Source of truth |
|---------|---------|-----------------|
| `prism-core` (workspace member) | workspace version | `Cargo.toml` workspace resolver |
| `rmcp` | `1.7` | ARCH-INDEX pinned version (ADR-041) |

No new external dependencies are introduced by this story.

## §File Structure Requirements

| File | Action | Change Description |
|------|--------|-------------------|
| `crates/prism-mcp/src/error_mapping.rs` | Modify | Add 1 net-new test function (`test_zero_PrismError_Internal_in_serde_context_server_rs`) to `#[cfg(test)] mod tests`; NO changes to production arm (arm is already correct per PR #222) |
| `crates/prism-mcp/src/server.rs` | Modify | Replace 18 `PrismError::Internal` constructions with `PrismError::McpSerializationError` |

No new files are created. No other files are touched.
The VariantMeta arm in `error_mapping.rs` must NOT be modified by this story — it is owned by PR #222.

## §Changelog

| Version | Burst | Date | Change |
|---------|-------|------|--------|
| v0.8 | DEFECT-MCP-ROWSHAPE-NULLS-001-FB22-F-MCPRS-PRL10-OBS-003 | 2026-07-14 | POL-23 pin refresh: BC-2.10.007 v1.17→v1.18 (F-MCPRS-PRL10-OBS-003 — §Rule 2 catch-all now FUTURE-ONLY; §Category table synced with 28 explicit-arm groups; no scope change to this story's OBS-002 McpSerializationError 18-site migration). 9 live pins updated. |
| v0.7 | DEFECT-MCP-ROWSHAPE-NULLS-001-FB20-OBS-002 | 2026-07-14 | POL-23 pin refresh: BC-2.10.007 v1.16→v1.17 (F-MCPRS-PRL8-OBS-002 snippet parity — `.as_u16()` removed; no semantic change to retryable rule; OBS-002 scope unaffected). 9 live pins updated. |
| v0.6 | DEFECT-MCP-ROWSHAPE-NULLS-001-FB18-RETRYABLE-503-RULE | 2026-07-14 | POL-23 pin refresh: BC-2.10.007 v1.15→v1.16 (§RETRYABLE-503 rule corrected from overbroad `!matches!(401\|403)` to transient-only `matches!(408\|425\|429\|500\|502\|503\|504)` — coordinator-raised finding; OBS-002 scope unaffected). 9 live pins updated. |
| v0.5 | DEFECT-MCP-ROWSHAPE-NULLS-001-FB18 | 2026-07-14 | POL-23 pin refresh: BC-2.10.007 v1.14→v1.15 (F-MCPRS-PRL6-MED-001 QueryDenylisted vector struct corrected + POL-29 sweep + RETRYABLE-503 adjudication; OBS-002 scope unaffected). |
| v0.4 | DEFECT-MCP-ROWSHAPE-NULLS-001-FB16 | 2026-07-13 | POL-23 pin refresh: BC-2.10.007 v1.13→v1.14 (§MED-001 safety-category arm added; §LOW-001 test vectors completed). No scope change — this story's OBS-002 scope (McpSerializationError 18-site migration) is unaffected by the safety arm addition. |
