# TD-S305-CAP-COMPILE-FAIL-001: Cap-bypass compile-fail test for CursorEntry

**Filed:** 2026-05-07
**Story:** S-3.05
**Adversary pass:** LOCAL pass-1 observation O-1
**Severity:** tech_debt
**Status:** open / deferred

## Finding

`CursorEntry` fields were narrowed to `pub(crate)` (I-3 fix), preventing external crates
from constructing the struct directly without going through `QueryCursorRegistry`. However,
no compile-fail test enforces this boundary.

## Why Deferred

Compile-fail tests (trybuild / ui-test pattern) require additional test infrastructure not
yet wired in the prism-query test harness. The field visibility fix already enforces the
boundary at the type-system level.

## Resolution Path

Add a `tests/external/perimeter-violation/` compile-fail case that attempts to construct
`CursorEntry { ... }` from outside `prism-query` and verifies it produces a
`field ... is private` compile error.

## Related

- I-3 fix: `CursorEntry` fields changed to `pub(crate)` in `cursor.rs`
- Existing compile-fail harness: `tests/external/perimeter-violation/`
