---
document_type: pr-level-security-review
cycle: wave-0-plugin-prereqs
story: S-CONFIG-MULTI-TENANT-OVERRIDE-001
pr: 155
pass: 1-redux
reviewer: security-reviewer
fresh_context: true
post_clear_recovery: true
feature_head: 515fdc2e
develop_baseline: f19575ff
timestamp: 2026-05-24T18:00:00Z
input-hash: d24980752c2a2f04e60b77e3d2e86bfc2c7650d910900e146bfef1ba5d315bac
total_findings: 7
critical: 1
high: 2
medium: 2
low: 1
obs: 1
files_reviewed: 12
---

# PR #155 Security Review — S-CONFIG-MULTI-TENANT-OVERRIDE-001 (Pass 1-Redux)

**Story:** Per-Org Sensor Endpoint Overlay Loading (ADR-029 Hybrid Sensor Instance)
**Feature HEAD:** `515fdc2e`
**Develop baseline:** `f19575ff`
**Review type:** PR-LEVEL fresh-context security review
**Context:** Post-`/clear` recovery — re-discovering all pass-1 findings with full body detail. Feature HEAD unchanged.

## Source Files Reviewed

| File | Role |
|------|------|
| `crates/prism-spec-engine/src/overlay.rs` | Core overlay types + OverlayLoader (new) |
| `crates/prism-bin/src/boot.rs` | step4 extension + BootContext/RunningServer (modified) |
| `crates/prism-sensors/src/fanout.rs` | resolve_spec_for_fanout + fan_out_with_overlay_map (new) |
| `crates/prism-query/src/materialization.rs` | fan_out_with_overlay_map dispatch wiring |
| `crates/prism-query/src/engine.rs` | resolved_spec_map threading |
| `crates/prism-sensors/src/auth/armis.rs` | ArmisAdapter::fetch + reqwest::Client construction |
| `crates/prism-sensors/src/auth/claroty.rs` | ClarotyAdapter + reqwest::Client construction |
| `crates/prism-core/src/error.rs` | SpecErrorCode enum additions |
| `crates/prism-core/src/tenant.rs` | OrgSlug::new validation |
| `crates/prism-spec-engine/tests/overlay_loading_tests.rs` | Red Gate tests |
| `crates/prism-sensors/specs/customers/acme/armis.sensor.toml` | Overlay fixture |
| `crates/prism-sensors/specs/customers/contoso/armis.sensor.toml` | Overlay fixture |

---

## Findings

---

## SEC-REDUX-001 — base_url overlay override is a NO-OP at adapter execution layer

**Severity:** CRIT
**CWE / CVE / OWASP:** CWE-706 (Use of Incorrectly-Resolved Name or Reference), OWASP A05:2021 Security Misconfiguration
**File:lines:** `crates/prism-sensors/src/fanout.rs:622-636`, `crates/prism-sensors/src/auth/armis.rs:377-390`, `crates/prism-sensors/src/auth/claroty.rs:183-195`
**Finding:**

This is the most severe finding in the PR. The multi-tenant endpoint routing that this entire story implements is functionally inert at the adapter layer.

**Evidence — injection side (overlay.rs + fanout.rs):**

`resolve_spec_for_fanout` in `fanout.rs` correctly injects the overlay `base_url` into `sensor_config["base_url"]`:

```rust
// fanout.rs:622-636
if let serde_json::Value::Object(ref mut map) = resolved_adapter_spec.sensor_config {
    map.insert(
        "base_url".to_string(),
        serde_json::Value::String(resolved.spec.base_url.clone()),
    );
}
```

This is correct wiring. The overlay `base_url` lands in `SensorSpec.sensor_config["base_url"]`.

**Evidence — consumption side (ArmisAdapter — does NOT read sensor_config["base_url"]):**

```rust
// armis.rs:377-390
pub fn new(org_id: prism_core::OrgId, auth: &ArmisAuth, bearer_token: SecretString) -> Self {
    let http = Client::builder()
        .cookie_store(false)
        .build()
        .unwrap_or_default();
    Self {
        org_id,
        instance_url: auth.instance_url.clone(),  // SOURCE: ArmisAuth, NOT sensor_config
        http,
        bearer_token,
    }
}
```

At fetch time (armis.rs:517):
```rust
let url = format!("{}/api/v1/search", self.instance_url);  // uses hardcoded instance_url
```

`ArmisAdapter` uses `self.instance_url` which is set from `ArmisAuth.instance_url` at construction time. The adapter NEVER reads `spec.sensor_config["base_url"]` in its `fetch()` method. The injected value is silently ignored.

**Evidence — ClarotyAdapter has the same defect:**

```rust
// claroty.rs:183-195
let http = Client::builder()
    .cookie_store(false)
    .build()
    .unwrap_or_default();
Self {
    org_id,
    instance_url: auth.instance_url.clone(),  // SOURCE: ClarotyAuth, NOT sensor_config
    http, bearer_token,
}
// claroty.rs:204
let url = format!("{}{}", self.instance_url, endpoint);  // hardcoded instance_url used
```

**Attack vector:** An MSSP operator installs overlay files for `customers/acme/armis.sensor.toml` and `customers/contoso/armis.sensor.toml` with distinct `base_url` values, expecting query routing to go to per-org Armis instances. Instead, ALL orgs route to the single `instance_url` baked into `ArmisAuth` at boot — which comes from credentials/config, not from the overlay file. There is no multi-tenant endpoint isolation despite the boot-time validation succeeding and the overlay map being correctly built.

**Impact:** The primary security and correctness invariant of ADR-029 — per-org endpoint isolation — is violated at runtime. Org Acme's queries may reach Org Contoso's Armis instance if `ArmisAuth.instance_url` was configured for a different tenant. This is a data boundary violation (BC-2.06.014 not implemented end-to-end).

**Paper-fix-consumer-doesnt-read class?:** YES — `resolve_spec_for_fanout` injects the overlay base_url into `sensor_config["base_url"]` (the producer writes), but `ArmisAdapter::fetch` never reads `sensor_config["base_url"]` (the consumer does not read). Classic paper-fix per lesson 50 / SAP-3.

**Suggested fix:** The fix requires one of two approaches:

1. **Preferred:** At `ArmisAdapter::fetch` time, read `spec.sensor_config["base_url"]` and use it as the request URL if present, falling back to `self.instance_url`. This wires the overlay at the consumption point.

2. **Alternative (architecturally cleaner):** Change `ArmisAdapter::new` to accept an `Option<String>` `override_base_url` parameter. At fanout time (in `fan_out_with_overlay_map`), construct a new adapter instance for each overlay-bearing org using the overlay URL. This requires `AdapterRegistry` to support per-org construction.

The simplest load-bearing fix per AC-003 spec: at `ArmisAdapter::fetch`, extract `spec.sensor_config["base_url"]` as `Option<String>`. If present and not empty, use it as the host for the request URL. The existing `instance_url` becomes the static fallback.

A Red Gate test must verify the adapter received the overlay URL (the `CapturingAdapter` pattern in `fanout.rs::test_F_LP2_CRIT_001_fan_out_with_overlay_map_routes_to_overlay_url` is a model — the same pattern must be reproduced for the real ArmisAdapter path).

---

## SEC-REDUX-002 — Symlink-following in overlay file discovery (file level)

**Severity:** HIGH
**CWE / CVE / OWASP:** CWE-59 (Improper Link Resolution Before File Access / Link Following), CWE-22 (Path Traversal), OWASP A01:2021 Broken Access Control
**File:lines:** `crates/prism-spec-engine/src/overlay.rs:333-360`
**Finding:**

The overlay loader's inner file walk does not check whether a discovered `.sensor.toml` entry is a symlink before reading it. `std::fs::read_to_string(file_entry.path())` follows symlinks on all POSIX platforms.

**Evidence:**

```rust
// overlay.rs:333-360 — inner file loop
for file_entry_result in file_entries {
    let file_entry = match file_entry_result { Ok(e) => e, ... };

    let file_name = file_entry.file_name().to_string_lossy().to_string();

    // Only check extension. NO file_type() call. NO symlink check.
    if !file_name.ends_with(".sensor.toml") {
        continue;
    }

    // read_to_string follows symlinks silently.
    let toml_content = match std::fs::read_to_string(file_entry.path()) {
        Ok(c) => c,
        ...
    };
```

**Partial protection at org-directory level (good):**

The outer loop does check `file_type().is_dir()` (line 283). On POSIX, `DirEntry::file_type()` uses `lstat()` — it does NOT follow symlinks. A symlink to a directory at the org-directory level reports as `Symlink` (not `Dir`), so `is_dir()` returns `false` and the symlink is skipped. This provides correct protection at the org-slug directory level.

**Gap at file level (the finding):**

No equivalent check exists for individual `.sensor.toml` files within an org directory. An attacker (or misconfigured operator) could create:
```
customers/acme/evil.sensor.toml -> /etc/passwd
customers/acme/evil.sensor.toml -> /proc/1/environ
customers/acme/evil.sensor.toml -> /var/run/prism/keyring.db
```
The TOML parser would fail on non-TOML content and emit an `E-SPEC-001` error, which prevents the overlay from loading. However:
1. The file content is passed to `toml::from_str()` after being read entirely into memory — a large file (e.g., a 100MB sparse file) could be read completely before the parse failure occurs, causing a denial-of-service.
2. TOML parse errors include the file path in the error message (overlay_file_path), which leaks the resolved path. If the symlink target contains newline characters in its content, the TOML error message may embed them in structured logs (log injection vector — see SEC-REDUX-004).
3. In a future refactor where `.sensor.toml` content is interpreted differently (e.g., if a YAML fallback or a binary format is added), the lack of symlink protection becomes a silent data exfiltration vector.

**TOCTOU note:** Even with a `symlink_metadata()` check before `read_to_string()`, there is a TOCTOU window. The correct fix uses `O_NOFOLLOW` (via `std::fs::OpenOptions` with a platform-specific flag) or reads via a file descriptor opened with `O_NOFOLLOW`. On stable Rust, the practical approach is `std::fs::read_link()` or checking `file_entry.file_type().is_symlink()` (which uses `lstat()` on the entry and does not require a separate `stat()` call).

**Suggested fix:**

```rust
// Before the file_name.ends_with() check, add:
let file_type = match file_entry.file_type() {
    Ok(ft) => ft,
    Err(io_err) => {
        errors.push(PrismError::Io(io_err.to_string()));
        continue;
    }
};

if !file_type.is_file() {
    // Skip symlinks and special files (directories within an org dir are unexpected but safe to skip)
    continue;
}
```

`DirEntry::file_type()` uses `lstat()` on POSIX and `FindData` on Windows, so `is_file()` returns `false` for symlinks — this is the correct guard without introducing TOCTOU.

**Paper-fix-consumer-doesnt-read class?:** NO — this is a file-system access control issue.

---

## SEC-REDUX-003 — timeout_secs overlay field is accepted but never applied to HTTP client

**Severity:** HIGH
**CWE / CVE / OWASP:** CWE-400 (Uncontrolled Resource Consumption), CWE-770 (Allocation of Resources Without Limits), OWASP A05:2021 Security Misconfiguration
**File:lines:** `crates/prism-spec-engine/src/overlay.rs:86,624-626`, `crates/prism-sensors/src/auth/armis.rs:377-390`, `crates/prism-sensors/src/auth/claroty.rs:183-195`
**Finding:**

The `SensorInstanceOverlay` struct exposes `timeout_secs: Option<u64>` as a documented, user-facing field, and the overlay validation allowlist includes it as a legitimate field name. The `merge_overlay_onto_type_spec` function tracks it in provenance:

```rust
// overlay.rs:624-626
if overlay.timeout_secs.is_some() {
    provenance.timeout_secs_from_overlay = true;
}
```

But `timeout_secs` is NEVER applied to any HTTP client. There is no code path that reads `provenance.timeout_secs_from_overlay` and adjusts the reqwest client timeout. The field is accepted, tracked in provenance, and then silently discarded.

**Compound issue:** Both `ArmisAdapter` and `ClarotyAdapter` build their reqwest clients without ANY timeout:

```rust
// armis.rs:377-380
let http = Client::builder()
    .cookie_store(false)
    .build()
    .unwrap_or_default();

// claroty.rs:183-185
let http = Client::builder()
    .cookie_store(false)
    .build()
    .unwrap_or_default();
```

These have no `.timeout()` call. This is a pre-existing gap (TD-S-PLUGIN-PREREQ-B-005), but this PR introduces `timeout_secs` as an overlay-level tunable that operators will expect to work. Advertising a feature that silently does nothing is more dangerous than not advertising the feature at all, because operators will believe they have configured a safety boundary when none exists.

**Attack vector:** An operator configures `timeout_secs = 5` in an overlay file expecting a 5-second HTTP timeout for a slow Armis on-premise instance. The sensor hangs for minutes on a network partition, holding a semaphore permit and blocking other queries. The operator has a false sense of security that the timeout is active.

**Impact:** Hang-based denial of service of the fan-out semaphore pool. Under the ADR-022 §D concurrency limit (8 permits for sensor fetch), 8 hung connections can exhaust the pool entirely.

**Paper-fix-consumer-doesnt-read class?:** YES — `timeout_secs` is accepted as a legitimate overlay field (the writer writes the provenance flag), but the consumer (the HTTP client builder) never reads it. This is the SAP-3 / lesson 50 paper-fix class.

**Suggested fix:**

Two independent fixes required:

1. **Apply timeout_secs in ResolvedSensorSpec:** Pass `overlay.timeout_secs` through to `ResolvedSensorSpec` as a concrete `Option<Duration>` field (not just provenance). At fanout time, read this field and pass it to the adapter.

2. **Fix base production timeouts:** `ArmisAdapter::new` and `ClarotyAdapter::new` must call `.timeout(Duration::from_secs(30))` (or read from the resolved spec's `timeout_secs` field) per CLAUDE.md convention. The pre-existing gap (TD-S-PLUGIN-PREREQ-B-005) is now reachable through this PR's timeout_secs semantics and must be resolved here.

---

## SEC-REDUX-004 — Log injection via user-controlled instance_id and field_name in error messages

**Severity:** MED
**CWE / CVE / OWASP:** CWE-117 (Improper Output Neutralization for Logs), OWASP A09:2021 Security Logging and Monitoring Failures
**File:lines:** `crates/prism-spec-engine/src/overlay.rs:547-555,704-720`
**Finding:**

Two error messages embed user-controlled values from overlay TOML file content without sanitization. If these error messages are forwarded to a log aggregator (Elasticsearch, Splunk, Datadog), an attacker controlling overlay file content can inject structured content into log records.

**Evidence — E-SPEC-020 (instance_id mismatch):**

```rust
// overlay.rs:712-716
message: format!(
    "Per-org overlay '{file_path}' declares instance_id='{actual_instance_id}' but \
     expected '{expected_instance_id}' (derived from filename and parent directory). \
     Rename or correct the instance_id field."
),
```

`actual_instance_id` comes directly from `overlay.instance_id` which is the raw TOML-deserialized `instance_id` field value. An attacker can set:
```toml
instance_id = "armis@acme\nENVIRONMENT=prod\nSECRET_KEY=leaked"
```
This embeds newlines in the error message. The `SpecError` is eventually formatted via `Display` and logged as a `BootError::ConfigInvalid` string.

**Evidence — E-SPEC-023 (unrecognized field):**

```rust
// overlay.rs:686-694
message: format!(
    "Per-org overlay '{file_path}' contains unrecognized field '{field_name}'. \
     Allowed overlay fields are: extends, instance_id, base_url, timeout_secs, \
     rate_limit_hints (with sub-fields: requests_per_second, burst_size)."
),
```

`field_name` is a key from the raw TOML table. While TOML keys are constrained in practice, TOML specification allows Unicode bare keys (including control characters in quoted keys). A crafted overlay TOML with a key containing newlines or JSON-like structures could inject into structured logs.

**Mitigating factors:**

- `OrgSlug::new()` validation (`^[a-zA-Z0-9_-]{1,64}$`) prevents injection via `org_slug` / `dir_display` (the validator explicitly notes log injection risk at tenant.rs:68-71).
- The `org_slug` field in the `overlay.loaded` tracing event is correctly validated before use.
- Boot-time errors rather than runtime errors: log injection here affects boot logs, not per-query logs.

**Impact:** Log injection can corrupt SIEM audit trails, enable log forging (injecting false records that appear legitimate), or trigger false positive alerts in security monitoring systems. In MSSP context where prism audit logs are customer-facing evidence, injected content could constitute evidence tampering.

**Paper-fix-consumer-doesnt-read class?:** NO.

**Suggested fix:**

Apply sanitization to user-controlled values before embedding in error messages:

```rust
fn sanitize_for_log(value: &str) -> String {
    // Replace control characters (including \n, \r, \t, null) with replacement char
    value.chars()
        .map(|c| if c.is_control() { '\u{FFFD}' } else { c })
        .take(256)  // cap length to prevent DoS via giant log entries
        .collect()
}
```

Apply to `actual_instance_id` in `make_e_spec_020_instance_id_mismatch` and `field_name` in `make_e_spec_023_unrecognized_field`. The length cap also addresses the file size concern (if an overlay contains a 10MB `instance_id` value, the error message won't be 10MB).

---

## SEC-REDUX-005 — No file size limit before overlay file is read into memory

**Severity:** MED
**CWE / CVE / OWASP:** CWE-400 (Uncontrolled Resource Consumption), CWE-770 (Allocation of Resources Without Limits), OWASP A05:2021 Security Misconfiguration
**File:lines:** `crates/prism-spec-engine/src/overlay.rs:357-362`
**Finding:**

`std::fs::read_to_string(file_entry.path())` reads the entire file into a `String` before any size check. There is no maximum file size enforced before or after the read. A maliciously large overlay file (or a symlink to a large file — see SEC-REDUX-002) can cause the process to allocate arbitrarily large amounts of memory at boot time.

**Evidence:**

```rust
// overlay.rs:357-362
let toml_content = match std::fs::read_to_string(file_entry.path()) {
    Ok(c) => c,       // no size check; entire file in c
    Err(io_err) => {
        errors.push(PrismError::Io(io_err.to_string()));
        continue;
    }
};
```

A legitimate Armis overlay file is approximately 100 bytes. An attacker who can write to `customers/acme/armis.sensor.toml` can replace it with a 1GB file. At boot, `read_to_string` will allocate 1GB of RAM before the TOML parser runs.

**Compound risk with symlinks:** If SEC-REDUX-002 is not fixed and a symlink can target `/dev/zero` (on Linux) or a large sparse file, the read will block or exhaust memory.

**Impact:** Boot-time denial of service. On constrained deployments (512MB process limit per prism memory budget), OOM kill during boot is possible.

**Paper-fix-consumer-doesnt-read class?:** NO.

**Suggested fix:**

Add a pre-read size check using `metadata()`:

```rust
// Before read_to_string
const MAX_OVERLAY_FILE_BYTES: u64 = 64 * 1024; // 64 KiB — generous for a scalar-only overlay
match file_entry.metadata() {
    Ok(meta) if meta.len() > MAX_OVERLAY_FILE_BYTES => {
        errors.push(PrismError::Spec(SpecError {
            code: SpecErrorCode::ESpec001,
            message: format!(
                "Per-org overlay '{overlay_file_path}' exceeds maximum allowed size \
                 ({} bytes > {} bytes limit). Overlay files must be scalar-only tunables.",
                meta.len(), MAX_OVERLAY_FILE_BYTES
            ),
            ..Default::default()
        }));
        continue;
    }
    Err(io_err) => {
        errors.push(PrismError::Io(io_err.to_string()));
        continue;
    }
    Ok(_) => {} // size OK, proceed
}
```

Note: using `metadata()` (not `symlink_metadata()`) here intentionally follows symlinks for the size check — this is acceptable because we want to know the size of the eventual file content. The symlink protection (SEC-REDUX-002) should be a separate `file_type()` check that runs before this.

---

## SEC-REDUX-006 — No URL scheme validation on overlay base_url (permits non-HTTP schemes)

**Severity:** LOW
**CWE / CVE / OWASP:** CWE-601 (URL Redirection to Untrusted Site), CWE-918 (Server-Side Request Forgery), OWASP A10:2021 SSRF
**File:lines:** `crates/prism-spec-engine/src/overlay.rs:596-603`, `crates/prism-spec-engine/src/validation.rs:132-144`
**Finding:**

The TYPE spec `base_url` is validated to start with `http://` or `https://` (validation.rs:133-140). The overlay `base_url` field is NOT run through the same validation. After `merge_overlay_onto_type_spec` replaces `merged_spec.base_url` with the overlay value:

```rust
// overlay.rs:600-602
if let Some(ref overlay_base_url) = overlay.base_url {
    merged_spec.base_url = overlay_base_url.clone();
    provenance.base_url_from_overlay = true;
}
```

The merged spec's `base_url` is not re-validated. An overlay file could set:
- `base_url = "file:///etc/shadow"` — local file read
- `base_url = "http://169.254.169.254/latest/meta-data/"` — cloud metadata SSRF
- `base_url = "ftp://internal-server/"` — unexpected protocol

The impact is presently low because SEC-REDUX-001 confirms that the overlay `base_url` is NOT actually consumed by adapters at this PR's HEAD. However, once SEC-REDUX-001 is fixed, this gap becomes a HIGH-severity SSRF vector.

**Paper-fix-consumer-doesnt-read class?:** NO (related to SEC-REDUX-001 but independent gap).

**Suggested fix:**

In `validate_overlay_toml`, after deserializing the overlay struct, validate `overlay.base_url` (when `Some`) with the same scheme check used for TYPE specs:

```rust
if let Some(ref url) = overlay.base_url {
    if !url.starts_with("https://") && !url.starts_with("http://") {
        validation_errors.push(PrismError::Spec(SpecError {
            code: SpecErrorCode::ESpec001,
            message: format!(
                "Per-org overlay '{overlay_file_path}' base_url '{}' is not a valid URL \
                 (must start with http:// or https://)",
                sanitize_for_log(url)
            ),
            toml_path: Some("base_url".to_string()),
            file_path: Some(overlay_file_path.to_string()),
            line_number: None,
        }));
    }
}
```

---

## SEC-REDUX-007 — OBS: OrgSlug::new_unchecked documentation gap — test-only intent not feature-gated

**Severity:** OBS
**CWE / CVE / OWASP:** CWE-284 (Improper Access Control — test-only interface accessible from production)
**File:lines:** `crates/prism-core/src/tenant.rs:77-99`
**Finding:**

`OrgSlug::new_unchecked` is documented as "test fixtures only" but is NOT protected by `#[cfg(test)]` or a feature gate (the doc comment explains this is intentional because cross-crate test callers need it). The audit allowlist check at `crates/prism-core/tests/new_unchecked_audit.rs` provides a compile-time gate. This is a pre-existing condition documented at tenant.rs:86-91.

The specific concern from this PR: `write_dispatch.rs:455` uses `OrgSlug::new_unchecked("acme")` inside a `#[cfg(test)]` module (verified: the enclosing `mod` at line 405 is `#[cfg(test)]`), so this call is test-only and does not appear in production binaries. No production regression in this PR.

**Rating as OBS (not finding):** The pre-existing gap is known and monitored by the allowlist. This PR introduces no new production callers. The observation is that the allowlist mechanism is a documentation convention rather than a compiler-enforced guarantee, which is an accepted risk per the tenant.rs comment.

**Paper-fix-consumer-doesnt-read class?:** NO.

**Suggested fix:** No action required in this PR. Future hardening: consider `#[cfg(any(test, feature = "test-helpers"))]` for `new_unchecked` with a corresponding `test-helpers` feature gate that CI enforces is never enabled in release builds.

---

## Probe Results

### Probe 1: Credential Safety

- `OrgSlug::new_unchecked`: Only test-module callers in this PR (write_dispatch.rs is inside `#[cfg(test)]`). PASS.
- `ArmisAuth.instance_url`, `ClarotyAuth.instance_url`: These are plain `String` fields not protected by `SecretString`. However, they are endpoint URLs (not credentials), so this is expected. PASS.
- Error messages: SEC-REDUX-004 covers log injection via user-controlled `instance_id` and `field_name` values. INV-ERR-002 (no credential values in error messages) is satisfied — no credential values appear in any error message body. PASS on credential safety; PARTIAL on injection safety.
- `OrgSlug::new()` at line 68-71 explicitly avoids echoing raw input to prevent log injection. PASS.

### Probe 2: HTTP Client Timeout (CLAUDE.md Convention)

- `ArmisAdapter::new` (armis.rs:377): `Client::builder().cookie_store(false).build()` — NO `.timeout()`. **FAIL** (TD-S-PLUGIN-PREREQ-B-005 pre-existing + SEC-REDUX-003 timeout_secs paper-fix compound).
- `ClarotyAdapter` construction (claroty.rs:183): `Client::builder().cookie_store(false).build()` — NO `.timeout()`. **FAIL** (same as above).
- `ClarotyAdapter` auth client (claroty.rs:288): `Client::builder().default_headers(...).build()` — NO `.timeout()`. **FAIL** (third site).
- Boot plugin HTTP client (boot.rs:1186, 1209): `.timeout(Duration::from_secs(PLUGIN_HTTP_CLIENT_TIMEOUT_SECS))`. **PASS**.
- The new `fan_out_with_overlay_map` function does not create new HTTP clients. **PASS**.

### Probe 3: File I/O Perimeter (overlay loading)

- Symlink at org-directory level: Protected. `DirEntry::file_type()` at line 275-283 uses `lstat()` — symlinks-to-dirs report as `Symlink`, not `Dir`, so `is_dir()` returns `false`. PASS.
- Symlink at file level: **FAIL** (SEC-REDUX-002). No `file_type()` call on file entries before `read_to_string`.
- Path traversal via `..` in directory names: Protected. `OrgSlug::new()` validates against `^[a-zA-Z0-9_-]{1,64}$`, rejecting directory names containing `.` (verified: the pattern requires only alnum + `_` + `-`). PASS.
- File size limit: **FAIL** (SEC-REDUX-005). No pre-read size check.
- TOCTOU: Low-risk at org-directory level (the `file_type()` call and subsequent `read_dir` are separate, but exploiting this TOCTOU requires replacing a directory with a symlink between two kernel calls — impractical in most deployment models). Noted as accepted risk.
- Cross-tenant escape: Protected by OrgSlug validation + OrgRegistry cross-check. PASS.

### Probe 4: Log Injection

**FAIL** (SEC-REDUX-004). Two error message builders embed unsanitized TOML-sourced values:
- `make_e_spec_020_instance_id_mismatch`: embeds `actual_instance_id` from TOML content.
- `make_e_spec_023_unrecognized_field`: embeds `field_name` from raw TOML table keys.

The `overlay.loaded` tracing event is SAFE: `org_slug` is validated, `sensor_id` is the filename stem (constrained by the `.sensor.toml` extension strip), and `instance_id` is from the validated overlay struct (but still TOML-sourced — see SEC-REDUX-004).

### Probe 5: Multi-Tenant Isolation

- Overlay map construction: PASS. Unregistered org slugs trigger E-SPEC-022 and their overlays are NOT inserted into the resolved map (overlay.rs:377-382 `if !is_registered { continue; }`).
- Caching: No overlay-level caching introduced. The resolved map is rebuilt on each boot/config reload. PASS.
- Revocation: On config reload, `step4_load_sensor_specs_with_overlays` is not yet wired to hot-reload (EC-010 is a deferred edge case per story). The boot path correctly rebuilds the map. Single-tenant deployments unaffected. PARTIAL — revocation works at boot; hot-reload revocation is deferred per story design.
- Cross-tenant read: The resolved map key is `(OrgSlug, sensor_id_string)` — strict tuple equality. No partial-match or prefix-match risk. PASS.

### Probe 6: Paper-Fix Consumer-Doesn't-Read Class (SAP-3 lesson 50)

Three findings in this review are of this class:
- **SEC-REDUX-001** (CRIT): `resolve_spec_for_fanout` writes to `sensor_config["base_url"]`; `ArmisAdapter::fetch` never reads it.
- **SEC-REDUX-003** (HIGH): `validate_overlay_toml` accepts `timeout_secs`; no HTTP client ever reads it.
- **SEC-REDUX-006** (LOW, potential HIGH post-fix): `merge_overlay_onto_type_spec` stores unvalidated `base_url`; validation.rs is never called on it.

### SAP-1: Tracing Emission Catalog Completeness

New emission in this PR: `event_type = "overlay.loaded"` in `overlay.rs:418`. Verified in BC-2.16.002 §Postconditions at version 1.44 (changelog entry dated 2026-05-23). PASS.

All other `event_type` values in the diff are pre-existing. PASS.

### SAP-2: DTU↔TOML Schema Parity

Not applicable to this PR — no DTU clone sensor TOML `[[tables]]` columns were modified. The overlay files (`customers/acme/armis.sensor.toml`, `customers/contoso/armis.sensor.toml`) contain no `[[tables]]` blocks (they are scalar-only overlays per BC-2.06.013 enforcement). PASS.

---

## Summary

| Severity | Count |
|----------|-------|
| CRIT | 1 |
| HIGH | 2 |
| MED | 2 |
| LOW | 1 |
| OBS | 1 |

**CLEAN(strict): NO** — 6 findings of severity CRIT through LOW.

**CLEAN(PR-merge): NO** — 1 CRIT + 2 HIGH + 2 MED findings present. PR-merge gate requires zero MED+ findings.

### Blocking Summary (cannot merge in current state)

| ID | Severity | Description |
|----|----------|-------------|
| SEC-REDUX-001 | CRIT | base_url overlay override is a NO-OP at ArmisAdapter + ClarotyAdapter — multi-tenant routing inert |
| SEC-REDUX-002 | HIGH | Symlink-following in overlay file-level walk — potential path escape and file disclosure |
| SEC-REDUX-003 | HIGH | timeout_secs overlay field accepted and documented but never applied to HTTP client — paper-fix class |
| SEC-REDUX-004 | MED | Log injection via TOML-sourced instance_id and field_name in E-SPEC-020 and E-SPEC-023 messages |
| SEC-REDUX-005 | MED | No overlay file size limit before read_to_string — boot-time DoS vector |

### Non-Blocking (fix before next major story)

| ID | Severity | Description |
|----|----------|-------------|
| SEC-REDUX-006 | LOW | No URL scheme validation on overlay base_url — becomes HIGH-severity SSRF once SEC-REDUX-001 is fixed |
| SEC-REDUX-007 | OBS | OrgSlug::new_unchecked not feature-gated — pre-existing, monitored by allowlist |
