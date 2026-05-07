# TD-S304-FUZZ-001: Implement VP-037 fuzz target for alias system

**Story:** S-3.04
**Status:** open
**Severity:** tech_debt
**Filing:** pass-1 review CR-016

## Description

VP-037 requires a fuzz target to exercise alias expansion with adversarial inputs.
The docstring for the VP-037 fuzz target stub is present but the decoder is not implemented —
the target currently treats bytes as raw query strings without alias token injection.

## Current Behavior

Fuzz target `vp037_alias_no_panic` exercises `AliasResolver::expand` with arbitrary
printable ASCII but does not construct alias stores with cross-referencing entries.

## Required Fix

Implement a structured fuzz input decoder that:
1. Deserializes a `FuzzAliasInput { store_entries: Vec<(name, query)>, query: String }`
   from the fuzz corpus bytes.
2. Builds an in-memory `AliasStore` with those entries (skipping invalid ones).
3. Calls `AliasResolver::expand(&input.query, &store, ...)` and verifies no panic.

This exercises the cycle detection, depth limiting, and expansion paths with rich inputs.

## References

- VP-037 (alias system no-panic verification property)
- CR-016 (pass-1 review finding)
- BC-2.11.009 (expansion invariants)
