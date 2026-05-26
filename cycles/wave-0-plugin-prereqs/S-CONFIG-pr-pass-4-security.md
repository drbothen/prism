---
document_type: pr-level-security-review
cycle: wave-0-plugin-prereqs
story: S-CONFIG-MULTI-TENANT-OVERRIDE-001
pr: 155
pass: 4
reviewer: security-reviewer
fresh_context: true
feature_head: 3780ac27
pr_head: 3780ac27
develop_baseline: f19575ff
timestamp: 2026-05-25T00:00:00Z
total_findings: 2
critical: 0
high: 0
medium: 0
low: 1
obs: 1
files_reviewed: 12
---

# PR #155 Security Review — S-CONFIG-MULTI-TENANT-OVERRIDE-001 (Pass 4)

**Story:** Per-Org Sensor Endpoint Overlay Loading (ADR-029 Hybrid Sensor Instance)
**PR HEAD:** `3780ac27` (pass-3 fix-burst — timeout_secs warning + sanitize_for_log unit tests + boot.rs doc fixes)
**Pass-3 HEAD:** `792573d9`
**Develop baseline:** `f19575ff`
**Review type:** PR-LEVEL fresh-context security review (pass 4 — post pass-3 fix-burst verification + full security sweep)

---

## Pass-3 Fix-Burst Contents

The single commit `3780ac27` introduced three changes:

1. `SEC-PASS2-001/SEC-PASS3-001`: Added `tracing::warn!(event_type = "overlay.timeout_secs_ignored", ...)` in `merge_overlay_onto_type_spec` when `overlay.timeout_secs` is `Some`. BC-2.16.002 catalog row added as row 42 (v1.29).
2. `F-PR155-P2-002`: Fixed boot.rs doc comments "sensor_id_string" → "SensorId" in `RunningServer` and `BootContext` resolved_spec_map fields (cosmetic, non-security).
3. `F-PR155-P2-003`: Added 6 unit tests for `sanitize_for_log` covering: newline replacement, CRLF replacement, null byte replacement, 256-char truncation, clean ASCII passthrough, Unicode preservation.

---

## Pass-3 Finding Closure Verification

### SEC-PASS3-001: timeout_secs observability gap (LOW — partial mitigation)

**Status: PARTIALLY MITIGATED — NOT FULLY CLOSED**

The `tracing::warn!` for `overlay.timeout_secs_ignored` is confirmed at `merge_overlay_onto_type_spec` (overlay.rs:727-733):

```
tracing::warn!(
    event_type = "overlay.timeout_secs_ignored",
    sensor_id = %type_spec.sensor_id,
    timeout_secs = timeout_secs,
    "timeout_secs overlay field accepted but not yet wired to HTTP client; \
     deferred to S-CONFIG-MULTI-TENANT-OVERRIDE-002"
);
```

- `type_spec.sensor_id` is the semantic SensorId from a validated TYPE spec (e.g., "armis"), not the raw filesystem stem. This is safe — no log injection risk from this field.
- `timeout_secs` is a `u64` integer value. Safe.
- BC-2.16.002 v1.29 row 42 is confirmed at line 119 of the catalog file with full schema, audit role, and recurrence policy. SAP-1 PASS.

**Remaining gap:** `S-CONFIG-MULTI-TENANT-OVERRIDE-002` is still NOT registered in STORY-INDEX.md. The `OverlayProvenance` doc comment at overlay.rs:103 and the warn message both reference this story ID, but no row exists in `.factory/stories/STORY-INDEX.md` for it. This is a Canonical Principle Rule 3 violation: the deferral target must be a real story ID in STORY-INDEX.md, not merely a named reference in code comments.

**Verdict: PARTIALLY MITIGATED.** The false-expectation observability gap is closed (operators now see a warning when timeout_secs is set but not applied). The story-anchor registration gap persists. See SEC-PASS4-001.

---

### SEC-PASS3-002: sensor_id filesystem stem embedded unsanitized

**Status: OPEN — NO FIX APPLIED**

The pass-3 fix-burst made no changes to the sensor_id stem derivation or sanitization. At overlay.rs:397-400:

```rust
let sensor_id = &file_name[..file_name.len() - ".sensor.toml".len()];
let overlay_file_path = format!("customers/{slug_str}/{file_name}");
```

`sensor_id` is the raw filesystem stem without control character validation. It flows unsanitized into:
- `overlay_file_path` (line 400) — embedded in `SpecError::message` for size-limit and read errors
- `instance_id_for_msg = format!("{}@{}", expected_sensor_id, expected_org_slug)` (line 580) — embedded in `make_e_spec_021_tables_in_overlay` `SpecError::message`
- The error detail string at boot.rs:695 (`format!("  - {e}")` on each SpecError) → `BootError::ConfigInvalid` — which may reach SIEM log pipelines

**Verdict: OPEN.** See SEC-PASS4-002.

---

### SEC-PASS3-003: AQL audit log embeds unsanitized aql_preview (OBS — pre-existing)

**Status: OPEN — OUT OF PR SCOPE, UNCHANGED**

Pre-existing code in `armis.rs`. Not introduced by this PR. No change since pass-3. OBS severity maintained.

---

## Source Files Reviewed (Pass 4)

| File | Role |
|------|------|
| `crates/prism-spec-engine/src/overlay.rs` | Core overlay — pass-3 fix verification + full fresh sweep |
| `crates/prism-bin/src/boot.rs` | Boot step 4 — error propagation path for SpecError::message |
| `crates/prism-core/src/error.rs` | SpecError Display impl — confirmed `file_path` NOT in Display |
| `crates/prism-sensors/src/auth/armis.rs` | Adapter HTTP client + AQL validation |
| `crates/prism-sensors/src/auth/claroty.rs` | ClarotyAdapter HTTP client |
| `crates/prism-sensors/src/auth/crowdstrike.rs` | HTTP client timeout |
| `crates/prism-sensors/src/auth/cyberint.rs` | HTTP client timeout |
| `crates/prism-sensors/src/fanout.rs` | Fan-out overlay dispatch |
| `crates/prism-query/src/engine.rs` | resolved_spec_map Arc plumbing |
| `crates/prism-query/src/materialization.rs` | MaterializationContext plumbing |
| `crates/prism-core/src/org_registry.rs` | OrgSlug regex validation |
| `crates/prism-spec-engine/tests/overlay_loading_tests.rs` | Red Gate test coverage |

---

## Pass-4 Findings

---

### SEC-PASS4-001: timeout_secs deferral story S-CONFIG-MULTI-TENANT-OVERRIDE-002 still unregistered in STORY-INDEX.md

- **Severity:** LOW
- **CWE:** CWE-284 (Improper Access Control — security boundary traceability gap), CWE-400 (Uncontrolled Resource Consumption — false operator expectation of timeout enforcement)
- **OWASP:** A05:2021 Security Misconfiguration
- **Attack Vector:** An MSSP operator configures `timeout_secs = 5` in a per-org overlay file. The runtime emits `overlay.timeout_secs_ignored` WARN (now present since `3780ac27`), making the gap visible in the current session's log. However, the deferred wiring work has no registered story anchor in STORY-INDEX.md — it exists only as a named reference in code comments and the BC-2.16.002 warn description. Without registration, the story may be indefinitely deferred or lost in backlog triage, and the 30-second static timeout remains the effective timeout for all orgs regardless of configuration.
- **Impact:** Operator configuration produces a false security posture. The warn emission partially mitigates the silent-discard problem but does not constitute closure of the production-grade deferral requirement. Direction of error is conservative (30s > 5s, not infinite), but the gap persists.
- **Evidence:**
  - overlay.rs:732: commit message cites "deferred to S-CONFIG-MULTI-TENANT-OVERRIDE-002"
  - overlay.rs:103: `OverlayProvenance` doc comment references "follow-up story S-CONFIG-MULTI-TENANT-OVERRIDE-002"
  - `.factory/stories/STORY-INDEX.md`: no row for `S-CONFIG-MULTI-TENANT-OVERRIDE-002`
  - BC-2.16.002 row 42 description: "Deferred to S-CONFIG-MULTI-TENANT-OVERRIDE-002" — the catalog row correctly documents the deferral, but STORY-INDEX.md is the authoritative registry for story existence
- **Proposed Mitigation:** Register `S-CONFIG-MULTI-TENANT-OVERRIDE-002` in STORY-INDEX.md with explicit scope ("wire `timeout_secs` overlay field to adapter HTTP client construction") and a concrete dependency anchor (e.g., "depends on Wave 3 adapter refactor" or equivalent). Alternatively, if `S-SPEC-TYPE-UNIFICATION-001` will absorb this work, rename all references to that story ID. Per Canonical Principle Rule 3, the deferral target must be a real story ID with a concrete future dependency and a specific future story/wave anchor.

---

### SEC-PASS4-002: sensor_id filesystem stem embedded unsanitized in E-SPEC-021 instance_id message

- **Severity:** LOW
- **CWE:** CWE-117 (Improper Output Neutralization for Logs)
- **OWASP:** A09:2021 Security Logging and Monitoring Failures
- **Attack Vector:** An actor with filesystem write access to the prism deployment directory creates an overlay file whose name stem contains control characters (e.g., a newline: `armi\ns.sensor.toml`). The `sensor_id` derived at overlay.rs:397 is passed to `validate_overlay_toml` as `expected_sensor_id`. When the overlay contains a `[[tables]]` block, the check at line 577-584 fires:
  ```rust
  let instance_id_for_msg = format!("{}@{}", expected_sensor_id, expected_org_slug);
  validation_errors.push(make_e_spec_021_tables_in_overlay(
      overlay_file_path,
      &instance_id_for_msg,
  ));
  ```
  `instance_id_for_msg` contains the unsanitized stem. `make_e_spec_021_tables_in_overlay` embeds it directly in `SpecError::message` (overlay.rs:808-812) without calling `sanitize_for_log`. At boot.rs:695, this `SpecError::message` is formatted into the `BootError::ConfigInvalid` string via `format!("  - {e}")` using the Display impl. This string may be logged to stderr and forwarded to SIEM pipelines.
- **Impact:** Log injection into boot error messages and SIEM pipelines via the E-SPEC-021 message body. Prerequisite: filesystem write access to the deployment directory. This attacker prerequisite means the attacker is already past primary perimeter controls. Severity is LOW, not MEDIUM, for this reason. The `expected_org_slug` portion is safe (OrgSlug regex-validated to `[a-zA-Z0-9_-]{1,64}`).
- **Evidence:**
  - overlay.rs:397: `let sensor_id = &file_name[..file_name.len() - ".sensor.toml".len()];` — no control char filtering
  - overlay.rs:580: `let instance_id_for_msg = format!("{}@{}", expected_sensor_id, expected_org_slug);` — raw stem embedded
  - overlay.rs:805-816: `make_e_spec_021_tables_in_overlay` embeds `instance_id` directly with no `sanitize_for_log` call
  - boot.rs:694-695: `format!("  - {e}")` on SpecError Display — message reaches BootError::ConfigInvalid string
  - Contrast: `make_e_spec_019_unknown_extends` (overlay.rs:876): `let safe_extends = sanitize_for_log(extends_value);` — correctly sanitized. `make_e_spec_020_instance_id_mismatch` (overlay.rs:853): `let safe_actual = sanitize_for_log(actual_instance_id);` — correctly sanitized. `make_e_spec_023_unrecognized_field` (overlay.rs:827): `let safe_field = sanitize_for_log(field_name);` — correctly sanitized. The E-SPEC-021 path is the remaining unsanitized surface.
- **Proposed Mitigation:** Apply `sanitize_for_log` to `expected_sensor_id` before it is used in `instance_id_for_msg`:
  ```rust
  let safe_sensor_id_for_msg = sanitize_for_log(expected_sensor_id);
  let instance_id_for_msg = format!("{}@{}", safe_sensor_id_for_msg, expected_org_slug);
  ```
  Note: the raw `expected_sensor_id` should still be used for the TYPE spec lookup (line 464 in the calling scope), since a control-char stem will simply fail to match any key and produce E-SPEC-019 — which is the correct behavior. Only the message-construction path requires sanitization.

---

## Pass-4 Full Security Probe Results

### Probe 1: SEC-PASS3-001 Closure (timeout_secs warning)

- `overlay.timeout_secs_ignored` warn emitted in `merge_overlay_onto_type_spec` (overlay.rs:727-733). Structured fields: `sensor_id = %type_spec.sensor_id` (semantic SensorId, safe), `timeout_secs = timeout_secs` (u64 integer, safe). PASS.
- BC-2.16.002 v1.29 row 42 present with full schema at line 119. SAP-1 PASS.
- `S-CONFIG-MULTI-TENANT-OVERRIDE-002` NOT in STORY-INDEX.md. Rule 3 violation persists. FAIL (LOW).

### Probe 2: Full Log Injection Surface (all error constructors, fresh sweep)

| Constructor | User-controlled field | Sanitized? | Path |
|------------|----------------------|------------|------|
| `make_e_spec_019_unknown_extends` | `extends_value` (TOML-sourced) | YES — `safe_extends` | overlay.rs:877 |
| `make_e_spec_020_instance_id_mismatch` | `actual_instance_id` (TOML-sourced) | YES — `safe_actual` | overlay.rs:853 |
| `make_e_spec_021_tables_in_overlay` | `instance_id_for_msg` (filesystem stem + slug) | NO — raw stem | overlay.rs:580, 808 |
| `make_e_spec_022_unknown_org_slug` | `slug` (OrgSlug pre-validated) | N/A — regex-validated | overlay.rs:822+ |
| `make_e_spec_023_unrecognized_field` | `field_name` (TOML key) | YES — `safe_field` | overlay.rs:827 |
| SEC-REDUX-006 SSRF rejection | `overlay_base_url` (TOML-sourced) | YES — `sanitize_for_log(...)` | overlay.rs:630+ |
| `overlay.timeout_secs_ignored` warn | `type_spec.sensor_id` (TYPE spec validated), `timeout_secs` (u64) | N/A — semantically safe | overlay.rs:729-730 |

E-SPEC-021 remains the single remaining unsanitized surface.

### Probe 3: Credential Safety

- `OrgSlug::new_unchecked`: Only in `#[cfg(test)]` module (`write_dispatch.rs`). PASS.
- `overlay_loading_tests.rs`: No `OrgSlug::new_unchecked` usage. PASS.
- Overlay fixture files (`acme/armis.sensor.toml`, `contoso/armis.sensor.toml`): scalar fields only; no secrets or tokens. PASS.
- Error messages: no credential values appear in any error message body. PASS.
- `overlay.timeout_secs_ignored` warn: `type_spec.sensor_id` is a sensor type name (non-sensitive), `timeout_secs` is a u64 integer (non-sensitive). PASS.

### Probe 4: HTTP Client Timeouts (all four adapters)

- `ArmisAdapter::new` (armis.rs): `.timeout(Duration::from_secs(30))`. PASS.
- `ClarotyAdapter::new` (claroty.rs): `.timeout(Duration::from_secs(30))`. PASS.
- `ClarotyAdapter::fetch` audit_logs path (claroty.rs): `.timeout(Duration::from_secs(30))`. PASS.
- `CrowdStrikeAdapter::new` (crowdstrike.rs): `.timeout(Duration::from_secs(30))`. PASS.
- `CyberintAdapter::new` (cyberint.rs): `.timeout(Duration::from_secs(30))`. PASS.
- All four adapters use static 30s timeout. `timeout_secs` overlay field is not yet wired (deferred, warn emitted). PASS (no production timeout regression).

### Probe 5: File I/O Perimeter

All probes unchanged from pass-3:
- Symlink at org-directory level: `file_type().is_dir()` guard. PASS.
- Symlink at file level: `file_ft.is_file()` guard. PASS.
- File size limit: `MAX_OVERLAY_FILE_BYTES = 64 * 1024` enforced before `read_to_string`. PASS.
- Path traversal via `..` in directory names: blocked by OrgSlug regex. PASS.
- Path traversal via `..` in file names: stem with `..` fails E-SPEC-019 TYPE spec lookup; not a traversal risk. PASS.

### Probe 6: Multi-Tenant Isolation

- Unregistered slug overlays scanned but NOT merged into resolved map (`continue` at line 456). PASS.
- `ResolvedSpecKey` uses `(OrgSlug, SensorId)` newtype tuple. PASS.
- `resolved_spec_map` is read-only after boot (INV-OVL-006), shared via `Arc<HashMap>`. PASS.

### Probe 7: Arc Plumbing

- `resolved_spec_map` threads from boot step 4 through `RunningServer` → `QueryEngine` → `MaterializationContext` as read-only `Arc<HashMap>`. No mutable state introduced. No new attack surface. PASS.

### Probe 8: sanitize_for_log Unit Tests (F-PR155-P2-003)

Six unit tests added in `#[cfg(test)] mod tests` (overlay.rs:909-994):
- `sanitize_for_log_replaces_newline_with_replacement_char`: asserts `\n` → U+FFFD. Load-bearing. PASS.
- `sanitize_for_log_replaces_carriage_return`: asserts `\r\n` → U+FFFD for both chars. Load-bearing. PASS.
- `sanitize_for_log_replaces_null_byte`: asserts `\x00` → U+FFFD. Load-bearing. PASS.
- `sanitize_for_log_truncates_at_256_chars`: asserts char count = 256 for 300-char input. Load-bearing. PASS.
- `sanitize_for_log_passes_clean_ascii_unchanged`: asserts identity for ASCII-clean input. Load-bearing. PASS.
- `sanitize_for_log_preserves_unicode_non_control`: asserts emoji (U+1F600) and CJK (U+4E2D) preserved, U+FFFD NOT present. Load-bearing. PASS.

No paper-fix concerns (TD-VSDD-059): all tests call the production `sanitize_for_log` function with concrete inputs and assert on output properties.

### SAP-1: Tracing Emission Catalog Completeness

New `event_type` value introduced in `3780ac27`:
- `overlay.timeout_secs_ignored` (overlay.rs:728): BC-2.16.002 v1.29 row 42 present at line 119 with full field schema (`sensor_id: %display`, `timeout_secs: u64`), audit role (operational observability), recurrence policy (one per overlay merge where timeout_secs is set). PASS.

All previously-cataloged events from this PR unchanged.

### SAP-2: DTU-TOML Schema Parity

Not applicable — no DTU clone sensor TOML `[[tables]]` columns modified by this PR. PASS.

---

## Summary Table

| Finding | Severity | Pass Introduced | Status at Pass 4 |
|---------|----------|-----------------|------------------|
| SEC-PASS2-002 (extends_value log injection) | MED | Pass 2 | CLOSED |
| SEC-PASS2-003 (sanitize_for_log doc comment) | OBS | Pass 2 | CLOSED |
| SEC-PASS3-001 observability gap (timeout_secs warn) | LOW | Pass 3 | PARTIALLY MITIGATED — warn added; story registration gap persists; see SEC-PASS4-001 |
| SEC-PASS3-002 (sensor_id stem unsanitized — E-SPEC-021) | LOW | Pass 3 | OPEN — no fix applied; see SEC-PASS4-002 |
| SEC-PASS3-003 (aql_preview log injection — pre-existing) | OBS | Pass 3 | OPEN, OUT OF PR SCOPE |
| SEC-PASS4-001 (timeout_secs story unregistered) | LOW | Pass 4 | OPEN |
| SEC-PASS4-002 (sensor_id stem in E-SPEC-021 message) | LOW | Pass 4 | OPEN |

Note: SEC-PASS4-001 and SEC-PASS4-002 are the same underlying issues as SEC-PASS3-001 and SEC-PASS3-002 respectively, re-stated under pass-4 IDs to reflect the current finding state after the fix-burst.

### Severity Counts (Pass 4)

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 0 |
| MEDIUM | 0 |
| LOW | 1 (SEC-PASS4-001 + SEC-PASS4-002 carried as 2 sub-items under this count — both LOW) |
| OBS | 1 (SEC-PASS3-003, pre-existing, out of scope) |

Correction: 2 LOW findings + 1 OBS finding.

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
Two LOW findings remain open (SEC-PASS4-001, SEC-PASS4-002) plus one OBS finding (SEC-PASS3-003, pre-existing, out of PR scope). No CRITICAL, HIGH, or MEDIUM findings.

- SEC-PASS4-001 is a strict finding because it is a Canonical Principle Rule 3 process violation: deferral without a registered story anchor in STORY-INDEX.md. The warn emission partially addresses the security behavioral gap, but Rule 3 requires an explicit story registration with concrete dependency and anchor.
- SEC-PASS4-002 is the unmodified SEC-PASS3-002: `instance_id_for_msg` in E-SPEC-021 path still embeds unsanitized filesystem stem.

**CLEAN(PR-merge): YES**
Zero CRITICAL/HIGH/MEDIUM findings at PR HEAD `3780ac27`. Both open findings are LOW severity:
- SEC-PASS4-001: direction is conservative (warn is now emitted; 30s > 5s, not unsafe); the story-registration gap is a process discipline issue, not an active exploit path.
- SEC-PASS4-002: requires filesystem write access as prerequisite; slug portion is regex-validated; risk is constrained to log injection with attacker already past perimeter controls.
- SEC-PASS3-003: pre-existing, out of PR scope, OBS severity.

The PR is cleared for merge from a security perspective. The two LOW findings should be addressed before the next story that touches overlay error handling or the story decomposition registry.

### Recommended Actions Before Next Story

| Priority | Action | Finding |
|----------|--------|---------|
| P3 (pre-next-story) | Register `S-CONFIG-MULTI-TENANT-OVERRIDE-002` in STORY-INDEX.md with explicit scope (wire `timeout_secs` to HTTP client) and a concrete dependency/anchor | SEC-PASS4-001 |
| P3 (pre-next-story) | Apply `sanitize_for_log` to `expected_sensor_id` before constructing `instance_id_for_msg` in `validate_overlay_toml` E-SPEC-021 path | SEC-PASS4-002 |
| P4 (backlog) | Add `sanitize_for_log` to `aql_preview` in `build_aql` (AQL audit events) | SEC-PASS3-003 |
