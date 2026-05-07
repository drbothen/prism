# TD-S305-CURSOR-ERR-CODE-001: Exhausted-vs-expired cursor error code mismatch (O-7)

**Filed:** 2026-05-07
**Story:** S-3.05
**Adversary pass:** LOCAL pass-1 observation O-7
**Severity:** tech_debt
**Status:** open / deferred

## Finding

`QueryCursorRegistry::next_page()` returns `Ok((Vec::new(), None))` when a cursor is
already exhausted (offset >= total), but `Err(E-QUERY-004)` when a cursor has expired
by TTL. The MCP caller has no way to distinguish "no more data" from "cursor expired
before being exhausted". This may cause confusing behavior if a caller retries on
empty-vec rather than checking the token.

Additionally, BC-2.07.002 does not define a separate error code for exhaustion vs expiry.
A future BC revision should clarify whether exhausted-before-expiry should return a
different sentinel (e.g., a new E-QUERY-006) or keep `Ok([], None)` semantics.

## Why Deferred

The current behavior is internally consistent: `None` token means "no continuation"
regardless of reason. The MCP layer (S-5.x) can document the `([], None)` contract.

## Resolution Path

In S-5.x MCP integration: review BC-2.07.002 with the spec author and decide whether
exhausted vs expired needs distinct error codes. If distinct codes are needed, add a new
`E-QUERY-006: cursor already exhausted` error code and update `next_page()`.

## Related

- `cursor.rs::next_page()` — exhausted branch returns `Ok((Vec::new(), None))`
- `cursor.rs::next_page()` — expired branch returns `Err(E-QUERY-004)`
- BC-2.07.002 §Forward-Only Progress
