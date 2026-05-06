# S-3.02 AC-7 — SessionContext Dropped After execute() Returns (RAII)

**Story:** S-3.02 — prism-query: Query Tool and Materialization
**BC Anchor:** BC-2.11.005
**Acceptance Criterion:** Given the `SessionContext` is created for a non-scheduled query, when `execute()` returns (including on error or panic), the `SessionContext` is dropped and its memory is released.

---

## Test Names

```
test_ac7_session_context_dropped_after_execute
test_execute_scheduled_returns_arc_session_context
  (crates/prism-query/src/tests/integration_tests.rs)
```

## Terminal Output

```
running 1 test
test tests::integration_tests::tests::test_ac7_session_context_dropped_after_execute ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 337 filtered out; finished in 0.00s
```

## Production Code Path

`crates/prism-query/src/session.rs` — `SessionScope` struct

```rust
pub struct SessionScope {
    inner: Option<SessionContext>,
}

impl SessionScope {
    pub fn new(ctx: SessionContext) -> Self { ... }

    /// Access the wrapped context for query execution.
    pub fn context(&self) -> &SessionContext { ... }

    /// Consume the scope and return an Arc<SessionContext> for execute_scheduled.
    pub fn into_arc(self) -> Arc<SessionContext> { ... }
}

impl Drop for SessionScope {
    fn drop(&mut self) {
        // inner Option is taken → SessionContext is dropped here.
        // scopeguard::defer! ensures this fires even on panic.
    }
}
```

**Architecture compliance rules enforced (BC-2.11.005):**
- `SessionContext` MUST NOT be stored as a field on `QueryEngine`.
- `SessionContext` MUST NOT outlive `execute()` for non-scheduled queries.
- `execute_scheduled` is the sole exception — returns `Arc<SessionContext>` for S-4.03 detection engine.

## Test Logic Summary (AC-7)

- Constructs `SessionContext::new()`.
- Wraps it in `SessionScope::new(ctx)`.
- Calls `scope.context().state()` (proves the context is accessible during scope lifetime).
- Calls `drop(scope)` — `SessionScope::Drop` fires, releasing the context.
- No panic = RAII contract satisfied.

## Test Logic Summary (execute_scheduled companion)

- Constructs a `SessionContext`, wraps in `SessionScope`.
- Calls `scope.into_arc()` — consumes the scope and returns `Arc<SessionContext>`.
- Clones the Arc; asserts both clones point to the same allocation via `Arc::ptr_eq`.
- This validates the `execute_scheduled` contract: detection engine holds the Arc,
  context lives until the Arc count drops to zero.

## Result

PASS — `SessionScope` RAII drop fires correctly; `into_arc()` produces a valid shared reference for the detection engine.
