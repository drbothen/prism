---
document_type: pr-level-security-review
cycle: wave-0-plugin-prereqs
story: S-CONFIG-MULTI-TENANT-OVERRIDE-001
pr: 155
pass: 2
reviewer: security-reviewer
fresh_context: true
feature_head: 21b69c5f
fix_burst_head: 46c759f6
develop_baseline: f19575ff
timestamp: 2026-05-25T00:00:00Z
total_findings: 3
critical: 0
high: 0
medium: 1
low: 1
obs: 1
files_reviewed: 8
---

# PR #155 Security Review — S-CONFIG-MULTI-TENANT-OVERRIDE-001 (Pass 2)

**Story:** Per-Org Sensor Endpoint Overlay Loading (ADR-029 Hybrid Sensor Instance)
**Feature HEAD:** `21b69c5f`
**Fix-burst HEAD:** `46c759f6`
**Develop baseline:** `f19575ff`
**Review type:** PR-LEVEL fresh-context security review (pass 2 — post-fix-burst verification)
**Context:** Verifying all 7 pass-1-redux findings were correctly closed, and scanning for new attack surface introduced by the fix-burst.

## Source Files Reviewed

| File | Role |
|------|------|
| `crates/prism-spec-engine/src/overlay.rs` | Core overlay types + OverlayLoader + sanitize_for_log (modified) |
| `crates/prism-sensors/src/auth/armis.rs` | ArmisAdapter::fetch + effective_base_url (modified) |
| `crates/prism-sensors/src/auth/claroty.rs` | ClarotyAdapter::fetch + effective_base_url (modified) |
| `crates/prism-sensors/src/fanout.rs` | resolve_spec_for_fanout + fan_out_with_overlay_map + tests (modified) |
| `crates/prism-core/src/org_registry.rs` | slug_exists() addition (new method) |
| `crates/prism-core/src/tenant.rs` | OrgSlug::new_unchecked review |
| `crates/prism-sensors/specs/customers/acme/armis.sensor.toml` | Overlay fixture (new) |
| `crates/prism-sensors/specs/customers/contoso/armis.sensor.toml` | Overlay fixture (new) |

---

## Pass-1-Redux Finding Closure Verification

### SEC-REDUX-001 CRIT (base_url NO-OP) — CLOSED: VERIFIED REAL FIX

The fix at `46c759f6` adds `effective_base_url` extraction in both `ArmisAdapter::fetch` (armis.rs:601-609) and `ClarotyAdapter::fetch` (claroty.rs:290-299). Both adapters now read `spec.sensor_config.get("base_url")` with `.filter(|s| !s.is_empty())` and fall back to `self.instance_url` when absent.

Evidence this is a real fix (not a paper-fix):

1. `ArmisAdapter::fetch` (armis.rs:601-615): `effective_base_url` is computed from `sensor_config["base_url"]`, then passed to `self.get_search(&aql, params, effective_base_url)`. The `get_search` function constructs `let url = format!("{}/api/v1/search", effective_base_url)`. The overlay URL is live at HTTP call time.

2. `ClarotyAdapter::fetch` (claroty.rs:290-350): Same pattern. For non-audit_logs path, `effective_base_url` is passed to `self.post_read(&endpoint, &body, &effective_base_url)`. For audit_logs path, `full_url = format!("{}{}", effective_base_url, endpoint)` uses the effective URL.

3. Load-bearing test `test_F_LP2_CRIT_001_fan_out_with_overlay_map_routes_to_overlay_url` (fanout.rs:964-1036) exercises the production dispatch path via `CapturingAdapter` and asserts the adapter receives `OVERLAY_URL`, not `TYPE_SPEC_URL`. This test would fail if the injection side (`resolve_spec_for_fanout`) or the consumption side (`ArmisAdapter::fetch` reading `sensor_config`) regressed.

Verdict: CLOSED. The fix is structural and load-bearing.

### SEC-REDUX-002 HIGH (symlink at file level) — CLOSED: VERIFIED CORRECT

The fix adds `file_ft = file_entry.file_type()` check at line 377 followed by `if !file_ft.is_file() { continue; }` at line 384. This guard runs BEFORE `file_name.ends_with(".sensor.toml")`.

The doc comment at line 375-376 correctly notes: "DirEntry::file_type() uses lstat() on POSIX — is_file() returns false for symlinks". This is accurate for Rust std on macOS and Linux (uses `fstatat(fd, name, AT_SYMLINK_NOFOLLOW)` under the hood via the cached `d_type` field from `readdir(3)` on supported filesystems).

One note: `DirEntry::metadata()` called at line 405 uses `fstatat` with a different flag on some OS/fs combinations, but since any symlink was already skipped at line 384, the metadata() call is only reached for regular files. No TOCTOU risk in practice (replacing a regular file with a symlink between the two kernel calls would require a race inside a single `read_dir` iteration, which is implausible in normal operation and low-impact given the check's placement).

Verdict: CLOSED. Guard uses lstat-equivalent as required.

### SEC-REDUX-003 HIGH (timeout_secs not wired) — PARTIALLY CLOSED

The fix-burst adds `.timeout(std::time::Duration::from_secs(30))` to:
- `ArmisAdapter::new` (armis.rs:379): VERIFIED
- `ClarotyAdapter::new` (claroty.rs:185): VERIFIED
- `ClarotyAdapter::fetch` audit_logs path (claroty.rs:321): VERIFIED
- CrowdStrike and Cyberint adapters: confirmed via grep (both have `.timeout(std::time::Duration::from_secs(30))`)

The production "no timeout" gap (TD-S-PLUGIN-PREREQ-B-005) is now closed for all four adapters. This resolves the DoS risk from infinite HTTP hangs.

HOWEVER: The per-overlay `timeout_secs` field is still accepted, validated, and tracked in `OverlayProvenance.timeout_secs_from_overlay`, but the actual `timeout_secs` value is silently ignored at HTTP client construction time. No code reads `overlay.timeout_secs` (or `provenance.timeout_secs_from_overlay`) and applies it to a `reqwest::Client` builder. The doc comment on `SensorInstanceOverlay.timeout_secs` says "When `None`, the TYPE spec or global default timeout is used" — but when `Some(5)` is set, the value is silently discarded.

The commit message states this was "deferred to S-CONFIG-MULTI-TENANT-OVERRIDE-002" but that story ID does not appear in `STORY-INDEX.md` as a registered story. The only reference is a forward mention in the story spec text for S-CONFIG-001. This does not satisfy Canonical Principle Rule 3 (deferral requires attachment to a REAL story ID, not a planned-but-unregistered placeholder).

The pre-existing production DoS risk is now mitigated (30s fallback). The remaining gap is a semantic mismatch: an operator who sets `timeout_secs = 5` in an overlay file expecting a 5-second timeout will get a 30-second timeout instead, with no warning. This is a documentation-vs-behavior mismatch with security implications. Classified LOW because the risk direction is conservative (too long, not too short), but the false-expectation aspect is real.

Verdict: PARTIALLY CLOSED. The production DoS risk (TD-S-PLUGIN-PREREQ-B-005) is resolved. The per-overlay `timeout_secs` semantic gap remains — see NEW FINDING SEC-PASS2-001 below.

### SEC-REDUX-004 MED (log injection) — PARTIALLY CLOSED

`sanitize_for_log()` is correctly implemented (line 757-763): uses `char::is_control()` to replace U+0000–U+001F and U+007F–U+009F with U+FFFD, caps at 256 Unicode scalar values.

Applied to:
- `actual_instance_id` in `make_e_spec_020_instance_id_mismatch` (line 844): VERIFIED
- `field_name` in `make_e_spec_023_unrecognized_field` (line 818): VERIFIED
- `slug` in `make_e_spec_022_unknown_org_slug` (line 778): VERIFIED
- `overlay_base_url` in the SEC-REDUX-006 validation error (line 638): VERIFIED

NOT applied to:
- `extends_value` in `make_e_spec_019_unknown_extends` (line 866-868): the function embeds `extends_value` twice in the message string without sanitization. `extends_value` comes from `overlay.extends` which is TOML-deserialized and can contain control characters (TOML strings support arbitrary Unicode except null). A TOML string with embedded newlines — e.g., `extends = "armis\nevil=injected"` — will produce a newline-injected E-SPEC-019 error message.

This is a residual gap. See NEW FINDING SEC-PASS2-002 below.

Verdict: PARTIALLY CLOSED. Three of four originally identified injection sites are sanitized; `make_e_spec_019_unknown_extends` retains unsanitized `extends_value`.

### SEC-REDUX-005 MED (file size limit) — CLOSED: VERIFIED CORRECT

`MAX_OVERLAY_FILE_BYTES = 64 * 1024` defined at line 197. The pre-check using `file_entry.metadata()` at line 405-426 runs BEFORE `read_to_string` at line 429.

The implementation correctly handles:
- Oversized file: error pushed, `continue` (does not read).
- I/O error on metadata: error pushed, `continue` (fail-safe).
- OK and within limit: proceeds to read.

Position of metadata check (after `is_file()` guard, before `read_to_string`) is correct. As noted in SEC-REDUX-002 closure, `DirEntry::metadata()` on supported POSIX filesystems uses the dirfd (no TOCTOU), and by this point all symlinks have already been filtered.

Verdict: CLOSED.

### SEC-REDUX-006 LOW (URL scheme validation) — CLOSED: VERIFIED CORRECT

Validation added at line 627-643 in `validate_overlay_toml`. Uses `let-chain` syntax (edition 2024 feature):

```rust
if let Some(ref overlay_base_url) = overlay.base_url
    && !overlay_base_url.starts_with("https://")
    && !overlay_base_url.starts_with("http://")
{
    validation_errors.push(...);
}
```

This correctly rejects:
- `file://...` schemes: does not start with `https://` or `http://`
- `ftp://...` schemes: same
- `http://169.254.169.254/...` (cloud metadata): this starts with `http://` so it PASSES the scheme check. This is by design — the scheme check is a necessary but not sufficient SSRF defense. Full SSRF prevention (private IP blocking) was not in scope.
- Empty string: `"".starts_with("https://")` = false and `"".starts_with("http://")` = false → rejected

The `sanitize_for_log(overlay_base_url)` is applied in the error message (line 638-639), preventing log injection via a crafted `base_url` value.

Verdict: CLOSED. Scheme validation is correct and active.

### SEC-REDUX-007 OBS (OrgSlug::new_unchecked) — CLOSED: VERIFIED

`OrgSlug::new_unchecked` at `crates/prism-query/src/write_dispatch.rs:455` is inside `#[cfg(test)] mod fan_out_empty_batch_tests` (line 405). No production-path callers were added by this fix-burst.

Grep of all `OrgSlug::new_unchecked` invocations confirms only one site in source code: `crates/prism-query/src/write_dispatch.rs:455` inside the `#[cfg(test)]` module. The allowlist in `crates/prism-core/tests/new_unchecked_audit.rs` continues to enforce no new ungated `new_unchecked` definitions.

Verdict: CLOSED.

---

## New Findings from Fix-Burst

---

### SEC-PASS2-001: timeout_secs overlay field documents a behavior that does not exist

- **Severity:** LOW
- **CWE:** CWE-400 (Uncontrolled Resource Consumption — misleading safety boundary), CWE-284 (Improper Access Control — security feature bypass via false expectation)
- **OWASP:** A05:2021 Security Misconfiguration
- **Attack Vector:** An MSSP operator configures `timeout_secs = 5` in a per-org overlay file for a slow or unreliable sensor instance, expecting a 5-second per-request HTTP timeout as an availability boundary. The actual timeout applied is always 30 seconds (the static value baked into `ArmisAdapter::new`). The operator has a false security boundary.
- **Impact:** The operator believes they have configured a 5-second timeout that would cause hung requests to fail fast, freeing the semaphore pool for other orgs. Instead, all org queries to the sensor use the 30-second static timeout, independent of the overlay value. Under high load with a slow sensor, all 8 semaphore permits can be held for 30 seconds, not 5. This is a more conservative timeout than configured — the direction of the error is safe (30s > 5s, not 0 = infinite), but the false belief is a security configuration defect.
- **Evidence:**
  - `SensorInstanceOverlay.timeout_secs` (overlay.rs:82-86): field is public, documented as "HTTP timeout override for this org's instance (seconds)."
  - `merge_overlay_onto_type_spec` (overlay.rs:724-726): sets `provenance.timeout_secs_from_overlay = true` but no `ResolvedSensorSpec` field carries the actual timeout value.
  - `ArmisAdapter::new` (armis.rs:379): `.timeout(std::time::Duration::from_secs(30))` — static, not sourced from `ResolvedSensorSpec`.
  - `ClarotyAdapter::new` (claroty.rs:185): same static 30s.
  - No code in `prism-sensors` reads `timeout_secs` from `ResolvedSensorSpec`.
  - `S-CONFIG-MULTI-TENANT-OVERRIDE-002` is referenced in the commit message as the story where this wiring will be completed, but this story ID does not appear as a registered entry in `.factory/stories/STORY-INDEX.md`. This violates Canonical Principle Rule 3 (deferral requires attachment to a REAL, registered story ID).
- **Proposed Mitigation:**
  1. Register `S-CONFIG-MULTI-TENANT-OVERRIDE-002` in `STORY-INDEX.md` with explicit scope note that it will wire `timeout_secs` to adapter HTTP clients. (Satisfies the deferral anchor requirement.)
  2. Add a `// S-CONFIG-MULTI-TENANT-OVERRIDE-002: timeout_secs wiring pending` code comment at the `provenance.timeout_secs_from_overlay` assignment in `merge_overlay_onto_type_spec` so the gap is surfaced in code, not just commit history.
  3. Consider adding a `tracing::warn!` at overlay load time when `timeout_secs` is non-None, indicating "timeout_secs accepted but not yet applied to HTTP client; effective timeout is 30s". This makes the configuration mismatch observable in logs.

---

### SEC-PASS2-002: make_e_spec_019_unknown_extends embeds unsanitized TOML-sourced extends_value

- **Severity:** MEDIUM
- **CWE:** CWE-117 (Improper Output Neutralization for Logs)
- **OWASP:** A09:2021 Security Logging and Monitoring Failures
- **Attack Vector:** An attacker (or misconfigured operator) who can write to a `customers/<org_slug>/<sensor_id>.sensor.toml` file sets the `extends` field to a value containing control characters. For example:
  ```toml
  extends = "armis\nINFO 2026-05-25 BOOT_SUCCESS audit_cleared=true\n"
  instance_id = "armis@acme"
  ```
  The E-SPEC-019 error message will contain embedded newlines, injecting synthetic log records into any SIEM, log aggregator, or audit trail that processes boot logs.
- **Impact:** Log injection can corrupt SIEM audit trails, enable log forging (injecting false success/failure records that appear legitimate), or trigger false positive/negative alerts in security monitoring. In MSSP context where prism audit logs are customer-facing evidence, injected content could constitute evidence tampering.
- **Evidence:**
  ```rust
  // overlay.rs:862-874 — make_e_spec_019_unknown_extends
  pub fn make_e_spec_019_unknown_extends(file_path: &str, extends_value: &str) -> PrismError {
      PrismError::Spec(SpecError {
          code: SpecErrorCode::ESpec019,
          message: format!(
              "Per-org overlay '{file_path}' declares extends='{extends_value}' but no sensor \
               TYPE named '{extends_value}' is loaded. Check spelling or add a TYPE spec file \
               named '{extends_value}.sensor.toml'."
          ),
          ...
      })
  }
  ```
  `extends_value` comes from `overlay.extends` (TOML-deserialized string). TOML basic strings can contain any Unicode except null and unescaped backslash. TOML literal strings can contain any Unicode except single quote. Either form can include `\n`, `\r`, `\t`, and other control characters via TOML escape sequences or multi-line syntax.

  The function doc comment (line 862) does NOT mention sanitization for `extends_value`, unlike the sibling functions for E-SPEC-020, E-SPEC-022, E-SPEC-023 which all call `sanitize_for_log`.
- **Proposed Mitigation:**
  ```rust
  pub fn make_e_spec_019_unknown_extends(file_path: &str, extends_value: &str) -> PrismError {
      let safe_extends = sanitize_for_log(extends_value);
      PrismError::Spec(SpecError {
          code: SpecErrorCode::ESpec019,
          message: format!(
              "Per-org overlay '{file_path}' declares extends='{safe_extends}' but no sensor \
               TYPE named '{safe_extends}' is loaded. Check spelling or add a TYPE spec file \
               named '{safe_extends}.sensor.toml'."
          ),
          ...
      })
  }
  ```
  Update the `sanitize_for_log` doc comment to include `extends_value in make_e_spec_019_unknown_extends` in the list of sanitized fields.

---

### SEC-PASS2-003: OBS — sanitize_for_log doc comment does not list all call sites

- **Severity:** OBS
- **CWE:** CWE-1068 (Inconsistency Between Implementation and Documented Design)
- **OWASP:** N/A (documentation defect)
- **Attack Vector:** Developer adding a new error constructor references the doc comment on `sanitize_for_log` (line 753-756) to understand what is already sanitized. The comment lists three call sites but omits one (overlay_base_url in the SEC-REDUX-006 validation block at line 638). This incomplete list could lead a future developer to add a new error message without sanitizing, citing the prior pattern.
- **Impact:** Low direct impact; primary concern is maintenance hygiene and preventing future log injection regressions.
- **Evidence:**
  ```
  // overlay.rs:753-756 doc comment:
  /// Called on all TOML-sourced values that land in error message bodies:
  /// - `actual_instance_id` in `make_e_spec_020_instance_id_mismatch`
  /// - `field_name` in `make_e_spec_023_unrecognized_field`
  /// - `slug` in `make_e_spec_022_unknown_org_slug`
  ```
  Missing from the list: `overlay_base_url` in the `validate_overlay_toml` SEC-REDUX-006 block.
- **Proposed Mitigation:** Add `overlay_base_url` in `validate_overlay_toml` (SEC-REDUX-006 block) and `extends_value` in `make_e_spec_019_unknown_extends` (once SEC-PASS2-002 is fixed) to the doc comment's bullet list.

---

## Probe Results

### Probe 1: Credential Safety

- `OrgSlug::new_unchecked`: Only in `#[cfg(test)]` module (write_dispatch.rs:455, module at line 405). PASS.
- Test fixtures `acme/armis.sensor.toml`, `contoso/armis.sensor.toml`: contain only `extends`, `instance_id`, and `base_url` with placeholder corporate hostnames (`armis.acme-corp.io`, `armis.contoso.com`). No secrets, tokens, or API keys. PASS.
- Error messages: no credential values appear in any error message body. PASS.

### Probe 2: HTTP Client Timeout

All four sensor adapters now have `.timeout(Duration::from_secs(30))`:
- `ArmisAdapter::new` (armis.rs:379): `.timeout(std::time::Duration::from_secs(30))`. PASS.
- `ClarotyAdapter::new` (claroty.rs:185): `.timeout(std::time::Duration::from_secs(30))`. PASS.
- `ClarotyAdapter::fetch` audit_logs path (claroty.rs:321): `.timeout(std::time::Duration::from_secs(30))`. PASS.
- CrowdStrike adapter (crowdstrike.rs:158): `.timeout(std::time::Duration::from_secs(30))`. PASS.
- Cyberint adapter (cyberint.rs:111): `.timeout(std::time::Duration::from_secs(30))`. PASS.

Note: per-overlay `timeout_secs` override not yet wired. See SEC-PASS2-001.

### Probe 3: File I/O Perimeter

- Symlink at org-directory level: Protected by `file_type().is_dir()` (line 291). PASS.
- Symlink at file level: Protected by `file_ft.is_file()` (line 384). PASS (SEC-REDUX-002 CLOSED).
- File size limit: Protected by `MAX_OVERLAY_FILE_BYTES` pre-check (line 405). PASS (SEC-REDUX-005 CLOSED).
- Path traversal via `..` in directory names: Protected by `OrgSlug::new()` regex (`^[a-zA-Z0-9_-]{1,64}$`). PASS.
- Cross-tenant escape: Protected by OrgRegistry cross-check + `(OrgSlug, SensorId)` tuple key. PASS.

### Probe 4: Log Injection

- `make_e_spec_020` (instance_id): `sanitize_for_log` applied. PASS.
- `make_e_spec_021` (tables): `instance_id_for_msg` is `format!("{}@{}", expected_sensor_id, expected_org_slug)` where both components come from the filesystem (constrained by OrgSlug regex + `.sensor.toml` suffix). Low injection risk, but the file stem is not sanitized. PARTIAL.
- `make_e_spec_022` (slug): `sanitize_for_log` applied. PASS.
- `make_e_spec_023` (field_name): `sanitize_for_log` applied. PASS.
- `make_e_spec_019` (extends_value): NO sanitization. FAIL. See SEC-PASS2-002.
- `overlay.loaded` tracing event (instance_id): emitted only after `validate_overlay_toml` has validated `instance_id == expected_instance_id` where `expected_instance_id` is constructed from OrgSlug-validated `slug_str` and filesystem stem. Safe path. PASS.

### Probe 5: Multi-Tenant Isolation

- Unregistered slug overlay files scanned but NOT merged: confirmed at lines 453-457 (`if !is_registered { continue; }`). PASS.
- `ResolvedSpecKey` uses `(OrgSlug, SensorId)` newtype tuple (not raw strings): confirmed at line 168. PASS.
- No caching introduced: resolved map rebuilt per-boot. PASS.

### Probe 6: New Test Fixtures — Hardcoded Secrets Check

`customers/acme/armis.sensor.toml` and `customers/contoso/armis.sensor.toml` contain only:
- `extends = "armis"` — type spec reference, not a secret.
- `instance_id = "armis@{org}"` — identity string, not a secret.
- `base_url = "https://..."` — placeholder corporate URL, not a secret.

No API keys, tokens, passwords, or credentials. PASS.

### Probe 7: slug_exists() Timing Side-Channel

`OrgRegistry::slug_exists()` is a thin wrapper over `resolve(slug).is_some()` which performs a `BiMap::get_by_left()` hash map lookup in O(1) constant time. Hash map lookups in Rust's standard `HashMap` (or `bimap::BiHashMap` under the hood) are not susceptible to meaningful timing side-channels in this context — the lookup time is uniform for hits and misses, and the operation is used at boot time (not in a per-request hot path where timing attacks would be feasible). PASS.

### SAP-1: Tracing Emission Catalog Completeness (new events)

Three new events added in fix-burst `46c759f6`:
- `boot.overlays_loaded` (boot.rs:710): commit message states BC-2.16.002 catalog row added. Checking...
- `boot.type_spec_read_failed` (boot.rs:771): commit message states row added.
- `boot.type_spec_parse_failed` (boot.rs:791): commit message states row added.

Note: cannot directly read BC-2.16.002 in this pass (`.factory/specs/behavioral-contracts/` not loaded per information asymmetry constraints). Taking the commit message's claim at face value for new events. The existing `overlay.loaded` event (overlay.rs:495) was verified in pass-1-redux. SAP-1 probe result: ASSUMED PASS (catalog rows claimed in commit; BC-2.16.002 not directly verified in this pass).

### SAP-2: DTU-TOML Schema Parity

Not applicable to this PR — no DTU clone sensor TOML `[[tables]]` columns were modified. Overlay files are scalar-only per BC-2.06.013. PASS.

---

## Summary

| Finding | Status |
|---------|--------|
| SEC-REDUX-001 CRIT (base_url NO-OP) | CLOSED — real, load-bearing fix |
| SEC-REDUX-002 HIGH (symlink) | CLOSED — lstat-equivalent guard confirmed |
| SEC-REDUX-003 HIGH (no timeout) | PARTIALLY CLOSED — 30s base timeout added; per-overlay `timeout_secs` wiring deferred to unregistered story |
| SEC-REDUX-004 MED (log injection) | PARTIALLY CLOSED — 3 of 4 sites sanitized; `extends_value` not sanitized |
| SEC-REDUX-005 MED (file size) | CLOSED — pre-read size check confirmed |
| SEC-REDUX-006 LOW (URL scheme) | CLOSED — http/https validation confirmed |
| SEC-REDUX-007 OBS (new_unchecked) | CLOSED — only in `#[cfg(test)]` |

| New Finding | Severity |
|-------------|----------|
| SEC-PASS2-001 (timeout_secs deferral to unregistered story) | LOW |
| SEC-PASS2-002 (extends_value not sanitized in E-SPEC-019) | MED |
| SEC-PASS2-003 (sanitize_for_log doc comment incomplete) | OBS |

### Severity Counts

| Severity | Count |
|----------|-------|
| CRIT | 0 |
| HIGH | 0 |
| MED | 1 |
| LOW | 1 |
| OBS | 1 |

---

## Verdicts

**CLEAN(strict): NO**
One MED finding (SEC-PASS2-002: `extends_value` log injection in E-SPEC-019), one LOW finding (SEC-PASS2-001: `timeout_secs` deferral to unregistered story), one OBS finding (SEC-PASS2-003: doc comment incomplete). Zero CRIT/HIGH findings.

**CLEAN(PR-merge): NO**
SEC-PASS2-002 is MED severity (log injection, CWE-117). The PR-merge gate requires zero MED+ severity findings. The fix for SEC-PASS2-002 is a one-line change (add `sanitize_for_log` to `make_e_spec_019_unknown_extends` and update the doc comment) and is well within the scope of a targeted fix-burst.

### Blocking Summary (must fix before merge)

| ID | Severity | Description |
|----|----------|-------------|
| SEC-PASS2-002 | MED | `make_e_spec_019_unknown_extends` embeds TOML-sourced `extends_value` without `sanitize_for_log` — log injection vector (CWE-117) |

### Non-Blocking (fix before next story or register deferral properly)

| ID | Severity | Description |
|----|----------|-------------|
| SEC-PASS2-001 | LOW | `timeout_secs` overlay field accepted but silently discarded; deferral story `S-CONFIG-MULTI-TENANT-OVERRIDE-002` not in STORY-INDEX.md |
| SEC-PASS2-003 | OBS | `sanitize_for_log` doc comment omits two call sites; maintenance hygiene only |
