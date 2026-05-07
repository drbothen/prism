# TD-S303-ALIAS-PROVENANCE-001: alias field provenance tracking in field_resolution

**Story:** S-3.03
**Status:** open
**Severity:** tech_debt

## Description

`explain` labels all non-virtual fields as `resolution_method: "direct"` regardless
of whether they originated from an alias expansion. The previous code attempted to
detect alias-expanded fields by checking `alias_expansion.contains_key(field_name)`,
but that map uses alias *names* as keys (e.g. `"critical"`) while AST field names
are the *expansion targets* (e.g. `"severity"`). They never match, so the `"alias"`
branch was unreachable.

## Current Behavior

`field_resolution` entries for fields that arrived via alias expansion are labeled
`"direct"` instead of `"alias"`. The `"alias"` resolution method value is defined
in the public struct documentation (`FieldResolution::resolution_method`) but is
never produced at runtime.

## Required Fix

When `expand_query_with_aliases` performs a substitution, parse the expansion text
to extract field references (identifiers matching `[a-zA-Z_][a-zA-Z0-9_]*` that
are not keywords or literals) and return them as a `HashSet<String>` alongside the
expanded query string. In the `field_resolution` loop, check membership in that set
to assign `resolution_method: "alias"`.

Alternatively, the parser could retain origin metadata (a source-range annotation
indicating which tokens came from alias expansion), which would be more precise but
requires parser changes.

## Deferred In

Pass-11 (CR-026). Option B was chosen: the broken branch was removed and replaced
with a `"direct"` label plus this TODO. The public struct field documentation still
lists `"alias"` as a valid value to preserve the API contract for the future fix.

## References

- BC-2.11.010 postcondition `field_resolution`
- `crates/prism-query/src/explain.rs` — `field_resolution` loop (Step 9)
- `crates/prism-query/src/explain.rs` — `expand_query_with_aliases`
- CR-026 (S-3.03 pass-11 adversarial finding)
