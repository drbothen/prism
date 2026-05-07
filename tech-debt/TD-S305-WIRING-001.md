# TD-S305-WIRING-001: QueryEngine cursor_registry and cache fields are stub-wired but execute() not yet plumbed

## Severity
MEDIUM — functional gap; cursor cleanup runs but execute() still returns todo!()

## Discovered
S-3.05 pass-1 review finding CR-003

## Description
`QueryEngine` now owns `cursor_registry`, `cache`, and `cleanup_handle` fields
(wired in CR-003 fix). The cursor cleanup background task is started in `new()`
and cancelled on `Drop`. However, `QueryEngine::execute()` and
`execute_scheduled()` still return `todo!()` (S-3.02 scope).

Until S-3.02 materializes execute(), neither the cursor registry nor the cache
are reachable from query call paths — they exist structurally but are functionally
dead code. When execute() is implemented:

1. Cache lookup: `self.cache.get(&cache_key)` before fan-out
2. Cache store: `self.cache.put(cache_key, rows)?` after sensor API response
3. Cursor allocation: `self.cursor_registry.lock()?.create(...)` for multi-page results
4. CacheInvalidator wiring: `CacheInvalidator::new(Arc::clone(&self.cache))` in
   write tool handlers

## Affected Code
- `crates/prism-query/src/engine.rs` — `QueryEngine::execute()` stub
- `crates/prism-query/src/cache.rs` — `QueryCache` methods ready but uncalled
- `crates/prism-query/src/cursor.rs` — `QueryCursorRegistry` ready but uncalled

## Resolution Path
S-3.02 execute() implementation must wire cache lookup/store and cursor allocation.
Add regression tests to confirm cache hit path is exercised.

## References
- CR-003 (S-3.05 pass-1 review)
- BC-2.07.003 (cache TTL)
- BC-2.07.001/002 (pagination cursor lifecycle)
- BC-2.11.005 (materialization pipeline)
