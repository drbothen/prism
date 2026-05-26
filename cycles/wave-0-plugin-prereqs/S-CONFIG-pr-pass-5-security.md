---
document_type: pr-level-security-review
cycle: wave-0-plugin-prereqs
story: S-CONFIG-MULTI-TENANT-OVERRIDE-001
pr: 155
pass: 5
reviewer: security-reviewer
fresh_context: true
feature_head: 7406458a
pr_head: 7406458a
develop_baseline: f19575ff
timestamp: 2026-05-25T00:00:00Z
total_findings: 3
critical: 0
high: 0
medium: 0
low: 2
obs: 1
files_reviewed: 7
---

# PR #155 Security Review — S-CONFIG-MULTI-TENANT-OVERRIDE-001 (Pass 5)

**Story:** Per-Org Sensor Endpoint Overlay Loading (ADR-029 Hybrid Sensor Instance)
**PR HEAD:** `7406458a` (pass-4 fix-burst — SEC-PASS4-002 sanitize expected_sensor_id/org_slug in E-SPEC-021 error path)
**Pass-4 HEAD:** `3780ac27`
**Develop baseline:** `f19575ff`
**Review type:** PR-LEVEL fresh-context security review (pass 5 — SEC-PASS4-002 closure verification + full security sweep)

---

## SEC-PASS4-002 Closure Verification

### SEC-PASS4-002: sensor_id filesystem stem embedded unsanitized in E-SPEC-021 instance_id message

**Status: CLOSED — VERIFIED REAL FIX**

The single commit `7406458a` applies `sanitize_for_log` to both `expected_sensor_id` and `expected_org_slug` before building `instance_id_for_msg` in the E-SPEC-021 error path of `validate_overlay_toml`.

Diff confirmed at overlay.rs:585-589:

```rust
let instance_id_for_msg = format!(
    "{}@{}",
    sanitize_for_log(expected_sensor_id),
    sanitize_for_log(expected_org_slug)
);
```

Evidence this is a real fix (not a paper-fix, TD-VSDD-059):

1. `sanitize_for_log` applies `char::is_control()` replacement with U+FFFD and 256-char cap — the same production function that was tested in 6 unit tests verified in pass-4 (lines 925-1005). The call is structural, not a doc-comment rename.
2. Both arguments to `format!` now use sanitized values. The `sanitize_for_log` doc comment at lines 775-776 was updated to enumerate this as "the final TD-VSDD-060 sibling-sweep site."
3. The commit message accurately describes the scope: "sanitize expected_sensor_id/org_slug in E-SPEC-021 error path."

**Note on scope:** The fix-burst correctly applies `sanitize_for_log` to `expected_org_slug` in addition to `expected_sensor_id`, even though `expected_org_slug` was previously assessed as safe via OrgSlug regex validation. This is defense-in-depth at the concatenation site (mentioned in the inline comment at lines 583-584). The addition of `sanitize_for_log(expected_org_slug)` is appropriate: for unregistered slug directories (where `is_registered = false`), the raw `slug_str` from the filesystem has NOT passed OrgSlug regex validation and CAN contain control chars. Sanitizing at this site closes a subtle gap.

**Verdict: CLOSED. Load-bearing fix with defense-in-depth coverage.**

---

## Pass-5 Source Files Reviewed

| File | Role |
|------|------|
| `crates/prism-spec-engine/src/overlay.rs` | Core overlay — SEC-PASS4-002 fix verification + full fresh sweep |
| `crates/prism-core/src/error.rs` | SpecError Display impl — confirmed `file_path` NOT in Display |
| `crates/prism-core/src/tenant.rs` | OrgSlug regex validation pattern |
| `crates/prism-bin/src/boot.rs` | Boot step 4 — overlay error propagation path |
| `crates/prism-sensors/src/auth/armis.rs` | Adapter HTTP client + AQL |
| `crates/prism-sensors/src/fanout.rs` | Fan-out overlay dispatch |
| `crates/prism-query/src/engine.rs` | resolved_spec_map Arc plumbing |

---

## Pass-5 Full Security Probe Results

### Probe 1: SEC-PASS4-002 Closure (instance_id_for_msg sanitization in E-SPEC-021 path)

`sanitize_for_log(expected_sensor_id)` and `sanitize_for_log(expected_org_slug)` applied at overlay.rs:587-588. PASS.

---

### Probe 2: Complete Log Injection Surface — Pass-5 Full Sweep

This probe examines ALL points where `overlay_file_path = format!("customers/{slug_str}/{file_name}")` is embedded in error message bodies. The `overlay_file_path` string is composed of:

- `slug_str`: raw OS directory name (`entry.file_name().to_string_lossy()` at line 296). Safe when `OrgSlug::new` validates it (matching `^[a-zA-Z0-9_-]{1,64}$`). Potentially unsafe when the OrgSlug is invalid — unregistered directories are still scanned for file-level errors (EC-016-002), and an invalid slug (e.g., `"evil\ndir"`) flows into `overlay_file_path`.
- `file_name`: raw OS filename (`file_entry.file_name().to_string_lossy()` at line 389). Constrained to end with `.sensor.toml`, but the stem prefix is unconstrained.

| Injection site | Line | Surface | Sanitized? | Status |
|---------------|------|---------|------------|--------|
| E-SPEC-001 (size exceeded) message body | 409-413 | `overlay_file_path` in message string | NO | OPEN (see SEC-PASS5-002) |
| E-SPEC-001 (read failure) PrismError::Io body | 432-434 | `overlay_file_path` in message string | NO | OPEN (see SEC-PASS5-002) |
| E-SPEC-001 (TOML parse failure) message body | 547-549 | `overlay_file_path` in message string | NO | OPEN (see SEC-PASS5-002) |
| E-SPEC-001 (not a TOML table) message body | 564-566 | `overlay_file_path` in message string | NO | OPEN (see SEC-PASS5-002) |
| E-SPEC-001 (deserialization failure) message body | 622-624 | `overlay_file_path` in message string | NO | OPEN (see SEC-PASS5-002) |
| E-SPEC-001 (SSRF rejection) message body | 646 | `overlay_file_path` in message string (`overlay_base_url` IS sanitized; path is not) | NO | OPEN (see SEC-PASS5-002) |
| E-SPEC-021 (tables in overlay) message via constructor | 591 | `overlay_file_path` as `file_path` param; `instance_id_for_msg` NOW SANITIZED | PARTIAL — `instance_id_for_msg` fixed; `overlay_file_path` in message still raw | OPEN (see SEC-PASS5-002) |
| E-SPEC-023 (unrecognized field) message via constructor | 604 | `overlay_file_path` as `file_path` param; `field_name` IS sanitized | PARTIAL — `field_name` fixed; `overlay_file_path` in message still raw | OPEN (see SEC-PASS5-002) |
| E-SPEC-001 (deserialization failure) message body | 624 | `overlay_file_path` in message string | NO | OPEN (see SEC-PASS5-002) |
| E-SPEC-020 (instance_id mismatch) message via constructor | 659 | `overlay_file_path` as `file_path` param; `actual_instance_id` IS sanitized | PARTIAL — `actual_instance_id` fixed; `overlay_file_path` in message still raw | OPEN (see SEC-PASS5-002) |
| E-SPEC-019 (unknown extends) message via constructor | 668 | `overlay_file_path` as `file_path` param; `extends_value` IS sanitized | PARTIAL — `extends_value` fixed; `overlay_file_path` in message still raw | OPEN (see SEC-PASS5-002) |
| `overlay.loaded` tracing field | 496-497 | `org_slug = %slug_str`, `sensor_id = %sensor_id` | NO | OBS — low exploitability (see Probe 3) |

**Conclusion:** SEC-PASS4-002 fixed the `instance_id_for_msg` injection point in the E-SPEC-021 call. The `overlay_file_path` string itself — which appears in EVERY error message body via `'{overlay_file_path}'` or `'{file_path}'` in the format strings of the error constructors — remains unsanitized. This is a residual from the original SEC-PASS3-002 finding.

---

### Probe 3: overlay.loaded Tracing Emission Safety (Fresh Observation)

The `overlay.loaded` INFO event at overlay.rs:494-500 emits:

```rust
tracing::info!(
    event_type = "overlay.loaded",
    org_slug = %slug_str,
    sensor_id = %sensor_id,
    instance_id = %overlay.instance_id,
    "per-org overlay loaded and merged"
);
```

- `org_slug = %slug_str`: `slug_str` here is the registered org slug. Reaching this code requires `is_registered = true`, which requires `OrgSlug::new(slug_str).is_ok() = true`, which requires matching `^[a-zA-Z0-9_-]{1,64}$`. SAFE.
- `sensor_id = %sensor_id`: raw filesystem stem at this emission site. However, reaching this code requires `validate_overlay_toml` to return `Ok(...)` AND `type_specs.get(sensor_id)` to return `Some(...)`. A control-char `sensor_id` would not match any TYPE spec key (TYPE spec sensor IDs are validated at load time), so `type_specs.get(sensor_id)` would return `None`, the internal error fires, and `overlay.loaded` is NOT emitted. Exploitability is therefore theoretically nonzero but practically unreachable in correct operation.
- `instance_id = %overlay.instance_id`: TOML-sourced, constrained by E-SPEC-020 to match `"{expected_sensor_id}@{expected_org_slug}"` — if `sensor_id` contains a control char, E-SPEC-020 would fail unless the TOML also declares the malformed instance_id. SAFE in all normal paths.

**Verdict: OBS (low exploitability). Not a new OPEN finding at LOW severity.** The `overlay.loaded` emission is unreachable in practice for malformed `sensor_id` stems. Recorded for completeness.

---

### Probe 4: Credential Safety

- `OrgSlug::new_unchecked`: only in `#[cfg(test)]` contexts. PASS.
- Overlay fixture files (`acme/armis.sensor.toml`, `contoso/armis.sensor.toml`): only `extends`, `instance_id`, `base_url` with placeholder corporate hostnames. No secrets, tokens, or credentials. PASS.
- Error messages: no credential values appear in any error message body. PASS.

---

### Probe 5: HTTP Client Timeouts (all four adapters)

All adapters confirmed to have `.timeout(Duration::from_secs(30))` — unchanged from pass-4. PASS.

---

### Probe 6: File I/O Perimeter

Unchanged from pass-4; no new code in this path:
- Symlink at org-directory level: `file_type().is_dir()` guard. PASS.
- Symlink at file level: `file_ft.is_file()` guard. PASS.
- File size limit: `MAX_OVERLAY_FILE_BYTES = 64 * 1024` enforced. PASS.
- Path traversal via `..` in directory names: blocked by OrgSlug regex for registered orgs; unregistered dirs scanned for file errors but overlays not merged. PASS.

---

### Probe 7: Multi-Tenant Isolation

- Unregistered slug overlays scanned but NOT merged (`continue` at line 456). PASS.
- `ResolvedSpecKey` uses `(OrgSlug, SensorId)` newtype tuple. PASS.
- `resolved_spec_map` read-only after boot (INV-OVL-006), shared via `Arc<HashMap>`. PASS.

---

### Probe 8: SSRF Prevention

- `overlay_base_url` validated at overlay.rs:636-652: only `http://` and `https://` schemes accepted.
- `overlay_base_url` sanitized via `sanitize_for_log` in the SSRF rejection error message. PASS.
- `overlay_file_path` in the same message: not sanitized (see Probe 2 / SEC-PASS5-002). Open but LOW severity.

---

### SAP-1: Tracing Emission Catalog Completeness

No new `event_type` values introduced in `7406458a` (the commit only modifies one code block and a doc comment; no new tracing emissions). Previously verified events:
- `overlay.loaded` (row 38) — PASS
- `boot.overlays_loaded` (row 39) — PASS
- `boot.type_spec_read_failed` (row 40) — PASS
- `boot.type_spec_parse_failed` (row 41) — PASS
- `overlay.timeout_secs_ignored` (row 42) — PASS

SAP-1: PASS.

---

### SAP-2: DTU-TOML Schema Parity

Not applicable — no DTU clone sensor TOML `[[tables]]` columns modified by this PR. PASS.

---

## Pass-5 Findings

---

### SEC-PASS5-001: timeout_secs deferral story S-CONFIG-MULTI-TENANT-OVERRIDE-002 still unregistered in STORY-INDEX.md

**Carries forward from SEC-PASS4-001. No change since pass-4.**

- **Severity:** LOW
- **CWE:** CWE-284 (Improper Access Control — security boundary traceability gap), CWE-400 (Uncontrolled Resource Consumption — false operator expectation of timeout enforcement)
- **OWASP:** A05:2021 Security Misconfiguration
- **Attack Vector:** An MSSP operator configures `timeout_secs = 5` in a per-org overlay file. The runtime emits `overlay.timeout_secs_ignored` WARN (as of `3780ac27`), but the deferred wiring work has no registered story anchor in STORY-INDEX.md. Without registration, the story may be indefinitely deferred or lost in backlog triage, and the 30-second static timeout remains the effective timeout for all orgs regardless of configuration.
- **Impact:** Operator configuration produces a false security posture. The warn emission partially mitigates the silent-discard problem, but the deferral target is not a real story in STORY-INDEX.md, violating Canonical Principle Rule 3.
- **Evidence:**
  - overlay.rs:737-743: `tracing::warn!(event_type = "overlay.timeout_secs_ignored", ...)` — deferred to S-CONFIG-MULTI-TENANT-OVERRIDE-002.
  - `.factory/stories/STORY-INDEX.md`: no row for `S-CONFIG-MULTI-TENANT-OVERRIDE-002` (confirmed at pass-5 HEAD `7406458a`).
- **Proposed Mitigation:** Register `S-CONFIG-MULTI-TENANT-OVERRIDE-002` in STORY-INDEX.md with explicit scope ("wire `timeout_secs` overlay field to adapter HTTP client construction") and a concrete dependency/anchor. Per Canonical Principle Rule 3, the deferral target must be a real story ID with a concrete future dependency and a specific future story/wave anchor.

---

### SEC-PASS5-002: overlay_file_path containing raw filesystem data embedded unsanitized in multiple error message bodies

**Residual from SEC-PASS3-002, partially addressed by SEC-PASS4-002. SEC-PASS4-002 closed the `instance_id_for_msg` injection point; this finding identifies the `overlay_file_path` string itself as a remaining unsanitized surface.**

- **Severity:** LOW
- **CWE:** CWE-117 (Improper Output Neutralization for Logs)
- **OWASP:** A09:2021 Security Logging and Monitoring Failures
- **Attack Vector:** An actor with filesystem write access to the prism deployment directory creates a directory or overlay file whose name contains control characters (e.g., a directory `"customers/evil\ndir/"` or a file `"armi\ns.sensor.toml"`). The `overlay_file_path = format!("customers/{slug_str}/{file_name}")` string at line 400 is constructed without sanitization of either `slug_str` or `file_name`. This string is then embedded in the message bodies of at least 9 error constructors (E-SPEC-001 size, E-SPEC-001 parse, E-SPEC-001 table-type, E-SPEC-001 deserialization, E-SPEC-001 SSRF, E-SPEC-019, E-SPEC-020, E-SPEC-021, E-SPEC-023) via format strings like `"Per-org overlay '{overlay_file_path}'..."`. These messages appear in:
  1. `BootError::ConfigInvalid` via boot.rs:695 `format!("  - {e}")` on SpecError Display, which uses `{message}` (confirmed: SpecError Display at error.rs:961 does NOT include `file_path` field, only `code`, `toml_path`, and `message`).
  2. SIEM/log pipelines that consume boot error output.
  
  For `slug_str` to contain control chars, the OrgSlug validation at line 318 must return `is_ok() = false`. Such directories are still scanned for file-level errors (EC-016-002 requirement), so an invalid slug CAN produce `overlay_file_path` with control chars in message bodies.
  
  For `file_name` to contain control chars, the file must exist in the directory (POSIX allows all bytes except `\0` and `/` in filenames). The `ends_with(".sensor.toml")` filter reduces the attack surface but does not eliminate it.

- **Impact:** Log injection into `BootError::ConfigInvalid` text and SIEM pipelines at boot time. Attacker prerequisite: filesystem write access to the deployment directory. This is past primary perimeter controls, consistent with the LOW severity assigned to SEC-PASS3-002 and SEC-PASS4-002.
- **Evidence:**
  - overlay.rs:296: `let dir_name = entry.file_name().to_string_lossy().to_string();` — raw OS name for `slug_str`
  - overlay.rs:389: `let file_name = file_entry.file_name().to_string_lossy().to_string();` — raw OS name
  - overlay.rs:400: `let overlay_file_path = format!("customers/{slug_str}/{file_name}");` — no sanitization
  - overlay.rs:409-413: `"Per-org overlay '{overlay_file_path}' exceeds..."` — raw path in message
  - overlay.rs:549: `"Per-org overlay '{}' failed TOML parse: {}", overlay_file_path, e` — raw path in message
  - overlay.rs:566: `"Per-org overlay '{}' is not a TOML table", overlay_file_path` — raw path in message
  - overlay.rs:624: `"Per-org overlay '{}' failed deserialization: {}", overlay_file_path, e` — raw path in message
  - overlay.rs:646: `"Per-org overlay '{}' base_url '{}' ...", overlay_file_path, ...` — raw path in message
  - Constructors `make_e_spec_021_tables_in_overlay`, `make_e_spec_023_unrecognized_field`, `make_e_spec_020_instance_id_mismatch`, `make_e_spec_019_unknown_extends`: all embed `file_path` (= `overlay_file_path`) as `"Per-org overlay '{file_path}'"` in their message format strings — raw, unsanitized
  - Contrast with the closed fields: `instance_id_for_msg` (SEC-PASS4-002), `extends_value` (SEC-PASS2-002), `actual_instance_id` (SEC-REDUX-004), `field_name`, `slug`, `overlay_base_url` — all now sanitized via `sanitize_for_log`. The `overlay_file_path` identifier itself is the remaining surface.
- **Proposed Mitigation:** Two options, either is acceptable:

  Option A — Sanitize at derivation point (recommended, least change):
  ```rust
  // overlay.rs near line 400
  let overlay_file_path_raw = format!("customers/{}/{}", slug_str, file_name);
  let overlay_file_path = sanitize_for_log(&overlay_file_path_raw);
  ```
  The `overlay_file_path` is used only in error messages (never as an actual filesystem path for further I/O after construction), so sanitizing it at derivation is safe. It is the path displayed TO the operator, not the path used FROM the operator.

  Option B — Sanitize at each embedding site:
  Apply `sanitize_for_log(overlay_file_path)` inside each inline format string and each error constructor call. This is more verbose but keeps the raw string available if needed for other purposes (none currently exist).

  Note: The `file_path: Some(overlay_file_path.clone())` field of `SpecError` does NOT appear in SpecError's Display impl (`#[error("spec error {code:?} at {toml_path:?}: {message}")]`), so that field is only used by callers that access `SpecError.file_path` directly (IDE integrations, structured logging of the full struct). Sanitizing it too is reasonable but lower priority than the message bodies.

---

### SEC-PASS3-003 (OBS): AQL audit log embeds unsanitized aql_preview (pre-existing, out of PR scope)

**Unchanged from pass-3/4.**

- **Severity:** OBS
- **CWE:** CWE-117 (Improper Output Neutralization for Logs)
- Pre-existing code in `armis.rs`. Not introduced by this PR. No change since pass-3. Out of PR scope.

---

## Pass-5 Summary Table

| Finding ID | Severity | First Filed | Status at Pass 5 |
|-----------|----------|-------------|------------------|
| SEC-PASS2-002 (extends_value log injection) | MED | Pass 2 | CLOSED |
| SEC-PASS2-003 (sanitize_for_log doc missing sites) | OBS | Pass 2 | CLOSED |
| SEC-REDUX-001..006 (base_url, symlinks, timeouts, SSRF, size limit) | CRIT/HIGH/MED | Pass 1-redux | CLOSED |
| SEC-PASS3-001 (timeout_secs observability gap) | LOW | Pass 3 | PARTIALLY MITIGATED — warn emitted; story registration gap persists |
| SEC-PASS3-002 (overlay_file_path containing file_name unsanitized) | LOW | Pass 3 | PARTIALLY MITIGATED — `instance_id_for_msg` fixed by SEC-PASS4-002; `overlay_file_path` string in message bodies still raw; see SEC-PASS5-002 |
| SEC-PASS3-003 (aql_preview unsanitized — pre-existing) | OBS | Pass 3 | OPEN, OUT OF PR SCOPE |
| SEC-PASS4-001 (timeout_secs story unregistered) | LOW | Pass 4 | OPEN; carries as SEC-PASS5-001 |
| SEC-PASS4-002 (instance_id_for_msg E-SPEC-021 sanitization) | LOW | Pass 4 | CLOSED by `7406458a` fix-burst |
| SEC-PASS5-001 (S-CONFIG-MULTI-TENANT-OVERRIDE-002 unregistered) | LOW | Pass 5 | OPEN |
| SEC-PASS5-002 (overlay_file_path in message bodies — residual from SEC-PASS3-002) | LOW | Pass 5 | OPEN |

---

## Severity Counts (Pass 5)

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 0 |
| MEDIUM | 0 |
| LOW | 2 (SEC-PASS5-001, SEC-PASS5-002) |
| OBS | 1 (SEC-PASS3-003, pre-existing, out of PR scope) |

---

## Verdicts

**CLEAN(strict): NO**
Two LOW findings remain open (SEC-PASS5-001, SEC-PASS5-002) plus one OBS finding (SEC-PASS3-003, pre-existing, out of PR scope). No CRITICAL, HIGH, or MEDIUM findings.

- **SEC-PASS5-001** is a Canonical Principle Rule 3 process violation: deferral of `timeout_secs` wiring without a registered story anchor in STORY-INDEX.md. The `overlay.timeout_secs_ignored` warn partially addresses the operational gap but Rule 3 requires a real story registration.
- **SEC-PASS5-002** is a residual from SEC-PASS3-002: the `overlay_file_path` string (containing raw OS filesystem data) is embedded unsanitized in the message bodies of 9 error constructors. SEC-PASS4-002 fixed the `instance_id_for_msg` injection point within one of those constructors, but the `overlay_file_path` path identifier itself was not sanitized at derivation. This is a structural gap, not a paper-fix. Attacker prerequisite (filesystem write access) places this firmly in LOW severity.

**CLEAN(PR-merge): YES**
Zero CRITICAL/HIGH/MEDIUM findings at PR HEAD `7406458a`. Both open findings are LOW severity with the same mitigating factors that applied in passes 3 and 4 (filesystem write access required, direction of error is conservative, no credential disclosure).

The PR is cleared for merge from a security perspective. Both LOW findings should be addressed in a follow-up fix-burst before or alongside the next story that touches overlay error handling.

---

## Recommended Actions

| Priority | Action | Finding |
|----------|--------|---------|
| P3 (pre-next-story) | Register `S-CONFIG-MULTI-TENANT-OVERRIDE-002` in STORY-INDEX.md with explicit scope (wire `timeout_secs` to HTTP client) and a concrete dependency/anchor | SEC-PASS5-001 |
| P3 (pre-next-story) | Apply `sanitize_for_log` to `overlay_file_path` at derivation (line 400) or at each embedding site — closes the `overlay_file_path` message-body injection surface | SEC-PASS5-002 |
| P4 (backlog) | Add `sanitize_for_log` to `aql_preview` in `build_aql` (AQL audit events) | SEC-PASS3-003 |
