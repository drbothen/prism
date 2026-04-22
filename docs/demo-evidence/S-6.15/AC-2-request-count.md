# AC-2: Request Counter Tracks Per-CVE Lookups for Cache-Hit Assertion

## AC Statement

Given the same CVE is requested twice, then `GET /dtu/request-count/CVE-2024-0001`
returns `count: 2` (if Prism does not cache) or `count: 1` (if Prism hits cache after
first request). The DTU-unit test asserts `count == 2` to verify the counter increments
correctly; the cache-hit assertion (`count == 1`) is validated in the Prism integration
suite where caching is controlled.

## Test File

`crates/prism-dtu-nvd/tests/ac_2_request_count_cache_hit.rs`

## Test Function

`ac_2_request_count_increments_per_cve_lookup`

## Implementation Excerpt

`crates/prism-dtu-nvd/src/state.rs` — `lookup_and_count`:

```rust
pub fn lookup_and_count(&self, cve_id: &str) -> Option<CveRecord> {
    let normalized = cve_id.to_uppercase();
    let record = self.cve_registry.get(&normalized).cloned();

    let mut counters = self.request_counters.lock().expect("poisoned");
    *counters.entry(normalized).or_insert(0) += 1;

    record
}

pub fn request_count_for(&self, cve_id: &str) -> u32 {
    let normalized = cve_id.to_uppercase();
    let counters = self.request_counters.lock().expect("poisoned");
    *counters.get(&normalized).unwrap_or(&0)
}
```

## Test Run Output

```
Running tests/ac_2_request_count_cache_hit.rs

running 1 test
test ac_2_request_count_increments_per_cve_lookup ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

## Mapping

AC-2 is satisfied: `lookup_and_count` atomically increments the per-CVE counter on every
request; `GET /dtu/request-count/{cve_id}` returns the current count; `NvdClone::request_count_for`
exposes the same value in-process for integration test assertions.
