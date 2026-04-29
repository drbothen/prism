---
document_type: red-gate-log
level: ops
version: "1.0"
status: merged
producer: test-writer
timestamp: "2026-04-29T19:47:53Z"
phase: 3
inputs: []
input-hash: "[live-state]"
traces_to: ""
stub_architect_agent: "[wave-3-phase-c-batch-3]"
stub_compile_verified: true
test_writer_agent: "[wave-3-phase-c-batch-3]"
red_gate_verified: true
---

# Red Gate Log: S-3.2.07 — Jira OrgId Ingress Tagging

## Summary

| Story | Tests Written | All Fail (Red)? | Gate |
|-------|--------------|-----------------|------|
| S-3.2.07 | 8 (org_tagging) | yes — red gate confirmed pre-impl | PASS — PR #91 merged (9c1ecec0) |

## Stubs Created

### S-3.2.07: prism-dtu-jira shared-mode OrgId tagging

- `fn capture_issue_tagged(org_id: OrgId, ...) -> Result<IssueRecord>` — stub returning unimplemented!(); gated #[cfg(feature = "dtu")]
- `IssueRecord.org_id: Option<String>` — field stub added, default None

## Red Gate Verification

### S-3.2.07

- AC-001 (BC-3.2.004): org_tagging::test_org_id_populated — FAIL (expected)
- AC-002 (BC-3.2.004): org_tagging::test_dedup_key_isolation — FAIL (expected)
- AC-003 (BC-3.2.005): org_tagging::test_dtu_mode_shared_const — FAIL (expected)
- AC-004 (BC-3.2.004): org_tagging::test_capture_issue_tagged_signature — FAIL (expected)
- AC-005 (BC-3.2.004): org_tagging::test_no_org_id_leakage — FAIL (expected)
- AC-006 (BC-3.2.004): org_tagging::test_backward_compat_default_none — FAIL (expected)
- AC-007 (BC-3.2.005): org_tagging::test_prism_core_optional_dep — FAIL (expected)
- AC-008 (BC-3.2.004): org_tagging::test_forbidden_deps_absent — FAIL (expected)

## Regression Check

| Existing Tests | Status |
|---------------|--------|
| 1619 pre-existing workspace tests (post-S-3.2.06) | all pass — 0 regressions |

## Hand-Off to Implementer

- Stories ready for implementation: S-3.2.07 (COMPLETE — merged PR #91, SHA 9c1ecec0)
- Implementation guidance: mirror S-3.2.06 pattern; X_DTU_MODE naming avoids prism-core enforcement-test scanner conflict (D-153)
- Review note: PRF-001 (stale doc comment nit) + PRF-002 (unused dev-dep) accepted as cosmetic, no fix required
