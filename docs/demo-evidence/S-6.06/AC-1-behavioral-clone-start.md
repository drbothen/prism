# AC-1: BehavioralClone::start() binds and bound_addr() is reachable

## Acceptance Criterion

Given `BehavioralClone::start()` is called on a per-surface DTU instance
that depends on `prism-dtu-common`, When the server binds, Then `bound_addr()` returns
a valid `SocketAddr` and the server is reachable at `http://127.0.0.1:{port}`.

## Test

- File: `crates/prism-dtu-common/tests/ac_1_behavioral_clone_start.rs`
- Function: `ac_1_behavioral_clone_start_binds_and_bound_addr_is_reachable`
- Test command: `cargo test --features prism-dtu-common/dtu --test ac_1_behavioral_clone_start`

## Implementation (excerpt)

File: `crates/prism-dtu-common/src/clone.rs`

```rust
#[async_trait]
pub trait BehavioralClone: Send + Sync + 'static {
    /// Start the stub server and bind to a local port.
    async fn start(&mut self) -> anyhow::Result<()>;

    /// Reset all captured state (requests, counters, injected errors) to initial values.
    async fn reset(&self) -> anyhow::Result<()>;

    /// Reconfigure the stub at runtime (e.g. change failure mode, latency).
    async fn configure(&self, config: serde_json::Value) -> anyhow::Result<()>;

    /// Return the `SocketAddr` the stub is actually bound to.
    fn bound_addr(&self) -> SocketAddr;

    /// Convenience: HTTP base URL derived from `bound_addr`.
    fn base_url(&self) -> String {
        format!("http://{}", self.bound_addr())
    }
}
```

## Test output

```
running 1 test
test ac_1_behavioral_clone_start_binds_and_bound_addr_is_reachable ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

## Mapping

The `BehavioralClone` trait enforces that every DTU implementation must call `start()` before `bound_addr()` is meaningful; the test constructs a minimal `TestClone`, calls `start()`, verifies the returned port is non-zero on loopback, and confirms the server responds 200 to a GET at `base_url()/health`.
