---
document_type: pr-level-security-review
cycle: wave-0-plugin-prereqs
story: S-CONFIG-MULTI-TENANT-OVERRIDE-001
pr: 155
pass: 3
reviewer: security-reviewer
fresh_context: true
feature_head: 2d4f82aa
pr_head: 792573d9
develop_baseline: f19575ff
timestamp: 2026-05-25T00:00:00Z
total_findings: 3
critical: 0
high: 0
medium: 0
low: 2
obs: 1
files_reviewed: 10
---

# PR #155 Security Review — S-CONFIG-MULTI-TENANT-OVERRIDE-001 (Pass 3)

**Story:** Per-Org Sensor Endpoint Overlay Loading (ADR-029 Hybrid Sensor Instance)
**PR HEAD:** `792573d9` (docs commit — evidence-report SHA bump; no code change)
**Fix-burst HEAD (pass-2):** `2d4f82aa` (sole code change since pass-2)
**Develop baseline:** `f19575ff`
**Review type:** PR-LEVEL fresh-context security review (pass 3 — post pass-2 fix-burst verification + new attack surface sweep)

## Pass-2 Findings Closure Verification

### SEC-PASS2-002 (MED, CWE-117) — extends_value unsanitized in make_e_spec_019_unknown_extends

**Status: CLOSED — VERIFIED REAL FIX**

The fix at `2d4f82aa` applies `sanitize_for_log` to `extends_value` in `make_e_spec_019_unknown_extends`. Verified at `792573d9`:

1. `let safe_extends = sanitize_for_log(extends_value);` — line 870
2. All three occurrences of `extends_value` in the format string now use `safe_extends` (TD-VSDD-060 sibling-sweep compliance verified by doc comment "All three occurrences use the sanitized value")
3. The `sanitize_for_log` doc comment (lines 753-758) now lists all five call sites including `extends_value` and `overlay_base_url` — closing SEC-PASS2-003 OBS simultaneously.

Evidence this is a real fix (not a paper-fix, TD-VSDD-059):
- `sanitize_for_log` applies `char::is_control()` to replace U+0000–U+001F and U+007F–U+009F with U+FFFD and caps at 256 scalar values.
- The `format!` macro receives `safe_extends` (the sanitized copy), not the raw `extends_value` parameter.
- The doc comment explicitly cross-references the CWE number and the sibling-sweep compliance rule (TD-VSDD-060), confirming intentional structural coverage.

**Verdict: CLOSED. Load-bearing fix with correct sibling-sweep.**

---

### SEC-PASS2-003 (OBS) — sanitize_for_log doc comment missing two call sites

**Status: CLOSED — VERIFIED**

The `sanitize_for_log` doc comment at lines 753-758 now lists all five call sites:
- `actual_instance_id` in `make_e_spec_020_instance_id_mismatch`
- `field_name` in `make_e_spec_023_unrecognized_field`
- `slug` in `make_e_spec_022_unknown_org_slug`
- `extends_value` in `make_e_spec_019_unknown_extends`
- `overlay_base_url` in the SEC-REDUX-006 SSRF rejection branch of `validate_overlay_toml`

**Verdict: CLOSED.**

---

### SEC-PASS2-001 (LOW) — timeout_secs overlay field accepted but deferral story unregistered

**Status: PARTIALLY MITIGATED — NOT FULLY CLOSED**

The deferred story `S-CONFIG-MULTI-TENANT-OVERRIDE-002` (referenced in the `OverlayProvenance` doc comment at line 103 and in the commit message for `46c759f6`) is still NOT registered in `STORY-INDEX.md` at PR HEAD `792573d9`.

`STORY-INDEX.md` contains `S-SPEC-TYPE-UNIFICATION-001` as a registered follow-up story anchored to `S-CONFIG-MULTI-TENANT-OVERRIDE-001`, but that story covers type consolidation — not `timeout_secs` wiring to HTTP clients. The specific `timeout_secs` behavioral gap (operator sets `timeout_secs = 5`, gets 30s instead with no warning) has no registered story anchor.

The code comment at line 724 (`// timeout_secs provenance (stored in provenance; no SensorSpec field for it yet).`) was not updated with the deferral story reference per the proposed mitigation in SEC-PASS2-001.

This remains a Canonical Principle Rule 3 violation: deferral without a registered story anchor.

**Verdict: NOT CLOSED. See SEC-PASS3-001 below.**

---

## Source Files Reviewed (Pass 3)

| File | Role |
|------|------|
| `crates/prism-spec-engine/src/overlay.rs` | Core overlay — SEC-PASS2-002 closure + new sweep |
| `crates/prism-sensors/src/auth/armis.rs` | ArmisAdapter effective_base_url + AQL validation |
| `crates/prism-sensors/src/auth/claroty.rs` | ClarotyAdapter effective_base_url |
| `crates/prism-sensors/src/auth/crowdstrike.rs` | Timeout addition (30s) |
| `crates/prism-sensors/src/auth/cyberint.rs` | Timeout addition (30s) |
| `crates/prism-sensors/src/fanout.rs` | resolve_spec_for_fanout + fan_out_with_overlay_map |
| `crates/prism-bin/src/boot.rs` | step4_load_sensor_specs_with_overlays + build_type_spec_map_for_overlay |
| `crates/prism-query/src/engine.rs` | resolved_spec_map wiring |
| `crates/prism-query/src/materialization.rs` | MaterializationContext resolved_spec_map plumbing |
| `crates/prism-core/src/org_registry.rs` | slug_exists + OrgSlug regex constraint |

---

## Pass-3 Findings

---

### SEC-PASS3-001: timeout_secs deferral references unregistered story S-CONFIG-MULTI-TENANT-OVERRIDE-002

- **Severity:** LOW
- **CWE:** CWE-400 (Uncontrolled Resource Consumption — misleading safety boundary), CWE-284 (Improper Access Control — security feature bypass via false expectation)
- **OWASP:** A05:2021 Security Misconfiguration
- **Attack Vector:** An MSSP operator configures `timeout_secs = 5` in a per-org overlay file expecting a 5-second HTTP timeout. The timeout applied is always 30 seconds (static in `ArmisAdapter::new`, `ClarotyAdapter::new`, etc.). No log warning is emitted when `timeout_secs` is set but silently discarded.
- **Impact:** The operator has a false security boundary: they believe hung requests to a slow sensor instance will fail at 5 seconds (freeing semaphore permits for other orgs), but in practice the 30-second static timeout applies. Under high-concurrency load with a slow sensor, all 8 semaphore permits can be held 6x longer than the operator's configured intention. Direction of error is conservative (30s > 5s, not infinite), but the false expectation is a genuine configuration safety defect.
- **Evidence:**
  - `OverlayProvenance` doc comment (overlay.rs:103): "follow-up story S-CONFIG-MULTI-TENANT-OVERRIDE-002" — this story is NOT in STORY-INDEX.md.
  - `merge_overlay_onto_type_spec` (overlay.rs:724-726): sets `provenance.timeout_secs_from_overlay = true` but no code reads this provenance flag to apply a timeout.
  - `ArmisAdapter::new` (armis.rs:379): `.timeout(std::time::Duration::from_secs(30))` — static.
  - No code in `prism-sensors` reads `timeout_secs` from `ResolvedSensorSpec`.
- **Proposed Mitigation:**
  1. Register `S-CONFIG-MULTI-TENANT-OVERRIDE-002` in `STORY-INDEX.md` with explicit scope (wire `timeout_secs` to adapter HTTP client construction), OR rename the reference to the already-registered `S-SPEC-TYPE-UNIFICATION-001` if that story will absorb this work.
  2. Add a deferral code comment at the `provenance.timeout_secs_from_overlay` assignment: `// S-CONFIG-MULTI-TENANT-OVERRIDE-002: timeout_secs wiring to HTTP client pending`.
  3. Consider adding `tracing::warn!(event_type = "overlay.timeout_secs_ignored", org_slug = ?, sensor_id = ?, timeout_secs = ?, "timeout_secs accepted but not yet applied to HTTP client; effective timeout is 30s")` at overlay load time when `timeout_secs` is `Some(...)`.

---

### SEC-PASS3-002: sensor_id filesystem stem embedded unsanitized in overlay error messages

- **Severity:** LOW
- **CWE:** CWE-117 (Improper Output Neutralization for Logs)
- **OWASP:** A09:2021 Security Logging and Monitoring Failures
- **Attack Vector:** An actor with filesystem write access to the prism deployment directory creates an overlay file whose name stem contains control characters (e.g., `armi\ns.sensor.toml` — a filename containing a literal newline byte). The `sensor_id` derived from this stem (`file_name[..file_name.len() - ".sensor.toml".len()]`) is embedded unsanitized in `overlay_file_path` (line 400: `format!("customers/{slug_str}/{file_name}")`), which then flows into E-SPEC-019, E-SPEC-020, E-SPEC-021, E-SPEC-023 error messages and `BootError::ConfigInvalid`.
- **Impact:** Log injection into boot error messages and `BootError::ConfigInvalid` text. In MSSP context where boot logs feed SIEM pipelines, injected synthetic log records could corrupt audit trails. This requires filesystem write access to the deployment directory as a prerequisite — an attacker who has achieved this level of access is already past primary perimeter controls. Severity is LOW rather than MEDIUM because of this prerequisite barrier.
- **Evidence:**
  - overlay.rs:397: `let sensor_id = &file_name[..file_name.len() - ".sensor.toml".len()];` — no character validation on the stem.
  - overlay.rs:400: `let overlay_file_path = format!("customers/{slug_str}/{file_name}");` — `file_name` (with stem) embedded unsanitized.
  - overlay.rs:580: `let instance_id_for_msg = format!("{}@{}", expected_sensor_id, expected_org_slug);` — `expected_sensor_id` is the raw stem, used in E-SPEC-021 message.
  - `slug_str` is safe: `OrgSlug::new()` enforces `^[a-zA-Z0-9_-]{1,64}$` which excludes control characters.
  - `file_name` constrained only by `ends_with(".sensor.toml")` — the stem prefix is unconstrained.
  - On Linux/macOS, filenames may contain any byte except `\0` and `/`, including `\n` (0x0A).
- **Proposed Mitigation:**
  Sanitize `sensor_id` after derivation from the filesystem stem:
  ```rust
  let sensor_id_raw = &file_name[..file_name.len() - ".sensor.toml".len()];
  // Validate sensor_id is safe to embed in error messages (filesystem stem, not TOML-sourced).
  // Control chars in filenames are adversarial; the TYPE spec lookup below enforces semantic
  // validity. A sensor_id with control chars will produce E-SPEC-019 regardless.
  let sensor_id_for_msg: String = sensor_id_raw.chars()
      .map(|c| if c.is_control() { '\u{FFFD}' } else { c })
      .take(128).collect();
  let sensor_id = sensor_id_raw; // TYPE spec lookup uses the raw stem
  ```
  Use `sensor_id_for_msg` in `overlay_file_path` and `instance_id_for_msg` rather than the raw stem. Pass `sensor_id_for_msg` as `expected_sensor_id` to `validate_overlay_toml` (which uses it only in error messages, not lookups). The TYPE spec lookup (line 464: `type_specs.get(sensor_id)`) uses the raw `sensor_id`, which is correct — a control-char stem will simply fail to match any registered TYPE spec and produce E-SPEC-019 with the sanitized value.

---

### SEC-PASS3-003: OBS — AQL audit log embeds unsanitized aql_preview (pre-existing, out of PR scope)

- **Severity:** OBS
- **CWE:** CWE-117 (Improper Output Neutralization for Logs)
- **OWASP:** A09:2021 Security Logging and Monitoring Failures
- **Attack Vector:** An AQL query string with control characters in the first 64 characters is passed via `spec.sensor_config["aql_query"]`. The `aql_preview` (line 429: `aql.chars().take(64).collect()`) is embedded in `aql_query_rejected` and `aql_query_execution` tracing events without sanitization.
- **Impact:** Log injection via the AQL audit trail. However, the AQL validator (`validate_aql`) blocks many ASCII control-character-like sequences via its allowlist (single-quote rejection, comment-marker rejection, stacked-query separator rejection). A newline injected in the first 64 chars would evade the validator's checks (newlines are not explicitly blocked by `validate_aql`). Severity is OBS because: (1) this is pre-existing code not introduced by this PR, (2) `aql_preview` is a structured tracing field (not a format string), and (3) the AQL path requires spec authoring access.
- **Evidence:**
  - armis.rs:429: `let aql_preview: String = aql.chars().take(64).collect();` — no control char filtering.
  - armis.rs:437: `aql_preview = %aql_preview` — structured field in `tracing::warn!`.
- **Note:** This finding is OUT OF SCOPE for PR #155 (not introduced by this PR). Included for completeness and cross-reference. Recommend a follow-up story or the existing AQL validation hardening story to address.

---

## Pass-3 Full Security Probe Results

### Probe 1: SEC-PASS2-002 Closure (extends_value sanitization)

- `make_e_spec_019_unknown_extends` (overlay.rs:869-882): `safe_extends = sanitize_for_log(extends_value)` applied; all three format string occurrences use `safe_extends`. PASS.
- `sanitize_for_log` doc comment: all 5 call sites listed. PASS.

### Probe 2: Full Log Injection Surface (all error constructors)

| Constructor | TOML-sourced field | Sanitized? |
|------------|-------------------|------------|
| `make_e_spec_019_unknown_extends` | `extends_value` | YES — `safe_extends` |
| `make_e_spec_020_instance_id_mismatch` | `actual_instance_id` | YES — `safe_actual` |
| `make_e_spec_021_tables_in_overlay` | `instance_id` (derived from filesystem) | NO — `instance_id_for_msg` uses raw `sensor_id` stem |
| `make_e_spec_022_unknown_org_slug` | `slug` | YES — `safe_slug` |
| `make_e_spec_023_unrecognized_field` | `field_name` | YES — `safe_field` |
| SEC-REDUX-006 SSRF rejection | `overlay_base_url` | YES — `sanitize_for_log(overlay_base_url)` |

`make_e_spec_021_tables_in_overlay` uses `instance_id_for_msg = format!("{}@{}", expected_sensor_id, expected_org_slug)` where `expected_sensor_id` is the raw filesystem stem (unconstrained) and `expected_org_slug` is `slug_str` (constrained by OrgSlug regex to `[a-zA-Z0-9_-]{1,64}` — safe). See SEC-PASS3-002.

### Probe 3: Credential Safety

- `OrgSlug::new_unchecked`: Only in `#[cfg(test)]` module (`write_dispatch.rs:455`). PASS.
- `overlay_loading_tests.rs`: No `OrgSlug::new_unchecked` usage. PASS.
- Overlay fixtures (`acme/armis.sensor.toml`, `contoso/armis.sensor.toml`): only `extends`, `instance_id`, `base_url` with placeholder corporate hostnames. No secrets, tokens, or credentials. PASS.
- Error messages: no credential values appear in any error message body. PASS.

### Probe 4: HTTP Client Timeouts (all four adapters)

- `ArmisAdapter::new` (armis.rs:379): `.timeout(std::time::Duration::from_secs(30))`. PASS.
- `ClarotyAdapter::new` (claroty.rs:185): `.timeout(std::time::Duration::from_secs(30))`. PASS.
- `ClarotyAdapter::fetch` audit_logs path (claroty.rs:321): `.timeout(std::time::Duration::from_secs(30))`. PASS.
- `CrowdStrikeAdapter::new` (crowdstrike.rs:158): `.timeout(std::time::Duration::from_secs(30))`. PASS.
- `CyberintAdapter::new` (cyberint.rs:111): `.timeout(std::time::Duration::from_secs(30))`. PASS.
- Production DoS risk (TD-S-PLUGIN-PREREQ-B-005) CLOSED for all four adapters. PASS.

### Probe 5: File I/O Perimeter

- Symlink at org-directory level: Protected by `file_type().is_dir()` (line 291). PASS.
- Symlink at file level: Protected by `file_ft.is_file()` (line 384, uses lstat equivalent). PASS.
- File size limit: `MAX_OVERLAY_FILE_BYTES = 64 * 1024`, enforced before `read_to_string` (line 405). PASS.
- Path traversal via `..` in directory names: Protected by `OrgSlug::new()` regex (`^[a-zA-Z0-9_-]{1,64}$`) — `..` is rejected. PASS.
- Path traversal via `..` in file names: Files are iterated via `read_dir`; only `*.sensor.toml` files processed; the stem would contain `..` which will not match any TYPE spec key, producing E-SPEC-019. Not a traversal risk (file content is read, not path-joined to another directory). PASS.

### Probe 6: Multi-Tenant Isolation

- Unregistered slug overlays scanned for structural errors but NOT merged into resolved map (lines 453-457: `if !is_registered { continue; }`). PASS.
- `ResolvedSpecKey` uses `(OrgSlug, SensorId)` newtype tuple (not raw strings). PASS.
- O(1) HashMap lookup, no filesystem I/O at fan-out time (INV-FANOUT-002). PASS.
- `resolved_spec_map` is read-only after boot (INV-OVL-006), shared via `Arc<HashMap>` without mutex. PASS.

### Probe 7: Arc Plumbing (engine.rs + materialization.rs)

New Arc-plumbing additions thread `resolved_spec_map` from boot step 4 through `RunningServer` → `QueryEngine` → `MaterializationContext`. These are read-only Arc shares with no mutable state introduced. No new attack surface. PASS.

### Probe 8: OrgRegistry slug_exists Timing Side-Channel

`slug_exists` → `resolve` → `BiMap::get_by_left()` hash map lookup. O(1), uniform for hits and misses, used at boot time not in per-request hot path. Not a meaningful timing side-channel. PASS.

### SAP-1: Tracing Emission Catalog Completeness

New `event_type` values introduced by this PR:
- `boot.overlays_loaded` (boot.rs:710): SAP-1 compliance comment and catalog row claim present in code. ASSUMED PASS.
- `boot.type_spec_read_failed` (boot.rs:771): SAP-1 compliance comment and catalog row claim present. ASSUMED PASS.
- `boot.type_spec_parse_failed` (boot.rs:791): SAP-1 compliance comment and catalog row claim present. ASSUMED PASS.
- `overlay.loaded` (overlay.rs:495): SAP-1 compliance comment present (overlay.rs:491-494). ASSUMED PASS.

Note: BC-2.16.002 catalog not directly loaded per information asymmetry constraints. Catalog row claims accepted at face value; adversary independently verified these in prior passes.

### SAP-2: DTU-TOML Schema Parity

Not applicable — no DTU clone sensor TOML `[[tables]]` columns modified. Overlay files are scalar-only per BC-2.06.013. PASS.

---

## Summary Table

| Finding | Severity | Status |
|---------|----------|--------|
| SEC-PASS2-002 (extends_value log injection) | MED | CLOSED — fix verified |
| SEC-PASS2-003 (sanitize_for_log doc comment) | OBS | CLOSED — fix verified |
| SEC-PASS2-001 (timeout_secs unregistered deferral) | LOW | NOT CLOSED — see SEC-PASS3-001 |
| SEC-PASS3-001 (timeout_secs LOW — unregistered deferral story) | LOW | OPEN |
| SEC-PASS3-002 (sensor_id filesystem stem unsanitized LOW) | LOW | OPEN |
| SEC-PASS3-003 (aql_preview log injection OBS — pre-existing) | OBS | OPEN, OUT OF PR SCOPE |

### Severity Counts (Pass 3)

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 0 |
| MEDIUM | 0 |
| LOW | 2 |
| OBS | 1 |

---

## Verdicts

**CLEAN(strict): NO**
Two LOW findings remain (SEC-PASS3-001, SEC-PASS3-002) plus one OBS finding (SEC-PASS3-003, out of PR scope). No CRITICAL or HIGH findings. Zero MEDIUM findings (SEC-PASS2-002 closure verified).

**CLEAN(PR-merge): YES**
Zero CRITICAL/HIGH/MEDIUM findings at PR HEAD `792573d9`. Both open findings are LOW severity:
- SEC-PASS3-001 (timeout_secs unregistered deferral): direction is conservative (30s > 5s, not unsafe); the behavioral gap does not create an active exploit path.
- SEC-PASS3-002 (sensor_id stem unsanitized): requires filesystem write access as prerequisite; slug_str is regex-validated; risk is constrained.
- SEC-PASS3-003 (aql_preview log injection): pre-existing, out of PR scope, OBS severity.

The PR is cleared for merge from a security perspective. The two LOW findings should be addressed before or during the next story that touches overlay error handling.

### Recommended Actions Before Next Story

| Priority | Action | Finding |
|----------|--------|---------|
| P3 (pre-next-story) | Register `S-CONFIG-MULTI-TENANT-OVERRIDE-002` in STORY-INDEX.md OR anchor timeout_secs wiring to `S-SPEC-TYPE-UNIFICATION-001`; add code comment at provenance assignment | SEC-PASS3-001 |
| P3 (pre-next-story) | Sanitize `sensor_id` stem before constructing `overlay_file_path` and `instance_id_for_msg` | SEC-PASS3-002 |
| P4 (backlog) | Add `sanitize_for_log` to `aql_preview` in `build_aql` (AQL audit events) | SEC-PASS3-003 |
