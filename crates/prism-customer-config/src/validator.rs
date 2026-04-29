use std::path::Path;

use crate::error::ConfigError;
use crate::schema::CustomerConfig;

/// Validate all `*.toml` files in `dir`.
///
/// Files are processed in lexicographic filename order (BC-3.3.004 Invariant 4).
/// Non-`.toml` files are silently skipped (EC-3.3.001-07).
///
/// Validation pass order per file (BC-3.3.003 Invariant 4, ADR-010 §2.6):
/// 1. TOML parse → `E-CFG-000` on failure
/// 2. `schema_version` check first — absent → `E-CFG-030`; ≠ 1 → `E-CFG-031`
/// 3. Credential heuristic pass (BC-3.3.002) on raw TOML value tree
/// 4. Structural validation (R-CUST-001 through R-CUST-017)
///
/// Cross-file after all per-file passes:
/// - Duplicate `org_id` → `E-CFG-011`
/// - Duplicate `org_slug` → `E-CFG-012`
///
/// Returns ALL errors collected across ALL files (multi-error, not fail-fast).
/// Returns empty vec on success.
pub fn validate_all(_dir: &Path) -> Vec<ConfigError> {
    todo!("validator::validate_all — implemented in Red Gate phase")
}

/// Parse and validate a single TOML file, returning the deserialized config and
/// any validation errors. Called by `validate_all` for each file in the directory.
#[allow(dead_code)]
fn validate_file(_path: &Path) -> (Option<CustomerConfig>, Vec<ConfigError>) {
    todo!("validator::validate_file — implemented in Red Gate phase")
}
