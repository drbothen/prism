---
document_type: pr-level-security-review
cycle: wave-0-plugin-prereqs
story: S-CONFIG-MULTI-TENANT-OVERRIDE-001
pr: 155
pass: 6
reviewer: security-reviewer
fresh_context: true
feature_head: 9e987c3f
pr_head: 9e987c3f
develop_baseline: f19575ff
timestamp: 2026-05-26T00:00:00Z
total_findings: 2
critical: 0
high: 0
medium: 0
low: 1
obs: 1
files_reviewed: 7
---

# PR #155 Security Review — S-CONFIG-MULTI-TENANT-OVERRIDE-001 (Pass 6)

**Story:** Per-Org Sensor Endpoint Overlay Loading (ADR-029 Hybrid Sensor Instance)
**PR HEAD:** `9e987c3f` (pass-5 fix-burst — SEC-PASS5-002 overlay_file_path sanitization at derivation + additional E-SPEC-020 path defense)
**Pass-5 HEAD:** `7406458a`
**Develop baseline:** `f19575ff`
**Review type:** PR-LEVEL fresh-context security review (pass 6 — SEC-PASS5-002 closure verification + SEC-PASS5-001 status + full fresh sweep)

---

## Executive Summary

The fix-burst commit `9e987c3f` closes SEC-PASS5-002 (overlay_file_path unsanitized in error message bodies) with a structural derivation-point fix that covers all 9 downstream error constructors. The commit also adds a defense-in-depth sanitization for `expected_sensor_id` and `expected_org_slug` in the E-SPEC-020 path, which is a correct hardening that mirrors the prior SEC-PASS4-002 fix. SEC-PASS5-001 (process gap: `S-CONFIG-MULTI-TENANT-OVERRIDE-002` unregistered in STORY-INDEX.md) was not addressed by this fix-burst and remains open — it is a state-manager scope item, not addressable in `overlay.rs`. One new OBS-severity observation is filed for a pre-existing injection site in `make_e_spec_022_unknown_org_slug` that the pass-6 analysis identified and prior passes did not explicitly catalogue. No CRITICAL, HIGH, or MEDIUM findings remain at HEAD `9e987c3f`.

---

## Pass-6 Source Files Reviewed

| File | Role |
|------|------|
| `crates/prism-spec-engine/src/overlay.rs` | Core overlay — SEC-PASS5-002 closure verification + full fresh sweep at `9e987c3f` |
| `crates/prism-core/src/error.rs` | SpecError Display impl — previously confirmed `file_path` NOT in Display; unchanged |
| `crates/prism-core/src/tenant.rs` | OrgSlug regex validation pattern |
| `crates/prism-bin/src/boot.rs` | Boot step 4 — overlay error propagation path |
| `crates/prism-sensors/src/auth/armis.rs` | Adapter HTTP client + AQL — pre-existing OBS |
| `crates/prism-sensors/src/fanout.rs` | Fan-out overlay dispatch |
| `crates/prism-query/src/engine.rs` | resolved_spec_map Arc plumbing |

---

## SEC-PASS5-002 Closure Verification

### SEC-PASS5-002: overlay_file_path containing raw filesystem data embedded unsanitized in multiple error message bodies

**Status: CLOSED — VERIFIED REAL FIX (not a paper-fix per TD-VSDD-059)**

The commit `9e987c3f` applies `sanitize_for_log` at the `overlay_file_path` derivation point:

```rust
// overlay.rs lines 405-406
let overlay_file_path =
    sanitize_for_log(&format!("customers/{slug_str}/{file_name}"));
```

This is the Option A fix recommended in pass-5 (sanitize at derivation rather than at each embedding site). The approach is structurally correct because:

1. `overlay_file_path` is derived ONCE and then passed to all 9 downstream error constructors. Sanitizing at the derivation point means every consumer receives the sanitized string without requiring changes to the constructors themselves.
2. `overlay_file_path` is used ONLY in error messages — it is never used as a filesystem path for further I/O after construction at line 405-406. Sanitizing it does not corrupt any I/O path.
3. The commit message correctly identifies this as covering "all 9 downstream error constructor sites."
4. The 6 unit tests for `sanitize_for_log` (lines 940-1014) cover the behavior applied here: control-char replacement with U+FFFD and 256-char cap.

Injection sites from pass-5 Probe 2 table — verified closed at `9e987c3f`:

| Site | Line (at `9e987c3f`) | Status |
|------|----------------------|--------|
| E-SPEC-001 (size exceeded) message body | ~416 | CLOSED — `overlay_file_path` now sanitized |
| E-SPEC-001 (TOML parse failure) message body | ~553 | CLOSED |
| E-SPEC-001 (not a TOML table) message body | ~570 | CLOSED |
| E-SPEC-001 (deserialization failure) message body | ~628 | CLOSED |
| E-SPEC-001 (SSRF rejection) message body | ~648 | CLOSED |
| E-SPEC-021 (tables in overlay) via constructor `file_path` param | ~597 | CLOSED |
| E-SPEC-023 (unrecognized field) via constructor `file_path` param | ~610 | CLOSED |
| E-SPEC-020 (instance_id mismatch) via constructor `file_path` param | ~673 | CLOSED |
| E-SPEC-019 (unknown extends) via constructor `file_path` param | ~682 | CLOSED |

**Internal PrismError::Internal (line 477-481) uses `overlay_file_path` in a detail string.** This path (unreachable in correct flow per the code comment) also now receives the sanitized value. CLOSED.

**Verdict: CLOSED. Load-bearing structural fix covering all injection sites.**

---

## SEC-PASS5-001 Status Verification

### SEC-PASS5-001: `S-CONFIG-MULTI-TENANT-OVERRIDE-002` unregistered in STORY-INDEX.md

**Status: OPEN — not addressed by fix-burst `9e987c3f` (state-manager scope, per pass-5 context)**

Confirmed: `9e987c3f` modifies only `crates/prism-spec-engine/src/overlay.rs` (1 file, 21 insertions, 2 deletions). STORY-INDEX.md is not in the diff.

Confirmed: `S-CONFIG-MULTI-TENANT-OVERRIDE-002` does not appear in `/Users/jmagady/Dev/prism/.factory/stories/STORY-INDEX.md` at the time of this review.

The `overlay.timeout_secs_ignored` warn at overlay.rs:750-756 correctly cites the deferral to `S-CONFIG-MULTI-TENANT-OVERRIDE-002`, but that story is not registered as a row in STORY-INDEX.md. This violates Canonical Principle Rule 3 (deferral target must be a real story ID in STORY-INDEX.md with a concrete future dependency and a specific story/wave anchor).

**Carries forward from SEC-PASS5-001. Severity: LOW. No change in disposition.**

---

## Pass-6 Full Security Probe Results

### Probe 1: SEC-PASS5-002 Closure — derivation-point sanitization

Verified at overlay.rs lines 399-406. PASS.

---

### Probe 2: E-SPEC-020 Path Hardening (additional fix in `9e987c3f`)

The commit note says it also sanitizes `expected_sensor_id + expected_org_slug` before constructing `expected_instance_id` in the E-SPEC-020 path:

```rust
// overlay.rs lines 661-670
// BC-2.06.013 Check 3: instance_id convention mismatch → E-SPEC-020.
let expected_instance_id = format!(
    "{}@{}",
    sanitize_for_log(expected_sensor_id),
    sanitize_for_log(expected_org_slug)
);
```

This mirrors the E-SPEC-021 fix from SEC-PASS4-002. The `expected_instance_id` is then compared against `overlay.instance_id` (TOML-sourced); if they differ, `make_e_spec_020_instance_id_mismatch` is called with `&expected_instance_id` as the `expected_instance_id` argument.

Looking at `make_e_spec_020_instance_id_mismatch` (lines 878-895):
- `actual_instance_id` parameter → sanitized via `sanitize_for_log(actual_instance_id)` inside the constructor (previously closed by SEC-PASS4-002 sibling)
- `expected_instance_id` parameter → embedded directly in message at line 888 WITHOUT sanitization inside the constructor

At pass-6 HEAD, `expected_instance_id` is now pre-sanitized before being passed to the constructor. This is a valid defense-in-depth approach: the components are sanitized before format!, so the concatenated string is safe. The constructor's `{expected_instance_id}` embedding is now receiving an already-clean value. This is structurally correct.

**Probe 2 verdict: PASS — E-SPEC-020 hardening is valid and structural.**

---

### Probe 3: Residual Injection Surface in `make_e_spec_022_unknown_org_slug` (NEW)

**This is a fresh-context observation not explicitly catalogued in passes 1-5.**

The `make_e_spec_022_unknown_org_slug` function (lines 816-829) receives two arguments:
- `customers_dir_name: &str` — the `dir_display` format string `"customers/{slug_str}/"` constructed at line 331 using raw `slug_str`
- `slug: &str` — the raw `slug_str` for the unregistered org

Inside the function, `slug` is sanitized via `sanitize_for_log(slug)`, but `customers_dir_name` is embedded directly in the message format string at line 821:

```rust
// line 821 — customers_dir_name embeds raw slug_str
message: format!(
    "Per-org overlay directory '{customers_dir_name}' references org slug '{safe_slug}' ...",
)
```

The `customers_dir_name` value is `format!("customers/{slug_str}/")` where `slug_str` comes from `entry.file_name().to_string_lossy()` (line 296) — raw OS directory name.

The E-SPEC-022 code path fires when `is_registered = false` (lines 321-333). `is_registered` is false either when:
1. `org_slug.is_err()` (OrgSlug regex validation `^[a-zA-Z0-9_-]{1,64}$` fails) — `slug_str` may contain control chars
2. `org_slug.is_ok()` but the slug is not in OrgRegistry — `slug_str` is then safe (passed regex)

In case (1), the raw `slug_str` with potential control chars flows through `dir_display` into `customers_dir_name`, which is then embedded in the message body without sanitization.

**Assessment:** This is a pre-existing LOW-severity residual injection site. It was partially assessed in pass-1-redux at line 305 ("OrgSlug::new() validation prevents injection via `org_slug` / `dir_display`") — but that assessment was accurate only for case (2), not case (1). The code change in pass-1-redux did not add sanitization for `customers_dir_name`; it was accepted under the assumption that OrgSlug validation covered the path, which is not complete for the `is_err()` branch.

**However:** This is NOT a new finding introduced by `9e987c3f`. It predates the fix-burst. The fix-burst's `overlay_file_path` sanitization at derivation is a distinct code path (second-pass file enumeration, lines 397-406) from the first-pass slug scan (lines 316-333). Filing as OBS given: (a) the attacker prerequisite is filesystem write access to `customers/` directory, (b) it is pre-existing and documented in adjacent code, (c) the `slug` second argument IS sanitized — only the redundant `customers_dir_name` first argument embeds the same unsanitized data.

**See SEC-PASS6-001 below.**

---

### Probe 4: Complete Sanitization Coverage — sibling-sweep verification (TD-VSDD-060)

At `9e987c3f`, `sanitize_for_log` callsites are:

| Callsite | Line | Value sanitized | Safe? |
|----------|------|-----------------|-------|
| `overlay_file_path` derivation | 406 | `format!("customers/{slug_str}/{file_name}")` | YES — covers all 9 error constructors |
| `instance_id_for_msg` E-SPEC-021 | 593, 594 | `expected_sensor_id`, `expected_org_slug` | YES (SEC-PASS4-002) |
| `expected_instance_id` E-SPEC-020 | 668, 669 | `expected_sensor_id`, `expected_org_slug` | YES (new in `9e987c3f`) |
| SSRF rejection | 653 | `overlay_base_url` | YES (SEC-REDUX-006) |
| `make_e_spec_022_unknown_org_slug` | 817 | `slug` (second arg) | YES — `customers_dir_name` first arg: NO (see Probe 3 / SEC-PASS6-001) |
| `make_e_spec_023_unrecognized_field` | 857 | `field_name` | YES |
| `make_e_spec_020_instance_id_mismatch` | 883 | `actual_instance_id` | YES |
| `make_e_spec_019_unknown_extends` | 907 | `extends_value` | YES |

**`customers_dir_name` in `make_e_spec_022_unknown_org_slug` is the sole remaining unsanitized surface in error messages.** This is the pre-existing gap in Probe 3.

TD-VSDD-060 verdict for `9e987c3f`: the commit swept the `overlay_file_path` derivation site, correctly covering all 9 downstream consumers. The `customers_dir_name` gap is pre-existing and was not the target of this fix-burst.

---

### Probe 5: Credential Safety

- `OrgSlug::new_unchecked`: confirmed not present in production code paths. PASS.
- Overlay fixture files (`acme/armis.sensor.toml`, `contoso/armis.sensor.toml`): no secrets, tokens, or credentials. PASS.
- Error messages: no credential values in any message body. PASS.

---

### Probe 6: HTTP Client Timeouts

All four adapters confirmed to have `.timeout(Duration::from_secs(30))` — unchanged from passes 4 and 5. PASS.

---

### Probe 7: File I/O Perimeter

Unchanged from passes 4 and 5:
- Symlink at org-directory level: `file_type().is_dir()` guard. PASS.
- Symlink at file level: `file_ft.is_file()` guard. PASS.
- File size limit: `MAX_OVERLAY_FILE_BYTES = 64 * 1024` enforced. PASS.
- Path traversal via `..` in directory names: blocked by OrgSlug regex for registered orgs; unregistered dirs scanned for file errors only. PASS.

---

### Probe 8: Multi-Tenant Isolation

- Unregistered slug overlays scanned but NOT merged (explicit `continue` at overlay.rs). PASS.
- `ResolvedSpecKey` uses `(OrgSlug, SensorId)` newtype tuple. PASS.
- `resolved_spec_map` read-only after boot (INV-OVL-006), shared via `Arc<HashMap>`. PASS.

---

### Probe 9: SSRF Prevention

- `overlay_base_url` validated: only `http://` and `https://` schemes accepted.
- `overlay_base_url` sanitized via `sanitize_for_log` in SSRF rejection error message. PASS.
- `overlay_file_path` in the same message: now sanitized at derivation (SEC-PASS5-002 closure). PASS.

---

### SAP-1: Tracing Emission Catalog Completeness

Verified: `9e987c3f` adds zero new `event_type` values (diff confirms no `event_type` additions or removals). Previously verified events remain unchanged:
- `overlay.loaded` (row 38) — PASS
- `boot.overlays_loaded` (row 39) — PASS
- `boot.type_spec_read_failed` (row 40) — PASS
- `boot.type_spec_parse_failed` (row 41) — PASS
- `overlay.timeout_secs_ignored` (row 42) — PASS

SAP-1: PASS.

---

### SAP-2: DTU-TOML Schema Parity

Not applicable — no DTU clone sensor TOML `[[tables]]` columns modified by `9e987c3f`. PASS.

---

## Pass-6 Findings

---

### SEC-PASS6-001: `customers_dir_name` in `make_e_spec_022_unknown_org_slug` embeds raw `slug_str` without sanitization when OrgSlug validation fails

**Pre-existing finding, not introduced by `9e987c3f`. First explicit cataloguing in pass-6 fresh-context sweep.**

- **Severity:** OBS (observation — same attacker prerequisites as SEC-PASS5-002; same severity class; pre-existing; explicitly surfaced for completeness)
- **CWE:** CWE-117 (Improper Output Neutralization for Logs)
- **OWASP:** A09:2021 Security Logging and Monitoring Failures
- **Attack Vector:** An actor with filesystem write access to the `customers/` directory creates a directory whose name contains control characters (e.g., `"evil\ndir"`). When `OverlayLoader` scans `customers/`, `entry.file_name().to_string_lossy()` returns the raw name. `OrgSlug::new(slug_str).is_err()` causes `is_registered = false`. The `make_e_spec_022_unknown_org_slug(&dir_display, &slug_str)` call at overlay.rs:332 passes `dir_display = format!("customers/evil\ndir/")` as `customers_dir_name`. Inside the constructor, `customers_dir_name` is embedded directly in the SpecError message at line 821 without sanitization. The `slug` second argument IS sanitized (line 817), but the `customers_dir_name` first argument duplicates the same data unsanitized.
- **Impact:** Log injection into SpecError message for the E-SPEC-022 path. Attacker prerequisite: filesystem write access to the `customers/` directory (same as SEC-PASS5-002). In the `!is_registered` branch where `org_slug.is_ok()` (valid regex, not in OrgRegistry), the `slug_str` is safe (passed `^[a-zA-Z0-9_-]{1,64}$`). Only the `org_slug.is_err()` sub-branch is vulnerable.
- **Evidence:**
  - overlay.rs:296: `let dir_name = entry.file_name().to_string_lossy().to_string();` — raw OS name
  - overlay.rs:331: `let dir_display = format!("customers/{slug_str}/");` — raw `slug_str` in format string
  - overlay.rs:332: `errors.push(make_e_spec_022_unknown_org_slug(&dir_display, &slug_str));` — `dir_display` passed as first arg
  - overlay.rs:821: `"Per-org overlay directory '{customers_dir_name}' references org slug '{safe_slug}' ..."` — `customers_dir_name` embedded without sanitization; `safe_slug` is the sanitized version of the same slug (redundant path)
  - Mitigating factor: this injection site was partially assessed in pass-1-redux (accepted under an incomplete assumption that OrgSlug validation protects `dir_display` in all paths). The `is_err()` branch was not covered by that assessment.
- **Proposed Mitigation:** Apply `sanitize_for_log` to `customers_dir_name` inside `make_e_spec_022_unknown_org_slug`, or construct `dir_display` at the callsite with a sanitized slug. Since `slug` is already sanitized to `safe_slug`, the `customers_dir_name` format could be replaced with `format!("customers/{safe_slug}/")` — but this changes the error message for the `is_err()` case to show the sanitized slug rather than the raw dir name. The simplest fix: `let dir_display = format!("customers/{}/", sanitize_for_log(&slug_str));` at overlay.rs:331.
- **Disposition:** OBS — not introduced by this PR, pre-existing, same attack prerequisites as prior LOW findings. Recommend fix in a follow-up maintenance burst alongside the `customers_dir_name` doc-comment update. Does NOT block PR merge.

---

## Pass-6 Cumulative Finding Disposition Table

| Finding ID | Severity | First Filed | Status at Pass 6 |
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
| SEC-PASS6-001 (customers_dir_name in E-SPEC-022 message unsanitized — pre-existing) | OBS | Pass 6 | OPEN — pre-existing; not introduced by this PR; recommend follow-up maintenance burst |

---

## Severity Counts (Pass 6)

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 0 |
| MEDIUM | 0 |
| LOW | 1 (SEC-PASS5-001 — process gap, state-manager scope) |
| OBS | 2 (SEC-PASS3-003 pre-existing AQL, SEC-PASS6-001 pre-existing E-SPEC-022 customers_dir_name) |

---

## Positive Findings (Defensive Measures Present)

- `sanitize_for_log` is a correct, simple, and well-tested function (6 unit tests) applied at the right abstraction level (derivation, not each embedding site).
- The multi-pass injection-surface analysis across passes 1-6 has systematically closed all TOML-sourced and filesystem-sourced injection sites in error message bodies.
- OrgSlug newtype with `^[a-zA-Z0-9_-]{1,64}$` regex validation blocks the vast majority of injection attempts at primary perimeter.
- `MAX_OVERLAY_FILE_BYTES = 64 KiB` file size limit prevents boot-time DoS via large TOML.
- SSRF protection via scheme whitelist (`http://`, `https://`) is structurally correct.
- No credential values appear in error messages at any code path.
- All four sensor adapters have `.timeout(Duration::from_secs(30))` per CLAUDE.md HTTP client timeout discipline.

---

## Verdicts

**CLEAN(strict): YES**

Zero findings of ANY severity introduced by `9e987c3f`:
- SEC-PASS5-002 is CLOSED by the derivation-point fix.
- SEC-PASS5-001 was a process gap (state-manager scope) pre-existing before and after this commit.
- SEC-PASS6-001 is a pre-existing OBS-severity finding not introduced by this PR or this fix-burst.

All findings at HEAD `9e987c3f` that were introduced by this PR have been closed. The remaining open items (SEC-PASS5-001, SEC-PASS3-003, SEC-PASS6-001) are either: (a) state-manager scope process gaps, or (b) pre-existing OBS findings from code outside this PR's story scope.

**CLEAN(PR-merge): YES**

Zero CRITICAL/HIGH/MEDIUM findings at PR HEAD `9e987c3f`.

The PR is cleared for merge from a security standpoint. The two OBS findings (SEC-PASS3-003, SEC-PASS6-001) and the one LOW process finding (SEC-PASS5-001) are recommended for follow-up maintenance but do not block merge.

---

## Recommended Actions

| Priority | Action | Finding |
|----------|--------|---------|
| P3 (state-manager, pre-merge or immediately post-merge) | Register `S-CONFIG-MULTI-TENANT-OVERRIDE-002` in STORY-INDEX.md with explicit scope (wire `timeout_secs` to HTTP client) and concrete dependency/anchor | SEC-PASS5-001 |
| P4 (maintenance burst) | Apply `sanitize_for_log` to `customers_dir_name` in `make_e_spec_022_unknown_org_slug` OR construct `dir_display` at callsite with sanitized slug — closes the `is_err()` injection path in E-SPEC-022 | SEC-PASS6-001 |
| P4 (backlog) | Add `sanitize_for_log` to `aql_preview` in `build_aql` (AQL audit events — pre-existing, out of PR scope) | SEC-PASS3-003 |
