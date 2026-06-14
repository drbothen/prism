# PR-LEVEL Adversary Pass 1 — S-DEMO-MULTI-TENANT-DTU-001

**Date:** 2026-06-14
**Story:** S-DEMO-MULTI-TENANT-DTU-001 — Per-DTU-Instance Multi-Address Binding for Multi-Tenant Overlay Testing
**PR:** #187 (feature/S-DEMO-MULTI-TENANT-DTU-001 → develop)
**PR HEAD at pass time:** 846c21dc (on remote)
**develop HEAD:** f7400f83
**Story version at pass time:** v1.10 (pre-fix; OBS-2 + F-PR2-MED-001 closures land in this burst → v1.12)
**BC-2.06.017 version at pass time:** v1.7 (pre-fix; F-PR2-MED-001 lands → v1.8)
**BC-5.39.001 streak entering this pass:** 0/3
**Adversary context:** PR-LEVEL (fresh context, information asymmetry; different perimeter from LOCAL)

---

## Part A — Post-LOCAL-Convergence Commit Verification

Three commits landed on the feature branch after LOCAL 3-CLEAN convergence (passes 9/10/11, D-1153). Adversary verified each commit is sound before examining PR diff.

### Commit 96fce1ad — tls-removal: no-default-features E0053 fix

- **Context:** LOCAL convergence reached at story v1.10/BC v1.7; devops-engineer pushed feature branch. CI surfaced E0053 compile error: `prism-dtu-harness` with `--no-default-features` triggered a `tls` feature conditional referencing a type removed in the no-tls path.
- **Adversary assessment:** Fix correct — removed `#[cfg(feature="tls")]`-gated code block that referenced a type only present when tls feature is enabled. No behavioral change; `just check` GREEN. No story/BC amendment required (CI-path-only fix; no behavioral contract surface affected).
- **Verdict:** SOUND.

### Commit 74d0bd4c — SEC fix: validate_harness_key input validation (closes SEC-001 + SEC-002)

- **Context:** Security reviewer found SEC-001 (TOML injection CWE-93/74) and SEC-002 (path traversal CWE-22) BLOCKING in initial PR security review. The `validate_harness_key` function accepted arbitrary string keys without sanitization; crafted keys could inject TOML control characters or traverse paths.
- **Fix details:** Added input validation: (a) allowlist check — key must match `^[a-zA-Z0-9_-]+$` (alphanumeric + underscore + hyphen only); (b) path-component validation — reject any key containing `/`, `\`, or `..` sequences; (c) 6 load-bearing unit tests added (`test_validate_harness_key_*`): valid key, empty key, TOML injection character, path separator, double-dot traversal, whitespace. Returns `HarnessError::InvalidKey(String)` on rejection.
- **Adversary assessment:** Fix correct. Injection and traversal surfaces closed. Tests are load-bearing (each exercises a distinct rejection path). SEC-001 CLOSED. SEC-002 CLOSED.
- **Verdict:** SOUND.

### Commit 89764cda — brotli pins: fuzz + perimeter-violation crates (CI E0277 fix)

- **Context:** Environmental rustc-1.95.0 E0277 surfaced in CI (not story-specific — repo-wide brotli version conflict). `brotli` 3.x pulled transitively; `brotli-decompressor` 4.x API incompatible with `brotli` 3.x in CI rustc-1.95.0. Fix: explicit pins in `fuzz/` and `tests/external/perimeter-violation/` `Cargo.toml` files.
- **Pin versions:** `brotli = "8.0.2"`, `brotli-decompressor = "5.0.0"`, `alloc-stdlib = "0.2.2"`.
- **Adversary assessment:** Environmental pin fix; not story-specific code. Correct approach to repo-wide transitive dep conflict. No behavioral change. SOUND.
- **Verdict:** SOUND.

**Part A summary:** All 3 post-LOCAL-convergence commits verified SOUND. PR diff review proceeds.

---

## Part B — PR Diff Adversarial Review

### Findings

#### SEC-006 — LOW — CWE-209 Error Message Input Value Disclosure

**Severity:** LOW
**CWE:** CWE-209 (Generation of Error Message Containing Sensitive Information)
**Location:** `crates/prism-dtu-harness/src/lib.rs` — `validate_harness_key` error path
**Description:** When `validate_harness_key` rejects a key, the returned `HarnessError::InvalidKey(String)` error carries the full raw input value. If the raw input contains sensitive data (e.g., a credential value accidentally passed as a key), the error message discloses it in logs and in any MCP tool response that surfaces the error text.
**Impact:** LOW — harness keys are expected to be structural identifiers (org_slug/sensor_id composites), not credentials. The scenario where a credential transits this surface is unlikely. However, CWE-209 is a standing project rule (CLAUDE.md §Forbidden patterns `unwrap/expect on Result` → error taxonomy; AD-017 credential opacity). The error message should redact or truncate the input value.
**Fix direction:** Redact input: display only the first 32 chars (or truncate to `[REDACTED]` if length > 32), or emit only the character-class description of the violation rather than the literal input.
**Status:** CLOSED — commit 846c21dc applied redaction (see PR-LEVEL Pass 2 Part A verification).

#### OBS-1 — PROCESS-GAP — CI Status Unverifiable at PR-LEVEL Review Time

**Severity:** PROCESS-GAP (not a code finding; orchestrator-owned)
**Description:** Adversary cannot fetch live CI run status at PR-LEVEL review time. GitHub Actions runs are external; adversary is restricted to diff analysis. CI verification is orchestrator's gate, not adversary's.
**Status:** CLOSED — orchestrator documents CI status separately. Known limitation per DRIFT-D904-001 (adversary diff-tooling limitation). Non-blocking.

#### OBS-2 — LOW — Timeout Wording Mismatch: test-client 10s vs story 30s

**Severity:** LOW
**Location:** Story S-DEMO-MULTI-TENANT-DTU-001 v1.10 §Architecture Compliance Rules + test file comment vs BC-2.06.017 v1.7 note
**Description:** Story §Architecture Compliance Rules states "HTTP timeout: 30s per CLAUDE.md §Conventions (reqwest::Client with `.timeout(Duration::from_secs(30))`)". The test client in `crates/prism-dtu-harness/tests/multi_instance_integration.rs` used `.timeout(Duration::from_secs(10))` for the test HTTP client. Story claims "30s timeout" but test uses 10s. This is a test-infra-only discrepancy (tests use shorter timeouts for speed); however, the story's Architecture Compliance wording was ambiguous — it should distinguish production client timeout from test-client timeout.
**Fix direction:** Story: add a parenthetical to the Architecture Compliance row clarifying "30s for production client; test clients may use shorter timeouts (10s) for test responsiveness."
**Status:** CLOSED — story-writer applied wording clarification in story v1.10→v1.11.

---

## Verdicts

| Criterion | Result |
|-----------|--------|
| CLEAN (strict) — ZERO findings ANY severity | NO (SEC-006 LOW + OBS-2 LOW + OBS-1 process-gap) |
| CLEAN (PR-merge) — ZERO findings CRIT+HIGH+MED | YES |
| BC-5.39.001 streak advance | NO (CLEAN strict = NO) |

**CLEAN(strict):** NO
**CLEAN(PR-merge):** YES
**Streak after pass 1:** 0/3

---

## Security Review Outcomes (Recorded Here for Completeness)

| Finding | Severity | CWE | Status |
|---------|----------|-----|--------|
| SEC-001 — TOML injection via harness key | BLOCKING | CWE-93/74 | CLOSED (commit 74d0bd4c; 6 load-bearing tests) |
| SEC-002 — path traversal via harness key | BLOCKING | CWE-22 | CLOSED (commit 74d0bd4c; path-component validation) |
| SEC-003 — test fixture cleanup race | LOW | test-infra | ACCEPTED (test-infra; non-production) |
| SEC-004 — temp-dir collision in parallel test runs | LOW | test-infra | ACCEPTED (test-infra; non-production) |
| SEC-005 — hardcoded test port range | LOW | test-infra | ACCEPTED (test-infra; ephemeral ports; no prod risk) |
| SEC-006 — error message discloses input value | LOW | CWE-209 | CLOSED (commit 846c21dc redaction) |

**Security re-review outcome:** APPROVE (all BLOCKING findings closed; LOW findings accepted per SEC-003/004/005 test-infra rationale + SEC-006 closed).

---

## Pass 1 Closure

All 3 PR-LEVEL Pass 1 findings accounted for. SEC-006 → commit 846c21dc. OBS-1 → orchestrator-owned (non-blocking). OBS-2 → story v1.11 wording fix. CLEAN(strict)=NO. Pass 2 next.
