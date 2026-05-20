---
document_type: adversarial-review-pass
story_id: PLUGIN-MIGRATION-001-D
pass_number: 3
pass_scope: LOCAL-SPEC-LEVEL
pass_date: 2026-05-20
adversary_model: Claude Opus 4.7 (1M context, fresh)
streak_before: 0/3
streak_after: 0/3
findings_summary: "3 CRIT + 2 HIGH + 1 MED + 6 OBS (12 total; 6 actionable)"
---

# PLUGIN-MIGRATION-001-D Pass-3 Adversarial Review (LOCAL Spec-Level)

## Scope
Story v1.2, BC-2.16.013 v1.2, BC-2.16.001 v1.4, BC-2.16.009 v1.4, error-taxonomy.md v1.41, HS-013..018, BC anchors (2.01.013/2.01.016/2.16.002/2.16.012), ADR-022/ADR-023/TS-PLUGIN-PARITY-001, code references (prism-sensors auth modules, prism-spec-engine).

## Methodology
Two-phase POL-22 verification. Code-grounded URL pattern audit for all 4 sensors.

## Findings

### CRITICAL

#### F-LP3-CRIT-001 PolicyViolation:POL-22 Phase C — `spec_parser::parse_spec_file()` phantom symbol at 11 sites
**Locations:** Story lines 233, 248, 263, 278, 326, 370, 393, 418; BC-2.16.013 lines 179, 281; HS-017 lines 66, 81, 103.
**Evidence:** Real symbols: `SpecLoader::parse(toml_input: &str)` (spec_parser.rs:655), `parse_spec_directory(&Path)` (config_manager.rs:75), `parse_and_validate_spec_toml(&str, &str)` (add_sensor_spec.rs). No `parse_spec_file` exists.
**Routing:** product-owner + story-writer.

#### F-LP3-CRIT-002 PolicyViolation:POL-22, POL-4 — CrowdStrike URL paths phantom
**Locations:** BC-2.16.013 lines 114-118; story Task 3.
**Evidence:** BC claims `/detects/queries/detects/v1`, `/devices/queries/devices/v1`. Actual code (crowdstrike.rs:262, 315, 369): `/queries/{resource}` + `/entities/{resource}/GET` where resource = detections/devices/incidents.
**Routing:** product-owner + story-writer.

#### F-LP3-CRIT-003 PolicyViolation:POL-22, POL-4 — Claroty `/xdome/` prefix phantom
**Locations:** BC-2.16.013 lines 125-127; story Task 4.
**Evidence:** BC claims `/xdome/api/v1/assets`. Actual code (claroty.rs:244): `/api/v1/{resource}s`. No `/xdome` segment exists.
**Routing:** product-owner + story-writer.

### HIGH

#### F-LP3-HIGH-001 PolicyViolation:POL-22, POL-4 — Cyberint `/v1/` segment phantom
**Locations:** BC-2.16.013 lines 133-134; story Task 5.
**Evidence:** BC claims `/api/v1/alerts`. Actual (cyberint.rs:251): `/api/{resource}s` — NO `/v1` segment.
**Routing:** product-owner + story-writer.

#### F-LP3-HIGH-002 PolicyViolation:POL-22, POL-4 — Armis `alerts` endpoint phantom (single-endpoint with AQL filter)
**Locations:** BC-2.16.013 lines 144-148; story Task 6.
**Evidence:** BC claims separate `/api/v1/alerts/` endpoint with trailing slash. Actual (armis.rs:517, 72, 469): single `/api/v1/search` (no trailing slash) for all tables; discriminated by AQL `in:{table}` filter.
**Routing:** product-owner + story-writer.

### MED

#### F-LP3-MED-001 PolicyViolation:POL-22 — `OrgSlug::new_unchecked` comment misleading
**Locations:** Story AC code samples (line 333-336 + 425).
**Evidence:** Comment says "test-helpers feature; NOT in production code". Reality (tenant.rs:97, 84-86): no Cargo feature gate; audit-allowlisted by `tests/new_unchecked_audit.rs` per AD-017.
**Routing:** story-writer.

### OBS

- O-1: BC §Related BCs uses abbreviated source-BC titles (POL-7 borderline, observation only)
- O-2: §query.aql correction (v1.2) verified against pipeline.rs:248 — no defect
- O-3: HS bring-up scenarios use illustrative-not-Rust shorthand for start_on — no defect
- O-4: AuthType enum (spec_parser.rs:29) has 4 variants but VALID_AUTH_TYPES (line 932-938) lists 5 — code-side issue, not spec; route to architect (out of scope)
- O-5: Story line 526 already TD-VSDD-091-compliant — good
- O-6: Claroty docstring inconsistency (`claroty.rs:8` says "Static bearer token" but `auth_type_name()` returns `"cookie_roundtrip"`) — code-side tech-debt, parallel to Cyberint pattern from FB-IMPL-P2; route to cycle-close

## Verdict
**BLOCKED-soft.** 6 actionable findings. Streak 0/3.

## Routing Summary
All 6 actionable findings closed via FB-IMPL-P3 (PO + story-writer). 2 code-side tech-debt items (O-4 AuthType variants; O-6 Claroty docstring) forwarded to cycle-close.

## Novelty Assessment
**HIGH.** Pass-3 surfaced complete URL drift cluster across all 4 sensors that pass-1 and pass-2 did not detect (anchored to API existence rather than path correctness). Plus a fresh `parse_spec_file` phantom that survived earlier sibling-sweeps. Confirms fresh-context compounding value principle — each pass with different model perspective catches new defect class.

## Streak Update
- streak_before: 0/3
- streak_after: 0/3
- next_action: FB-IMPL-P3 → pass-4 with fresh context
