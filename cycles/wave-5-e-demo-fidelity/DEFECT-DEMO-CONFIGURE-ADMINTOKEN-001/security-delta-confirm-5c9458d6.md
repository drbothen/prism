---
document_type: security-delta-confirm
story_id: DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001
pr_number: 225
reviewed_head: "5c9458d6"
status: "APPROVE"
producer: security-reviewer
timestamp: "2026-07-18T00:00:00Z"
---

# SECURITY DELTA-CONFIRM — PR #225 (5c9458d6)
fix/DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001

## Verdict: APPROVE

Zero CRITICAL or IMPORTANT findings. Both prior suggestions closed. One new LOW observation.

## Prior Finding Dispositions

### SEC-001 (CWE-117 Log Injection via Unvalidated Clone Name) — CLOSED
**Placement.** `validate_clone_name(&clone_name)?;` is the literal first statement in `cmd_configure` (line 642). It executes before `resolve_configure_url` (first file I/O, 693–704), `resolve_configure_token` (second file I/O, 715–719), `tracing::debug!(clone = %clone_name, ...)` (first log emission, 720–724), and the HTTP client build/POST (728–740). No path reaches any log sink, file read, or network operation with an unvalidated `clone_name`. Early-return via `?` on validation failure.
**Charset allowlist.** `c.is_ascii_alphanumeric() || c == '-' || c == '_'` accepts exactly `[a-zA-Z0-9_-]`; all non-ASCII (confusables, Cyrillic lookalikes) fail. Correct.
**Error message sanitization.** `validate_clone_name` embeds `sanitize_clone_name(name)` output (disallowed chars → `?`) in the `anyhow::bail!` message — attacker bytes cannot survive into the rejection error string.
**Bypass vectors:** CR, LF, null byte, ANSI escape, slash/path traversal, space, Unicode confusables, multi-byte sequences — all REJECTED. Very long names of valid chars pass validation (see NEW-001).
**SEC-001: CLOSED.**

### SEC-002 (Missing Timeout Rationale Comment) — CLOSED
Comment at 726–728: "SEC-002: 10s timeout — local loopback demo server (not a production sensor API client). Production sensor clients use 30s per CLAUDE.md §HTTP client timeout." Accurate — cmd_configure POSTs to 127.0.0.1 only. **SEC-002: CLOSED.**

## New Finding

### NEW-001: Empty string passes `validate_clone_name` (vacuous truth)
- **Severity:** LOW · **CWE:** CWE-20 (Improper Input Validation)
- **Evidence:** `"".chars().all(...)` evaluates true (vacuous universality); `validate_clone_name("")` returns `Ok(())`.
- **Exploitability:** None for CWE-117 — empty string carries no injection chars; downstream `resolve_configure_url("")` fails noisily. Local CLI attack surface only. Functional correctness gap, not a security issue.
- **Mitigation:** `if name.is_empty() { anyhow::bail!("configure: clone name must not be empty"); }` first line of `validate_clone_name`. Non-blocking for the security gate.

**[ORCHESTRATOR NOTE: closed same-session by implementer @dac830d1 + test_validate_clone_name_rejects_empty; codified in story v0.21 EC-008.]**

## New Issues Sweep (+136 lines)
unwrap()/expect() in production paths: none (test-only). AD-017: token never logged; `token_present = true` flag only. New injection surfaces: none beyond gated clone_name. Error handling: `?` + `anyhow::bail!` throughout. SAP-1: no event_type in delta.

## Test Quality
Both tests call the production `validate_clone_name` directly; neither tautological. Rejects-test verifies sanitization actually runs (`?` echo assertion). Accepts-test guards against over-restriction. Gap: no empty-string case (NEW-001) [closed @dac830d1].

## Summary
| Finding | Severity | Status |
|---|---|---|
| SEC-001 CWE-117 | prior SUGGESTION | CLOSED |
| SEC-002 timeout rationale | prior SUGGESTION | CLOSED |
| NEW-001 empty string vacuous truth | LOW | Open at review time (non-blocking) [closed @dac830d1 same-session] |

CLEAN (PR-merge): yes — zero CRITICAL/IMPORTANT/MEDIUM.
CLEAN (strict): no — NEW-001 (LOW) present at review time.
