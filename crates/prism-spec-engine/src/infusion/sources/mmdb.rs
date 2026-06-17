//! MaxMind MMDB infusion source.
//!
//! Loads a MaxMind GeoIP2/GeoLite2 `.mmdb` file and implements `InfusionSource`.
//! Input: IP address string. Output: `serde_json::Value` with declared fields
//! (country ISO code, city, ASN, is_tor flag).
//!
//! Uses `maxminddb = "0.28"` which returns `LookupResult<'_, T>` from `lookup()`.
//! NOT the bare-T API of 0.24 — implementer must call `.deserialize::<T>()` on the
//! `LookupResult` (story Task 4 / S-1.14-REDO § Dev Notes).
//!
//! Supports `refresh_interval_secs` — schedules re-read via tokio interval.
//!
//! # Stub (S-1.14-REDO)
//! All non-trivial bodies are `todo!()`. The correct return type is now
//! `Result<Self, prism_core::InfusionError>` — the original stub had the WRONG return type
//! `Result<Self, crate::infusion::loader::InfusionLoader>` which is fixed here.
//! Implementation dispatched to S-1.14-REDO implementer.

use std::path::Path;

use super::super::InfusionSource;

/// MaxMind MMDB-backed infusion source.
///
/// Holds the `maxminddb::Reader<Vec<u8>>` (in-memory, loaded at construction time)
/// and the declared `field_names` from the `[[infusion.fields]]` spec entries.
///
/// `field_names` determines which columns the source tries to extract from each
/// MMDB lookup result.
#[derive(Debug)]
pub struct MmdbSource {
    /// Path of the loaded `.mmdb` file (for diagnostics; not used after load).
    pub mmdb_path: String,
    /// Declared output field names from the infusion spec.
    pub field_names: Vec<String>,
    // NOTE: the real `maxminddb::Reader<Vec<u8>>` is NOT stored as a field here in the stub
    // because maxminddb::Reader does not implement Debug; adding it requires a custom Debug impl.
    // The implementer must add it and provide a manual Debug impl (or use `#[allow(dead_code)]`
    // and `derive(Debug)` after wrapping the reader in a newtype).
    // Reference: maxminddb 0.28 docs / story Task 4.
}

impl MmdbSource {
    /// Load a MaxMind MMDB file and return an `MmdbSource`.
    ///
    /// Uses `maxminddb::Reader::open_readfile(path)` (0.28 API).
    /// Maps MMDB open errors to `InfusionError::MissingRequiredField` with a
    /// `"mmdb_open_failed: {e}"` field descriptor (story Task 4 code skeleton).
    ///
    /// # Return type
    /// FIXED from the original stub: return type is `Result<Self, prism_core::InfusionError>`,
    /// NOT `Result<Self, crate::infusion::loader::InfusionLoader>` (that was a bug).
    pub fn load(
        _mmdb_path: &Path,
        _field_names: Vec<String>,
    ) -> Result<Self, prism_core::InfusionError> {
        todo!(
            "S-1.14-REDO: implement MmdbSource::load using maxminddb 0.28 API \
             (Reader::open_readfile, LookupResult deserialization) — BC-2.19.001 / AC-1"
        )
    }
}

impl InfusionSource for MmdbSource {
    /// Enrich a single IP address string via MMDB lookup.
    ///
    /// maxminddb 0.28: `reader.lookup::<serde_json::Value>(addr)` returns
    /// `Result<LookupResult<'_, serde_json::Value>, MaxMindDbError>`.
    /// The `LookupResult` must be deserialized — it does NOT auto-coerce to `T`.
    /// (story Task 4 note on LookupResult API).
    fn enrich_single(&self, _input: &str, _input_type: &str) -> Option<serde_json::Value> {
        todo!(
            "S-1.14-REDO: implement MmdbSource::enrich_single using maxminddb 0.28 \
             LookupResult deserialization API — BC-2.19.001 / AC-1 / AC-2"
        )
    }

    fn enrich_batch(
        &self,
        _inputs: &[String],
        _input_type: &str,
    ) -> Vec<Option<serde_json::Value>> {
        todo!(
            "S-1.14-REDO: implement MmdbSource::enrich_batch (delegate to enrich_single per input) \
             — BC-2.19.001 / AC-1 / AC-2"
        )
    }
}
