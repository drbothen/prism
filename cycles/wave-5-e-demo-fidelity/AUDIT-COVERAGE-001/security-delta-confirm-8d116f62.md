# Security Delta-Confirm — `0fbef7db..8d116f62`
**Story:** AUDIT-COVERAGE-001 · **PR:** #226 · **Date:** 2026-07-18 · **Reviewer:** security-reviewer

---

## Delta Summary

Change: BASE_URL > PORT > default precedence fix in `scripts/t13-preflight-audit.py`.
Commit: `8d116f62`. Prior commit: `0fbef7db`. Delta is 5 sites in a single Python script.

---

## Assessment

1. **SEC-001 (credential redaction in audit output) — CLOSED, HOLDING.** No new print sites that could expose credential values. Print statements in the delta show URLs (not tokens); SEC-001 closure undisturbed.

2. **SEC-002 (audit script env var access scope) — CLOSED, HOLDING.** Delta reads `PRISM_*_BASE_URL` environment variables (same trust boundary as the prior `_PORT` reads). No new env var access patterns; no shell expansion of values; no subprocess invocation with env pass-through. SEC-002 closure undisturbed.

3. **BASE_URL passthrough — NO-NEW-RISK.** The BASE_URL values are passed verbatim to the `httpx` call as the URL target — they are not passed to shell, not passed to subprocess.Popen, not logged. The values originate from the operator's own shell environment (set by `demo-run.sh` output). No trust boundary crossing; no injection surface. `execve` env-dict usage (direct httpx) is the clean path — assessed no-new-risk.

4. **Print site audit (new delta sites) — CLEAN.** Five `BASE_URL` construction sites produce URL strings only; no token or secret concatenation at any site.

5. **No new dependencies introduced.** No new `import` statements. No dependency version changes.

---

## VERDICT: APPROVE

CLEAN (strict): **yes** — zero findings at any severity level.

SEC-001 and SEC-002 closures undisturbed. BASE_URL passthrough is no-new-risk. Delta is minimal and security-orthogonal.
