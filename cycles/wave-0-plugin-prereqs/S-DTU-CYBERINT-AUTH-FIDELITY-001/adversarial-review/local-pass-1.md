---
document_type: adversarial-review
cycle: wave-0-plugin-prereqs
story_id: S-DTU-CYBERINT-AUTH-FIDELITY-001
pass: 1
type: LOCAL
date: 2026-05-30
feature_head: "dba6eb95"
clean_strict: false
clean_pr_merge: false
findings_count: 13
findings_by_severity:
  CRIT: 2
  HIGH: 4
  MED: 4
  LOW: 2
  OBS: 1
streak_after_pass: 0
target_streak: 3
status: "BLOCKED_ON_ARCHITECT — F-LP1-CRIT-001 required harness-clone audit before fix-burst"
architect_dispatch_triggered: true
architect_dispatch_agent_id: "a1161dc86ddae5c53"
architect_decision_committed: "12378e35 (factory-artifacts)"
---

# LOCAL Adversary Pass 1 — S-DTU-CYBERINT-AUTH-FIDELITY-001

**Feature HEAD:** `dba6eb95`
**Pass date:** 2026-05-30
**CLEAN(strict):** NO (13 findings)
**CLEAN(PR-merge):** NO (2 CRIT + 4 HIGH + 4 MED)
**Streak:** 0/3

## Findings (13 total)

| ID | Sev | Title | Route | Status |
|----|-----|-------|-------|--------|
| F-LP1-CRIT-001 | CRIT | Parallel Cyberint DTU clone at `prism-dtu-harness/src/clones/cyberint.rs` still uses `cyberint_session` + `POST /login` (ADR-031 §D1 binds ALL DTU clones). 18+ harness_tests.rs sites affected. | architect (decide pattern/scope) → implementer | RESOLVED by architect at 12378e35 — Pattern B Scope-1 expansion into current story |
| F-LP1-CRIT-002 | CRIT | `cyberint.sensor.toml` grounding comment cites stale `extract_session_token` / `cyberint_session` cookie — SoT precedence violation (CLAUDE.md §Source-of-Truth Precedence) | implementer | OPEN — fix in Pass 1 fix-burst |
| F-LP1-HIGH-001 | HIGH | `dtu.rs::post_reset` doc-comment claims `session_store` clearing semantics (field renamed to `access_token_allowlist`) | implementer | OPEN — fix in Pass 1 fix-burst |
| F-LP1-HIGH-002 | HIGH | `state.rs::reset_all` doc-comment claims "Clears session_store entirely" (now `access_token_allowlist`) | implementer | OPEN — fix in Pass 1 fix-burst |
| F-LP1-HIGH-003 | HIGH | `state.rs::reset_for` doc-comment references "both stores" / `session_store` — doc promises behavior that doesn't exist | implementer | OPEN — fix in Pass 1 fix-burst |
| F-LP1-HIGH-004 | HIGH | `alerts.rs` module-doc auth-model preamble contradictory — cites BC-3.2.003 per-session-per-org routing model superseded by ADR-031 §D3-a account-level auth | implementer | OPEN — fix in Pass 1 fix-burst |
| F-LP1-MED-001 | MED | DRY violation: auth check logic duplicated between `alerts.rs::check_auth` and `threats.rs::get_threat_intel` (line-by-line parallel) | implementer (extract shared helper) | OPEN — fix in Pass 1 fix-burst |
| F-LP1-MED-002 | MED | BC vs implementation: empty-resolved api_key returns E-AUTH-005 (resolver path) but BC-2.01.017 §Error Cases EC-017-005 mandates E-AUTH-006. Test was tuned to behavior (TD-VSDD-059 paper-fix risk). | product-owner (decide: BC amendment vs impl fix; production-grade default favors impl fix since BC is canonical source-of-truth) | OPEN — PO dispatch required |
| F-LP1-MED-003 | MED | `tests/parity/cyberint.rs` line 144 NullAuthProvider comment cites `cyberint_session` cookie (test `#[ignore]`'d but delayed-fuse defect) | implementer | OPEN — fix in Pass 1 fix-burst |
| F-LP1-MED-004 | MED | `MockCredentialResolver::resolve` uses deprecated `secrecy::SecretString::new(value)` (replaced by `::from(value)` in secrecy >= 0.10) | implementer (verify version + migrate) | OPEN — fix in Pass 1 fix-burst |
| F-LP1-LOW-001 | LOW | `routes/auth.rs` retained as empty placeholder; story Risk Mitigation §1 preferred full removal | implementer | OPEN — fix in Pass 1 fix-burst |
| F-LP1-LOW-002 | LOW | `auth_provider.rs` unit tests use `unsafe { std::env::set_var }` with brittle "nextest process isolation" SAFETY justification — refactor to MockCredentialResolver injection | implementer | OPEN — fix in Pass 1 fix-burst |
| F-LP1-OBS-001 | OBS [process-gap] | ADR-031 §D1 binding scope under-specified; missed harness-clones path → architect's prior audit (POLLER-DTU-FIDELITY-AUDIT v1.1) missed F-LP1-CRIT-001. Codification candidate. | architect (ADR-031 amendment) | RESOLVED by architect at 12378e35 — ADR-031 §D7 added; HARNESS-DTU-FIDELITY-AUDIT-2026-05-30.md authored |

## Cascade State After Pass 1

**Architect response (12378e35):**
- HARNESS-DTU-FIDELITY-AUDIT-2026-05-30.md v1.0 authored — all 4 sensor harness clones audited
- ADR-031 v1.0 → v1.1: §D7 added enumerating harness-clones scope; `related_bcs` updated
- Pattern decision: Pattern B (in-place rewrite) for Cyberint; Pattern C partial (shared fixtures) preserved for Claroty/Armis; CrowdStrike already correct
- Scope decision: Scope-1 (in-current-story) for Cyberint harness; Claroty audit_log harness gap stays with S-DEMO-CLAROTY-AUDIT-DTU-001
- POLLER-DTU-FIDELITY-AUDIT-2026-05-29.md v1.1 → v1.2 addendum note added

**F-LP1-CRIT-001 resolution path:**
Implementer must fix BOTH `crates/prism-dtu-cyberint/` AND `crates/prism-dtu-harness/src/clones/cyberint.rs` in the same commit per architect §8 scope expansion deliverables. `rg 'cyberint_session|/login' crates/prism-dtu-harness/` must return zero hits.

**F-LP1-MED-002 routing:**
Product-owner dispatch required. Production-grade default (CLAUDE.md Canonical Principle Rule 1): impl should match BC. BC-2.01.017 EC-017-005 mandates E-AUTH-006; impl returns E-AUTH-005. Unless PO amends BC, implementer must fix impl. PO has final authority on BC amendment.

## Next Steps

1. Dispatch product-owner: F-LP1-MED-002 (E-AUTH-005 vs E-AUTH-006) — PO adjudication
2. Dispatch implementer: all remaining 10 findings (CRIT-002, HIGH-001..004, MED-001/003/004, LOW-001/002) + harness scope expansion per 12378e35
3. After fix-burst: Pass 2 adversary dispatch
