# TD-S304-TMPFILE-PID-001: Use mkstemp/tempfile crate for temp files instead of PID nonce

**Story:** S-3.04
**Status:** open
**Severity:** tech_debt
**Filing:** S-3.04 local adversary pass-1 OBS-002

## Description

`AliasStore::write_entries_to_file` generates a temp filename using the system time
as a nonce:

```rust
let tmp_path = parent.join(format!(
    "aliases.toml.tmp.{}",
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
));
```

Using nanosecond timestamp as a nonce is fragile:
- On systems with low-resolution clocks (some VMs), two writes within the same
  nanosecond window would generate the same filename.
- `unwrap_or(0)` maps clock errors to a predictable nonce of 0.

Similarly, test files use `std::process::id()` which is unique per process but not
per thread or per test case within a single test binary run.

## Required Fix

Option A (production code): use the `tempfile` crate's `NamedTempFile` for atomic
writes, which uses OS `mkstemp` for guaranteed-unique paths.

Option B (minimal): combine timestamp with thread ID and a counter:
```rust
use std::sync::atomic::{AtomicU64, Ordering};
static TMP_COUNTER: AtomicU64 = AtomicU64::new(0);
let nonce = TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
let tmp_path = parent.join(format!("aliases.toml.tmp.{nonce}"));
```

Option C (test code): use `tempfile::tempdir()` for test isolation, which is
cleaned up automatically on drop.

## References

- S-3.04 local adversary pass-1 OBS-002
- `crates/prism-query/src/alias_store.rs::write_entries_to_file`
- `crates/prism-query/src/tests/alias_tests.rs` (per-test paths)
