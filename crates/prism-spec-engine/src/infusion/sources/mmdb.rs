//! MaxMind MMDB infusion source.
//!
//! Loads a MaxMind GeoIP2/GeoLite2 `.mmdb` file and implements `InfusionSource`.
//! Input: IP address string. Output: `serde_json::Value` with declared fields
//! (country ISO code, city, ASN, is_tor flag).
//!
//! Supports `refresh_interval_secs` — schedules re-read via tokio interval.
//!
//! # Stub
//! All methods are `unimplemented!()` — implementation in S-1.14.

use super::super::InfusionSource;

/// MaxMind MMDB-backed infusion source.
#[derive(Debug)]
pub struct MmdbSource {
    pub mmdb_path: String,
}

impl MmdbSource {
    /// Load a MaxMind MMDB file and return an `MmdbSource`.
    pub fn load(_mmdb_path: &str) -> Result<Self, crate::infusion::loader::InfusionLoader> {
        unimplemented!("MmdbSource::load — implement in S-1.14 (BC-2.19.001)")
    }
}

impl InfusionSource for MmdbSource {
    fn enrich_single(&self, _input: &str, _input_type: &str) -> Option<serde_json::Value> {
        unimplemented!("MmdbSource::enrich_single — implement in S-1.14 (BC-2.19.001)")
    }

    fn enrich_batch(
        &self,
        _inputs: &[String],
        _input_type: &str,
    ) -> Vec<Option<serde_json::Value>> {
        unimplemented!("MmdbSource::enrich_batch — implement in S-1.14 (BC-2.19.001)")
    }
}
