---
document_type: adversarial-review-pass
story: DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001
pass: 14
scope: LOCAL
reviewer: general-purpose-as-adversary
frozen_head: 558d5881
story_version: v0.14
bc_versions:
  BC-2.06.017: v1.12
date: 2026-07-17
clean_strict: false
clean_pr_merge: true
findings_summary: "0 CRIT / 0 HIGH / 0 MED / 0 LOW / 1 OBS — single observation, Test J inventory row absent from module-header test table"
streak_after: "0/3 (new HEAD 803db300 from fb-13; DRIFT-ORCH-PRLEVEL-PUSH-001 streak reset)"
next_pass: LOCAL pass-15 on frozen 803db300
---

# Adversarial Review — DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001 LOCAL Pass 14

**Reviewer:** general-purpose-as-adversary (fresh context; no prior pass reports read)
**Frozen HEAD:** `558d5881`
**Story version:** v0.14
**BC versions:** BC-2.06.017 v1.12; BC-3.6.001 (current)
**Date:** 2026-07-17
**Cascade tally:** 14 passes / 12 fix-bursts (pass-14 on story v0.14 + BC-2.06.017 v1.12)

## Verdict

```
CLEAN (strict):    NO   — 1 OBS present
CLEAN (PR-merge):  YES  — 0 CRIT / 0 HIGH / 0 MED / 0 LOW
```

Finding trajectory: 4 → 5 → 5 → 1

Streak: 0/3 (reset by fb-13 push at 803db300 per DRIFT-ORCH-PRLEVEL-PUSH-001).

---

## Positive Verifications (24-item battery — highlights)

- **Sweep counts reproduce at HEAD:** `write_multi_admin_token_sidecar_to_path` 447 grep hits, `write_token_sidecar_to_path` 131 hits, `token_map()` 6 hits, `TOKEN_MULTI_FILE` 8 hits — all four commands reproduce the reported values at `558d5881`.
- **SWEEP-MIRROR byte-identity:** Disposition table in story AC-004 ¶1 matches the code across all three artifacts (story, code, pass-13 report) — byte-identical counts confirmed.
- **Fixture-gen suite 99/99 incl. Test G:** `write_multi_admin_token_sidecar_to_path` fixture-generation suite passes all 99 cases including Test G (10-org fixture set, all at correct path patterns, all at 0600 permissions).
- **Default suite 59 pass + 3 known Red Gate:** `cargo nextest run -p prism-dtu-demo-server` returns 59 passing + 3 pre-existing Red Gate fails (T-01 through T-09 all GREEN; T-10 known DTU-EXT-001 integration skip).
- **Determinism sweep — zero unsorted user-facing lists remain:** All 8 `collect()` hits in `harness.rs` are either `.sorted()` before use or are not user-facing (internal Vec accumulations with no observable order contract). No unsorted user-facing list paths remain.
- **POL-22 Phase A/C verbatim:** ADR-003 Amendment #5 quotation in story AC-001, BC-2.06.017 Postcondition-1 quotation in AC-003, BC-3.6.001 Postcondition-4 v0.8 pin in AC-004 — all three passages verified verbatim against current spec files.
- **POL-21 all live §-refs resolve:** Every `§` section cross-reference in story v0.14 resolves to an existing heading in the cited file.
- **POL-24 E-DEMO-007 byte-verbatim:** Error taxonomy `E-DEMO-007` message string in story matches the emitted string in `resolve_configure_token` byte-for-byte, confirming the EC-005 v0.13 rewrite landed correctly.
- **POL-13 STORY-INDEX v0.14 @558d5881 match:** STORY-INDEX row for DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001 records `**draft v0.14** (558d5881 D-1799 ...)` — version and SHA match the frozen HEAD.
- **SAP-1 zero new emissions:** `rg 'event_type\s*=' crates/prism-dtu-demo-server/src/ --type rust` returns 0 hits in the fb-12 diff; no new tracing emissions were introduced and no BC-2.16.002 catalog update is needed.
- **AD-017 clean:** No credential values in story text; diagnostics emit key-names-only (not token values); 0600 permissions gate confirmed on all sidecar-write paths.
- **POL-12 no stub residue:** No `todo!()`, `unimplemented!()`, or `FIXME` stubs remain in any file touched by the cascade.
- **Clippy clean:** `cargo clippy -p prism-dtu-demo-server -- -D warnings` exits 0; no new clippy lints introduced by fb-12.

---

## Findings

### F-ADMTOK-P14-OBS-001 [OBS] [doc-completeness] — Test J absent from module-header "## Test inventory" table

**Severity:** OBS (observational; doc-completeness)

**Location:** `crates/prism-dtu-demo-server/tests/defect_demo_configure_admintoken_001.rs` — module-header `## Test inventory` table.

**Description:**

fb-11 (pass-12 fix-burst) added Test J — `test_resolve_configure_url_ambiguity_message_uses_sorted_org_list` — as the F-ADMTOK-P12-OBS-001 closure lock: a load-bearing assertion that the ambiguity-resolution error message lists candidate org slugs in sorted lexicographic order. Test J is the key regression-guard for the determinism property established across this cascade.

However, neither fb-11 (which created the test) nor fb-12 (which swept sorted-order paths) updated the `## Test inventory` table at the top of the test module. The table is the canonical enumeration site for all tests in the file (POL-29 single-enumeration-site discipline). Test J appears in the function body but is invisible to readers using the inventory table as the authoritative test list.

This is a within-burst sibling-sweep residue: the implementer correctly added the test and its behavioral assertion, but did not sweep the inventory table as the companion enumeration site.

**Scope:** Documentation gap only. Test J exists, runs, and passes. No behavioral correctness risk.

**Required fix:** Add Test J row to the `## Test inventory` table with schema-matched columns (test ID, function name, description, gated/ungated status). Verify POL-29 single-enumeration-site: exactly one place lists all test names.

---

## Resolution

| Finding | Severity | Closed by | Mechanism |
|---------|----------|-----------|-----------|
| F-ADMTOK-P14-OBS-001 | OBS [doc-completeness] | implementer fb-13 @803db300 | Test J row added to `## Test inventory` table (schema-matched); POL-29 sweep confirmed single enumeration site; 9/9 ungated + Test G gated as expected |

Finding closed. Streak reset to 0/3 by fb-13 push (803db300). Next = LOCAL pass-15 on frozen 803db300.
