---
document_type: security-review
cycle: wave-0-plugin-prereqs
story_id: S-DTU-CYBERINT-AUTH-FIDELITY-001
review_number: 1
step: 5
date: 2026-05-30
feature_head: "d09bdfa9"
pr_number: 164
base_branch: develop
base_head: "e898c3c9"
verdict: "may proceed"
crit_count: 0
important_count: 0
suggestion_count: 2
status: "SEC-001 CLOSED by FB-PR4 implementer 8f6f4e91; SEC-002 CLOSED by FB-PR4 implementer 8f6f4e91"
---

# PR Security Review 1 — S-DTU-CYBERINT-AUTH-FIDELITY-001 PR #164

## Header

- **Review:** Step 5 Security Review (first occurrence)
- **Date:** 2026-05-30
- **Feature HEAD at review:** d09bdfa9 (FB-PR3: 9 anti-volatile-pin fixes; story v1.7 e9827961; at 3-CLEAN convergence streak 3/3)
- **PR:** #164 (feature/S-DTU-CYBERINT-AUTH-FIDELITY-001 → develop)
- **Base develop HEAD:** e898c3c9
- **Reviewer:** security-reviewer (Step 5 protocol)
- **Verdict:** May proceed (zero CRIT/IMPORTANT findings; 2 SUGGESTION findings)

> **Note:** Per project protocol, security findings of any non-zero severity trigger fix-burst dispatch under user directive "No pragmatic convergence. Fix all issues before build." Both SUGGESTIONs were dispatched to FB-PR4 implementer despite the SUGGESTION severity — production-grade default applies.

## Findings

### SEC-001 [SUGGESTION] — E-AUTH-006 Header Injection Path: CTL/CRLF Character Rejection Gap

**CWE:** CWE-93 (Improper Neutralization of CRLF Sequences), CWE-113 (Improper Neutralization of CRLF Sequences in HTTP Headers)
**Severity:** SUGGESTION (not IMPORTANT because the `access_token` value originates from the operator-controlled credential store, not user input; blast radius is operator-misconfigured credentials, not end-user injection)
**Status:** CLOSED by FB-PR4 implementer commit 8f6f4e91

**Description:** `PipelineExecutor::build_request` constructs a `Cookie: access_token={token}` header by string interpolation. If the `access_token` value (retrieved from the credential store) contained `\r`, `\n`, or `:` characters, this could result in HTTP header injection or response splitting. The `StaticCookieAuthProvider::acquire_token` path did not validate the token value before returning it, and `build_request` did not sanitize before header construction.

**Attack surface:** Operator-misconfigured or compromised credential store supplying a malicious token value. Not end-user input. Risk is low in practice but the validation gap exists at the code level.

**Resolution (FB-PR4):** Implementer commit 8f6f4e91 added validation in `StaticCookieAuthProvider::acquire_token` that rejects tokens containing `\r`, `\n`, or `:` characters before returning the `AuthToken`. Returns `E-AUTH-006` (allowlist-reject) on invalid characters. Load-bearing test `test_static_cookie_auth_rejects_crlf_in_token` added. pr-security-review-2.md confirms closure.

---

### SEC-002 [SUGGESTION] — DTU access_token Allowlist Unbounded (CWE-400)

**CWE:** CWE-400 (Uncontrolled Resource Consumption)
**Severity:** SUGGESTION (not IMPORTANT because the allowlist is populated at DTU startup from a static config, not at runtime from API requests; an attacker cannot trigger allowlist growth without restarting the DTU process with adversary-controlled config)
**Status:** CLOSED by FB-PR4 implementer commit 8f6f4e91

**Description:** `CyberintState.access_tokens` (a `HashSet<String>`) is populated at DTU startup with no explicit capacity bound. In a long-running DTU process where the config is reloaded (future feature, not yet implemented), unbounded token accumulation could occur. Even without config reload, having no capacity assertion is a CWE-400 code-level gap.

**Resolution (FB-PR4):** Implementer commit 8f6f4e91 added a maximum capacity constant `MAX_ACCESS_TOKENS` (default: 1000) and a capacity check at startup that returns `E-DTU-INIT-001` if the configured token count exceeds the bound. Prevents unbounded accumulation if config-reload is added in the future. Load-bearing test added. pr-security-review-2.md confirms closure.

> **Sibling-sweep note:** FB-PR4 addressed SEC-002 in `prism-dtu-cyberint` but missed the `prism-dtu-harness/src/clones/cyberint.rs` sibling. This gap was caught by Pass 7 (F-P7-HIGH-001) and closed by FB-PR5 implementer 44aa7fed.

---

## Threat Model Summary

| Surface | Risk Level | Notes |
|---------|-----------|-------|
| `StaticCookieAuthProvider::acquire_token` | LOW | Returns from in-memory credential store (no HTTP). AD-017: no credential in AI context. |
| `PipelineExecutor::build_request` Cookie injection | LOW (mitigated) | CTL/CRLF rejection added in FB-PR4. |
| `CyberintState` access_token allowlist | LOW (mitigated) | Capacity bound added in FB-PR4. |
| `OrgSlug::new_unchecked` | NONE | Not used in production paths; `#[cfg(feature = "test-helpers")]` gate confirmed. |
| Session fixation | NONE | No session state; `access_token` is static key per request. |
| Token leakage in logs | NONE | `AuthToken` uses redacted `Debug` impl; no token values in tracing output. |

## Verdict

**May proceed.** Zero CRIT/IMPORTANT findings. 2 SUGGESTIONs dispatched to FB-PR4 per production-grade default. Both closed and re-verified in pr-security-review-2.md.
