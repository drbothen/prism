//! SafetyEnvelope — MCP response envelope with trust annotations (BC-2.09.008).
//!
//! Stub: `unimplemented!()` bodies. Red Gate — tests must fail.
//!
//! Wraps all tool responses with:
//! ```json
//! {
//!   "_meta": {
//!     "tool": "<tool_name>",
//!     "data_source": "<sensor_id>",
//!     "query_time": "<ISO8601>",
//!     "trust_level": "untrusted_external" | "internal",
//!     "safety_flags": [{...}, ...],
//!     "total_results": <integer>,
//!     "page": <integer>,
//!     "has_more": <boolean>,
//!     "next_cursor": "<cursor>" | null
//!   },
//!   "results": [...]
//! }
//! ```

use prism_core::{SafetyFlag, TrustLevel};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The `_meta` section of a Prism MCP response envelope (BC-2.09.008).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMeta {
    pub tool: String,
    pub data_source: DataSource,
    pub query_time: String,
    pub trust_level: TrustLevel,
    pub safety_flags: Vec<SafetyFlag>,
    pub total_results: u64,
    pub page: u64,
    pub has_more: bool,
    pub next_cursor: Option<String>,
}

/// Data source: single sensor or multiple sensors (cross-client query).
///
/// BC-2.09.008 EC-09-019: cross-client queries report an array.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DataSource {
    Single(String),
    Multiple(Vec<String>),
}

/// The full response envelope (BC-2.09.008).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseEnvelope {
    #[serde(rename = "_meta")]
    pub meta: ResponseMeta,
    pub results: Value,
}

/// Builder for `ResponseEnvelope` — applies injection scanning and constructs
/// the `_meta` envelope (BC-2.09.008 + BC-2.09.003 + BC-2.09.004).
pub struct SafetyEnvelopeBuilder;

impl SafetyEnvelopeBuilder {
    /// Wrap raw sensor results in the safety envelope.
    ///
    /// ## Procedure
    /// 1. Run `InjectionScanner::scan_record` over all string fields in `results`.
    /// 2. Collect all `SafetyFlag`s into `_meta.safety_flags`.
    /// 3. Set `_meta.trust_level` based on the tool name.
    /// 4. Set `_meta.query_time` to the current timestamp.
    /// 5. Never modify `results` values (flag-don't-strip).
    pub fn wrap(
        tool: &str,
        data_source: DataSource,
        results: Value,
        page: u64,
        has_more: bool,
        next_cursor: Option<String>,
    ) -> ResponseEnvelope {
        unimplemented!("SafetyEnvelopeBuilder::wrap — stub (Red Gate)")
    }

    /// Returns `true` if `envelope._meta.safety_flags` is always present
    /// (even as an empty array) for the given envelope.
    ///
    /// BC-2.09.008: `_meta.safety_flags` is always present.
    pub fn safety_flags_always_present(envelope: &ResponseEnvelope) -> bool {
        unimplemented!("SafetyEnvelopeBuilder::safety_flags_always_present — stub (Red Gate)")
    }
}
