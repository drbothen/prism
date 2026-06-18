//! MaxMind MMDB infusion source.
//!
//! Loads a MaxMind GeoIP2/GeoLite2 `.mmdb` file and implements `InfusionSource`.
//! Input: IP address string. Output: `serde_json::Value` with declared fields.
//!
//! Uses `maxminddb = "0.28"` which returns `LookupResult<'_, S>` from `lookup()`.
//! The `LookupResult::decode::<T>()` method returns `Result<Option<T>, MaxMindDbError>`.

use std::path::Path;

use super::super::InfusionSource;

/// Wrapper that implements Debug for maxminddb::Reader.
struct ReaderWrapper(maxminddb::Reader<Vec<u8>>);

impl std::fmt::Debug for ReaderWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MmdbReader").finish_non_exhaustive()
    }
}

/// MaxMind MMDB-backed infusion source.
///
/// Holds the `maxminddb::Reader<Vec<u8>>` (in-memory, loaded at construction time).
/// Column projection (field selection) happens at the UDF layer via
/// `InfusionUdfDescriptor::source_column` — not at this source level.
/// The MMDB lookup returns the full GeoIP record as JSON; the descriptor's
/// `source_column` selects which field to surface in the query result.
///
/// `#[non_exhaustive]`: forward-compat for infusion engine evolution — fields may expand
/// (e.g., reload policy, cache warm-up flag, MMDB metadata) without a breaking semver change.
/// External callers must use `MmdbSource::load()` for construction.
#[non_exhaustive]
#[derive(Debug)]
pub struct MmdbSource {
    /// Path of the loaded `.mmdb` file (for diagnostics).
    pub mmdb_path: String,
    /// The MMDB reader.
    reader: ReaderWrapper,
}

impl MmdbSource {
    /// Load a MaxMind MMDB file and return an `MmdbSource`.
    ///
    /// SEC-001 (CWE-400): checks `fs::metadata().len()` against `MAX_SOURCE_FILE_BYTES`
    /// BEFORE opening the MMDB reader. Files exceeding the limit are rejected with
    /// `InfusionError::SourceFileTooLarge` (E-INFUSE-012) to prevent unbounded-memory OOM.
    ///
    /// Uses `maxminddb::Reader::open_readfile(path)` (0.28 API).
    /// Maps MMDB open errors to `InfusionError::MissingRequiredField` with a
    /// `"mmdb_open_failed: {e}"` field descriptor.
    ///
    /// Column projection is handled at the UDF layer via `InfusionUdfDescriptor::source_column`;
    /// `MmdbSource` itself returns the full GeoIP record as JSON from every lookup.
    pub fn load(mmdb_path: &Path) -> Result<Self, prism_core::InfusionError> {
        // SEC-001 (CWE-400): size guard — BEFORE maxminddb::Reader::open_readfile.
        let file_size = std::fs::metadata(mmdb_path)
            .map_err(|e| prism_core::InfusionError::MissingRequiredField {
                field: format!("mmdb_metadata_failed: {}", e),
                spec_path: mmdb_path.to_string_lossy().to_string(),
            })?
            .len();
        if file_size > super::MAX_SOURCE_FILE_BYTES {
            return Err(prism_core::InfusionError::SourceFileTooLarge {
                path: mmdb_path.to_string_lossy().to_string(),
                size: file_size,
                limit: super::MAX_SOURCE_FILE_BYTES,
            });
        }

        let reader = maxminddb::Reader::open_readfile(mmdb_path).map_err(|e| {
            prism_core::InfusionError::MissingRequiredField {
                field: format!("mmdb_open_failed: {}", e),
                spec_path: mmdb_path.to_string_lossy().to_string(),
            }
        })?;

        Ok(MmdbSource {
            mmdb_path: mmdb_path.to_string_lossy().to_string(),
            reader: ReaderWrapper(reader),
        })
    }
}

impl InfusionSource for MmdbSource {
    /// Enrich a single IP address string via MMDB lookup.
    ///
    /// Uses maxminddb 0.28 `LookupResult::decode::<serde_json::Value>()`.
    /// Returns `None` on IP parse error, lookup miss, or decode error.
    fn enrich_single(&self, input: &str, _input_type: &str) -> Option<serde_json::Value> {
        let addr: std::net::IpAddr = input.parse().ok()?;
        // lookup() returns Result<LookupResult, MaxMindDbError>
        let lookup = self.reader.0.lookup(addr).ok()?;
        // decode::<T>() returns Result<Option<T>, MaxMindDbError>
        lookup.decode::<serde_json::Value>().ok()?
    }

    fn enrich_batch(&self, inputs: &[String], input_type: &str) -> Vec<Option<serde_json::Value>> {
        inputs
            .iter()
            .map(|i| self.enrich_single(i, input_type))
            .collect()
    }
}
