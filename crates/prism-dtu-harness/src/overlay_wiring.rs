//! Overlay TOML wiring helper for multi-tenant DTU test harnesses.
//!
//! Writes per-org sensor overlay TOML files to a caller-supplied directory,
//! suitable for consumption by `SpecLoader::load_all` from `prism-spec-engine`.
//!
//! # Story anchor
//!
//! S-DEMO-MULTI-TENANT-DTU-001 (BC-2.06.017 Postcondition 3)
//!
//! # Perimeter constraint (BC-2.06.017 INV-PERIMETER-001)
//!
//! This module writes raw TOML strings only. It does NOT import
//! `prism-spec-engine`, `prism-sensors`, or `prism-query` types.
//! The `tempfile` crate is a `[dev-dependency]` only — the caller (test code)
//! owns the `TempDir` and passes `dir.path()` to this function (U-005).
//!
//! # Output layout
//!
//! For each `(org_slug, sensor_id)` entry in the harness socket map, writes:
//!
//! ```text
//! {dir}/customers/{org_slug}/{sensor_id}.sensor.toml
//! ```
//!
//! with content:
//!
//! ```toml
//! extends = "{sensor_id}"
//! instance_id = "{sensor_id}@{org_slug}"
//! base_url = "http://{socket_addr}"
//! ```
//!
//! This format is consumed by `SpecLoader::load_all` overlay walk semantics
//! (S-CONFIG-MULTI-TENANT-OVERRIDE-001 / BC-2.06.012).

use std::collections::HashMap;
use std::net::SocketAddr;

use crate::multi_instance::MultiInstanceHarness;

/// Validate that a harness key component (org_slug or sensor_id) is safe to use
/// in both filesystem paths and raw TOML string values.
///
/// # Security rationale (SEC-001 / SEC-002)
///
/// These values are interpolated into:
/// - Filesystem paths: `{dir}/customers/{org_slug}/{sensor_id}.sensor.toml`  (CWE-22 path traversal)
/// - Raw TOML content: `extends = "{sensor_id}"`, `instance_id = "{sensor_id}@{org_slug}"` (CWE-93/74 TOML injection)
///
/// Allowed charset: ASCII alphanumeric, `-`, `_`. Length: 1..=64.
/// This excludes `..`, `/`, `\`, `"`, `\n`, `\r`, `[`, `]`, `@`, whitespace —
/// closing both the path-traversal and TOML-injection attack surfaces.
fn validate_harness_key(field: &str, value: &str) -> std::io::Result<()> {
    if value.is_empty() || value.len() > 64 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "overlay_wiring: {field} must be 1–64 characters, got length {}",
                value.len()
            ),
        ));
    }
    for ch in value.chars() {
        if !matches!(ch, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_') {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "overlay_wiring: {field} contains disallowed character {ch:?} \
                     (input redacted; length {}); \
                     allowed charset is [A-Za-z0-9_-], length 1..=64 \
                     (SEC-001 TOML-injection / SEC-002 path-traversal defense)",
                    value.len()
                ),
            ));
        }
    }
    Ok(())
}

/// Write per-org sensor overlay TOML files from a `MultiInstanceHarness` socket map.
///
/// For each `(org_slug, sensor_id)` → `SocketAddr` entry in
/// `harness.socket_map()`, creates the directory
/// `{dir}/customers/{org_slug}/` and writes the file
/// `{sensor_id}.sensor.toml` containing:
///
/// ```toml
/// extends = "{sensor_id}"
/// instance_id = "{sensor_id}@{org_slug}"
/// base_url = "http://{socket_addr}"
/// ```
///
/// The caller is responsible for creating and owning the `TempDir`; this
/// function receives only the `&Path` (U-005: no `tempfile` import in `src/`).
///
/// Returns `Ok(())` once all overlay files are written, or the first
/// `std::io::Error` encountered during validation, directory creation, or file write.
///
/// # Security (SEC-001 / SEC-002)
///
/// Both `org_slug` and `sensor_id` are validated before any path join or TOML
/// string construction. Only ASCII alphanumeric, `-`, and `_` are accepted
/// (length 1–64). This rejects path components containing `..`, `/`, `"`, newlines,
/// or TOML structural characters, closing the CWE-22 path-traversal and CWE-93/74
/// TOML-injection vectors.
///
/// (BC-2.06.017 Postcondition 3 — after this function + `SpecLoader::load_all`,
/// `ResolvedSensorSpec` entries for each org carry the correct distinct `base_url`)
pub fn write_overlay_temp_dir(
    harness: &MultiInstanceHarness,
    dir: &std::path::Path,
) -> std::io::Result<()> {
    // Delegate to the map-key variant; avoids duplicating the write logic.
    // (BC-2.06.017 Postcondition 3 / BC-2.06.012)
    write_overlay_from_socket_map(harness.socket_map(), dir)
}

/// Write per-org sensor overlay TOML files from a pre-extracted socket map.
///
/// This is the map-key variant of [`write_overlay_temp_dir`] for callers that
/// have already extracted the socket addresses from a `MultiInstanceHarness`
/// (e.g., `BackgroundHarness::socket_map()` which runs the harness on a separate
/// tokio runtime). Semantics are identical to `write_overlay_temp_dir`.
///
/// # Security (SEC-001 / SEC-002)
///
/// Both `org_slug` and `sensor_id` are validated before any path join or TOML
/// string construction. Only ASCII alphanumeric, `-`, and `_` are accepted
/// (length 1–64). This rejects path components containing `..`, `/`, `"`, newlines,
/// or TOML structural characters.
///
/// (BC-2.06.017 Postcondition 3)
pub fn write_overlay_from_socket_map(
    socket_map: &HashMap<(String, String), SocketAddr>,
    dir: &std::path::Path,
) -> std::io::Result<()> {
    // Validate ALL entries BEFORE any filesystem mutation (fail-fast, atomicity).
    for (org_slug, sensor_id) in socket_map.keys() {
        validate_harness_key("org_slug", org_slug)?;
        validate_harness_key("sensor_id", sensor_id)?;
    }

    for ((org_slug, sensor_id), socket_addr) in socket_map {
        let customer_dir = dir.join("customers").join(org_slug);
        std::fs::create_dir_all(&customer_dir)?;

        let toml_path = customer_dir.join(format!("{sensor_id}.sensor.toml"));
        let toml_content = format!(
            "extends = \"{sensor_id}\"\n\
             instance_id = \"{sensor_id}@{org_slug}\"\n\
             base_url = \"http://{socket_addr}\"\n"
        );
        std::fs::write(&toml_path, toml_content)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_harness_key;

    /// SEC-002: org_slug with path traversal component is rejected.
    #[test]
    fn test_validate_harness_key_rejects_path_traversal_org_slug() {
        let result = validate_harness_key("org_slug", "../etc");
        assert!(
            result.is_err(),
            "org_slug '../etc' must be rejected (SEC-002 path traversal)"
        );
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("org_slug"),
            "error message must name the offending field; got: {msg}"
        );
    }

    /// SEC-001: sensor_id with TOML injection payload (quote + newline) is rejected.
    #[test]
    fn test_validate_harness_key_rejects_toml_injection_sensor_id() {
        // A value that would break out of a TOML string value: `"\nmalicious = "true"`.
        let injection = "arm\"\nmalicious = \"true";
        let result = validate_harness_key("sensor_id", injection);
        assert!(
            result.is_err(),
            "sensor_id containing quote+newline must be rejected (SEC-001 TOML injection)"
        );
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("sensor_id"),
            "error message must name the offending field; got: {msg}"
        );
    }

    /// Happy path: valid org_slug and sensor_id values are accepted.
    #[test]
    fn test_validate_harness_key_accepts_valid_values() {
        assert!(
            validate_harness_key("org_slug", "acme").is_ok(),
            "org_slug 'acme' must be accepted"
        );
        assert!(
            validate_harness_key("sensor_id", "armis").is_ok(),
            "sensor_id 'armis' must be accepted"
        );
        assert!(
            validate_harness_key("org_slug", "contoso-corp_01").is_ok(),
            "org_slug with hyphens and underscores must be accepted"
        );
    }

    /// SEC-002: forward slash in org_slug is rejected.
    #[test]
    fn test_validate_harness_key_rejects_forward_slash() {
        let result = validate_harness_key("org_slug", "acme/etc");
        assert!(
            result.is_err(),
            "org_slug 'acme/etc' must be rejected (SEC-002 path traversal via slash)"
        );
    }

    /// Edge: empty string is rejected.
    #[test]
    fn test_validate_harness_key_rejects_empty() {
        let result = validate_harness_key("org_slug", "");
        assert!(
            result.is_err(),
            "empty org_slug must be rejected (length constraint)"
        );
    }

    /// Edge: string exceeding 64 characters is rejected.
    #[test]
    fn test_validate_harness_key_rejects_too_long() {
        let long_value = "a".repeat(65);
        let result = validate_harness_key("org_slug", &long_value);
        assert!(
            result.is_err(),
            "org_slug of 65 characters must be rejected (length > 64)"
        );
    }
}
