# Public API Snapshot — prism-dtu-common

> cargo doc deferred — nightly-only rustfmt flags cause warnings in stable; API extracted
> directly from `src/lib.rs` re-exports and module sources. Will be validated in CI's
> docs job (stable toolchain + `--features dtu`).

Crate path: `crates/prism-dtu-common/`
Feature gate: `#[cfg(any(test, feature = "dtu"))]` — never links into a production binary.

---

## Exported items

```
prism_dtu_common
├── BehavioralClone (trait, re-export from clone)
│   ├── async fn start(&mut self) -> anyhow::Result<()>
│   ├── async fn reset(&self) -> anyhow::Result<()>
│   ├── async fn configure(&self, config: serde_json::Value) -> anyhow::Result<()>
│   ├── fn bound_addr(&self) -> SocketAddr
│   └── fn base_url(&self) -> String   [provided default]
│
├── StubConfig (struct, re-export from config)
│   ├── seed: u64
│   ├── latency_ms: u64
│   └── failure_mode: FailureMode
│   └── impl Default (seed=42, latency_ms=0, failure_mode=None)
│
├── FailureMode (enum, re-export from config)
│   ├── None
│   ├── RateLimit { after_n_requests: u32, retry_after_secs: u32 }
│   ├── InternalError { at_request_n: u32 }
│   ├── NetworkTimeout { after_ms: u64 }
│   └── AuthReject
│
├── LatencyLayer (struct, re-export from layers)
│   └── latency_ms: u64
│   └── impl tower::Layer<S> -> LatencyMiddleware<S>
│
├── FailureLayer (struct, re-export from layers)
│   └── mode: FailureMode
│   └── impl tower::Layer<S> -> FailureMiddleware<S>
│
├── SyslogReceiver (struct, re-export from syslog)
│   ├── async fn start(addr: SocketAddr) -> anyhow::Result<Self>
│   ├── fn bound_addr(&self) -> SocketAddr
│   ├── fn received_messages(&self) -> Vec<String>
│   └── fn reset(&self)
│
├── WebhookReceiver (struct, re-export from webhook)
│   ├── async fn start() -> anyhow::Result<Self>
│   ├── fn bound_addr(&self) -> SocketAddr
│   ├── fn received_payloads(&self) -> Vec<CapturedRequest>
│   └── fn reset(&self)
│
├── CapturedRequest (struct, re-export from webhook)
│   ├── path: String
│   ├── body: Bytes
│   └── headers: HeaderMap
│
├── FidelityValidator (struct, re-export from fidelity)
│   └── async fn run(base_url: &str, checks: Vec<FidelityCheck>) -> FidelityReport
│
├── FidelityCheck (struct, re-export from fidelity)
│   ├── endpoint: String
│   ├── method: http::Method
│   ├── body: Option<serde_json::Value>
│   ├── expected_status: u16
│   └── required_fields: Vec<String>
│
├── FidelityFailure (struct, re-export from fidelity)
│   ├── endpoint: String
│   └── reason: String
│
├── FidelityReport (struct, re-export from fidelity)
│   ├── checks_passed: usize
│   ├── checks_failed: usize
│   └── failures: Vec<FidelityFailure>
│
├── seeded_rng(seed: u64) -> ChaCha20Rng  (fn, re-export from seed)
│
├── load_fixture(crate_dir: &str, name: &str) -> serde_json::Value  (fn, re-export from fixture)
│
├── load_fixture_as::<T>(crate_dir: &str, name: &str) -> T  (fn, re-export from fixture)
│
└── test_utils (module, re-exports)
    ├── fn assert_field_present(body: &serde_json::Value, field: &str)
    ├── fn assert_header_present(headers: &HeaderMap, name: &str)
    ├── fn assert_status(resp: &reqwest::Response, expected: u16)
    └── fn build_test_client() -> reqwest::Client
```

---

## Key design constraints (from lib.rs rustdoc)

- All 13 per-surface DTU crates (S-6.07 through S-6.19) build on this foundation.
- Gated behind `#[cfg(any(test, feature = "dtu"))]` — MUST NEVER link into a production binary.
- All randomness in DTU stubs MUST flow through `seeded_rng()` to guarantee reproducible test runs.
