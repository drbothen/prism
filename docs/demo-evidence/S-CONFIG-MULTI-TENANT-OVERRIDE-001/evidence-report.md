# Demo Evidence Report — S-CONFIG-MULTI-TENANT-OVERRIDE-001

**Story:** S-CONFIG-MULTI-TENANT-OVERRIDE-001 — Per-Org Sensor Endpoint Overlay Loading (ADR-029)
**Branch:** feature/S-CONFIG-MULTI-TENANT-OVERRIDE-001
**Feature HEAD:** 46c759f6 (PR #155 fix-burst — 24-finding consolidation commit)
**Demo captured at:** d600f7f4 — AC-001..007 demos reflect behavior at that SHA. The fix-burst at 46c759f6 addresses security and code-quality findings (SEC-REDUX-001..006, ADV-010, ADV-011, PRR-004..013) but does not change the overlay loading behavior observed in the recordings — the demos remain valid evidence for all ACs.
**Recording tool:** VHS 0.10.0
**Font:** FiraCode Nerd Font Mono
**Recorded:** 2026-05-24

---

## Coverage Summary

| AC | Title | Red Gate Test(s) | Recording | Status |
|----|-------|------------------|-----------|--------|
| AC-001 | Overlay file discovery and scalar merge | `test_BC_2_06_012_overlay_discovered_and_merged` | AC-001-overlay-discovery-and-merge.gif/.webm | PASS |
| AC-002 | Scalar-only overlay enforcement | `test_BC_2_06_013_tables_in_overlay_rejects_with_e_spec_021`, `test_BC_2_06_013_unrecognized_field_rejects_with_e_spec_023`, `test_BC_2_06_013_wrong_instance_id_rejects_with_e_spec_020` | AC-002-scalar-only-enforcement.gif/.webm | PASS (3/3) |
| AC-003 | Instance identity resolution at fanout uses overlay base_url | `test_BC_2_06_014_resolved_spec_overlays_base_url`, `test_F_LP2_CRIT_001_fan_out_with_overlay_map_routes_to_overlay_url` | AC-003-fanout-overlay-base-url.gif/.webm | PASS |
| AC-004 | OrgRegistry cross-validation — unknown dir aborts boot | `test_BC_2_06_015_unknown_org_dir_aborts_boot_with_e_spec_022` | AC-004-org-registry-cross-validation.gif/.webm | PASS |
| AC-005 | Error taxonomy SpecErrorCode byte-equality safety net | `test_BC_2_06_016_error_messages_match_canonical_templates`, `test_BC_2_06_016_EC_016_001_tables_and_unrecognized_field_both_collected`, `test_BC_2_06_016_EC_016_002_unknown_org_and_tables_both_collected`, `test_BC_2_06_016_EC_016_003_all_five_codes_in_same_boot` | AC-005-error-taxonomy-byte-equality.gif/.webm | PASS (4/4) |
| AC-006 | Backwards compatibility — no customers/ directory | `test_BC_2_06_012_backcompat_no_customers_dir_uses_type_spec_only` | AC-006-backwards-compat-no-customers-dir.gif/.webm | PASS |
| AC-007 | Two-org overlays produce distinct ResolvedSensorSpec entries | `test_S_CONFIG_MULTI_TENANT_OVERRIDE_001_007_two_org_overlays_produce_distinct_resolved_specs` | AC-007-two-org-distinct-resolved-specs.gif/.webm | PASS |

All 7 acceptance criteria have passing demo recordings. All 11 Red Gate tests pass.

---

## AC-001: Overlay file discovery and scalar merge

**BC trace:** BC-2.06.012 postconditions 1–7
**Recording:** `AC-001-overlay-discovery-and-merge.gif` / `.webm`
**Tape:** `AC-001-overlay-discovery-and-merge.tape`

Demonstrates `OverlayLoader::load_overlays` discovering `customers/acme/armis.sensor.toml`,
parsing it into `SensorInstanceOverlay`, merging `base_url` onto the Armis TYPE spec,
and producing a `ResolvedSensorSpec` indexed at `(acme, armis)` with:
- `spec.base_url = "https://armis.acme-corp.io"` (from overlay, not TYPE spec default)
- `spec.tables` identical to TYPE spec (INV-OVL-001 — schema immutability)
- `spec.auth_type` unchanged from TYPE spec (INV-OVL-002)
- `provenance.base_url_from_overlay = true`
- `instance_id = "armis@acme"`

---

## AC-002: Scalar-only overlay enforcement (3 error paths)

**BC trace:** BC-2.06.013 failure paths
**Recording:** `AC-002-scalar-only-enforcement.gif` / `.webm`
**Tape:** `AC-002-scalar-only-enforcement.tape`

Demonstrates all three boot-time rejection paths (all 3 tests pass simultaneously):

1. `test_BC_2_06_013_tables_in_overlay_rejects_with_e_spec_021` — `[[tables]]` block in overlay
   produces `E-SPEC-021` (BC-2.06.016 §Error Catalog)
2. `test_BC_2_06_013_unrecognized_field_rejects_with_e_spec_023` — `auth_type` field in overlay
   produces `E-SPEC-023` with field name in error message
3. `test_BC_2_06_013_wrong_instance_id_rejects_with_e_spec_020` — `instance_id = "armis@wrongorg"`
   in `customers/acme/` produces `E-SPEC-020` with expected value `"armis@acme"` in message

---

## AC-003: Instance identity resolution at fanout uses overlay base_url

**BC trace:** BC-2.06.014 Case A (overlay) + Case B (no overlay)
**Recording:** `AC-003-fanout-overlay-base-url.gif` / `.webm`
**Tape:** `AC-003-fanout-overlay-base-url.tape`

Demonstrates two tests:

1. `test_BC_2_06_014_resolved_spec_overlays_base_url`:
   - Case A: overlay `base_url` used at HTTP dispatch; `provenance.base_url_from_overlay = true`
   - Case B: minimal overlay (no `base_url`) falls back to TYPE spec `base_url`; `provenance.base_url_from_overlay = false`

2. `test_F_LP2_CRIT_001_fan_out_with_overlay_map_routes_to_overlay_url` (F-LP2-CRIT-001
   end-to-end): full Arc-DI plumbing through `MaterializationContext` → `QueryEngine` →
   `RunningServer` → per-org `base_url` routing verified against HTTP mock. This test closed
   the most critical pass-2 paper-fix finding — the `resolved_spec_map` is now threaded
   through `FanOutTarget` via the real production wiring path, not a placeholder return.

---

## AC-004: OrgRegistry cross-validation — unknown customers/<slug>/ directory aborts boot

**BC trace:** BC-2.06.015 failure path
**Recording:** `AC-004-org-registry-cross-validation.gif` / `.webm`
**Tape:** `AC-004-org-registry-cross-validation.tape`

Demonstrates that `customers/unknown-org/armis.sensor.toml` with an OrgRegistry containing
only `"acme"` produces:
- `result.errors` containing `E-SPEC-022`
- Error message includes slug `"unknown-org"` (BC-2.06.016 canonical template)
- `result.resolved` is empty — unregistered slug blocks the entire walk (INV-SCALAR-003)

---

## AC-005: Error taxonomy and SpecErrorCode byte-equality safety net

**BC trace:** BC-2.06.016 §Error Catalog (E-SPEC-019 through E-SPEC-023)
**Recording:** `AC-005-error-taxonomy-byte-equality.gif` / `.webm`
**Tape:** `AC-005-error-taxonomy-byte-equality.tape`

Demonstrates 4 tests passing simultaneously:

1. `test_BC_2_06_016_error_messages_match_canonical_templates` — triggers all five
   error conditions (E-SPEC-019..023) and **byte-compares** `SpecError::message` against
   the canonical template read from `error-taxonomy.md` at test runtime (POL-25 safety net).
   Any drift between production code and taxonomy causes a named assertion failure.

2. `test_BC_2_06_016_EC_016_001_tables_and_unrecognized_field_both_collected` — both
   `E-SPEC-021` and `E-SPEC-023` collected from the same overlay file (INV-ERR-003
   multi-error aggregation; no short-circuit at first error).

3. `test_BC_2_06_016_EC_016_002_unknown_org_and_tables_both_collected` — `E-SPEC-022`
   (unregistered slug) and `E-SPEC-021` (tables in overlay) both collected from the same
   file under an unregistered directory (post-F-LP1-HIGH-001 fix: no early-return guard).

4. `test_BC_2_06_016_EC_016_003_all_five_codes_in_same_boot` — all five codes
   (E-SPEC-019 through E-SPEC-023) appear in a single `load_overlays` call across five
   overlay files, with ≥5 errors total.

The byte-equality check in test 1 is highlighted as a unique safety net: it mechanically
prevents drift between production code, error-taxonomy.md, and BC-2.06.016 definitions,
catching TD-VSDD-059 paper-fix patterns at the test level.

---

## AC-006: Backwards compatibility — no customers/ directory

**BC trace:** BC-2.06.012 EC-012-001 + EC-012-002
**Recording:** `AC-006-backwards-compat-no-customers-dir.gif` / `.webm`
**Tape:** `AC-006-backwards-compat-no-customers-dir.tape`

Demonstrates two scenarios:
- Scenario A (EC-012-001): `customers/` directory entirely absent → zero `ResolvedSensorSpec`
  entries, zero errors, boot continues normally.
- Scenario B (EC-012-002): `customers/.gitkeep` present but no subdirectories → same zero-overlay
  result (plain file not treated as a slug per INV-COMPAT-004).

Verifies that existing single-tenant prism deployments are unaffected by this story.

---

## AC-007: Two-org overlays produce distinct ResolvedSensorSpec entries

**BC trace:** BC-2.06.012 §Canonical Test Vectors (two-org same-sensor, EC-012-006)
**Recording:** `AC-007-two-org-distinct-resolved-specs.gif` / `.webm`
**Tape:** `AC-007-two-org-distinct-resolved-specs.tape`

Demonstrates loading both fixture overlays:
- `customers/acme/armis.sensor.toml` → `(acme, armis)` with `base_url = "https://armis.acme-corp.io"`
- `customers/contoso/armis.sensor.toml` → `(contoso, armis)` with `base_url = "https://armis.contoso.com"`

Asserts:
- Exactly 2 `ResolvedSensorSpec` entries in the resolved map
- Distinct `base_url` per org (INV-FANOUT-004 — resolving one org does not affect the other)
- Identical `[[tables]]` schemas from TYPE spec for both orgs (INV-OVL-001)
- `instance_id` values: `"armis@acme"` and `"armis@contoso"`

---

## Artifact Index

| File | Type | AC | Size |
|------|------|-----|------|
| `AC-001-overlay-discovery-and-merge.gif` | GIF recording | AC-001 | 190 KB |
| `AC-001-overlay-discovery-and-merge.webm` | WebM recording | AC-001 | 228 KB |
| `AC-001-overlay-discovery-and-merge.tape` | VHS script | AC-001 | — |
| `AC-002-scalar-only-enforcement.gif` | GIF recording | AC-002 | 171 KB |
| `AC-002-scalar-only-enforcement.webm` | WebM recording | AC-002 | 223 KB |
| `AC-002-scalar-only-enforcement.tape` | VHS script | AC-002 | — |
| `AC-003-fanout-overlay-base-url.gif` | GIF recording | AC-003 | 303 KB |
| `AC-003-fanout-overlay-base-url.webm` | WebM recording | AC-003 | 559 KB |
| `AC-003-fanout-overlay-base-url.tape` | VHS script | AC-003 | — |
| `AC-004-org-registry-cross-validation.gif` | GIF recording | AC-004 | 189 KB |
| `AC-004-org-registry-cross-validation.webm` | WebM recording | AC-004 | 230 KB |
| `AC-004-org-registry-cross-validation.tape` | VHS script | AC-004 | — |
| `AC-005-error-taxonomy-byte-equality.gif` | GIF recording | AC-005 | 250 KB |
| `AC-005-error-taxonomy-byte-equality.webm` | WebM recording | AC-005 | 260 KB |
| `AC-005-error-taxonomy-byte-equality.tape` | VHS script | AC-005 | — |
| `AC-006-backwards-compat-no-customers-dir.gif` | GIF recording | AC-006 | 211 KB |
| `AC-006-backwards-compat-no-customers-dir.webm` | WebM recording | AC-006 | 211 KB |
| `AC-006-backwards-compat-no-customers-dir.tape` | VHS script | AC-006 | — |
| `AC-007-two-org-distinct-resolved-specs.gif` | GIF recording | AC-007 | 216 KB |
| `AC-007-two-org-distinct-resolved-specs.webm` | WebM recording | AC-007 | 287 KB |
| `AC-007-two-org-distinct-resolved-specs.tape` | VHS script | AC-007 | — |

---

## Out-of-Perimeter Findings

None surfaced during recording. All 11 Red Gate tests pass clean against the pre-built test
binaries. No source code modifications were made during this demo recording session.
