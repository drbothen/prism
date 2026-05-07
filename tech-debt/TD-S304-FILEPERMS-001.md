# TD-S304-FILEPERMS-001: Set aliases.toml file permissions to 0o600

**Story:** S-3.04
**Status:** open
**Severity:** tech_debt
**Filing:** pass-1 review SEC-007

## Description

`aliases.toml` is written via temp file + rename without setting OS-level permissions.
On Unix systems, file permissions default to the process umask (typically 0o644), which
allows group and world-read access to alias definitions.

## Current Behavior

`write_entries_to_file` creates the temp file via `std::fs::File::create`, which uses
the default umask. The file may be readable by other users on the system.

## Required Fix

After creating `tmp_file` and before writing, set permissions to 0600:

```rust
#[cfg(unix)]
{
    use std::os::unix::fs::PermissionsExt;
    let metadata = tmp_file.metadata()
        .map_err(|e| PrismError::Io(format!("cannot get tmp file metadata: {e}")))?;
    let mut perms = metadata.permissions();
    perms.set_mode(0o600);
    std::fs::set_permissions(&tmp_path, perms)
        .map_err(|e| PrismError::Io(format!("cannot set file permissions: {e}")))?;
}
```

Windows: no equivalent needed (file ACLs handle this differently).

## References

- SEC-007 (pass-1 review finding)
- BC-2.11.008 (file-first write sequence)
