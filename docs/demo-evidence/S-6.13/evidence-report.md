# Demo Evidence Report — S-6.13

**Story:** S-6.13 — prism-dtu-jira: DTU for Jira REST API v3 — L3 (behavioral)  
**Branch:** `feature/S-6.13-dtu-jira`  
**Impl commit:** `0e1dea7a` (Red Gate / fully implemented by stub-author; Step 4 skipped)  
**Tests:** 28 fidelity tests — 1086 PASS / 0 FAIL (workspace)  
**BCs:** None (DTU is test infrastructure, not a product BC boundary)  
**Recorded:** 2026-04-25  
**Tool:** VHS 0.10.0  

---

## AC Coverage Table

| AC | Description | Test(s) | Recording | Status |
|----|-------------|---------|-----------|--------|
| AC-1 | `POST /rest/api/3/issue` → 201, key `PROJ-NNNN`, `/dtu/issues` shows `Open` | `test_full_lifecycle_create_comment_transition` | [ac-1-create-issue.gif](ac-1-create-issue.gif) | PASS |
| AC-2 | `POST comment` → 201; `GET issue` shows `fields.comment.total: 1` | `test_get_issue_response_has_self_field_and_status_id` | [ac-2-get-issue.gif](ac-2-get-issue.gif) | PASS |
| AC-3 | `GET /transitions` when Open → ids `"11"` (Start Progress) and `"31"` (Close) present | `test_open_transition_names_are_start_progress_and_close` | [ac-3-list-transitions.gif](ac-3-list-transitions.gif) | PASS |
| AC-4 | `POST transition id "11"` → 204; `GET issue` shows status `In Progress` | `test_inprogress_to_done_transition_id_21` | [ac-4-execute-transition.gif](ac-4-execute-transition.gif) | PASS |
| AC-5 | Invalid transition from Done → 400 `{"errorMessages": ["Invalid transition id"]}` | `test_inprogress_to_inprogress_is_invalid_transition` | [ac-5-add-comment.gif](ac-5-add-comment.gif) | PASS |
| AC-6 | Missing `fields.project.key` → 400 `{"errors": {"project": "required"}}` | `test_missing_project_key_returns_400` | [ac-6-missing-project-key-400.gif](ac-6-missing-project-key-400.gif) | PASS |
| AC-7 | Unknown `issuetype.name: "Feature"` → 400 `{"errors": {"issuetype": "unknown"}}` | `test_unknown_issuetype_returns_400` | [ac-7-unknown-issuetype-400.gif](ac-7-unknown-issuetype-400.gif) | PASS |
| AC-8 | No `Authorization: Basic` header → 401 `{"errorMessages": ["Basic authentication required"]}` | `test_missing_auth_returns_401`, `test_bearer_scheme_returns_401`, `test_invalid_base64_in_basic_auth_returns_401` | [ac-8-missing-auth-401.gif](ac-8-missing-auth-401.gif) | PASS |
| AC-9 | `GET /issue/UNKNOWN-999` → 404 `{"errorMessages": ["Issue does not exist"]}` | `test_unknown_issue_key_returns_404`, `test_execute_transition_on_missing_issue_returns_404`, `test_list_transitions_on_missing_issue_returns_404`, `test_add_comment_on_missing_issue_returns_404` | [ac-9-unknown-issue-key-404.gif](ac-9-unknown-issue-key-404.gif) | PASS |
| AC-10 | `FailureMode::RateLimit` → 429; issue NOT persisted (EC-006 atomicity) | `test_ac10_rate_limit_429_returned_and_issue_not_persisted` | [ac-10-rate-limit-429.gif](ac-10-rate-limit-429.gif) | PASS |

---

## Edge Case Coverage (from fidelity tests)

| EC | Description | Test | Status |
|----|-------------|------|--------|
| EC-001 | Extra unknown fields silently ignored | `test_ec001_extra_fields_in_create_body_are_ignored` | PASS |
| EC-002 | `GET /transitions` when Done → empty list | `test_full_lifecycle_create_comment_transition` (inline) | PASS |
| EC-003 | Two creates same project → incremented keys (PROJ-1000, PROJ-1001) | `test_ec003_sequential_creates_get_incremented_keys` | PASS |
| EC-004 | Comment on Done issue → 201 (Jira permits comments on closed issues) | `test_ec004_comment_on_done_issue_returns_201` | PASS |
| EC-005 | `reset()` clears registry; subsequent GET → 404; counter resets to 1000 | `test_reset_clears_all_issues` | PASS |
| EC-006 | Rate limit → 429; issue NOT persisted (atomicity) | `test_ac10_rate_limit_429_returned_and_issue_not_persisted` | PASS (covered in AC-10 demo) |

---

## File Inventory

| File | Size | Purpose |
|------|------|---------|
| `ac-1-create-issue.tape` | 593 B | VHS script |
| `ac-1-create-issue.gif` | 147 KB | AC-1 recording |
| `ac-2-get-issue.tape` | 576 B | VHS script |
| `ac-2-get-issue.gif` | 145 KB | AC-2 recording |
| `ac-3-list-transitions.tape` | 606 B | VHS script |
| `ac-3-list-transitions.gif` | 148 KB | AC-3 recording |
| `ac-4-execute-transition.tape` | 576 B | VHS script |
| `ac-4-execute-transition.gif` | 145 KB | AC-4 recording |
| `ac-5-add-comment.tape` | 599 B | VHS script |
| `ac-5-add-comment.gif` | 152 KB | AC-5 recording |
| `ac-6-missing-project-key-400.tape` | 585 B | VHS script |
| `ac-6-missing-project-key-400.gif` | 142 KB | AC-6 recording |
| `ac-7-unknown-issuetype-400.tape` | 592 B | VHS script |
| `ac-7-unknown-issuetype-400.gif` | 147 KB | AC-7 recording |
| `ac-8-missing-auth-401.tape` | 685 B | VHS script |
| `ac-8-missing-auth-401.gif` | 184 KB | AC-8 recording |
| `ac-9-unknown-issue-key-404.tape` | 769 B | VHS script |
| `ac-9-unknown-issue-key-404.gif` | 209 KB | AC-9 recording |
| `ac-10-rate-limit-429.tape` | 590 B | VHS script |
| `ac-10-rate-limit-429.gif` | 151 KB | AC-10 recording |
| `evidence-report.md` | — | This file |

**Total: 10 tape + 10 gif + 1 report = 21 files**

---

## Notes

- Step 4 (implementer dispatch) was skipped — all routes and state machine were fully
  implemented by the stub-author in commit `0e1dea7a`. The 28 fidelity tests passed
  GREEN-BY-DESIGN prior to any Step 5 recording.
- S-6.13 has **no BCs** — this crate is test infrastructure only. The architecture anchor
  is `dtu-assessment.md §3.5.3`.
- All recordings show `cargo test` invocations with `--features dtu` against
  `prism-dtu-jira`'s `tests/fidelity.rs`.
- VHS settings: FiraCode Nerd Font Mono, 14pt, 1000x600, Dracula theme, 20px padding.
