# TD-S304-VISIBILITY-001: Re-evaluate pub vs pub(crate) for alias_* modules

**Story:** S-3.04
**Status:** open
**Severity:** tech_debt
**Filing:** pass-1 review CR-020

## Description

The alias system modules (`alias_resolver`, `alias_store`, `alias_tools`, `alias_types`,
`alias_capability`) are declared `pub mod` in `lib.rs`. Most of these should be
`pub(crate)` unless they are part of the intentional public API surface.

## Current Behavior

All alias modules are publicly exported. External crates depending on `prism-query` can
call internal functions directly, bypassing validation layers.

## Required Fix

1. Audit each module's public symbols against the intended API surface (documented in
   `architecture/api-surface.md`).
2. Change modules not in the public API to `pub(crate)`.
3. Mark any symbols that must remain public (MCP tool inputs/outputs) explicitly.
4. Add a `tests/external/` compile-fail test verifying internal symbols are not reachable.

## Notes

`create_or_update` was already narrowed to `pub(crate)` in pass-1 as part of CR-018.
This TD covers the broader module-level visibility audit.

## References

- CR-020 (pass-1 review finding)
- architecture/api-surface.md
