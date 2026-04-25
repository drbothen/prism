//! Internal table registry — RocksDB domains as PrismQL-queryable tables
//! (S-2.03, BC-2.15.011).
//!
//! Registers seven RocksDB domains as DataFusion table providers so analysts
//! can query internal Prism state (alerts, cases, rules, schedules, diff
//! results, audit log, aliases) using the same PrismQL syntax as external
//! sensor tables.
//!
//! ## Responsibilities in this story (S-2.03)
//!
//! - `INTERNAL_TABLES` static — all 7 descriptors with full column schemas.
//! - `get_descriptor(name)` — O(n) lookup (7 entries; linear scan is fine).
//! - `all_descriptors()` — returns the full static slice.
//! - `scan_limit()` — reads `PRISM_MAX_INTERNAL_TABLE_SCAN` from env; pure.
//! - `check_table_access(descriptor, capabilities)` — capability gate for
//!   `prism_audit` (E-QUERY-011, AC-8, AC-9).
//!
//! ## Not in this story
//!
//! - DataFusion `TableProvider` implementation — lives in prism-query (S-3.02).
//! - Arrow `RecordBatch` construction — lives in prism-query (S-3.02).
//! - Write-query rejection (E-QUERY-010) — enforced at SQL parse step in S-3.02.
//!
//! ## Architecture compliance
//!
//! No DataFusion, Arrow, or arrow-schema imports.  `scan_limit()` is a pure
//! function (no RocksDB access).

use prism_core::{
    CapabilityPath, ClientCapabilities, InternalColumnType, InternalTableDescriptor, PrismError,
    StorageDomain,
};

// ─────────────────────────────────────────────────────────────────────────────
// Column schema helpers (BC-2.15.011 Phase 4, Task 10)
// ─────────────────────────────────────────────────────────────────────────────

/// Columns for `prism_alerts` (StorageDomain::Alerts).
///
/// Sufficient to satisfy `SELECT * FROM prism_alerts`.
fn alerts_columns() -> Vec<(String, InternalColumnType)> {
    vec![
        ("alert_id".to_owned(), InternalColumnType::Text),
        ("severity_id".to_owned(), InternalColumnType::UInt64),
        ("device_ip".to_owned(), InternalColumnType::Text),
        ("device_hostname".to_owned(), InternalColumnType::Text),
        ("client_id".to_owned(), InternalColumnType::Text),
        ("created_at".to_owned(), InternalColumnType::Timestamp),
        ("rule_id".to_owned(), InternalColumnType::Text),
    ]
}

/// Columns for `prism_cases` (StorageDomain::Cases).
fn cases_columns() -> Vec<(String, InternalColumnType)> {
    vec![
        ("case_id".to_owned(), InternalColumnType::Text),
        ("title".to_owned(), InternalColumnType::Text),
        ("severity_id".to_owned(), InternalColumnType::UInt64),
        ("client_id".to_owned(), InternalColumnType::Text),
        ("created_at".to_owned(), InternalColumnType::Timestamp),
        ("status".to_owned(), InternalColumnType::Text),
    ]
}

/// Columns for `prism_rules` (StorageDomain::DetectionRules).
fn rules_columns() -> Vec<(String, InternalColumnType)> {
    vec![
        ("rule_id".to_owned(), InternalColumnType::Text),
        ("name".to_owned(), InternalColumnType::Text),
        ("client_id".to_owned(), InternalColumnType::Text),
        ("enabled".to_owned(), InternalColumnType::Bool),
        ("created_at".to_owned(), InternalColumnType::Timestamp),
    ]
}

/// Columns for `prism_schedules` (StorageDomain::Schedules).
fn schedules_columns() -> Vec<(String, InternalColumnType)> {
    vec![
        ("schedule_id".to_owned(), InternalColumnType::Text),
        ("name".to_owned(), InternalColumnType::Text),
        ("client_id".to_owned(), InternalColumnType::Text),
        ("query".to_owned(), InternalColumnType::Text),
        ("interval_secs".to_owned(), InternalColumnType::UInt64),
        ("last_run_at".to_owned(), InternalColumnType::Timestamp),
    ]
}

/// Columns for `prism_diff_results` (StorageDomain::DiffResults).
///
/// Metadata-only per BC-2.15.011: the raw `previous_results` blob is NOT
/// exposed as a queryable column (AC-14).  Use `get_diff_results` MCP tool
/// to inspect diff content.
fn diff_results_columns() -> Vec<(String, InternalColumnType)> {
    vec![
        ("query_hash".to_owned(), InternalColumnType::Text),
        ("client_id".to_owned(), InternalColumnType::Text),
        ("previous_results_hash".to_owned(), InternalColumnType::Text),
        ("epoch".to_owned(), InternalColumnType::UInt64),
        ("counter".to_owned(), InternalColumnType::UInt64),
        ("last_diff_time".to_owned(), InternalColumnType::Timestamp),
    ]
}

/// Columns for `prism_audit` (StorageDomain::AuditBuffer).
///
/// Requires `audit.read` capability (E-QUERY-011).
fn audit_columns() -> Vec<(String, InternalColumnType)> {
    vec![
        ("trace_id".to_owned(), InternalColumnType::Text),
        ("timestamp_ns".to_owned(), InternalColumnType::UInt64),
        ("operation".to_owned(), InternalColumnType::Text),
        ("client_id".to_owned(), InternalColumnType::Text),
        ("analyst_id".to_owned(), InternalColumnType::Text),
        ("outcome".to_owned(), InternalColumnType::Text),
        ("capability".to_owned(), InternalColumnType::Text),
    ]
}

/// Columns for `prism_aliases` (AliasStore-backed, NOT RocksDB).
///
/// `rocksdb_backed: false` — S-3.02 `TableProvider` reads from the in-memory
/// `AliasStore` (loaded from `aliases.toml`) rather than `StorageBackend::scan()`.
fn aliases_columns() -> Vec<(String, InternalColumnType)> {
    vec![
        ("alias_id".to_owned(), InternalColumnType::Text),
        ("alias".to_owned(), InternalColumnType::Text),
        ("expansion".to_owned(), InternalColumnType::Text),
        ("client_id".to_owned(), InternalColumnType::Text),
        ("created_at".to_owned(), InternalColumnType::Timestamp),
    ]
}

// ─────────────────────────────────────────────────────────────────────────────
// Static descriptor table (BC-2.15.011 postcondition — 7 entries, AC-13)
// ─────────────────────────────────────────────────────────────────────────────

/// All registered internal PrismQL tables.
///
/// A static slice of [`InternalTableDescriptor`] covering all seven RocksDB
/// domains exposed as queryable tables (BC-2.15.011, AC-13).
///
/// Note: `Vec` fields in `InternalTableDescriptor` mean we cannot make this a
/// true `static` with array literals.  We use `std::sync::OnceLock` to
/// initialize once at first access, keeping the data logically static.
static INTERNAL_TABLES_CELL: std::sync::OnceLock<Vec<InternalTableDescriptor>> =
    std::sync::OnceLock::new();

/// Return a reference to the slice of all internal table descriptors.
///
/// Initialized on first call; the `Vec` is owned by a `OnceLock` and lives
/// for the duration of the process.
pub fn all_descriptors() -> &'static [InternalTableDescriptor] {
    INTERNAL_TABLES_CELL.get_or_init(init_internal_tables)
}

/// Build the canonical list of internal table descriptors.
///
/// Called exactly once by `OnceLock::get_or_init`.
fn init_internal_tables() -> Vec<InternalTableDescriptor> {
    vec![
        InternalTableDescriptor {
            table_name: "prism_alerts",
            domain: Some(StorageDomain::Alerts),
            columns: alerts_columns(),
            requires_audit_read: false,
            rocksdb_backed: true,
        },
        InternalTableDescriptor {
            table_name: "prism_cases",
            domain: Some(StorageDomain::Cases),
            columns: cases_columns(),
            requires_audit_read: false,
            rocksdb_backed: true,
        },
        InternalTableDescriptor {
            table_name: "prism_rules",
            domain: Some(StorageDomain::DetectionRules),
            columns: rules_columns(),
            requires_audit_read: false,
            rocksdb_backed: true,
        },
        InternalTableDescriptor {
            table_name: "prism_schedules",
            domain: Some(StorageDomain::Schedules),
            columns: schedules_columns(),
            requires_audit_read: false,
            rocksdb_backed: true,
        },
        InternalTableDescriptor {
            table_name: "prism_diff_results",
            domain: Some(StorageDomain::DiffResults),
            columns: diff_results_columns(),
            requires_audit_read: false,
            rocksdb_backed: true,
        },
        InternalTableDescriptor {
            table_name: "prism_audit",
            domain: Some(StorageDomain::AuditBuffer),
            columns: audit_columns(),
            requires_audit_read: true,
            rocksdb_backed: true,
        },
        // prism_aliases: backed by AliasStore (TOML), NOT RocksDB.
        // Per BC-2.11.008, aliases persist in aliases.toml (S-3.04).
        // The Aliases StorageDomain and CF exist but are reserved for future use.
        InternalTableDescriptor {
            table_name: "prism_aliases",
            domain: None,
            columns: aliases_columns(),
            requires_audit_read: false,
            rocksdb_backed: false,
        },
    ]
}

// ─────────────────────────────────────────────────────────────────────────────
// Public registry API (BC-2.15.011)
// ─────────────────────────────────────────────────────────────────────────────

/// Look up a descriptor by PrismQL table name.
///
/// Returns `None` for unknown table names (EC-005 — callers handle gracefully
/// without panic).  Linear scan over 7 entries.
///
/// # AC-7
/// `get_descriptor("prism_alerts")` returns a descriptor with
/// `table_name == "prism_alerts"`, `domain == Some(StorageDomain::Alerts)`,
/// `requires_audit_read == false`, and `columns[0] == ("alert_id", Text)`.
///
/// # AC-8
/// `get_descriptor("prism_audit")` returns a descriptor with
/// `requires_audit_read == true`.
pub fn get_descriptor(_table_name: &str) -> Option<&'static InternalTableDescriptor> {
    todo!(
        "BC-2.15.011 postcondition: linear scan of all_descriptors() for table_name; \
         return Some(&descriptor) on match, None on miss (EC-005). \
         AC-7: prism_alerts returns domain=Alerts, requires_audit_read=false, first column=alert_id. \
         AC-8: prism_audit returns requires_audit_read=true."
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// Scan limit (BC-2.15.011 — configurable via PRISM_MAX_INTERNAL_TABLE_SCAN)
// ─────────────────────────────────────────────────────────────────────────────

/// Default row limit for internal table scans.
const DEFAULT_SCAN_LIMIT: usize = 50_000;

/// Return the maximum number of rows to return from an internal table scan.
///
/// Reads `PRISM_MAX_INTERNAL_TABLE_SCAN` from the environment. If the variable
/// is absent or unparseable, returns the default of `50_000`.
///
/// This is a pure function with no RocksDB access — the actual truncation is
/// enforced in prism-query (S-3.02) when iterating RocksDB results.
///
/// # AC-11
/// - Env var absent → `50_000`
/// - Env var `"1000"` → `1000`
/// - Env var `"abc"` → `50_000` (no panic)
pub fn scan_limit() -> usize {
    std::env::var("PRISM_MAX_INTERNAL_TABLE_SCAN")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(DEFAULT_SCAN_LIMIT)
}

// ─────────────────────────────────────────────────────────────────────────────
// Capability gate (BC-2.15.011 E-QUERY-011)
// ─────────────────────────────────────────────────────────────────────────────

/// Check whether the given client capabilities allow access to `descriptor`.
///
/// Returns `Ok(())` when access is permitted.  Returns
/// `Err(PrismError::AuditTableAccessDenied)` when the descriptor requires
/// `audit.read` and the client does NOT have `audit.read = Allow`
/// (E-QUERY-011, AC-9).
///
/// All non-audit tables return `Ok(())` for any capability set (AC-10).
///
/// # AC-9
/// `check_table_access(audit_descriptor, caps_without_audit_read)` →
/// `Err(PrismError::AuditTableAccessDenied)` where the error Display contains
/// "audit.read capability".
///
/// # AC-10
/// `check_table_access(alerts_descriptor, any_caps)` → `Ok(())`.
pub fn check_table_access(
    descriptor: &InternalTableDescriptor,
    capabilities: &ClientCapabilities,
) -> Result<(), PrismError> {
    if !descriptor.requires_audit_read {
        return Ok(());
    }

    // Construct the capability path — this is infallible for a known-valid
    // string literal; unwrap is safe here.
    let audit_path =
        CapabilityPath::new("audit.read").expect("'audit.read' is a valid capability path");
    let (allowed, _explanation) = capabilities.is_allowed(&audit_path);

    if allowed {
        Ok(())
    } else {
        Err(PrismError::AuditTableAccessDenied)
    }
}
