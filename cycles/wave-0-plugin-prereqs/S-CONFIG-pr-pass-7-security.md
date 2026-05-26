---
document_type: pr-level-security-review
cycle: wave-0-plugin-prereqs
story: S-CONFIG-MULTI-TENANT-OVERRIDE-001
pr: 155
pass: 7
reviewer: security-reviewer
fresh_context: true
feature_head: f66287df
pr_head: f66287df
develop_baseline: f19575ff
timestamp: 2026-05-26T00:00:00Z
total_findings: 0
critical: 0
high: 0
medium: 0
low: 0
obs: 0
files_reviewed: 1
---

# PR #155 Security Review — S-CONFIG-MULTI-TENANT-OVERRIDE-001 (Pass 7)

**Story:** Per-Org Sensor Endpoint Overlay Loading (ADR-029 Hybrid Sensor Instance)
**PR HEAD:** `f66287df` (pass-7 fix-burst — SEC-PASS6-001 dir_display sanitization at derivation)
**Pass-6 HEAD:** `9e987c3f`
**Develop baseline:** `f19575ff`
**Review type:** PR-LEVEL fresh-context security review (pass 7 — SEC-PASS6-001 closure verification + full fresh sweep)

---

## Executive Summary

The fix-burst commit `f66287df` closes SEC-PASS6-001 (`customers_dir_name` embedding raw `slug_str` in the E-SPEC-022 error message body) by applying `sanitize_for_log` at the `dir_display` derivation point (line 336), mirroring the SEC-PASS5-002 derivation-point pattern for `overlay_file_path`. The fix is structural, not a paper-fix: it changes the value that flows into `make_e_spec_022_unknown_org_slug` from potentially containing control characters to a sanitized form. The `sanitize_for_log` function is independently unit-tested (6 tests covering newline, CRLF, null byte, 256-char truncation, clean ASCII, and non-control Unicode). The E-SPEC-022 code path itself is exercised by the integration test at `overlay_loading_tests.rs:568`. No new security findings are introduced by `f66287df`.

The two remaining open items from prior passes (SEC-PASS5-001: process gap — `S-CONFIG-MULTI-TENANT-OVERRIDE-002` unregistered in STORY-INDEX.md; SEC-PASS3-003: `aql_preview` pre-existing OBS) are unchanged in scope and disposition. Neither was modified by this commit.

---

## Pass-7 Source Files Reviewed

| File | Role |
|------|------|
| `crates/prism-spec-engine/src/overlay.rs` | Core overlay — SEC-PASS6-001 closure verification + full fresh sibling sweep at `f66287df` |

Only 1 file changed in `f66287df` (confirmed by `git diff --stat`): `crates/prism-spec-engine/src/overlay.rs` — 9 insertions, 1 deletion.

---

## SEC-PASS6-001 Closure Verification

### SEC-PASS6-001: `customers_dir_name` in `make_e_spec_022_unknown_org_slug` embeds raw `slug_str` without sanitization when OrgSlug validation fails

**Status: CLOSED — VERIFIED REAL FIX (not a paper-fix per TD-VSDD-059)**

The commit `f66287df` applies `sanitize_for_log` at the `dir_display` derivation point:

```rust
// overlay.rs line 336 (at f66287df)
let dir_display = format!("customers/{}/", sanitize_for_log(&slug_str));
```

Previously (at `9e987c3f`):
```rust
let dir_display = format!("customers/{slug_str}/");
```

This is the same derivation-point fix pattern as SEC-PASS5-002 (`overlay_file_path` at line 411). It is correct because:

1. `dir_display` is derived ONCE and then passed as `customers_dir_name` to `make_e_spec_022_unknown_org_slug`. Sanitizing at derivation means the constructor receives a clean string.
2. `dir_display` is used ONLY as a display string in the error message body and `file_path` field — it is never used as an actual filesystem path for I/O after construction.
3. `slug_str` (line 296, `entry.file_name().to_string_lossy().to_string()`) is readdir-sourced on Linux, where directory names may contain any byte except `/` and `\0`. The `^[a-zA-Z0-9_-]{1,64}$` OrgSlug regex catches well-formed slugs but only AFTER this derivation — when `OrgSlug::new(slug_str).is_err()` (invalid slug), the raw `slug_str` is the value that flowed into `dir_display`. The fix closes this path.
4. The `make_e_spec_022_unknown_org_slug` constructor (line 824) independently sanitizes its `slug` second argument. The `dir_display` first argument receives the sanitized form — both arguments to the constructor are now clean.

**TD-VSDD-059 (paper-fix detection) assessment:**

The fix is behavioral, not documentary. The actual string value emitted in the E-SPEC-022 error message body changes when `slug_str` contains control characters: previously a control character would pass through verbatim; now it is replaced with U+FFFD. The `sanitize_for_log` function is independently load-bearing with 6 unit tests. There is no dedicated end-to-end test exercising a control-character directory name (such a test would require filesystem support for control chars in dir names, which varies by OS), but this is an acceptable gap given:

- The fix is in a single expression, not an architectural pattern
- The `sanitize_for_log` unit tests prove the function behavior
- The E-SPEC-022 integration test (`test_BC_2_06_015_unknown_org_dir_aborts_boot_with_e_spec_022`, line 568) exercises the code path end-to-end with a clean ASCII slug
- The attack prerequisite (filesystem write to `customers/` directory) is unchanged

**Verdict: CLOSED. Structural fix — not a paper-fix.**

---

## Pass-7 Full Security Probe Results

### Probe 1: SEC-PASS6-001 Closure — derivation-point sanitization

Verified at overlay.rs line 336. `format!("customers/{}/", sanitize_for_log(&slug_str))` replaces `format!("customers/{slug_str}/")`. PASS.

---

### Probe 2: TD-VSDD-060 Sibling-Sweep — exhaustive readdir-sourced value trace

At `f66287df`, the complete map of readdir-sourced values and their sanitization status:

| Readdir-sourced value | Derivation | Sanitization status | Flows to |
|-----------------------|------------|--------------------|-----------| 
| `dir_name` → `slug_str` (line 296) | `entry.file_name().to_string_lossy()` | Sanitized at line 336 via `sanitize_for_log` in `dir_display` | E-SPEC-022 message + `file_path` field — CLOSED at `f66287df` |
| `slug_str` (line 337 second arg) | same | Sanitized inside `make_e_spec_022_unknown_org_slug` via `safe_slug = sanitize_for_log(slug)` (line 825) | E-SPEC-022 message slug portion — CLOSED pre-f66287df |
| `slug_str` (line 411 in `overlay_file_path`) | same | Sanitized via `sanitize_for_log(&format!("customers/{slug_str}/{file_name}"))` | All 9 downstream error constructors — CLOSED at `9e987c3f` |
| `slug_str` (line 507, `tracing::info!`) | same | At `is_registered=true` branch only — OrgSlug regex validates chars to `[a-zA-Z0-9_-]` — no control chars possible | `overlay.loaded` tracing event — SAFE (validated by construction) |
| `file_name` (line 394) | `file_entry.file_name().to_string_lossy()` | Sanitized as part of `overlay_file_path` derivation at line 411 | All 9 downstream error constructors — CLOSED at `9e987c3f` |
| `sensor_id` = `&file_name[..]` (line 402) | derived from `file_name` | NOT sanitized at line 484 in `PrismError::Internal` | Logically unreachable internal error path — PRE-EXISTING (not modified by `f66287df` or `9e987c3f`) |
| `sensor_id` (line 508, `tracing::info!`) | same | At `is_registered=true` + `type_specs.get(sensor_id).is_some()` branch — type_specs is TOML-sourced; key match guarantees clean value | `overlay.loaded` tracing event — SAFE by construction |

The implementer's commit message claim — "exhaustive grep confirms this is the final unsanitized `format!` embedding a readdir-sourced value in overlay.rs" — is accurate for error-message-body surfaces. The `PrismError::Internal` at line 484 embeds `sensor_id` without sanitization, but this is:

1. Pre-existing (present at `9e987c3f`, `7406458a`, and earlier)
2. In a logically unreachable code path (the comment at line 479 explicitly states "This arm is logically unreachable: validate_overlay_toml returns Err(E-SPEC-019) when extends is unresolvable")
3. An internal error variant, not a user-facing error — `PrismError::Internal` is intended for bug detection in telemetry, not exposed to callers

This gap was identified in the pass-6 analysis of the Internal path but not filed as a finding because it was classified as pre-existing and out-of-scope for the fix-burst (which targeted `dir_display`). Its security severity is LOW-to-OBS given the unreachability and internal-telemetry-only exposure. It is not introduced by `f66287df`.

TD-VSDD-060 verdict: the sibling sweep is complete for the targeted code path. The `PrismError::Internal` unsanitized `sensor_id` is a pre-existing, logically-unreachable, internal-only surface.

---

### Probe 3: complete `sanitize_for_log` callsite sweep (from line 804)

Full list of callsites at `f66287df` HEAD:

| Callsite | Line | Value sanitized | Status |
|----------|------|-----------------|--------|
| `dir_display` derivation (E-SPEC-022 path) | 336 | `slug_str` | CLOSED at `f66287df` |
| `overlay_file_path` derivation | 411 | `format!("customers/{slug_str}/{file_name}")` | CLOSED at `9e987c3f` |
| `instance_id_for_msg` E-SPEC-021 (tables check) | 598, 599 | `expected_sensor_id`, `expected_org_slug` | CLOSED at `7406458a` |
| SSRF rejection (SEC-REDUX-006) | 658 | `overlay_base_url` | CLOSED at earlier fix-burst |
| `expected_instance_id` E-SPEC-020 (instance_id check) | 673, 674 | `expected_sensor_id`, `expected_org_slug` | CLOSED at `9e987c3f` |
| `make_e_spec_022_unknown_org_slug` constructor | 825 | `slug` (second arg to constructor) | Pre-existing — CLOSED |
| `make_e_spec_023_unrecognized_field` constructor | 865 | `field_name` | Pre-existing — CLOSED |
| `make_e_spec_020_instance_id_mismatch` constructor | 891 | `actual_instance_id` | Pre-existing — CLOSED |
| `make_e_spec_019_unknown_extends` constructor | 915 | `extends_value` | Pre-existing — CLOSED |

All readdir-sourced values that reach user-visible error message bodies are now sanitized. The `PrismError::Internal` surface (logically unreachable, internal telemetry only) is the sole remaining gap, and it is pre-existing.

---

### Probe 4: Credential Safety

- `OrgSlug::new_unchecked`: confirmed absent from production code paths. PASS.
- No change to fixture files — no new secrets or credentials introduced. PASS.
- Error messages: no credential values in any message body. PASS.

---

### Probe 5: HTTP Client Timeouts

Not applicable — `f66287df` does not modify any HTTP client code. All four sensor adapters confirmed to retain `.timeout(Duration::from_secs(30))` (unchanged from passes 4-6). PASS.

---

### Probe 6: File I/O Perimeter

Not applicable — `f66287df` does not modify any file I/O logic. Guards remain:
- Symlink at org-directory level: `file_type().is_dir()` (lstat). PASS.
- Symlink at file level: `file_ft.is_file()` (lstat). PASS.
- File size limit: `MAX_OVERLAY_FILE_BYTES = 64 * 1024`. PASS.

---

### Probe 7: Multi-Tenant Isolation

Not applicable — `f66287df` does not modify overlay merge logic. Unregistered slug overlays scanned but NOT merged (explicit `continue` at line 467). `ResolvedSpecKey` and `resolved_spec_map` unchanged. PASS.

---

### SAP-1: Tracing Emission Catalog Completeness

`f66287df` adds zero new `event_type` values. The diff confirms: only line 336 (behavioral change) and lines 801-803 (doc comment) are modified. Previously verified events `overlay.loaded` (row 38) and `overlay.timeout_secs_ignored` (row 42) remain unchanged. SAP-1: PASS.

---

### SAP-2: DTU-TOML Schema Parity

Not applicable — `f66287df` does not touch any sensor TOML specs or DTU crate source. PASS.

---

## Pass-7 Findings

No new findings introduced by `f66287df`.

---

## Pass-7 Cumulative Finding Disposition Table

| Finding ID | Severity | First Filed | Status at Pass 7 |
|-----------|----------|-------------|------------------|
| SEC-PASS2-002 (extends_value log injection) | MED | Pass 2 | CLOSED |
| SEC-PASS2-003 (sanitize_for_log doc missing sites) | OBS | Pass 2 | CLOSED |
| SEC-REDUX-001..006 (base_url, symlinks, timeouts, SSRF, size limit) | CRIT/HIGH/MED | Pass 1-redux | CLOSED |
| SEC-PASS3-001 (timeout_secs observability gap) | LOW | Pass 3 | PARTIALLY MITIGATED — warn emitted; story registration gap persists as SEC-PASS5-001 |
| SEC-PASS3-002 (overlay_file_path in error message bodies) | LOW | Pass 3 | CLOSED by `9e987c3f` fix-burst (SEC-PASS5-002 scope) |
| SEC-PASS3-003 (aql_preview unsanitized — pre-existing, out of PR scope) | OBS | Pass 3 | OPEN, OUT OF PR SCOPE |
| SEC-PASS4-001 (timeout_secs story unregistered) | LOW | Pass 4 | Carried as SEC-PASS5-001 |
| SEC-PASS4-002 (instance_id_for_msg E-SPEC-021 sanitization) | LOW | Pass 4 | CLOSED by `7406458a` fix-burst |
| SEC-PASS5-001 (S-CONFIG-MULTI-TENANT-OVERRIDE-002 unregistered) | LOW | Pass 5 | OPEN — state-manager scope; not code-addressable in overlay.rs |
| SEC-PASS5-002 (overlay_file_path in message bodies — residual from SEC-PASS3-002) | LOW | Pass 5 | CLOSED by `9e987c3f` fix-burst — derivation-point sanitization |
| SEC-PASS6-001 (customers_dir_name in E-SPEC-022 message unsanitized) | OBS | Pass 6 | CLOSED by `f66287df` fix-burst — derivation-point sanitization |

---

## Severity Counts (Pass 7)

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 0 |
| MEDIUM | 0 |
| LOW | 1 (SEC-PASS5-001 — process gap, state-manager scope) |
| OBS | 1 (SEC-PASS3-003 — pre-existing AQL, out of PR scope) |

**Findings introduced by this PR that remain open at `f66287df`: 0**

The LOW (SEC-PASS5-001) is a state-manager scope process gap — `S-CONFIG-MULTI-TENANT-OVERRIDE-002` not registered in STORY-INDEX.md. It is not addressable in `overlay.rs` and does not block merge.

The OBS (SEC-PASS3-003) is a pre-existing finding in `aql_preview` in `build_aql` (AQL audit events), which is out of scope for this PR/story.

---

## Positive Findings (Defensive Measures Present)

- The multi-pass sanitization closure campaign (passes 1-redux through 7) has systematically addressed every readdir-sourced and TOML-sourced injection path in `overlay.rs` error message bodies. The derivation-point pattern (sanitize once, cover all consumers) is now applied consistently across all three major surfaces: `overlay_file_path` (9 consumers), `dir_display` (1 consumer), and `expected_instance_id` components (E-SPEC-020/021 paths).
- `sanitize_for_log` has 6 unit tests proving control-char replacement and truncation. The function is simple, correct, and well-tested.
- OrgSlug `^[a-zA-Z0-9_-]{1,64}$` regex validation at primary perimeter remains intact.
- `MAX_OVERLAY_FILE_BYTES = 64 KiB` file size limit unchanged.
- SSRF protection via scheme whitelist unchanged.
- No credential values in any error message body.
- HTTP client timeout discipline (`30s`) maintained in all four sensor adapters.

---

## Verdicts

**CLEAN(strict): YES**

Zero findings of ANY severity introduced by `f66287df`:
- SEC-PASS6-001 is CLOSED by the derivation-point fix at line 336.
- SEC-PASS5-001 is a pre-existing state-manager scope process gap, not code-addressable in this commit.
- SEC-PASS3-003 is a pre-existing OBS-severity finding from code outside this PR's story scope.
- The `PrismError::Internal` unsanitized `sensor_id` at line 484 is a pre-existing, logically-unreachable, internal-telemetry-only surface — not introduced by any commit in this PR's fix-burst chain and not a user-facing injection path.

No findings of any severity were introduced by `f66287df`. All findings introduced by this PR have been closed.

**CLEAN(PR-merge): YES**

Zero CRITICAL/HIGH/MEDIUM findings at PR HEAD `f66287df`.

The PR is cleared for merge from a security standpoint.

---

## Recommended Actions

| Priority | Action | Finding |
|----------|--------|---------|
| P3 (state-manager, immediately post-merge) | Register `S-CONFIG-MULTI-TENANT-OVERRIDE-002` in STORY-INDEX.md with explicit scope (wire `timeout_secs` to HTTP client) and concrete dependency/anchor | SEC-PASS5-001 |
| P4 (maintenance burst) | Add `sanitize_for_log` to `sensor_id` in `PrismError::Internal` detail at overlay.rs line 484 — closes the logically-unreachable but technically-present injection surface | Pre-existing OBS (not SEC-filed; logically unreachable) |
| P4 (backlog) | Add `sanitize_for_log` to `aql_preview` in `build_aql` (AQL audit events — pre-existing, out of PR scope) | SEC-PASS3-003 |
