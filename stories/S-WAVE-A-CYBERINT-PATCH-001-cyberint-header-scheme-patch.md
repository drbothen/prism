---
document_type: story
story_id: S-WAVE-A-CYBERINT-PATCH-001
title: "Cyberint header_scheme Boot-Failure Patch — Add header_scheme = \"cookie:access_token\" to cyberint.sensor.toml"
version: "1.2"
status: draft
producer: story-writer
phase: 3
wave: wave-a
epic_id: E-WAVE-A-SENSOR-REMEDIATION
priority: P0
points: 1
tdd_mode: strict
target_module: prism-sensors
subsystems: ["SS-06 (SensorSpec)"]
depends_on: []
blocks:
  - S-WAVE-A-ENGINE-001    # ENGINE-001 MUST NOT merge without this story landing simultaneously.
                           # Rationale: see MERGE-GATE-ENGINE-001 section below.
behavioral_contracts:
  - BC-2.16.009
verification_properties: []
estimated_days: 0.5
assumption_validations: []
risk_mitigations: []
---

# S-WAVE-A-CYBERINT-PATCH-001: Cyberint header_scheme Boot-Failure Patch

## MERGE-GATE-ENGINE-001

**This story MUST merge in the same release batch as `S-WAVE-A-ENGINE-001`.**

After `S-WAVE-A-ENGINE-001` merges, Rule 9 is live inside `SpecLoader::parse()` —
the unconditional call point on every path that loads a sensor spec
(BC-2.16.009 §Integration function). Rule 9 fires on `auth_type = "cookie_roundtrip"` +
absent `header_scheme` → E-SPEC-027(c). The current `cyberint.sensor.toml` has exactly
this shape: `auth_type = "cookie_roundtrip"` with no `header_scheme` field. Because
`SpecLoader::parse()` is invoked unconditionally by `load_all()` at boot,
**every prism startup fails at spec load time with exit code 2** until this patch lands.

The fix is a one-line addition. The co-landing merge must remain small and reviewable.
Do not add this story's merge gate to `S-WAVE-A-CYBERINT-SPEC-001` (the full
dual-surface migration) — the large migration is independent ADR-053 work and must NOT
be forced into the atomic merge.

---

## Narrative

As a Prism maintainer, I want to add `header_scheme = "cookie:access_token"` to
`cyberint.sensor.toml` — the single one-line change required for Rule 9 compliance —
so that prism boots successfully after `S-WAVE-A-ENGINE-001` ships Rule 9 validation,
without requiring the full 8-point dual-surface migration to land at the same time.

---

## Why One Line is Sufficient

Rule 9 fires on exactly one condition: `auth_type = "cookie_roundtrip"` AND `header_scheme`
is absent. The existing `cyberint.sensor.toml` is otherwise valid:
- `version = "1.0.0"` — passes Rule 1 semver check
- `base_url = "https://${env.CYBERINT_ENVIRONMENT}.cyberint.io"` — resolves to an HTTPS
  URL after env-var expansion, passes Rule 1 scheme check
- All tables have at least one column and one step
- No dangling variable references

Adding `header_scheme = "cookie:access_token"` satisfies Rule 9 path (a) entirely. No
other field changes are required for boot compliance. The C2-class OpenAPI grounding fixes
(POST method, `$.alerts` path, page/size pagination, dual-surface split, credential rename)
are accuracy improvements deferred to `S-WAVE-A-CYBERINT-SPEC-001`.

---

## Acceptance Criteria

### AC-001: cyberint.sensor.toml declares header_scheme = "cookie:access_token"
(traces to BC-2.16.009 Rule 9 postcondition — `cookie_roundtrip` + `header_scheme` present
and valid → E-SPEC-027(c) does NOT fire)

`crates/prism-sensors/specs/cyberint.sensor.toml` contains the line:
```
header_scheme = "cookie:access_token"
```

The `header_scheme` value `"cookie:access_token"` matches the Rule 9 pattern for
`cookie_roundtrip`: `cookie:<name>` where `<name>` is the cookie key containing the
access token. Value `"cookie:access_token"` is correct — the DTU validates the
`access_token` cookie on every request.

### AC-002: Bundled spec load test passes with updated cyberint spec
(traces to BC-2.16.001 postcondition — all bundled specs load without error at startup)

The bundled spec load test (`tests/bc_2_16_001_bundled_spec_load.rs` in `prism-spec-engine`)
passes with the updated `cyberint.sensor.toml`. No E-SPEC-027(c) error is produced for the
Cyberint spec.

This test exercises the full load path (TOML parse → env-var resolution → Rule 7 → Rule 9
via `SpecLoader::parse()`) and confirms the patch is sufficient.

### AC-003: No other fields in cyberint.sensor.toml are changed
(traces to AC-007 of S-WAVE-A-CYBERINT-SPEC-001 — this patch is the MINIMAL change; all
C2-class fixes belong to the full migration story)

A diff of `cyberint.sensor.toml` shows exactly one addition: the `header_scheme` line.
No method changes (GET→POST), no response_path changes (`$.data` remains), no pagination
changes (cursor_token remains), no credential_refs changes (`api_key` name remains), no
table additions or deletions. The `incidents` table remains present (its removal is
S-WAVE-A-CYBERINT-SPEC-001 scope).

---

## Architecture Mapping

| Component | File | Pure/Effectful | Change |
|-----------|------|---------------|--------|
| `cyberint.sensor.toml` | `crates/prism-sensors/specs/` | Pure (config data) | Add `header_scheme = "cookie:access_token"` |

---

## Behavioral Contracts

| BC | Version | Relevance |
|----|---------|-----------|
| BC-2.16.009 | v1.28 | Rule 9: `cookie_roundtrip` + `header_scheme` → Rule 9 path (a) accepted |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | header_scheme = "cookie:access_token" present; S-WAVE-A-ENGINE-001 NOT yet merged | Rule 9 not yet live; field is parsed as an unknown field or ignored; spec loads without error (no regression) |
| EC-002 | header_scheme = "cookie:access_token" present; Rule 9 live (ENGINE-001 merged) | AC-001: passes Rule 9 path (a); spec loads cleanly |
| EC-003 | header_scheme field placed in wrong TOML section | Rule 9 fires with E-SPEC-027(a) (malformed); the field must be at the top level of the spec, not inside [[tables]] |

---

## Tasks

### T-01: Add header_scheme to cyberint.sensor.toml
**File:** `crates/prism-sensors/specs/cyberint.sensor.toml`

Add the line immediately after the `auth_type = "cookie_roundtrip"` line:
```toml
header_scheme = "cookie:access_token"
```

The placement immediately after `auth_type` is conventional (related auth fields grouped
together) and mirrors the pattern established in the ENGINE-001 spec.

### T-02: Verify no other changes are included (AC-003)
Run `git diff crates/prism-sensors/specs/cyberint.sensor.toml` and confirm exactly one
added line. No other fields may change.

### T-03: Run bundled spec load test (AC-002)
```
cargo nextest run -p prism-spec-engine -E 'test(bundled)'
```
Must pass. If it fails, the `header_scheme` field was placed incorrectly or a side-effect
of other editing introduced an error.

---

## Token Budget Estimate

| Context source | Estimated tokens |
|----------------|-----------------|
| This story spec | ~1,500 |
| `crates/prism-sensors/specs/cyberint.sensor.toml` | ~2,500 |
| Test output (nextest bundled) | ~500 |
| **Total estimate** | **~4,500** |

Well within any context window. No split required.

---

## Previous Story Intelligence

N/A — this is the first story in the Cyberint remediation sub-chain.

---

## Architecture Compliance Rules

1. **Placement rule.** `header_scheme` is a top-level field in the sensor spec TOML (not
   inside a `[[tables]]` or `[[credential_refs]]` block). It must be placed at the file's
   top-level section alongside `auth_type`.

2. **Minimality rule.** This patch is the co-land minimal change. Any additional changes
   to `cyberint.sensor.toml` (method, pagination, credential_refs, table structure) must be
   deferred to `S-WAVE-A-CYBERINT-SPEC-001`. The smaller this patch, the safer the atomic
   co-landing merge.

---

## Library & Framework Requirements

None — no Rust code changes, no dependency changes.

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-sensors/specs/cyberint.sensor.toml` | MODIFY | Add `header_scheme = "cookie:access_token"` line only |

---

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.2 | 2026-07-26 | story-writer | FB60 MED-008: pin BC-2.16.009 from `current` to v1.28 in §Behavioral Contracts table |
| 1.1 | 2026-07-26 | story-writer | FB55a HIGH-002: fix §MERGE-GATE-ENGINE-001 first paragraph — false claim that `validate_sensor_spec()` carries Rule 9 replaced with correct attribution: Rule 9 is inside `SpecLoader::parse()`, the unconditional call point on every spec-load path (BC-2.16.009 §Integration function); `load_all()` boot invocation cited as the causal chain for the unconditional boot failure. Removed stale "reciprocal edge" administrative paragraph (the ENGINE-001 → PATCH-001 `blocks:` edge it described is dropped by HIGH-001). Fix AC-002 second paragraph: wrong `validate_sensor_spec()` function attribution replaced with `SpecLoader::parse()`; false conditionality "when S-ADR055-WAVE-A-001 is also merged" removed. Internal contradiction resolved — §MERGE-GATE-ENGINE-001 unconditional boot-failure claim is now consistent with AC-002, which no longer makes Rule 9 liveness conditional on ADR-055 wiring. |
| 1.0 | 2026-07-25 | story-writer | Split from S-WAVE-A-CYBERINT-SPEC-001; minimal co-land patch; MERGE-GATE-ENGINE-001 boot-failure consequence documented |
