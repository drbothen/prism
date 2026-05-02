# Security Review Findings — W3-FIX-SEC-001

**Reviewer:** vsdd-factory:security-review (fresh-context dispatch, step 4)
**PR:** #113 — feature/W3-FIX-SEC-001 vs develop
**Date:** 2026-05-01
**Verdict:** RESOLVED — HIGH-001 fixed in commit e8ca86ae; all tests pass; APPROVE

---

## Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | — |
| HIGH | 1 | RESOLVED — commit e8ca86ae |
| MEDIUM | 0 | — |
| LOW | 0 | — |

---

## HIGH-001: CrowdStrike write/read endpoints bypass validate_org_id (W3-FIX-SEC-001 incomplete)

**Files:**
- `crates/prism-dtu-crowdstrike/src/routes/hosts.rs:294` — `get_host_details`
- `crates/prism-dtu-crowdstrike/src/routes/writes.rs:99` — `device_actions`
- `crates/prism-dtu-crowdstrike/src/routes/writes.rs:237` — `patch_detections`

**Severity:** HIGH
**Confidence:** 9/10
**CWE:** CWE-639 (Authorization Bypass Through User-Controlled Key)
**OWASP:** A01:2021 — Broken Access Control

**Description:**

`validate_org_id` was wired into `list_host_ids` (`GET /devices/queries/devices/v1`)
but NOT into the following three handlers:

1. `get_host_details` (`GET /devices/entities/devices/v2`, hosts.rs:294) — calls
   `extract_org_id`, the legacy fallback that accepts any caller-supplied UUID without
   comparing to `state.instance_org_id`.

2. `device_actions` (`POST /devices/entities/devices-actions/v2`, writes.rs:99) —
   calls `extract_org_id`; writes ContainmentStatus records keyed by `(org_id, device_id)`.

3. `patch_detections` (`PATCH /detects/entities/detects/v2`, writes.rs:237) — calls
   `extract_org_id`; writes detection status records keyed by `(org_id, detection_id)`.

The containment store and detection store are keyed by `(OrgId, ...)`, so a spoofed
`X-Org-Id` enables reading Org A's containment state or writing into Org A's
containment/detection namespace from Org B's clone port.

**Exploit scenario:**

In a network-harness multi-tenant run (Org A clone + Org B clone on separate ports):

1. Attacker on Org B's port: `GET /devices/entities/devices/v2?ids=h-001`
   with `-H "X-Org-Id: <OrgA-UUID>"` → reads Org A's containment state for h-001.
2. Attacker: `POST /devices/entities/devices-actions/v2?action_name=contain`
   with body `{"ids":["h-001"]}` and `-H "X-Org-Id: <OrgA-UUID>"` → writes
   "contained" into Org A's containment store, corrupting test isolation state.

This is the exact attack surface SEC-001 was designed to close. `list_host_ids` is
protected; the read-detail and both write endpoints are not.

**Tool evidence (grep output confirms):**
```
hosts.rs:147:   if let Err((status, body)) = validate_org_id(&headers, state.instance_org_id)  ← PROTECTED
hosts.rs:294:   let org_id = extract_org_id(&headers);  ← NOT PROTECTED
writes.rs:99:   let org_id = extract_org_id(&headers);  ← NOT PROTECTED
writes.rs:237:  let org_id = extract_org_id(&headers);  ← NOT PROTECTED
```

**Required fix:**

Apply the same guard used in `list_host_ids` (hosts.rs:146-150) to all three missing
handlers. Pattern:

```rust
// In get_host_details (hosts.rs), after check_auth:
if state.instance_org_id != OrgId::from_uuid(uuid::Uuid::nil()) {
    if let Err((status, body)) = validate_org_id(&headers, state.instance_org_id) {
        return (status, body).into_response();
    }
}

// In device_actions and patch_detections (writes.rs), after check_auth:
// Same pattern — CrowdStrike state needs instance_org_id accessible; check
// CrowdstrikeState struct for field presence.
```

**Routing:** implementer (prism-dtu-crowdstrike, hosts.rs + writes.rs)

---

## Gate-Step-D Carry-Over Context

The gate-step-d review (wave-3-multi-tenant) identified SEC-001 as HIGH (CWE-287/639).
This PR resolves the `list_host_ids` path but leaves three additional CrowdStrike
endpoints unguarded. The four other DTU clones (Claroty, Cyberint, Armis) are correctly
handled per their respective auth models.

---

_Generated: 2026-05-01 | Reviewer: vsdd-factory:security-review_
