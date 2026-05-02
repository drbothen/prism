# Security Findings — W3-FIX-SEC-003

## Review Summary

| Severity | Count | Blocking |
|----------|-------|----------|
| CRITICAL | 0 | — |
| HIGH | 0 | — |
| MEDIUM | 1 | No (acknowledged, tracked as tech-debt) |
| LOW | 0 | — |

**Verdict:** No CRITICAL/HIGH findings. PR may proceed to review convergence.

---

## Finding SEC-003-R1 (MEDIUM): Pre-Join Path Traversal Check Bypassed for Non-Existent Targets

- **File:** `crates/prism-customer-config/src/validator.rs:554`
- **Severity:** Medium
- **Category:** path_traversal / CWE-22
- **Confidence:** 8.5/10

### Description

The integration point in `validate_dtu_block` gates the call to `validate_spec_path`
behind a `resolved.exists()` check (line 554). When a customer TOML contains a traversal
path like `spec = "../../../../etc/nonexistent_file"` where the target does not yet
exist, the flow is:

1. `parent.join("../../../../etc/nonexistent_file")` produces an out-of-boundary path
2. `resolved.exists()` returns `false`
3. `validate_spec_path` is **never called** — pre-join `..` check is bypassed
4. `SpecFileNotFound` (E-CFG-015) is emitted instead of `SpecPathTraversal` (E-CFG-018)

### Impact

- For currently-non-existent traversal targets: path structure is not rejected; only
  a benign "file not found" error is emitted. Audit trail does not capture the attempted
  path escape.
- TOCTOU-adjacent: if the target file is created later, a subsequent reload or restart
  passes both the existence check and (potentially) the boundary check for a path that
  should have been permanently rejected.
- No immediate unauthorized file read is possible in the current startup flow because the
  file must exist for `validate_spec_path` to proceed to the post-join boundary check.
  However, the story spec requires the pre-join `..` check to fire unconditionally
  (no filesystem I/O dependency), and the current integration violates that requirement.

### Recommended Fix

Move the I/O-free pre-join checks outside the `resolved.exists()` gate:

```rust
Some(spec_path) => {
    // Pre-join checks: fire unconditionally, no filesystem I/O.
    use std::path::Component;
    let spec_as_path = Path::new(spec_path.as_str());
    if spec_as_path.is_absolute() {
        errors.push(ConfigError::SpecPathTraversal {
            file: config_path.to_path_buf(),
            spec_path: spec_path.clone(),
            message: "absolute paths are not permitted".to_string(),
        });
        return;
    }
    for component in spec_as_path.components() {
        if matches!(component, Component::ParentDir) {
            errors.push(ConfigError::SpecPathTraversal {
                file: config_path.to_path_buf(),
                spec_path: spec_path.clone(),
                message: "parent directory traversal (`..`) is not permitted".to_string(),
            });
            return;
        }
    }
    // Post-join existence + canonicalize boundary checks (require file to exist).
    let parent = config_path.parent().unwrap_or(Path::new("."));
    let resolved = parent.join(spec_as_path);
    if resolved.exists() {
        match validate_spec_path(config_path, spec_path) {
            Ok(_) => {}
            Err(e) => { errors.push(e); return; }
        }
    } else {
        errors.push(ConfigError::SpecFileNotFound { ... });
    }
}
```

Alternatively, refactor `validate_spec_path` to return a typed `PreJoinRejection` vs
`PostJoinRejection` and call it unconditionally, with the caller handling
`canonicalize` failure as `SpecFileNotFound`.

### Decision

**Acknowledged as tech-debt.** The immediate CWE-22 exploitability is mitigated by the
fact that the file must exist for a read to occur. No CRITICAL/HIGH vector exists in the
current implementation. The gap (missing pre-join rejection audit for non-existent targets)
will be addressed in a follow-up hardening story.

### Status: OPEN (non-blocking) — tracked as W3-FIX-SEC-003-FOLLOWUP

---

## Gate Finding Disposition

| Gate Finding | Status |
|-------------|--------|
| SEC-003 (HIGH, CWE-22) — dotdot traversal with existing target | RESOLVED |
| SEC-003 (HIGH, CWE-22) — absolute path with existing target | RESOLVED |
| SEC-003 (HIGH, CWE-22) — symlink escape | RESOLVED |
| SEC-003-R1 (MEDIUM) — dotdot bypass for non-existent targets | OPEN (non-blocking) |
