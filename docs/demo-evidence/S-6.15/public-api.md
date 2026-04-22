# Public API — prism-dtu-nvd

Crate gate: `#[cfg(any(test, feature = "dtu"))]` — never compiled into production binaries.

## NvdClone (implements BehavioralClone)

`crates/prism-dtu-nvd/src/clone.rs`

```rust
pub struct NvdClone { /* opaque */ }

impl NvdClone {
    /// Construct a new clone. Loads fixtures/cves.json from CARGO_MANIFEST_DIR.
    pub fn new() -> anyhow::Result<Self>;

    /// Return the per-CVE request count (for cache-hit assertion in integration tests).
    pub fn request_count_for(&self, cve_id: &str) -> u32;
}

#[async_trait]
impl BehavioralClone for NvdClone {
    /// Bind ephemeral TCP port and start the axum server.
    async fn start(&mut self) -> anyhow::Result<()>;

    /// Clear request counters and reset all rate-limit buckets.
    async fn reset(&self) -> anyhow::Result<()>;

    /// Apply JSON config patch: supports "auth_mode" and "exhaust_authenticated_bucket".
    async fn configure(&self, config: serde_json::Value) -> anyhow::Result<()>;

    /// Return the SocketAddr the server is listening on (panics if called before start).
    fn bound_addr(&self) -> SocketAddr;
}
```

`BehavioralClone` also provides the derived `base_url(&self) -> String` method returning
`http://{bound_addr}`.

## NvdState — Key Methods

`crates/prism-dtu-nvd/src/state.rs`

| Method | Signature | Purpose |
|--------|-----------|---------|
| `new` | `fn new(registry: HashMap<String, CveRecord>) -> Self` | Construct with pre-loaded CVE registry |
| `lookup_and_count` | `fn lookup_and_count(&self, cve_id: &str) -> Option<CveRecord>` | Case-normalized lookup; increments request counter |
| `request_count_for` | `fn request_count_for(&self, cve_id: &str) -> u32` | Read request counter (test API) |
| `check_rate_limit` | `fn check_rate_limit(&self, api_key: Option<&str>) -> Result<(), RateLimitError>` | Enforce dual buckets; honor auth_mode |
| `reset` | `fn reset(&self)` | Clear counters, reset buckets, restore AuthMode::Accept |
| `apply_config` | `fn apply_config(&self, config: &serde_json::Value) -> anyhow::Result<()>` | Toggle auth_mode, exhaust buckets |

## HTTP Routes

| Route | Method | Purpose |
|-------|--------|---------|
| `/rest/json/cves/2.0` | GET | Single CVE lookup (`cveId` param) or bulk paginated fetch |
| `/dtu/request-count/:cve_id` | GET | Returns `{"cve_id": "...", "count": N}` |
| `/dtu/configure` | POST | Applies JSON config patch |
| `/dtu/reset` | POST | Resets all mutable state |

## Re-exports

`crates/prism-dtu-nvd/src/lib.rs`:

```rust
pub use clone::NvdClone;
pub use state::NvdState;
pub use types::{CveRecord, CveResponse, NvdError};
```

## Key Types

`crates/prism-dtu-nvd/src/types.rs`

| Type | Purpose |
|------|---------|
| `CveResponse` | Top-level NVD API 2.0 response (`resultsPerPage`, `startIndex`, `totalResults`, `vulnerabilities`) |
| `VulnerabilityWrapper` | `{"cve": CveRecord}` wrapper in `vulnerabilities` array |
| `CveRecord` | Full NVD CVE object: `id`, `published`, `lastModified`, `vulnStatus`, `descriptions`, `metrics`, `weaknesses`, `cisaKevVulnAdded?` |
| `CveMetrics` | Contains `cvssMetricV31: Vec<CvssMetricV31>` |
| `CvssData` | `version`, `vectorString`, `baseScore`, `baseSeverity` |
| `NvdError` | `{"error": "...", "cveId"?: "..."}` — all 4xx/5xx bodies |
| `RateLimitBucket` | `{count, window_start, limit}` — 5/30s unauth, 50/30s auth |
| `RequestCountResponse` | `{"cve_id": "...", "count": N}` from test API |

## Fixture CVE Set

10 CVEs pre-loaded from `crates/prism-dtu-nvd/fixtures/cves.json`:

| CVE ID | CVSS Score | Severity | CISA KEV | CWE |
|--------|-----------|----------|----------|-----|
| CVE-2024-0001 | 9.8 | CRITICAL | Yes | CWE-89 |
| CVE-2024-0002 | 7.5 | HIGH | No | — |
| CVE-2024-0003 | 5.3 | MEDIUM | No | — |
| CVE-2024-0004 | 3.1 | LOW | No | — |
| CVE-2024-0005 | 0.0 | NONE | No | — |
| CVE-2024-0006 | 9.8 | CRITICAL | Yes | CWE-787 |
| CVE-2024-0007 | 8.8 | HIGH | No | — |
| CVE-2024-0008 | 6.5 | MEDIUM | No | — |
| CVE-2024-0009 | 4.3 | MEDIUM | No | — |
| CVE-2024-0010 | 10.0 | CRITICAL | Yes | — |
