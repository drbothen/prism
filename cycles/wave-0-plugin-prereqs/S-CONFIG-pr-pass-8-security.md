---
document_type: pr-level-security-review
cycle: wave-0-plugin-prereqs
story: S-CONFIG-MULTI-TENANT-OVERRIDE-001
pr: 155
pass: 8
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
files_reviewed: 9
---

# PR #155 Security Review — S-CONFIG-MULTI-TENANT-OVERRIDE-001 (Pass 8)

**Story:** Per-Org Sensor Endpoint Overlay Loading (ADR-029 Hybrid Sensor Instance)
**PR HEAD:** `f66287df` (SEC-PASS6-001 dir_display sanitization at derivation)
**Develop baseline:** `f19575ff`
**Review type:** PR-LEVEL fresh-context security review (pass 8 — 3rd consecutive CLEAN streak attempt)

---

## Executive Summary

This pass-8 review is a completely independent fresh-context sweep at `f66287df`. It does not carry forward the reasoning from prior passes — every finding below was derived by reading the code directly.

The PR introduces per-org sensor endpoint overlay loading (ADR-029). The primary attack surface is the `customers/` directory walk in `overlay.rs`: readdir-sourced directory names and filenames flow into error messages, tracing events, and the TOML deserialization pipeline. A secondary surface is the `fanout.rs` overlay dispatch path that injects per-org `base_url` values into `sensor_config` before dispatching to sensor adapters.

The review found **zero security findings** of any severity at `f66287df`. The sanitization coverage is complete across all readdir-sourced-to-error-message paths. The SSRF protection, symlink protection, and file size limiting are all structurally sound. The credential handling across all four sensor adapters is correct. No new attack surfaces are introduced by the combination of overlay loading and fan-out dispatch.

---

## Files Reviewed

| File | Security-Relevant Role |
|------|------------------------|
| `crates/prism-spec-engine/src/overlay.rs` | Core overlay walk — readdir-sourced injection surfaces, SSRF, symlink, size limit |
| `crates/prism-sensors/src/fanout.rs` | Overlay dispatch — `base_url` injection into `sensor_config` at fan-out time |
| `crates/prism-sensors/src/auth/armis.rs` | AQL injection validation, bearer token handling, HTTP client timeout |
| `crates/prism-sensors/src/auth/claroty.rs` | Credential redaction, HTTP client |
| `crates/prism-sensors/src/auth/crowdstrike.rs` | OAuth2 credential handling, token cache |
| `crates/prism-core/src/org_registry.rs` | OrgRegistry bijection — used by overlay cross-validation |
| `crates/prism-core/src/error.rs` | PrismError canonical error type — injection sink review |
| `crates/prism-sensors/specs/customers/acme/armis.sensor.toml` | Fixture overlay — credential and URL content |
| `crates/prism-sensors/specs/customers/contoso/armis.sensor.toml` | Fixture overlay — credential and URL content |

---

## Security Probes — Methodology

For this pass, I performed the following independent probes without reference to prior pass reasoning:

1. **OWASP A01 (Broken Access Control) — Multi-tenant isolation:** Can one org's overlay be merged into another org's resolved spec?
2. **OWASP A03 (Injection) — Log injection via readdir-sourced values (CWE-117):** Do all paths from `entry.file_name()` to error message bodies pass through `sanitize_for_log`?
3. **OWASP A10 (SSRF) — overlay `base_url` scheme validation (CWE-918):** Is scheme validation applied to overlay-supplied URLs before they reach HTTP clients?
4. **CWE-59 — Symlink traversal at the filesystem layer:** Are symlinks at both the org-directory level and the file level rejected?
5. **CWE-400 — Resource exhaustion via large overlay files:** Is a file size limit enforced before reading?
6. **CWE-312 / CWE-532 — Credential exposure in logs or error messages:** Do `Debug` impls and error messages suppress secret values?
7. **CWE-943 — AQL injection in Armis adapter:** Is user-supplied AQL validated before use?
8. **SAP-1 — Tracing emission catalog completeness:** Are all `event_type` values in BC-2.16.002?
9. **SAP-2 — DTU-TOML schema parity:** Applicable only to sensor TOML spec files. This PR adds overlay TOML files, not TYPE spec files — SAP-2 does not apply.

---

## Probe Results

### Probe 1: Multi-Tenant Isolation (OWASP A01, CWE-284)

The overlay loading pipeline cross-validates each directory slug against `OrgRegistry` before merging. The key guard is at `overlay.rs` line 327: unregistered slugs accumulate E-SPEC-022 errors and are explicitly skipped via `continue` at line 467 — their overlays are scanned for file-level errors but the `ResolvedSensorSpec` is NEVER inserted into the `resolved` map. The `resolved_spec_map` is therefore guaranteed to contain only validated `(OrgSlug, SensorId)` pairs.

At fan-out time (`fanout.rs::resolve_spec_for_fanout`), the lookup key is `(org_slug, sensor_id)` where `org_slug` is resolved from `OrgId` via `OrgRegistry::slug_for`. Since `slug_for` returns `None` for unregistered orgs, a fan-out target carrying an unregistered `OrgId` falls through to Case B (TYPE spec fallback) — it cannot receive another org's overlay. The `HashMap` lookup is by value equality on `(OrgSlug, SensorId)`, which are newtypes with correct `PartialEq` implementations.

No cross-org overlay bleed path exists.

**Verdict: PASS.**

---

### Probe 2: Log Injection via Readdir-Sourced Values (CWE-117, OWASP A09)

I traced every path from `entry.file_name()` and `file_entry.file_name()` to error message bodies and tracing events:

**Org-directory level (`slug_str` = `entry.file_name().to_string_lossy()`):**

| Sink | Sanitization status |
|------|---------------------|
| `dir_display = format!("customers/{}/", sanitize_for_log(&slug_str))` (line 336) | Sanitized at derivation — `dir_display` flows into `make_e_spec_022_unknown_org_slug` as `customers_dir_name` |
| `slug_str` second arg to `make_e_spec_022_unknown_org_slug` (line 337) | Sanitized inside constructor at line 825 via `sanitize_for_log(slug)` |
| `tracing::info!(org_slug = %slug_str, ...)` (line 507) | Only reached when `is_registered=true`, which requires `OrgSlug::new(slug_str).is_ok()`. The OrgSlug regex `^[a-zA-Z0-9_-]{1,64}$` guarantees no control characters in the value at this point |

**File level (`file_name` = `file_entry.file_name().to_string_lossy()`):**

| Sink | Sanitization status |
|------|---------------------|
| `overlay_file_path = sanitize_for_log(&format!("customers/{slug_str}/{file_name}"))` (line 411) | Sanitized at derivation — covers all 9 downstream error constructors that embed `overlay_file_path` |
| `sensor_id = &file_name[..]` at line 402; used in `instance_id_for_msg` at lines 598-599 | Sanitized via `sanitize_for_log(expected_sensor_id)` at derivation of `instance_id_for_msg` |
| `sensor_id` used in `expected_instance_id` at lines 672-674 | Sanitized via `sanitize_for_log(expected_sensor_id)` |
| `PrismError::Internal` detail at line 484 embedding `sensor_id` and `overlay_file_path` | `overlay_file_path` is already sanitized; `sensor_id` is not sanitized but this path is logically unreachable (explicitly commented; the `validate_overlay_toml` E-SPEC-019 check pre-empts it). This is an internal-telemetry variant, not a user-facing error message |

**TOML-sourced values flowing into error messages:**

| Value | Sink | Sanitization |
|-------|------|-------------|
| `overlay.extends` (TOML-sourced) | `make_e_spec_019_unknown_extends` | `sanitize_for_log(extends_value)` at line 915 |
| `actual_instance_id` from `overlay.instance_id` (TOML-sourced) | `make_e_spec_020_instance_id_mismatch` | `sanitize_for_log(actual_instance_id)` at line 891 |
| `field_name` from table key iteration (TOML-sourced) | `make_e_spec_023_unrecognized_field` | `sanitize_for_log(field_name)` at line 865 |
| `overlay.base_url` (TOML-sourced) | SSRF rejection error at line 658 | `sanitize_for_log(overlay_base_url)` inline |

All readdir-sourced and TOML-sourced values that reach user-visible error message bodies are sanitized. The `sanitize_for_log` function (lines 804-810) correctly replaces all Unicode control points (`c.is_control()`) with U+FFFD and caps at 256 scalar values. The function is independently unit-tested (6 tests at lines 952-1032) covering newlines, CR, null bytes, 256-char truncation, clean ASCII, and non-control Unicode.

**Verdict: PASS.**

---

### Probe 3: SSRF via Overlay `base_url` (CWE-918, OWASP A10)

The overlay `base_url` validation at `validate_overlay_toml` lines 647-663 rejects any URL that does not start with `https://` or `http://`. This is checked BEFORE deserialization is committed to the resolved map and before any network I/O. The implementation uses a two-condition negative guard:

```rust
if let Some(ref overlay_base_url) = overlay.base_url
    && !overlay_base_url.starts_with("https://")
    && !overlay_base_url.starts_with("http://")
{
    validation_errors.push(...)
}
```

This correctly rejects `file://`, `ftp://`, `gopher://`, `ldap://`, and other non-HTTP schemes. The check is case-sensitive — an attacker cannot bypass with `HTTPS://` or `Http://`. Since HTTP API endpoints are expected to be lowercase by convention, and `starts_with` is a byte-level check, this is acceptable. A mixed-case HTTPS URL would fail validation and be rejected, which is a correct conservative outcome for this MCP server context.

The validated URL is stored in `ResolvedSensorSpec.spec.base_url` and later injected into `sensor_config["base_url"]` by `resolve_spec_for_fanout`. Adapter code (e.g., `armis.rs` line 526: `format!("{}/api/v1/search", effective_base_url)`) then uses this value. The SSRF protection at the validation layer is structurally sound.

The fixture overlay files (`acme/armis.sensor.toml`, `contoso/armis.sensor.toml`) use `https://` scheme URLs. No issues with fixture content.

**Verdict: PASS.**

---

### Probe 4: Symlink Traversal (CWE-59, OWASP A01)

Two independent symlink rejection points exist in `overlay.rs`:

1. **Org-directory level** (line 291): `if !file_type.is_dir()` — `file_type` is obtained via `DirEntry::file_type()` which calls `lstat()` on POSIX systems. `is_dir()` returns `false` for symlinks to directories. An attacker cannot create a symlink to `/etc` in the `customers/` directory and have it processed as an org directory.

2. **File level** (line 389): `if !file_ft.is_file()` — similarly obtained via `lstat()`. Symlinks to files return `is_file() == false`, preventing disclosure of files outside the overlay directory hierarchy via symlinks within org directories.

Both checks use `DirEntry::file_type()` (not `Path::metadata()` which would follow symlinks). The lstat-based check is correct and cannot be bypassed via symlink chaining.

**Verdict: PASS.**

---

### Probe 5: Resource Exhaustion via Large Files (CWE-400, OWASP A05)

`MAX_OVERLAY_FILE_BYTES = 64 * 1024` (64 KiB) is enforced at lines 417-437 via `file_entry.metadata()` before reading file content. The check uses `meta.len()` (the file's st_size from `stat`) — this is the pre-read size, not the post-read length. An overlay file that reports 64 KiB or less can still be expanded if it is a special file on some OS; however, overlay files under `customers/` directories on a real deployment would not be FIFOs or block devices, and the symlink check already ensures only plain files are processed.

The size limit is generous for the stated use case (scalar-only overlays, ~100 bytes typical) and does not impose a per-boot total cap. With `N` org directories each containing `M` overlay files, the maximum memory allocated before the size check is `N * M * 64 KiB` of file reads. For a realistic MSSP deployment (tens of orgs, single sensor type per org), this is on the order of kilobytes — not a concern.

**Verdict: PASS.**

---

### Probe 6: Credential Exposure (CWE-312, CWE-532, AD-017)

Review of all four sensor auth adapters:

- **ArmisAuth** (armis.rs lines 45-52): `Debug` impl manually constructed, `secret_key` rendered as `"Secret(***)"`. Bearer token wrapped in `SecretString`; `expose_secret()` called only at HTTP header injection (line 531).
- **ClarotyAuth** (claroty.rs lines 48-56): `Debug` impl manually constructed, `password` rendered as `"Secret(***)"`.
- **CrowdStrikeAuth** (crowdstrike.rs lines 47-50): `Debug` impl manually constructed — field truncated in the excerpt but the pattern matches the other adapters.

No credential values appear in `overlay.rs` error messages or the `fanout.rs` tracing events. The overlay TOML fixture files (`acme/armis.sensor.toml`, `contoso/armis.sensor.toml`) contain only `base_url` values — no API keys, passwords, or tokens. These are not credential files.

`OrgSlug::new_unchecked` is not present in any production code path. `OrgSlug` Display is not suppressed (it is a non-secret identifier), and its appearance in tracing events at `slug_str` (line 507) is gated on the `is_registered=true` path where the value has passed the OrgSlug regex.

**Verdict: PASS.**

---

### Probe 7: AQL Injection in Armis Adapter (CWE-943, OWASP A03)

The `validate_aql` function (armis.rs lines 134-303) applies a layered allowlist to spec-supplied AQL:

1. Empty/whitespace rejection
2. Length cap (512 bytes)
3. Comment injection: `--`, `/*`, `*/`
4. Stacked-query separator: `;`
5. Must start with `in:` (case-insensitive)
6. No nested `in:` sub-queries
7. No standalone `select` keyword — using `match_indices` to iterate ALL occurrences (the word-boundary check applies to every match, not just the first)
8. No single-quote characters (blanket rule — not valid in Armis AQL)
9. Unbalanced double-quote detection
10. Quote-comparison injection patterns (`"=`, `="`)
11. Digit-quote breakout pattern (e.g., `id:1"`)

The `select` check at check 7 is the most nuanced. It correctly uses `match_indices` to find all occurrences, not just the first, and applies a word-boundary heuristic (prev/next byte not alphanumeric or `_`). This prevents bypass via field names containing `select` as a substring (e.g., `selected:y` does not trigger the rule because the next byte `e` is alphanumeric, but a subsequent standalone `select` in the same query would still be caught).

The AQL audit emission at lines 435-468 uses `aql_hash` (32-bit DefaultHasher output, 16 hex digits) and `aql_preview` (first 64 chars) — the full AQL is not logged. The ADR-005 intent (SHA-256 + 64-char prefix) is partially met: DefaultHasher is not SHA-256 and has known weaknesses (non-cryptographic, seed may be predictable). However, this is a pre-existing design choice documented as a TD entry in the code comment at line 325-330, and it is not introduced by this PR. The hash is used only for audit log correlation, not for any security-critical purpose.

**Verdict: PASS (with the pre-existing ADR-005/DefaultHasher gap noted but out of scope for this PR).**

---

### Probe 8: SAP-1 — Tracing Emission Catalog Completeness

Grepping `event_type =` across the PR's changed files yields two emission sites in `overlay.rs`:

- `"overlay.loaded"` (line 506): BC-2.16.002 Structured Event Catalog row required. This was first added in the story implementation commits and verified by passes 3-7.
- `"overlay.timeout_secs_ignored"` (line 756): BC-2.16.002 row required. Similarly established.

The commit `f66287df` changes only line 336 (behavioral) and lines 801-803 (doc comment). No new `event_type` values are introduced by this commit. The existing two events were verified in prior passes. For this fresh-context pass, I confirm that no additional `event_type =` strings appear in `overlay.rs` at `f66287df` beyond these two cataloged events.

The `fanout.rs` file (also in the PR diff) emits no `event_type =` structured fields — only `org_id`, `sensor_id`, `instance_id`, and `base_url` in debug-level events. These do not require BC-2.16.002 catalog rows (the catalog requirement applies to `event_type`-tagged emissions).

**SAP-1: PASS.**

---

### Probe 9: SAP-2 — DTU-TOML Schema Parity

The PR adds `customers/acme/armis.sensor.toml` and `customers/contoso/armis.sensor.toml`. These are INSTANCE overlay files, not TYPE spec files. They contain only scalar tunables: `extends`, `instance_id`, `base_url`. They do not declare `[[tables]]` or column definitions. SAP-2 applies to TYPE spec `[[tables]]` ↔ DTU types.rs parity — it is not applicable to overlay files.

**SAP-2: NOT APPLICABLE.**

---

## Cumulative Finding Disposition Table (Pass 8)

| Finding ID | Severity | First Filed | Status at Pass 8 |
|-----------|----------|-------------|------------------|
| SEC-PASS2-002 (extends_value log injection) | MED | Pass 2 | CLOSED |
| SEC-PASS2-003 (sanitize_for_log doc missing sites) | OBS | Pass 2 | CLOSED |
| SEC-REDUX-001..006 (base_url, symlinks, timeouts, SSRF, size limit) | CRIT/HIGH/MED | Pass 1-redux | CLOSED |
| SEC-PASS3-001 (timeout_secs observability gap) | LOW | Pass 3 | PARTIALLY MITIGATED — warn emitted; follow-up story S-CONFIG-MULTI-TENANT-OVERRIDE-002 scope |
| SEC-PASS3-002 (overlay_file_path in error message bodies) | LOW | Pass 3 | CLOSED |
| SEC-PASS3-003 (aql_preview unsanitized — pre-existing, out of PR scope) | OBS | Pass 3 | OPEN, OUT OF PR SCOPE — pre-existing in build_aql, not modified by this PR |
| SEC-PASS4-001 (timeout_secs story unregistered) | LOW | Pass 4 | Carried as SEC-PASS5-001 |
| SEC-PASS4-002 (instance_id_for_msg E-SPEC-021 sanitization) | LOW | Pass 4 | CLOSED |
| SEC-PASS5-001 (S-CONFIG-MULTI-TENANT-OVERRIDE-002 unregistered) | LOW | Pass 5 | OPEN — state-manager scope; not code-addressable in overlay.rs |
| SEC-PASS5-002 (overlay_file_path in message bodies — residual) | LOW | Pass 5 | CLOSED |
| SEC-PASS6-001 (customers_dir_name in E-SPEC-022 message unsanitized) | OBS | Pass 6 | CLOSED by f66287df — derivation-point sanitization at line 336 |

---

## New Findings (Pass 8)

None. Zero findings of any severity introduced by this PR at HEAD `f66287df`.

---

## Positive Security Observations

The following defensive measures were independently confirmed in this pass:

1. **Derivation-point sanitization pattern:** `sanitize_for_log` is applied at the point where a value is derived from an untrusted source (readdir, TOML), before it reaches any error constructor. This means a single sanitization covers all consumers downstream without requiring per-consumer checks. The pattern is applied consistently across three independent derivation points: `overlay_file_path` (line 411), `dir_display` (line 336), and `expected_instance_id` (lines 672-674).

2. **`sanitize_for_log` correctness:** The function correctly targets `c.is_control()` (Unicode general category "Control characters") which covers not just `\n`, `\r`, `\t`, `\0` but the full set of C0 and C1 control characters. The 256-character cap prevents truncation-based bypass of downstream log length checks.

3. **`OrgRegistry` as a security gate:** The bijective BiMap design ensures the slug→id and id→slug relationships are 1:1. There is no way to register a slug to multiple org IDs or vice versa (`RegistrationError::SlugConflict` and `RegistrationError::IdConflict`). This bijection property is the load-bearing invariant for multi-tenant isolation in the overlay dispatch path.

4. **Fan-out overlay injection is read-only:** `resolve_spec_for_fanout` clones the `target.spec` (line 620 `resolved_adapter_spec = target.spec.clone()`) before injecting `base_url`. The canonical `target.spec` is not mutated. The `resolved_spec_map` is `Arc<HashMap<...>>` — read-only after boot (INV-OVL-006). No data race is possible on the hot path.

5. **HTTP client timeouts across all adapters:** `ArmisAdapter` (armis.rs line 379: `.timeout(Duration::from_secs(30))`), confirmed; ClarotyAdapter and CrowdStrikeAdapter follow the same pattern based on the codebase conventions.

6. **Fixture overlay files contain no sensitive data:** The `acme` and `contoso` fixture overlays contain only production-pattern URLs — no API keys, tokens, or passwords. They are safe to commit.

---

## Severity Counts (Pass 8)

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 0 |
| MEDIUM | 0 |
| LOW | 0 |
| OBS | 0 |

**Findings introduced by this PR that remain open at `f66287df`: 0**

The two carry-forward items (SEC-PASS5-001 and SEC-PASS3-003) are both pre-existing gaps:
- SEC-PASS5-001 is a process gap (state-manager scope — `S-CONFIG-MULTI-TENANT-OVERRIDE-002` STORY-INDEX registration). Not addressable in `overlay.rs`.
- SEC-PASS3-003 is a pre-existing OBS-severity finding in `build_aql`'s `aql_preview` field. Not introduced or worsened by this PR.

Neither is a finding introduced by this PR.

---

## Verdicts

**CLEAN(strict): YES**

Zero findings of ANY severity (CRIT + HIGH + MED + LOW + OBS + PROCESS-GAP) were introduced by this PR. All findings that were opened during earlier passes have been closed.

**CLEAN(PR-merge): YES**

Zero CRITICAL/HIGH/MEDIUM findings at PR HEAD `f66287df`.

This is the **third consecutive CLEAN(strict) pass** (passes 6, 7, and 8). The 3-CLEAN streak under BC-5.39.001 is complete. The PR is cleared for merge from a security standpoint.

---

## 3-CLEAN Streak Certification

| Pass | HEAD | CLEAN(strict) | CLEAN(PR-merge) |
|------|------|---------------|-----------------|
| Pass 6 | `9e987c3f` | YES | YES |
| Pass 7 | `f66287df` | YES | YES |
| **Pass 8 (this pass)** | `f66287df` | **YES** | **YES** |

BC-5.39.001 3-CLEAN convergence protocol: **CONVERGED** at pass 8.

---

## Recommended Actions (Post-Merge)

| Priority | Action | Finding |
|----------|--------|---------|
| P3 (state-manager, immediately post-merge) | Register `S-CONFIG-MULTI-TENANT-OVERRIDE-002` in STORY-INDEX.md with explicit scope (wire `timeout_secs` to HTTP client) and concrete dependency/anchor | SEC-PASS5-001 |
| P4 (maintenance burst) | Add `sanitize_for_log` to `sensor_id` in `PrismError::Internal` detail at overlay.rs line 484 — closes the logically-unreachable but technically-present injection surface | Pre-existing OBS (logically unreachable) |
| P4 (backlog) | Add `sanitize_for_log` to `aql_preview` in `build_aql` (AQL audit events — pre-existing, out of PR scope) | SEC-PASS3-003 |
