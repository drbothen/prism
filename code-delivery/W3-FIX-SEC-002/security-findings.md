# Security Review Findings — W3-FIX-SEC-002

**Reviewer:** security-review skill (fresh-context sub-agent)
**Date:** 2026-05-01
**PR:** #119 — fix(W3-FIX-SEC-002): /dtu/reset admin token auth on 4 DTU clones
**Branch head reviewed:** 5f769db0
**Scope:** POST /dtu/reset gate on prism-dtu-{claroty,crowdstrike,armis,slack}

---

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 0 |
| MEDIUM | 0 |
| LOW | 0 |

**Total findings: 0. SEC-NEW-001 (CWE-306/A07) CLOSED.**

---

## Analysis

### Files Reviewed

- `crates/prism-dtu-armis/src/routes/dtu.rs` — `post_reset` handler
- `crates/prism-dtu-claroty/src/routes/devices.rs` — `dtu_reset` handler
- `crates/prism-dtu-crowdstrike/src/routes/mod.rs` — `dtu_reset` handler
- `crates/prism-dtu-slack/src/routes/dtu.rs` — `post_reset` handler
- `crates/prism-dtu-armis/tests/dtu_reset_auth.rs` — new test file (×3 tests)
- `crates/prism-dtu-claroty/tests/dtu_reset_auth.rs` — new test file (×3 tests)
- `crates/prism-dtu-crowdstrike/tests/dtu_reset_auth.rs` — new test file (×3 tests)
- `crates/prism-dtu-slack/tests/dtu_reset_auth.rs` — new test file (×3 tests)
- 7 backwards-compat test files (header additions only)

### Security Properties Verified

1. **No timing-safe comparison concern** — UUID v4 (36-char) comparison on loopback-only
   endpoint; theoretical timing attack is not in scope per false-positive rules.

2. **No whitespace trim (EC-002)** — `as_str()` comparison is exact byte match. Correct.

3. **Missing header → 401 (EC-001)** — `headers.get("x-admin-token")` returns `None`
   when absent; `None != Some(...)` is `true` → 401. Correct.

4. **Check-then-act ordering** — `state.reset()` only called after guard passes in all
   four handlers. Correct.

5. **No new injection surface** — header value is compared only; never parsed,
   deserialized, or logged.

6. **No new Cargo dependencies** — `HeaderMap` already in scope from `dtu_configure`.

7. **Token generation** — UUID v4 at clone startup, per-instance, not shared, not
   returned in any response body.

### Cosmetic Note (Not a Security Finding)

`dtu_configure` returns error body `"missing or invalid X-Admin-Token"` (includes header
name); `dtu_reset` returns `"missing or invalid admin token"`. Minor inconsistency in
error message wording. Not a security issue — both return 401 with a JSON error key.

---

## Verdict

**APPROVE** — No security findings. SEC-NEW-001 (HIGH, CWE-306/A07) is closed by this
change. Implementation is semantically identical to the existing `dtu_configure` gate.
