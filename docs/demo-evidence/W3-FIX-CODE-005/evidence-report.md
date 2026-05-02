# Demo Evidence Report — W3-FIX-CODE-005

| Field | Value |
|-------|-------|
| Story ID | W3-FIX-CODE-005 |
| Title | DTU harness + Armis/CrowdStrike: sibling poll-backoff propagation and missing org-id guards |
| Branch | feature/W3-FIX-CODE-005 |
| HEAD SHA | 652409cf |
| Wave | 3.3 |
| Priority | P1 (5 sub-fixes: 3 MEDIUM + 2 LOW) |
| Recorded | 2026-05-01 |
| Toolchain | VHS 0.10.0 (CLI product) |
| Font | FiraCode Nerd Font Mono |

## Sub-fixes resolved

| ID | Severity | Crate | Description | Status |
|----|----------|-------|-------------|--------|
| CR-016 | MEDIUM | prism-dtu-harness | 3 clone-specific `poll_test_hook` mirrors updated 10ms → 50ms | Closed |
| CR-017 / M-50-001 | MEDIUM | prism-dtu-armis | `validate_org_id` dual-mode guard applied to `tags.rs` + `alerts.rs` | Closed |
| CR-018 | MEDIUM | prism-dtu-crowdstrike | `validate_org_id` nil-instance guard applied to `detections.rs` | Closed |
| CR-020 | LOW | prism-customer-config | `validate_spec_path` pub-vs-pub(crate) deviation documented | Closed |
| L-50-004 | LOW | factory-specs | TD-W3-POLL-NOTIFY-001 filed in tech-debt-register.md | Closed |

## Coverage map

### AC-001 — CR-016: All three clone-specific poll mirrors updated to 50ms

**Behavioral contract:** BC-3.5.001 postcondition 5 (12-clone harness drops from ~300 to ~60 wake-ups/second)

| Recording | Path | What it shows |
|-----------|------|---------------|
| [AC-001-cr016-50ms-cadence.gif](AC-001-cr016-50ms-cadence.gif) | Success | `grep -r 'from_millis(10)'` returns zero matches in `clones/`; all three sibling files show `from_millis(50)` |
| [AC-001-cr016-50ms-cadence.webm](AC-001-cr016-50ms-cadence.webm) | Success (archival) | Same |
| [AC-001-cr016-50ms-cadence.tape](AC-001-cr016-50ms-cadence.tape) | Script source | — |

Error path: the `grep ... && echo FAIL || echo 'PASS'` idiom demonstrates that any surviving `from_millis(10)` would print `FAIL` to terminal — the recording shows `PASS`.

---

### AC-002 — CR-017 / M-50-001: Armis dual-mode guard on tag and alert endpoints

**Behavioral contract:** BC-3.5.002 precondition 3; BC-3.2.001 precondition 4

8 tests in `crates/prism-dtu-armis/tests/cr017_tag_alert_org_id_guard.rs`:

| Test | Endpoint | Condition | Expected |
|------|----------|-----------|----------|
| `test_post_device_tag_real_org_absent_header_returns_401` | `POST /api/v1/devices/{id}/tags/` | real-org, absent header | HTTP 401 |
| `test_post_device_tag_real_org_correct_header_returns_201` | `POST /api/v1/devices/{id}/tags/` | real-org, correct header | HTTP 201 |
| `test_post_device_tag_default_instance_absent_header_returns_201` | `POST /api/v1/devices/{id}/tags/` | default-instance, absent header | HTTP 201 (backward compat) |
| `test_delete_device_tag_real_org_absent_header_returns_401` | `DELETE /api/v1/devices/{id}/tags/{key}` | real-org, absent header | HTTP 401 |
| `test_delete_device_tag_default_instance_absent_header_allows_request` | `DELETE /api/v1/devices/{id}/tags/{key}` | default-instance, absent header | HTTP 200 |
| `test_get_alerts_real_org_absent_header_returns_401` | `GET /api/v1/alerts` | real-org, absent header | HTTP 401 |
| `test_get_alerts_real_org_correct_header_returns_200` | `GET /api/v1/alerts` | real-org, correct header | HTTP 200 |
| `test_get_alerts_default_instance_absent_header_returns_200` | `GET /api/v1/alerts` | default-instance, absent header | HTTP 200 |

| Recording | Path | What it shows |
|-----------|------|---------------|
| [AC-002-cr017-tag-alert-org-id-guard.gif](AC-002-cr017-tag-alert-org-id-guard.gif) | Success + Error | 8/8 pass; 401 error paths embedded in test assertions |
| [AC-002-cr017-tag-alert-org-id-guard.webm](AC-002-cr017-tag-alert-org-id-guard.webm) | Success + Error (archival) | Same |
| [AC-002-cr017-tag-alert-org-id-guard.tape](AC-002-cr017-tag-alert-org-id-guard.tape) | Script source | — |

---

### AC-003 — CR-018: CrowdStrike nil-instance guard on detection endpoints

**Behavioral contract:** BC-3.5.002 precondition 3; BC-3.2.001 precondition 4

6 tests in `crates/prism-dtu-crowdstrike/tests/cr018_detections_org_id_guard.rs`:

| Test | Endpoint | Condition | Expected |
|------|----------|-----------|----------|
| `test_list_detection_ids_real_org_absent_header_returns_401` | `GET /detects/queries/detects/v1` | real-org, absent header | HTTP 401 |
| `test_list_detection_ids_real_org_correct_header_returns_200` | `GET /detects/queries/detects/v1` | real-org, correct header | HTTP 200 |
| `test_list_detection_ids_nil_instance_absent_header_returns_200` | `GET /detects/queries/detects/v1` | nil-instance, absent header | HTTP 200 (backward compat) |
| `test_get_detection_summaries_real_org_absent_header_returns_401` | `POST /detects/entities/summaries/GET/v1` | real-org, absent header | HTTP 401 |
| `test_get_detection_summaries_real_org_correct_header_returns_200` | `POST /detects/entities/summaries/GET/v1` | real-org, correct header | HTTP 200 |
| `test_get_detection_summaries_nil_instance_absent_header_returns_200` | `POST /detects/entities/summaries/GET/v1` | nil-instance, absent header | HTTP 200 |

| Recording | Path | What it shows |
|-----------|------|---------------|
| [AC-003-cr018-detections-org-id-guard.gif](AC-003-cr018-detections-org-id-guard.gif) | Success + Error | 6/6 pass; 401 error paths embedded in test assertions |
| [AC-003-cr018-detections-org-id-guard.webm](AC-003-cr018-detections-org-id-guard.webm) | Success + Error (archival) | Same |
| [AC-003-cr018-detections-org-id-guard.tape](AC-003-cr018-detections-org-id-guard.tape) | Script source | — |

---

### AC-004 — CR-020: validate_spec_path deviation comment

**Behavioral contract:** BC-3.3.004 invariant 1

| Recording | Path | What it shows |
|-----------|------|---------------|
| [AC-004-cr020-validate-spec-path-deviation-comment.gif](AC-004-cr020-validate-spec-path-deviation-comment.gif) | Success | `grep -n -A10 'AC-005 deviation'` shows the 10-line comment block above `#[doc(hidden)]` in `validator.rs` |
| [AC-004-cr020-validate-spec-path-deviation-comment.webm](AC-004-cr020-validate-spec-path-deviation-comment.webm) | Success (archival) | Same |
| [AC-004-cr020-validate-spec-path-deviation-comment.tape](AC-004-cr020-validate-spec-path-deviation-comment.tape) | Script source | — |

Error path: absence of the comment would produce no `grep` output (the comment text is the evidence; any future regression would be caught by the same grep returning empty).

---

### AC-005 — L-50-004: TD-W3-POLL-NOTIFY-001 filed in tech-debt-register.md

**Behavioral contract:** BC-3.5.001 postcondition 5 (50ms acceptable; Notify-based replacement Wave 4)

| Recording | Path | What it shows |
|-----------|------|---------------|
| [AC-005-l50004-tech-debt-register.gif](AC-005-l50004-tech-debt-register.gif) | Success | `grep -c` returns 1 + `PASS: entry present`; second grep shows the TD row |
| [AC-005-l50004-tech-debt-register.webm](AC-005-l50004-tech-debt-register.webm) | Success (archival) | Same |
| [AC-005-l50004-tech-debt-register.tape](AC-005-l50004-tech-debt-register.tape) | Script source | — |

Error path: `grep -c` returning 0 would trigger the `|| echo 'FAIL: entry missing'` branch — demonstrated by the `PASS` outcome confirming it is present.

---

## Pass-50 finding closures

| Finding | AC | Resolution |
|---------|-----|-----------|
| CR-016 (MEDIUM) | AC-001 | 10ms → 50ms in `clones/armis.rs`, `clones/claroty.rs`, `clones/crowdstrike.rs` |
| CR-017 / M-50-001 (MEDIUM) | AC-002 | `is_real_org` guard added to `tags.rs:post_device_tag`, `tags.rs:delete_device_tag`, `alerts.rs:get_alerts` |
| CR-018 (MEDIUM) | AC-003 | nil-instance guard added to `detections.rs:list_detection_ids`, `detections.rs:get_detection_summaries` |
| CR-020 (LOW) | AC-004 | Deviation comment above `#[doc(hidden)]` on `validate_spec_path` in `validator.rs` |
| L-50-004 (LOW) | AC-005 | TD-W3-POLL-NOTIFY-001 row in `.factory/tech-debt-register.md` |

## Follow-up: TD-W3-POLL-NOTIFY-001

`TD-W3-POLL-NOTIFY-001` is filed in `.factory/tech-debt-register.md` at row 173. It tracks
replacement of the 50ms busy-wait loops in all four `poll_test_hook` variants with
`tokio::sync::Notify`-based cancellation. Severity P3, Wave 4 candidate. Non-blocking for
current Wave 3 gate.

## Test suite totals (HEAD 652409cf)

| Test suite | Tests | Result |
|-----------|-------|--------|
| `cr017_tag_alert_org_id_guard` (prism-dtu-armis) | 8 | all pass |
| `cr018_detections_org_id_guard` (prism-dtu-crowdstrike) | 6 | all pass |
| Full target suite (14 tests) | 14 | all pass, 0 regressions |
