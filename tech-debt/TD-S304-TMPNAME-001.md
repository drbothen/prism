# TD-S304-TMPNAME-001: Use UUID suffix for aliases.toml temp file naming

**Story:** S-3.04
**Status:** open
**Severity:** tech_debt
**Filing:** pass-1 review SEC-008 / CR-021

## Description

`write_entries_to_file` constructs the temp file name as `aliases.toml.tmp.<nanos>` using
`SystemTime::now().duration_since(UNIX_EPOCH)`. On systems where multiple processes write
simultaneously, nanosecond timestamps can collide. Additionally, the `.tmp.<nanos>` suffix
is predictable and could be used in a TOCTOU attack.

## Current Behavior

```rust
let tmp_path = parent.join(format!(
    "aliases.toml.tmp.{}",
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
));
```

## Required Fix

Use a UUID v4 (or UUID v7 for time-ordered) suffix:

```rust
use uuid::Uuid;
let tmp_path = parent.join(format!("aliases.toml.tmp.{}", Uuid::new_v4()));
```

`uuid` is already a workspace dependency.

## References

- SEC-008 / CR-021 (pass-1 review findings)
- BC-2.11.008 (atomic write pattern)
