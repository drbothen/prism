# TD-S305-FORMAT-MSG-001: Cosmetic format-string improvement in expect messages (O-6)

**Filed:** 2026-05-07
**Story:** S-3.05
**Adversary pass:** LOCAL pass-1 observation O-6
**Severity:** tech_debt
**Status:** open / deferred

## Finding

Several `expect()` messages in `cache.rs` and `cursor.rs` use static strings where
a format string with context values (key prefix, entry count) would improve
diagnostics in the rare event of a panic in test/debug builds.

Example:
```rust
// current
.expect("put must succeed");
// improved
.expect(&format!("put key={:?} must succeed", key));
```

## Why Deferred

Cosmetic quality improvement only — no correctness impact. Existing messages are
sufficient for identifying failures. Low priority.

## Resolution Path

During any S-3.05 maintenance sweep: update `expect()` messages in lib tests to
include key prefix context. Prefer `unwrap_or_else(|e| panic!(...))` pattern to
avoid allocating format strings on the hot path.
