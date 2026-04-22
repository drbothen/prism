//! InjectionScanner — suspicious pattern detection with NFKC normalization (BC-2.09.003).
//!
//! Stub: all method bodies are `unimplemented!()`. Red Gate — tests must fail.
//!
//! Architecture notes (from story spec):
//! - Regex patterns compiled ONCE via `OnceLock<RegexSet>` — never per-scan.
//! - NFKC normalization applied before every pattern match.
//! - Base64 heuristic: strings >20 chars matching `[A-Za-z0-9+/=]{20,}` decoded
//!   and re-scanned for injection markers.
//! - Original field values are NEVER modified (flag-don't-strip principle).
//! - `_meta.safety_flags` is additive; all detections appended.
//! - Scan limit: 10KB per field; beyond limit a `TruncatedScan` flag is added.

use prism_core::{PatternCategory, SafetyFlag};

/// Maximum number of bytes scanned per string field (BC-2.09.003 EC-09-006).
pub const SCAN_LIMIT_BYTES: usize = 10_240;

/// Input to `InjectionScanner::scan`.
///
/// Wraps a field name, item index (for the results array), and the raw string
/// value to scan.
pub struct ScanInput<'a> {
    /// Name of the field in the sensor record (e.g., `"hostname"`).
    pub field: &'a str,
    /// Zero-based index of the item in the results array.
    pub index: usize,
    /// Raw string value from the sensor record.
    pub value: &'a str,
}

/// Result of scanning one field value.
///
/// BC-2.09.004: flags are centralized in `_meta.safety_flags`; original value
/// is preserved unchanged.
pub struct ScanResult {
    /// All safety flags detected for this field value.
    /// Empty when no patterns matched.
    pub flags: Vec<SafetyFlag>,
    /// The original, unmodified field value (flag-don't-strip principle).
    pub original_value: String,
}

/// Injection scanner implementing BC-2.09.003 and BC-2.09.004.
///
/// The regex patterns are compiled once via `OnceLock` at first use.
/// The scanner is stateless after initialization — `scan()` takes `&self`.
pub struct InjectionScanner;

impl InjectionScanner {
    /// Returns the singleton scanner instance.
    ///
    /// Regex patterns are compiled once on first call via `OnceLock`.
    /// Subsequent calls return the same instance with zero overhead.
    pub fn global() -> &'static InjectionScanner {
        unimplemented!("InjectionScanner::global — stub (Red Gate)")
    }

    /// Scan a single field value for injection patterns.
    ///
    /// ## Procedure (BC-2.09.003)
    /// 1. Apply NFKC normalization to `input.value`.
    /// 2. If value exceeds `SCAN_LIMIT_BYTES`, truncate for scanning only;
    ///    original is preserved; a `TruncatedScan` flag is emitted.
    /// 3. Match normalized string against all compiled regex patterns.
    /// 4. Apply base64 heuristic on strings matching `[A-Za-z0-9+/=]{20,}`.
    /// 5. Append one `SafetyFlag` per pattern match to result.
    /// 6. Return `ScanResult` with flags and original (unmodified) value.
    ///
    /// This method NEVER panics (VP-038).
    pub fn scan<'a>(&self, input: ScanInput<'a>) -> ScanResult {
        unimplemented!("InjectionScanner::scan — stub (Red Gate)")
    }

    /// Scan a byte slice — used by the fuzz target (VP-038).
    ///
    /// Invalid UTF-8 is handled gracefully (lossy decode). Never panics.
    pub fn scan_bytes(&self, field: &str, index: usize, data: &[u8]) -> ScanResult {
        unimplemented!("InjectionScanner::scan_bytes — stub (Red Gate)")
    }

    /// Scan all string fields in a sensor record.
    ///
    /// Returns a flat list of all `SafetyFlag`s found across all fields.
    /// Fields are provided as `(field_name, item_index, value)` triples.
    pub fn scan_record(&self, fields: &[(&str, usize, &str)]) -> Vec<SafetyFlag> {
        unimplemented!("InjectionScanner::scan_record — stub (Red Gate)")
    }
}
