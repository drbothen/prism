# Security Review Findings — W3-FIX-CODE-005

**Reviewer:** pr-manager direct diff analysis (Step 4 — vsdd-factory:security-review skill unavailable)
**Date:** 2026-05-01
**PR:** #123 — feature/W3-FIX-CODE-005 → develop
**Diff reviewed:** git diff develop..feature/W3-FIX-CODE-005 — full diff examined inline

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 0 |
| MEDIUM | 0 |
| LOW | 0 |

**Verdict: CLEAN**

## Files Reviewed

| File | Change Type | Security Assessment |
|------|-------------|---------------------|
| `crates/prism-dtu-harness/src/clones/armis.rs` | Poll cadence 10ms→50ms | Zero security impact (pure timing constant) |
| `crates/prism-dtu-harness/src/clones/claroty.rs` | Poll cadence 10ms→50ms | Zero security impact (pure timing constant) |
| `crates/prism-dtu-harness/src/clones/crowdstrike.rs` | Poll cadence 10ms→50ms | Zero security impact (pure timing constant) |
| `crates/prism-dtu-armis/src/routes/tags.rs` | is_real_org guard added to post_device_tag + delete_device_tag | Additive restriction — closes A01 gap; no bypass path |
| `crates/prism-dtu-armis/src/routes/alerts.rs` | is_real_org guard added to get_alerts | Additive restriction — closes A01 gap; no bypass path |
| `crates/prism-dtu-armis/src/routes/devices.rs` | is_real_org guard added to get_device_activity + get_device_risk | Additive restriction — closes A01 gap; no bypass path |
| `crates/prism-dtu-crowdstrike/src/routes/detections.rs` | nil-instance guard added to list_detection_ids + get_detection_summaries | Additive restriction — closes A01 gap; no bypass path |
| `crates/prism-customer-config/src/validator.rs` | Doc comment only | Zero runtime change |
| `.factory/tech-debt-register.md` | Documentation only | N/A |

## Analysis Notes

- `validate_org_id` (Armis): missing header → 401; non-UUID → 401; UUID mismatch → 401. No injection vector. Correct UUID parsing via `uuid::Uuid::parse_str`.
- `validate_org_id` (CrowdStrike): identical pattern. Guard fires before any state access.
- Dual-mode Armis sentinel (`DTU_DEFAULT_INSTANCE_ORG_ID = 00000000-0000-7000-8000-0000000000AA`) and CrowdStrike nil-UUID sentinel (`00000000-0000-0000-0000-000000000000`) are correctly scoped to DTU simulation code; not reachable from production data paths.
- The `pub` visibility on `validate_spec_path` is pre-existing (documented by CR-020 comment). The function validates path traversal, not exposes it. Not introduced by this PR.
- OWASP A01 (Broken Access Control): risk REDUCED by this PR. CR-017/CR-018 close guard-bypass asymmetries.
- All other OWASP Top 10 items: not applicable (no new network calls, no new data paths, no user-controlled input paths introduced).
