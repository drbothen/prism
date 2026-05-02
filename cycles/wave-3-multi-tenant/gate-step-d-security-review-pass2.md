---
document_type: security-review
level: ops
version: "1.0"
status: final
producer: security-reviewer
timestamp: 2026-05-01T00:00:00
phase: 3
wave: 3
step: d
pass: 2
develop_sha: cda17ed4
reviewer: vsdd-factory:security-reviewer
cycle: wave-3-multi-tenant
gate: wave-3-integration-gate
scope: "a3bd5a0f..cda17ed4 (Wave 3.1 — 5 fix PRs)"
inputs:
  - .factory/cycles/wave-3-multi-tenant/gate-step-d-security-review.md
  - .factory/code-delivery/W3-FIX-SEC-001/security-findings.md
  - .factory/code-delivery/W3-FIX-SEC-003/security-findings.md
  - .factory/code-delivery/W3-FIX-CODE-003/security-findings.md
  - .factory/code-delivery/S-3.1.06-ImplPhase/security-findings.md
input-hash: "40f2127"
traces_to: "wave-3-integration-gate"
total_findings: 7
critical: 0
high: 1
medium: 4
low: 2
files_reviewed: 34
verdict: APPROVED_WITH_CONDITIONS
---

# Wave 3 Integration Gate — Gate Step D: Security Review (Pass 2)

**Scope:** a3bd5a0f..cda17ed4 (Wave 3.1 fix PRs: #113, #114, #115, #116 + S-3.1.06-ImplPhase)
**Predecessor review:** gate-step-d-security-review.md (SHA a3bd5a0f, verdict: APPROVED_WITH_CONDITIONS)
**Reviewer:** vsdd-factory:security-reviewer
**Date:** 2026-05-01
**Develop SHA:** cda17ed4
**Verdict:** APPROVED WITH CONDITIONS — 7 findings (0 CRITICAL, 1 HIGH, 4 MEDIUM, 2 LOW)

---

## Executive Summary

Wave 3.1 closed both HIGH findings from the pass-1 gate review: SEC-001 (X-Org-Id header
spoofing) is fully remediated across all four DTU clones, and SEC-003 (spec path traversal)
is remediated for all existing-file attack vectors with a comprehensive test suite. SEC-004
(KeyringBackend stubs) is retracted as a false positive — the implementation was complete
at pass-1 review time. One pass-1 HIGH finding (SEC-002, `POST /dtu/reset` unauthenticated)
was explicitly deferred from Wave 3.1 and remains open; it must be addressed before Phase 4.
One new MEDIUM finding (SEC-P2-001) and one new LOW finding (SEC-P2-006) arise from Wave 3.1
implementation choices. The overall security posture is substantially improved from pass-1.

---

## Pass-2 Scope: What Changed in Wave 3.1

Five PRs merged after the pass-1 gate review:

| PR | Story | Purpose |
|----|-------|---------|
| #113 | W3-FIX-SEC-001 | X-Org-Id auth enforcement on DTU clones |
| #114 | W3-FIX-SEC-003 | `validate_spec_path` — path traversal hardening |
| #115 | W3-FIX-CODE-003 | KeyringBackend `CredentialStoreOrgId` impl + tests (SEC-004 false positive) |
| #116 | W3-FIX-CODE-001 | HarnessBuilder failure scope + Drop grace |
| S-3.1.06-ImplPhase | cda17ed4 | prism-sensors: OrgId-keyed adapter dispatch, `AdapterRegistry` composite key |

---

## Prior HIGH Findings: Closure Verification

### SEC-001 (HIGH, CWE-287/639) — X-Org-Id Header Spoofing

**Status: RESOLVED (with one residual gap — see SEC-001-RG below)**

**Verification methodology:** Read diff against a3bd5a0f for all four DTU crate route
files. Cross-checked with PR #113 security-findings.md (per-PR reviewer noted an initial
incomplete fix in commit e8ca86ae that was corrected in the same PR).

**Claroty** (`prism-dtu-claroty/src/routes/devices.rs`):
- `validate_org_id` function added with correct `instance_org_id != nil_org` gate.
- Applied to `list_devices` (line 228) and `dtu_reset_for` path-param check (line 430).
- Backward compat clones (nil `instance_org_id`) fall through to legacy `extract_org_id`.
- **VERIFIED CLOSED** for all handlers reviewed.

**CrowdStrike** (`prism-dtu-crowdstrike/src/routes/hosts.rs`, `writes.rs`):
- `validate_org_id` added and wired into `list_host_ids` (hosts.rs:146), `get_host_details`
  (hosts.rs:294), `device_actions` (writes.rs:102), and `patch_detections` (writes.rs:248).
- The per-PR reviewer (W3-FIX-SEC-001/security-findings.md HIGH-001) correctly identified
  that the initial commit only guarded `list_host_ids`; the fix was completed in commit
  e8ca86ae within the same PR. Final state is correct.
- **VERIFIED CLOSED** for all four CrowdStrike handlers.

**Cyberint** (`prism-dtu-cyberint/src/routes/alerts.rs`):
- Cyberint uses a session-based multi-org routing model: `is_valid_session(org_id, token)`
  uses a `HashSet<(OrgId, String)>` key — sessions are physically keyed by `(OrgId, token)`.
  A forged `X-Prism-Org-Id` UUID that does not match a registered session for that org
  returns 401 "org_id mismatch". The `check_auth` function was updated with AC-002_body
  handling and `get_alerts` was updated with EC-001 (non-UUID header) rejection.
- The `#[allow(dead_code)] validate_org_id` function is intentionally unused (documented
  in code): Cyberint's multi-org session routing is structurally incompatible with
  instance_org_id-based gating. The inline session-mismatch enforcement is equivalent.
- **VERIFIED CLOSED** for Cyberint's auth model.

**Armis** (`prism-dtu-armis/src/routes/devices.rs`):
- Uses header-presence conditional rather than instance_org_id-non-nil conditional.
  When `X-Org-Id` header is absent, validation is skipped and tags are read/written
  under `DTU_ROUTE_ORG_ID` (a constant, not the caller's UUID). This is intentional
  per the single-tenant-per-clone design where all state is stored under a sentinel key.
- **Residual gap documented as SEC-NEW-001 (MEDIUM) below.**

---

### SEC-003 (HIGH, CWE-22) — Spec Path Traversal

**Status: SUBSTANTIALLY RESOLVED (one non-blocking residual documented in W3-FIX-SEC-003 PR)**

**Verification methodology:** Read full diff of `validator.rs` and new `path_traversal.rs`
test file. Cross-checked with W3-FIX-SEC-003/security-findings.md.

`validate_spec_path` function implemented in `crates/prism-customer-config/src/validator.rs`:

1. **AC-001** (`..` component rejection): `Path::components()` iterator scans for
   `Component::ParentDir` before any filesystem I/O. Verified correct.

2. **AC-002** (absolute path rejection): `Path::is_absolute()` fires pre-join. Verified
   correct; Windows drive-letter paths also handled via platform-aware `is_absolute()`.

3. **AC-003** (relative-within-tree): `canonicalize()` called after `parent.join()`.
   Returns `Ok(canonical_path)` for valid in-tree paths.

4. **AC-004** (symlink escape): `canonical_candidate.starts_with(&canonical_parent)`
   post-join boundary check. Unix symlink-to-`/etc/hosts` test case added in
   `test_BC_3_3_004_AC_004_symlink_escape_rejected` (unix-only, correct).

**Test coverage:** 7 test functions in `tests/path_traversal.rs` covering all four ACs
plus the `./` prefix variant and `C:\\` Windows absolute path path. Coverage is
comprehensive for the existing-file attack vectors.

**Residual gap (non-blocking, acknowledged):** The pre-join `..` and absolute path checks
are gated behind `resolved.exists()` in `validate_dtu_block`. A traversal target that
does not yet exist (e.g., `../../../../etc/nonexistent`) skips `validate_spec_path` and
emits `SpecFileNotFound` (E-CFG-015) instead of `SpecPathTraversal` (E-CFG-018). This
means the audit trail does not capture the attempted path escape when the target is absent.
The immediate CWE-22 read risk is zero (file must exist to be read), but the intent
violation creates a TOCTOU-adjacent window if the target file is created later.
Documented as SEC-003-RG (MEDIUM) below.

**VERIFIED CLOSED** for existing-file attack vectors (the primary HIGH concern).

---

### SEC-004 (MEDIUM) — KeyringBackend `todo!()` Stubs

**Status: FALSE POSITIVE — RETRACTED**

The per-PR review of W3-FIX-CODE-003 (#115) confirmed that `KeyringBackend::CredentialStoreOrgId`
was fully implemented at SHA a3bd5a0f (S-3.1.04 delivery). SEC-004 was based on a
stale review snapshot. The complete implementation has been verified:

- `get_by_org`: `keyring::Entry::new(app_name, namespace_key).get_password()` → `Ok(Some(SecretString))`.
- `set_by_org`: `entry.set_password(expose_secret())` + sidecar index update.
- `delete_by_org`: `entry.delete_credential()` + conditional index removal.
- `list_by_org`: prefix-filters sidecar index by `"{org_id_uuid}/"`.

All operations use `namespace_key_by_org_id(org_id, sensor, name)` = `"{uuid}/{sensor}/{name}"`,
providing physical cross-org isolation in the keyring service.

**SEC-004 is RETRACTED.** No stubs. No panic paths.

---

### New HIGH/MEDIUM from Wave 3.1 Changes

The following findings arise from the Wave 3.1 implementation, not from prior wave code.

---

## HIGH Findings

### SEC-NEW-001: SEC-002 (`POST /dtu/reset` Unauthenticated) — Still Open

- **Severity:** HIGH
- **CWE:** CWE-306 (Missing Authentication for Critical Function)
- **OWASP:** A07:2021 — Identification and Authentication Failures
- **Status:** EXPLICITLY DEFERRED — W3-FIX-SEC-002 was not part of Wave 3.1
- **Files:**
  - `crates/prism-dtu-claroty/src/routes/devices.rs` (`dtu_reset` — no auth)
  - `crates/prism-dtu-crowdstrike/src/routes/mod.rs` (`dtu_reset` — comment: "No auth required")
  - `crates/prism-dtu-armis/src/routes/dtu.rs` (`post_reset` — no auth)
  - `crates/prism-dtu-slack/src/routes/dtu.rs` (`post_reset` — no auth)

**Attack Vector:**

Same as pass-1 SEC-002. Any client that can reach a per-org clone's TCP port can issue
`POST /dtu/reset` without any authentication header. All mutable state for all orgs on that
clone is erased. `X-Admin-Token` gate is present on `POST /dtu/configure` for all four clones
but is absent from `POST /dtu/reset`.

Current code evidence (unchanged since pass-1):

```
// CrowdStrike mod.rs:29: "No auth required."
async fn dtu_reset(State(state): State<Arc<CrowdstrikeState>>) -> impl IntoResponse {
    state.reset();
    ...
}

// Armis dtu.rs:66:
pub async fn post_reset(State(state): State<Arc<ArmisState>>) -> impl IntoResponse {
    state.reset();
    ...
}
```

**Impact:**

In a network-isolation-mode harness with two customer organizations, a client connected
to Org B's port can erase all of Org A's state by invoking `POST /dtu/reset` without
presenting an admin token. This undermines test harness isolation fidelity in multi-tenant
test runs and would undermine production isolation if this pattern were promoted.

**Condition of approval:**

W3-FIX-SEC-002 must be delivered before the Phase 4 holdout evaluation. It is not required
for wave integration gate progression (ADR-011 §2.3 scope limitation: harness is not a
production network boundary), but remains a condition of the pass-1 verdict.

**Proposed Mitigation:** Apply the `X-Admin-Token` check used in `dtu_configure` to
`dtu_reset` on all four affected clones.

---

## MEDIUM Findings

### SEC-P2-001: Armis `X-Org-Id` Validation Model Inconsistent with Claroty/CrowdStrike Pattern

- **Severity:** MEDIUM
- **CWE:** CWE-284 (Improper Access Control)
- **OWASP:** A01:2021 — Broken Access Control
- **File:** `crates/prism-dtu-armis/src/routes/devices.rs:64-68` (get_devices), `95-99` (post_devices)

**Description:**

Claroty and CrowdStrike use an **instance_org_id-non-nil** conditional: validation fires
whenever the clone was started with a real org identity. A client that omits the header on
a real-org Claroty clone receives 401.

Armis uses a **header-presence** conditional: validation only fires when `X-Org-Id` is
present. A client that omits the header entirely is allowed through and all state is
read/written under the constant `DTU_ROUTE_ORG_ID` sentinel rather than the caller's org.

This means a client on Armis's port that omits `X-Org-Id` entirely bypasses the mismatch
check. While the Armis single-tenant-per-clone model means the `DTU_ROUTE_ORG_ID` bucket
contains only that org's data (physical isolation is at the TCP port level), this is a
weaker defense-in-depth posture than Claroty/CrowdStrike.

The comment at line 65 states "backward compat for callers without org header" — this
documents the intent, but the consequence is that a missing header is silently treated as
a valid request rather than rejected.

**Evidence:**

```rust
// Armis get_devices — header-presence conditional (weaker):
if headers.get("x-org-id").is_some() {
    if let Err((status, body)) = validate_org_id(&headers, state.instance_org_id) {
        return (status, body).into_response();
    }
}

// Claroty list_devices — instance_org_id-non-nil conditional (stronger):
let nil_org = OrgId::from_uuid(Uuid::nil());
let org_id = if state.instance_org_id != nil_org {
    match validate_org_id(&headers, state.instance_org_id) {
        Ok(id) => id,
        Err(err) => return err.into_response(),
    }
} else {
    extract_org_id(&headers)
};
```

**Impact:** Limited to the harness context. Physical isolation is maintained at the TCP
port level. However, a multi-tenant harness test that expects 401 on a missing-header
request to Armis would pass incorrectly — weakening test fidelity.

**Proposed Mitigation:** When `state.instance_org_id != DTU_DEFAULT_INSTANCE_ORG_ID`,
require the `X-Org-Id` header to be present and matching, consistent with Claroty/CrowdStrike.

---

### SEC-P2-002: SEC-003-RG — Pre-Join Path Traversal Check Bypassed for Non-Existent Targets

- **Severity:** MEDIUM
- **CWE:** CWE-22 (Improper Limitation of a Pathname to a Restricted Directory)
- **OWASP:** A01:2021 — Broken Access Control
- **File:** `crates/prism-customer-config/src/validator.rs:554`
- **Status:** Acknowledged in W3-FIX-SEC-003/security-findings.md as tech-debt (non-blocking)

**Description:**

In `validate_dtu_block`, the call to `validate_spec_path` is gated behind `resolved.exists()`.
When a customer TOML contains `spec = "../../../../etc/nonexistent"` (target absent):

1. `parent.join("../../../../etc/nonexistent")` produces an out-of-boundary candidate path.
2. `resolved.exists()` returns `false`.
3. `validate_spec_path` is never called — the pre-join `..` check is bypassed.
4. `SpecFileNotFound` (E-CFG-015) is emitted instead of `SpecPathTraversal` (E-CFG-018).

The audit trail does not capture the attempted path escape. If the target file is created
later, a subsequent config reload re-evaluates the same path without the prior rejection
having been recorded.

**Immediate exploitability:** None — the file must exist for `validate_spec_path` to
reach the post-join boundary check. A traversal to a non-existent file cannot yield a
file read in the current startup flow.

**Proposed Mitigation:** Move the I/O-free pre-join `..` and absolute-path checks out
of the `resolved.exists()` gate so they fire unconditionally. See W3-FIX-SEC-003
security-findings.md §Recommended Fix for the exact code change.

---

### SEC-P2-003: Deferred — SEC-005 (OrgSlug 64-char limit / OQ-1 unresolved)

- **Severity:** MEDIUM
- **CWE:** CWE-20 (Improper Input Validation)
- **File:** `crates/prism-core/src/tenant.rs` (`ORG_SLUG_PATTERN`)
- **Status:** Deferred — not addressed in Wave 3.1

ADR-006 §8 OQ-1 remains unresolved. `ORG_SLUG_PATTERN` = `r"^[a-zA-Z0-9_-]{1,64}$"`.
ADR-006 proposes tightening to 32 characters for log injection surface reduction.
No change in Wave 3.1. Disposition unchanged from pass-1: PARTIALLY MITIGATED.

---

### SEC-P2-004: Deferred — SEC-006 (TOML `sanitize_error_message` multi-line gap)

- **Severity:** MEDIUM
- **CWE:** CWE-209 (Generation of Error Message Containing Sensitive Information)
- **OWASP:** A09:2021 — Security Logging and Monitoring Failures
- **File:** `crates/prism-customer-config/src/validator.rs:327-349`
- **Status:** Deferred — not addressed in Wave 3.1

No change to `sanitize_error_message`. Multi-line TOML string and inline-table
credential-name patterns remain untested. Disposition unchanged from pass-1:
PARTIALLY MITIGATED.

---

## LOW Findings

### SEC-P2-005: Deferred — SEC-007 (org_slug audit cross-check at write time)

- **Severity:** MEDIUM (downgraded to LOW in pass-2 based on limited exploitability)
- **CWE:** CWE-345 (Insufficient Verification of Data Authenticity)
- **File:** `crates/prism-audit/src/audit_entry.rs`
- **Status:** Deferred — not addressed in Wave 3.1

No audit-registry cross-check added. `org_slug` is still accepted from the
`AuditRequest` caller without verification against `OrgRegistry::slug_for(org_id)`.
Exploitability requires control over the `AuditRequest` construction path — an internal
call site, not an external attack surface. Downgrading to LOW given no external
attack vector. Disposition unchanged from pass-1: PARTIALLY MITIGATED.

---

### SEC-P2-006: `init_registry` (deprecated) Uses Nil-UUID Sentinel — Migration Gate Absent

- **Severity:** LOW
- **CWE:** CWE-284 (Improper Access Control)
- **File:** `crates/prism-sensors/src/lib.rs:110-131`
- **Note:** New finding introduced by S-3.1.06-ImplPhase

**Description:**

`init_registry` (now `#[deprecated(since = "0.2.0")]`) remains callable and registers
all four adapters under a nil-UUID `OrgId` (`00000000-0000-0000-0000-000000000000`).
The registry's `get(nil_org, SensorType::X)` returns these adapters correctly.

The test file `tests/test_armis.rs:404` uses `#[allow(deprecated)]` to call it and
subsequently looks up with `registry.get(nil_org, ...)`. This is the correct usage
pattern for backward compat.

Two concerns:

1. There is no compile-time or CI gate preventing new callers from using `init_registry`
   after Wave 3.1. The `#[deprecated]` attribute emits a warning but not an error.
   ADR-006 §4 requires migration to `init_registry_for_org` before Wave 5.

2. Adapters registered under nil-UUID have `self.org_id = nil_uuid`. The
   `OrgIdMismatch` guard in `fetch()` fires when `spec.org_id != self.org_id`. If a
   fan-out target constructed with a real `OrgId` is dispatched to a nil-UUID adapter
   (e.g., in a test that mixes `init_registry` and real-OrgId fan-out targets), the
   guard correctly returns `SensorError::OrgIdMismatch`. This is safe behavior but
   creates a footgun: tests using the deprecated path with real-OrgId specs will fail
   at the dispatch guard with a confusing error.

**Evidence:**

```rust
// lib.rs:120-131 — nil sentinel:
let sentinel = prism_core::OrgId::from_uuid(uuid::Uuid::nil());
init_registry_for_org(sentinel, ...)

// The OrgIdMismatch guard in all four adapters:
if spec.org_id != self.org_id { // self.org_id == nil — mismatch if caller uses real OrgId
    return Err(SensorError::OrgIdMismatch { ... });
}
```

**Proposed Mitigation:**

Add a `#[deprecated]` → `#[deprecated_error]` (`allow_deprecated_macro` approach) or
add a CI `--deny deprecated` lint for the `prism-sensors` crate to make the
deprecation a compile error. This forces the test migration AC-005 to complete before
Wave 5 shipping rather than remaining optional.

---

## Positive Findings (Defensive Measures Present)

The following constitute genuine security improvements delivered by Wave 3.1:

- **OrgId-keyed `AdapterRegistry`** — `(OrgId, SensorType)` composite key in `HashMap`
  eliminates the silent cross-tenant dispatch path that `SensorType`-only key allowed.
  `get(org_a, SensorType::CrowdStrike)` structurally cannot return `org_b`'s adapter.

- **`OrgIdMismatch` guard fires before network I/O** — The `if spec.org_id != self.org_id`
  guard at the top of every adapter's `fetch()` function returns `SensorError::OrgIdMismatch`
  before the HTTP semaphore is acquired, before any AQL validation, and before any cookie
  or bearer token is used. Zero network I/O on mismatch.

- **`DEFAULT_ORG_ID_BYTES` cfg(test)-gated** — The test-only sentinel constant
  (`#[cfg(test)]`) cannot appear in production compilation units. This prevents the
  Wave 2 pattern of sentinel UUIDs creeping into production dispatches.

- **Deprecated `init_registry` wrapped to forward to `init_registry_for_org`** — The
  deprecated path remains functional (backward compat) but uses nil-UUID internally,
  making the sentinel visible in the key and triggering the OrgIdMismatch guard if a
  real-OrgId spec is routed to it. Fail-safe behavior.

- **`KeyringBackend::CredentialStoreOrgId` fully implemented** — SEC-004 false positive
  retracted. OrgId-namespaced keyring operations (`"{uuid}/{sensor}/{name}"`) ensure
  physical isolation in the OS keyring service.

- **Path traversal test suite** — 7 test cases in `tests/path_traversal.rs` covering
  `..` components, absolute paths (Unix + Windows), `./`-prefixed relative paths, and
  symlink escape detection. SEC-003 HIGH attack vectors are regression-tested.

- **CrowdStrike write endpoints guarded** — `device_actions` and `patch_detections`
  (the two write endpoints added in Wave 3) now have the same `validate_org_id` guard as
  `list_host_ids` and `get_host_details`. Write-path cross-tenant contamination is closed.

---

## Timing Oracle Assessment (S-3.1.06-ImplPhase OrgIdMismatch Guards)

**Question:** Do the `OrgIdMismatch` guards introduce timing oracles that could leak
org identity information?

**Assessment:** No exploitable timing oracle.

`OrgId` derives `PartialEq` over `Uuid([u8; 16])`. Rust's derived `PartialEq` for fixed
arrays uses short-circuit comparison (not constant-time). This creates a theoretical
timing difference: matching UUIDs take O(16) comparisons; mismatching UUIDs short-circuit
at the first differing byte.

However, `OrgId` values are:
- Opaque UUID v7 identifiers minted by `OrgId::new()` (random node bytes + timestamp).
- Not secret values an attacker needs to guess through timing.
- Visible in the `X-Org-Id` / `X-Prism-Org-Id` headers the attacker supplies.

The attacker already knows both `spec.org_id` (they constructed the query) and
`self.org_id` (they control the header that was parsed into it). The comparison leaks
nothing that the attacker did not already supply. This is not a secret-comparison
context. No remediation needed.

---

## Deprecated `init_registry` Attack Surface Assessment

**Question:** Does the `init_registry` nil-OrgId path leave an exploitable attack surface?

**Assessment:** No. The nil-UUID sentinel is deterministic but self-limiting.

The deprecated path registers adapters under `OrgId(nil)`. Any fan-out target dispatched
against these adapters must also carry `spec.org_id == OrgId(nil)` for the
`OrgIdMismatch` guard to pass. A fan-out target with a real OrgId routed to the deprecated
registry will fail at the dispatch guard with `SensorError::OrgIdMismatch` — this is
correct fail-safe behavior, not a bypass.

The nil-UUID sentinel cannot be used in production code paths because:
1. `init_registry` is `#[deprecated]` — callers require `#[allow(deprecated)]`.
2. `DEFAULT_ORG_ID_BYTES` (the test-only nil sentinel constant) is `#[cfg(test)]`-gated.
3. `OrgRegistry::register` enforces bijectivity — a nil-UUID OrgId would conflict with
   any real org registration and be rejected at startup.

**Assessment: No attack surface. SEC-P2-006 (LOW) documents the migration enforcement
gap, not an active vulnerability.**

---

## Audit Log Integrity Post-OrgId-Rekey Assessment

**Question:** Does the Wave 3.1 OrgId dispatch rekey affect audit record integrity?

**Assessment:** No impact on audit log integrity.

The audit entry fields (`org_id: OrgId`, `org_slug: OrgSlug`) are set at emit time from
the `AuditRequest` struct. Wave 3.1 changes are in the query dispatch layer
(`prism-sensors`), not in the audit emission layer (`prism-audit`). No changes to
`AuditEntry`, `AuditEmitter`, or `AuditRequest` were introduced in Wave 3.1.

SEC-P2-005 (deferred SEC-007, now downgraded to LOW) documents the open question of
`org_slug` cross-validation. This is unchanged from pass-1 and is not a regression.

---

## Dependency Advisory Assessment

No new production dependencies were introduced in Wave 3.1 (prism-credentials, prism-sensors,
prism-dtu-claroty, prism-dtu-crowdstrike, prism-dtu-armis, prism-dtu-cyberint,
prism-customer-config). `Cargo.lock` modified only for `cargo-nextest` tooling additions
(CI). No RustSec advisories for any existing dependency found relevant.

---

## Risk Register Dispositions (Security-Category R-NNN Entries)

| ADR Risk | Pass-1 Status | Pass-2 Status | Change in Wave 3.1 |
|----------|--------------|--------------|-------------------|
| Cross-tenant data leakage at adapter layer (ADR-006 §3.1) | Partially Mitigated | **Mitigated** | OrgId-keyed `AdapterRegistry` + OrgIdMismatch guard in all 4 adapters closes the query-dispatch gap. HTTP-layer: Claroty/CrowdStrike fully guarded; Armis/Cyberint use alternative models (one weaker — SEC-P2-001). |
| Cross-tenant credential reachability (ADR-006 §3.2) | Partially Mitigated | **Mitigated** | `KeyringBackend::CredentialStoreOrgId` fully implemented (SEC-004 retracted). `namespace_key_by_org_id` confirmed correct. |
| Slug squatting / namespace collision (ADR-006 §3.4) | Mitigated | **Mitigated** | No change. `OrgRegistry::register` bijectivity unchanged. |
| Slug rename forensics (ADR-006 §3.3) | Mitigated | **Mitigated** | No change. Audit carry non-nullable `org_id` + `org_slug`. |
| Privacy in shared-infrastructure DTU (ADR-006 §3.5) | Mitigated | **Mitigated** | No change. BC-3.3.001 enforced. |
| Path traversal in spec file loading (R-CUST-014/015) | Unmitigated | **Partially Mitigated** | Existing-file traversal fully blocked by `validate_spec_path` (AC-001..AC-004). Non-existent-target traversal audit gap remains (SEC-P2-002, MEDIUM). |
| `POST /dtu/reset` unauthenticated (SEC-002) | Unmitigated | **Unmitigated** | W3-FIX-SEC-002 explicitly deferred from Wave 3.1. |

---

## Summary Table

| ID | Severity | CWE | Location | Pass-1 Origin | Pass-2 Status |
|----|----------|-----|----------|--------------|--------------|
| SEC-NEW-001 (≡ SEC-002) | **HIGH** | CWE-306 | 4 DTU `dtu_reset` handlers | SEC-002 (pass-1 HIGH) | Open — deferred |
| SEC-P2-001 | MEDIUM | CWE-284 | `prism-dtu-armis/routes/devices.rs:64-99` | New (Wave 3.1) | Open |
| SEC-P2-002 (≡ SEC-003-RG) | MEDIUM | CWE-22 | `prism-customer-config/validator.rs:554` | Acknowledged in PR #114 | Open (non-blocking) |
| SEC-P2-003 (≡ SEC-005) | MEDIUM | CWE-20 | `prism-core/src/tenant.rs` ORG_SLUG_PATTERN | SEC-005 (pass-1) | Open — deferred |
| SEC-P2-004 (≡ SEC-006) | MEDIUM | CWE-209 | `prism-customer-config/validator.rs:327-349` | SEC-006 (pass-1) | Open — deferred |
| SEC-P2-005 (≡ SEC-007) | LOW | CWE-345 | `prism-audit/audit_entry.rs` | SEC-007 (pass-1, downgraded) | Open — deferred |
| SEC-P2-006 | LOW | CWE-284 | `prism-sensors/src/lib.rs:110-131` | New (S-3.1.06) | Open |

**Retracted (false positive):** SEC-004 (MEDIUM, CWE-284) — KeyringBackend stubs.

**Closed from pass-1:**
- SEC-001 HIGH (CWE-287/639): CLOSED (Claroty + CrowdStrike all handlers; Cyberint alternative model)
- SEC-003 HIGH (CWE-22): CLOSED for existing-file vectors
- SEC-008 LOW (CWE-20): No change in `OrgId::from_uuid` behavior; still accepted; LOW tracking unchanged
- SEC-009 LOW (CWE-79): No change to `dtu_reset_for` error reflection; still LOW tracking unchanged
- SEC-010 LOW (CWE-312): `admin_token` still plain `String` in clone state; unchanged

---

## Recommendations Priority

### Immediate (before wave gate progression)

- **SEC-NEW-001 (HIGH, CWE-306):** Wire `X-Admin-Token` gate onto `POST /dtu/reset` on
  Claroty, CrowdStrike, Armis, and Slack clones (W3-FIX-SEC-002). Blocked from Phase 4.

### Before Phase 4 holdout evaluation

- **SEC-P2-001 (MEDIUM, CWE-284):** Align Armis `X-Org-Id` validation to the
  instance_org_id-non-nil conditional used by Claroty/CrowdStrike.
- **SEC-P2-002 (MEDIUM, CWE-22):** Move I/O-free pre-join `..` and absolute-path checks
  outside the `resolved.exists()` gate in `validate_dtu_block`.
- **SEC-P2-003 (MEDIUM, CWE-20):** Resolve ADR-006 OQ-1; tighten `ORG_SLUG_PATTERN`
  to 32 characters if no fixture exceeds it.
- **SEC-P2-004 (MEDIUM, CWE-209):** Add multi-line and inline-table test cases for
  `sanitize_error_message`; fix if any case leaks credential values.

### Post-Release / Before Wave 5

- **SEC-P2-005 (LOW, CWE-345):** Add `OrgRegistry::slug_for` cross-check at audit-record
  write time (warning-only, never fail-stop).
- **SEC-P2-006 (LOW, CWE-284):** Add `deny(deprecated)` lint to `prism-sensors` to make
  `init_registry` a compile error and force migration to `init_registry_for_org`.

---

## Verdict and Conditions

**APPROVED WITH CONDITIONS**

Wave 3.1 delivers sound closure of the two most critical security findings from the
pass-1 gate review. The OrgId-keyed adapter registry and per-handler mismatch guards
structurally eliminate cross-tenant query dispatch. Path traversal defenses are
substantive and regression-tested for the primary attack vectors. SEC-004 is retracted.

The following conditions govern wave progression:

### Condition A — Blocking (before Phase 4 holdout evaluation):

**SEC-NEW-001 (HIGH, CWE-306) — `POST /dtu/reset` unauthenticated on four clones.**
W3-FIX-SEC-002 must be delivered before the Phase 4 holdout evaluation. This is an
explicit deferral from Wave 3.1 (not a regression). Apply `X-Admin-Token` gate to
`dtu_reset` on Claroty, CrowdStrike, Armis, and Slack clones.

### Condition B — Non-blocking (tracked as technical debt):

1. **SEC-P2-001 (MEDIUM)** — Armis header-presence conditional weaker than Claroty/CrowdStrike.
   Align to instance_org_id-non-nil model before Phase 4.

2. **SEC-P2-002 (MEDIUM, SEC-003-RG)** — Pre-join traversal check bypassed for
   non-existent spec paths. Move I/O-free checks outside `resolved.exists()` gate.

3. **SEC-P2-003 (MEDIUM, SEC-005)** — OrgSlug 64-char limit / ADR-006 OQ-1 unresolved.
   Resolve before Phase 4 holdout evaluation.

4. **SEC-P2-004 (MEDIUM, SEC-006)** — `sanitize_error_message` multi-line TOML gap.
   Add test cases and if needed fix before Phase 4.

5. **SEC-P2-005 (LOW, SEC-007)** — `org_slug` audit cross-check deferred.

6. **SEC-P2-006 (LOW)** — `init_registry` deprecation without compile-error enforcement.
   Add `deny(deprecated)` lint to `prism-sensors` before Wave 5.

Wave 3 integration gate step D (pass 2) is **conditionally approved**. The HIGH
condition (SEC-NEW-001 = deferred SEC-002) was explicitly scoped out of Wave 3.1 and
must be addressed before Phase 4. All MEDIUM/LOW conditions are non-blocking for wave
progression but should be resolved before the Phase 4 holdout evaluation begins.
