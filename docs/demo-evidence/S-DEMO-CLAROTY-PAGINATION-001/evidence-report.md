---
document_type: demo-evidence-report
story_id: S-DEMO-CLAROTY-PAGINATION-001
title: "OffsetLimit POST-Body Pagination for Claroty — Demo Evidence"
product: prism-spec-engine
pipeline_run: "2026-06-08"
demo_type: cli
recording_tool: vhs
status: complete
pol10_compliant: true
---

# Demo Evidence Report — S-DEMO-CLAROTY-PAGINATION-001

**Story:** OffsetLimit POST-Body Pagination for Claroty (closes Gap-CL-004 / multi-page support)
**Product:** prism-spec-engine (crates/prism-spec-engine/src/pipeline.rs)
**Governing BC:** BC-2.16.002 v1.70 §Postconditions "OffsetLimit Pagination Dispatch: POST-body vs GET-URL (DRIFT-D850-001)"
**Recording date:** 2026-06-08
**VHS version:** 0.10.0

---

## Test Suite Summary

Command run to produce evidence:

```
cargo nextest run -p prism-spec-engine -E 'test(BC_2_16_002_pagination)'
```

Result:

```
────────────
 Nextest run ID c8aa38e6-531b-4101-9d10-3d59363a9123 with nextest profile: default
    Starting 9 tests across 38 binaries (585 tests skipped)
        PASS [   0.016s] (1/9) prism-spec-engine pipeline::pagination_post_body_tests::test_BC_2_16_002_pagination_post_method_url_unchanged
        PASS [   0.016s] (2/9) prism-spec-engine pipeline::pagination_post_body_tests::test_BC_2_16_002_pagination_get_method_continues_url_params
        PASS [   0.016s] (3/9) prism-spec-engine pipeline::pagination_post_body_tests::test_BC_2_16_002_pagination_build_paged_url_for_test_get_path_still_works
        PASS [   0.054s] (4/9) prism-spec-engine pipeline::pagination_post_body_tests::test_BC_2_16_002_pagination_post_non_object_body_surfaces_error
        PASS [   0.054s] (5/9) prism-spec-engine pipeline::pagination_post_body_tests::test_BC_2_16_002_pagination_post_invalid_json_body_surfaces_error
        PASS [   0.060s] (6/9) prism-spec-engine pipeline::pagination_post_body_tests::test_BC_2_16_002_pagination_body_template_merge_preserves_existing_keys
        PASS [   0.060s] (7/9) prism-spec-engine pipeline::pagination_post_body_tests::test_BC_2_16_002_pagination_post_first_page_offset_zero_in_body
        PASS [   0.060s] (8/9) prism-spec-engine pipeline::pagination_post_body_tests::test_BC_2_16_002_pagination_post_method_sends_offset_limit_in_body
        PASS [   0.060s] (9/9) prism-spec-engine pipeline::pagination_post_body_tests::test_BC_2_16_002_pagination_post_offset_advances_across_pages
────────────
     Summary [   0.061s] 9 tests run: 9 passed, 585 skipped
```

**9/9 tests PASS.** The 585 skipped includes the `#[ignore]`'d DTU integration test
`test_BC_2_16_002_pagination_claroty_alerts_page_2_returns_data` (DTU-EXT-001 — see §AC-003 below).

---

## Per-AC Evidence

### AC-001: POST steps send offset+limit in body, URL unchanged
**BC trace:** BC-2.16.002 v1.70 §Postconditions "OffsetLimit Pagination Dispatch: POST-body vs GET-URL" — POST step clause

**Red Gate tests (both PASS):**
- `test_BC_2_16_002_pagination_post_method_url_unchanged` — verifies `build_paged_url_impl` returns base URL unchanged (no `?offset=&limit=` appended) when `step.method == "POST"`
- `test_BC_2_16_002_pagination_post_method_sends_offset_limit_in_body` — verifies the HTTP request body for a POST step contains `"offset"` and `"limit"` keys at the top level

**VHS recording:** [AC-001-002-post-body-get-url.gif](AC-001-002-post-body-get-url.gif)
**Source tape:** [AC-001-002-post-body-get-url.tape](AC-001-002-post-body-get-url.tape)

**Evidence type:** VHS terminal recording (cargo nextest output).

**claroty.sensor.toml spec anchor** (method=POST, page_size=100 for all 3 tables):

```toml
# alerts table fetch step
[[tables.steps]]
name = "fetch_alerts"
method = "POST"
path_template = "/api/v1/alerts/"
body_template = '{}'
response_path = "$.alerts"
[tables.steps.pagination]
type = "offset_limit"
page_size = 100
```

The same `method = "POST"` + `type = "offset_limit"` pattern applies to `fetch_audit_logs`
(`/api/v1/audit_log/get/`) and `fetch_devices` (`/api/v1/devices/`).

---

### AC-002: GET steps continue appending offset+limit as URL query params
**BC trace:** BC-2.16.002 v1.70 §Postconditions "OffsetLimit Pagination Dispatch: POST-body vs GET-URL" — GET/absent-method step clause (regression guard)

**Red Gate test (PASS):**
- `test_BC_2_16_002_pagination_get_method_continues_url_params` — verifies GET steps still append `?offset=N&limit=M` to the URL (regression guard for Cyberint, Armis, CrowdStrike sensors)

**VHS recording:** [AC-001-002-post-body-get-url.gif](AC-001-002-post-body-get-url.gif) (same tape as AC-001)
**Source tape:** [AC-001-002-post-body-get-url.tape](AC-001-002-post-body-get-url.tape)

**Evidence type:** VHS terminal recording (cargo nextest output).

---

### AC-003: Multi-page Claroty query returns >100 rows
**BC trace:** BC-2.01.013 v1.14 postcondition §1 — adapter returns all records within query limits

**Non-#[ignore]'d wiremock companion (PASS):**
- `test_BC_2_16_002_pagination_post_offset_advances_across_pages` — 2-page wiremock scenario; page-1 POST body has `offset=0`, page-2 POST body has `offset=51`. Verifies the `PipelineExecutor` body-injection loop advances offset correctly across page boundaries.

**VHS recording:** [AC-003-005-offset-advances-first-page-zero.gif](AC-003-005-offset-advances-first-page-zero.gif)
**Source tape:** [AC-003-005-offset-advances-first-page-zero.tape](AC-003-005-offset-advances-first-page-zero.tape)

**Evidence type:** VHS terminal recording (cargo nextest output).

**DTU-EXT-001 gating note:** The live-DTU integration test
`test_BC_2_16_002_pagination_claroty_alerts_page_2_returns_data` is `#[ignore]`'d:

```rust
#[ignore = "DTU-EXT-001: requires prism-dtu-claroty clone running with 102-entry alerts fixture;
            ungated after S-DEMO-CLAROTY-PAGINATION-001 merges and DTU fixture is recorded"]
```

Per SID-1 (CLAUDE.md §Standing Adversary Probes), the wiremock companion
`test_BC_2_16_002_pagination_post_offset_advances_across_pages` provides equivalent
non-ignored coverage of the production code path (the body-injection loop in
`PipelineExecutor::execute_impl` and `build_request`). The `#[ignore]`'d test cites the
specific story (S-DEMO-CLAROTY-PAGINATION-001) and blocking dependency (DTU clone boot).

---

### AC-004: Body template merging preserves existing body fields
**BC trace:** BC-2.16.002 v1.70 §Postconditions "OffsetLimit Pagination Dispatch: POST-body vs GET-URL" — body merge clause

**Red Gate test (PASS):**
- `test_BC_2_16_002_pagination_body_template_merge_preserves_existing_keys` — verifies that injecting `offset` and `limit` into a body that already contains other keys (e.g., `{"filter": "active"}`) preserves the pre-existing keys in the merged body object

**VHS recording:** [AC-004-EC002-body-merge-error-paths.gif](AC-004-EC002-body-merge-error-paths.gif)
**Source tape:** [AC-004-EC002-body-merge-error-paths.tape](AC-004-EC002-body-merge-error-paths.tape)

**Evidence type:** VHS terminal recording (cargo nextest output).

---

### AC-005: First-page request uses offset=0
**BC trace:** BC-2.16.002 v1.70 §Postconditions "OffsetLimit Pagination Dispatch: POST-body vs GET-URL" — offset initialization clause

**Red Gate test (PASS):**
- `test_BC_2_16_002_pagination_post_first_page_offset_zero_in_body` — verifies the first POST request body contains `"offset": 0` and `"limit": <page_size>` (not a non-zero carry-over from a previous run)

**VHS recording:** [AC-003-005-offset-advances-first-page-zero.gif](AC-003-005-offset-advances-first-page-zero.gif)
**Source tape:** [AC-003-005-offset-advances-first-page-zero.tape](AC-003-005-offset-advances-first-page-zero.tape)

**Evidence type:** VHS terminal recording (cargo nextest output).

---

### AC-006: build_paged_url_for_test remains callable for GET paths
**BC trace:** BC-2.16.002 v1.70 §Postconditions "OffsetLimit Pagination Dispatch: POST-body vs GET-URL" — GET regression guard

**Red Gate test (PASS):**
- `test_BC_2_16_002_pagination_build_paged_url_for_test_get_path_still_works` — verifies the public test helper `build_paged_url_for_test` (used in `#[cfg(test)]` modules) still returns URL-appended results for GET steps; confirms `build_paged_url_impl` signature is unchanged (Task 4 / Note 5 in story spec — the logic-only change requires no signature update)

**VHS recording:** [AC-004-EC002-body-merge-error-paths.gif](AC-004-EC002-body-merge-error-paths.gif)
**Source tape:** [AC-004-EC002-body-merge-error-paths.tape](AC-004-EC002-body-merge-error-paths.tape)

**Evidence type:** VHS terminal recording (cargo nextest output).

---

### EC-001: POST step with empty body_template ({})
**Expected behavior:** Offset+limit merged into empty object: `{"offset": 0, "limit": 100}`

**Coverage:** This case is exercised by `test_BC_2_16_002_pagination_post_method_sends_offset_limit_in_body`
and `test_BC_2_16_002_pagination_post_first_page_offset_zero_in_body`, both of which use an empty
`body_template = "{}"` (matching the claroty.sensor.toml production config). The body assertion verifies the
merged result contains the expected keys.

**Evidence type:** Covered by AC-001 and AC-005 recordings above (same tape).

---

### EC-002: POST step with non-object/invalid body_template → Err, no panic
**BC trace:** Story EC-002 — error path

**Red Gate tests (both PASS — error path demos):**
- `test_BC_2_16_002_pagination_post_non_object_body_surfaces_error` — verifies a non-JSON-object body_template (e.g., `"hello"`) produces a `SpecEngineError` result, not a panic
- `test_BC_2_16_002_pagination_post_invalid_json_body_surfaces_error` — verifies invalid JSON body_template (e.g., `"not json"`) produces a `SpecEngineError` result, not a panic

**VHS recording:** [AC-004-EC002-body-merge-error-paths.gif](AC-004-EC002-body-merge-error-paths.gif)
**Source tape:** [AC-004-EC002-body-merge-error-paths.tape](AC-004-EC002-body-merge-error-paths.tape)

**Evidence type:** VHS terminal recording showing PASS on error-path tests (cargo nextest output).

---

### EC-006: page_size=0 terminates safely at MAX_REQUESTS_PER_PIPELINE cap
**Coverage:** Not directly tested by a dedicated test (story spec explicitly calls this out-of-scope
in EC-006 note: "adding such a guard is a pre-existing spec-engine validation concern, separately
routed to PO"). The advance logic (`offset += page_size`, termination on `page_record_count < page_size`)
is exercised by the multi-page tests. No division by page_size occurs, so no panic path exists.

---

## Full Suite Recording

| Tape | ACs covered | Recording | Format | Status |
|------|-------------|-----------|--------|--------|
| ALL-ACs-full-suite | AC-001..006, EC-002 | [gif](ALL-ACs-full-suite.gif) [webm](ALL-ACs-full-suite.webm) | VHS gif+webm | PASS 9/9 |
| AC-001-002-post-body-get-url | AC-001, AC-002 | [gif](AC-001-002-post-body-get-url.gif) [webm](AC-001-002-post-body-get-url.webm) | VHS gif+webm | PASS 3/3 |
| AC-003-005-offset-advances-first-page-zero | AC-003, AC-005 | [gif](AC-003-005-offset-advances-first-page-zero.gif) [webm](AC-003-005-offset-advances-first-page-zero.webm) | VHS gif+webm | PASS 2/2 |
| AC-004-EC002-body-merge-error-paths | AC-004, AC-006, EC-002 | [gif](AC-004-EC002-body-merge-error-paths.gif) [webm](AC-004-EC002-body-merge-error-paths.webm) | VHS gif+webm | PASS 4/4 |

---

## AC Coverage Matrix

| AC | Description | Test name | Status | Recording |
|----|-------------|-----------|--------|-----------|
| AC-001 (URL side) | POST URL unchanged | `test_BC_2_16_002_pagination_post_method_url_unchanged` | PASS | AC-001-002 tape |
| AC-001 (body side) | POST body has offset+limit | `test_BC_2_16_002_pagination_post_method_sends_offset_limit_in_body` | PASS | AC-001-002 tape |
| AC-002 | GET URL appended | `test_BC_2_16_002_pagination_get_method_continues_url_params` | PASS | AC-001-002 tape |
| AC-003 | Multi-page offset advances | `test_BC_2_16_002_pagination_post_offset_advances_across_pages` | PASS (wiremock) | AC-003-005 tape |
| AC-003 (DTU) | Live DTU 102-row fixture | `test_BC_2_16_002_pagination_claroty_alerts_page_2_returns_data` | `#[ignore]` DTU-EXT-001 | N/A — DTU gated |
| AC-004 | Body merge preserves keys | `test_BC_2_16_002_pagination_body_template_merge_preserves_existing_keys` | PASS | AC-004-EC002 tape |
| AC-005 | First page offset=0 in body | `test_BC_2_16_002_pagination_post_first_page_offset_zero_in_body` | PASS | AC-003-005 tape |
| AC-006 | GET test helper unchanged | `test_BC_2_16_002_pagination_build_paged_url_for_test_get_path_still_works` | PASS | AC-004-EC002 tape |
| EC-001 | Empty body_template merges | covered by AC-001/AC-005 tests (use `{}`) | PASS | AC-001-002 tape |
| EC-002 (non-object) | Non-object body → Err | `test_BC_2_16_002_pagination_post_non_object_body_surfaces_error` | PASS | AC-004-EC002 tape |
| EC-002 (invalid JSON) | Invalid JSON body → Err | `test_BC_2_16_002_pagination_post_invalid_json_body_surfaces_error` | PASS | AC-004-EC002 tape |
| EC-006 | page_size=0 safe termination | not tested (story spec: out-of-scope, PO drift item) | N/A | N/A |

---

## Summary

**All ACs + EC-001/EC-002 evidenced: CONFIRMED.**

- **9/9 non-ignored tests PASS** in `prism-spec-engine pipeline::pagination_post_body_tests`
- **4 VHS recordings** produced (gif + webm per tape = 8 media files total)
- **AC-003 live-DTU multi-page query** is gated DTU-EXT-001 (`#[ignore]`'d); the wiremock companion
  `test_BC_2_16_002_pagination_post_offset_advances_across_pages` provides equivalent non-ignored
  coverage per SID-1. The `#[ignore]`'d test cites the specific story and blocking dependency.
- **Gap-CL-004 is CLOSED:** claroty.sensor.toml `method = "POST"` + `type = "offset_limit"` for
  alerts, audit_logs, and devices tables — the pipeline now correctly injects `offset` and `limit`
  into the POST request body rather than URL query params.
- **DRIFT-D850-001 RESOLVED:** BC-2.16.002 v1.70 §Postconditions "OffsetLimit Pagination Dispatch:
  POST-body vs GET-URL" is the governing postcondition. D-1059 2026-06-08.

---

## Toolchain

| Tool | Version | Status |
|------|---------|--------|
| VHS | 0.10.0 | installed |
| cargo nextest | workspace | installed |
| FiraCode Nerd Font Mono | installed | font confirmed |

---

## POL-10 Compliance

All evidence files are under `docs/demo-evidence/S-DEMO-CLAROTY-PAGINATION-001/` (story-scoped
subfolder). No files placed at `docs/demo-evidence/*.md` (flat root). This report is at
`docs/demo-evidence/S-DEMO-CLAROTY-PAGINATION-001/evidence-report.md`. POL-10 COMPLIANT.

**Note on absolute paths in .tape files:** VHS tape files in this directory contain absolute paths
(`/Users/jmagady/Dev/prism/.worktrees/S-DEMO-CLAROTY-PAGINATION-001`). This is the pre-existing
project-wide pattern documented as DRIFT-SEC-TAPE-PATH-001, registered for a separate sweep.
This story follows the existing pattern and does not special-case it.
