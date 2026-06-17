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
/// Holds the `maxminddb::Reader<Vec<u8>>` (in-memory, loaded at construction time)
/// and the declared `field_names` from the `[[infusion.fields]]` spec entries.
///
/// `#[non_exhaustive]`: forward-compat for infusion engine evolution — fields may expand
/// (e.g., reload policy, cache warm-up flag, MMDB metadata) without a breaking semver change.
/// External callers must use `MmdbSource::load()` for construction.
#[non_exhaustive]
#[derive(Debug)]
pub struct MmdbSource {
    /// Path of the loaded `.mmdb` file (for diagnostics).
    pub mmdb_path: String,
    /// Declared output field names from the infusion spec.
    pub field_names: Vec<String>,
    /// The MMDB reader.
    reader: ReaderWrapper,
}

impl MmdbSource {
    /// Load a MaxMind MMDB file and return an `MmdbSource`.
    ///
    /// Uses `maxminddb::Reader::open_readfile(path)` (0.28 API).
    /// Maps MMDB open errors to `InfusionError::MissingRequiredField` with a
    /// `"mmdb_open_failed: {e}"` field descriptor.
    pub fn load(
        mmdb_path: &Path,
        field_names: Vec<String>,
    ) -> Result<Self, prism_core::InfusionError> {
        let reader = maxminddb::Reader::open_readfile(mmdb_path).map_err(|e| {
            prism_core::InfusionError::MissingRequiredField {
                field: format!("mmdb_open_failed: {}", e),
                spec_path: mmdb_path.to_string_lossy().to_string(),
            }
        })?;

        Ok(MmdbSource {
            mmdb_path: mmdb_path.to_string_lossy().to_string(),
            field_names,
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
