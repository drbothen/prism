# Public API — prism-dtu-threatintel

Crate: `prism-dtu-threatintel`
Feature gate: `dtu` (compile-time) and `#[cfg(any(test, feature = "dtu"))]`
Dependency: `prism-dtu-common` (path dep) for `BehavioralClone`, `StubConfig`, `build_test_client`

## Exported Items

### `ThreatIntelClone` (re-exported via `pub use clone::ThreatIntelClone`)

Primary struct. Implements `BehavioralClone` from `prism-dtu-common`.

```rust
pub struct ThreatIntelClone {
    pub config: StubConfig,
    pub state: Arc<ThreatIntelState>,
    pub bound_addr: Option<SocketAddr>,
}

impl ThreatIntelClone {
    pub fn new() -> Self
    pub fn with_config(config: StubConfig) -> Self
}

#[async_trait]
impl BehavioralClone for ThreatIntelClone {
    async fn start(&mut self) -> anyhow::Result<()>
    async fn reset(&self) -> anyhow::Result<()>
    async fn configure(&self, config: serde_json::Value) -> anyhow::Result<()>
    fn bound_addr(&self) -> SocketAddr
}
```

`start()` binds an ephemeral `127.0.0.1:0` TCP listener and spawns the Axum server.
`base_url()` (provided by `BehavioralClone` blanket) returns `http://127.0.0.1:{port}`.

### `ThreatIntelState` (pub, in `state` module)

Shared server state, Arc-wrapped internally.

```rust
pub struct ThreatIntelState {
    pub fixture_registry: Mutex<HashMap<String, FixtureKey>>,
    pub request_counter: AtomicU32,
    pub rate_limit_after: Mutex<Option<u32>>,
}

impl ThreatIntelState {
    pub fn new() -> Self
    pub fn reset(&self)
    pub fn increment_counter(&self) -> u32
    pub fn is_rate_limited(&self, current_count: u32) -> bool
    pub fn lookup_fixture(&self, key: &str) -> Option<FixtureKey>
}
```

### `FixtureKey` (pub, in `types` module)

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FixtureKey { Malicious, Benign, Unknown }
```

### HTTP API Surface

| Method | Path | Auth Required | Description |
|--------|------|---------------|-------------|
| GET | `/v3/ip/:ip` | Yes (key or Bearer) | IP threat lookup |
| GET | `/v3/domain/:domain` | Yes | Domain threat lookup |
| GET | `/v3/hash/:hash` | Yes | File hash lookup (VirusTotal shape) |
| POST | `/dtu/configure` | No | Set rate_limit_after or add registry entry |

### Default Fixture Registry

| Lookup Value | Fixture |
|---|---|
| `45.55.100.1` | Malicious |
| `8.8.8.8` | Benign |
| `0.0.0.0` | Unknown |
| `evil.example.com` | Malicious |
| `safe.example.com` | Benign |
