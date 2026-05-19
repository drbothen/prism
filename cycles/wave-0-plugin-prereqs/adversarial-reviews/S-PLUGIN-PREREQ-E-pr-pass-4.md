---
document_type: adversarial-review
producer: adversary (security/edge-case fresh-context; pr-manager reified)
pass: 4
cascade_scope: PR-LEVEL
story_id: S-PLUGIN-PREREQ-E
pr: 151
feature_head_reviewed: a4c048ce
factory_head_at_review: 7fc27d09
version: "1.0"
timestamp: 2026-05-19T13:00:00Z
verdict: CLEAN
streak_before: "2/3"
streak_after: "3/3 CONVERGED"
finding_counts:
  critical: 0
  high: 0
  medium: 0
  low: 0
  observation: 1
  process_gap: 0
fix_burst: none
bc_5_39_001_streak: "3/3 CONVERGED"
local_cascade_converged_at: "pass-16 (D-721)"
ci_platforms_failing: 0
ci_job_pass_count: 36
ci_job_total: 36
---

# S-PLUGIN-PREREQ-E PR-LEVEL Adversarial Pass-4 Report

**Verdict: CLEAN. Streak: 3/3 CONVERGED. BC-5.39.001 PR-LEVEL satisfied.**

Security and edge-case focused final fresh-context pass. Zero blocking findings.
BC-5.39.001 PR-LEVEL cascade converges at pass-4 (3 consecutive CLEAN passes:
pass-2, pass-3, pass-4). Ready for squash-merge per D-716 Option A authorization.

---

## §1 Focus Areas Examined

- AD-017 credential redaction compliance (new SpecEngineError variants)
- #[non_exhaustive] discipline on new public types (WriteToolInvalidationMap)
- validate_cross_composition Rule B condition consistency across call paths
- OnceLock vs RwLock — correct choice for DYNAMIC_WRITE_TOOLS
- No production panic/unwrap paths introduced

---

## §2 Findings

### AD-017 Credential Redaction

New SpecEngineError variants (AuthTypeCrossComposition, MultipleCredentialRefs,
AuthTypeCredentialMismatch, DuplicateWriteToolRegistration, WriteToolRegistrationAfterBoot)
carry only structural descriptor strings and counts — never credential values. Doc comments
explicitly note AD-017 compliance. Auto-derived Debug is safe by design. Story spec
requirement for "redacted Debug" is satisfied by the fields containing no secrets.
CLEAN.

### #[non_exhaustive] on WriteToolInvalidationMap

WriteToolInvalidationMap (prism_query::invalidation) is pub but not re-exported at
the prism-query crate root. Not in the scope of the non-exhaustive-violation compile-fail
gate (which covers prism-spec-engine TOML-deserialized types). The struct is consumed
by prism-bin boot.rs which constructs it via struct literal. Adding #[non_exhaustive]
now would break boot.rs struct literal construction without adding production value
(only prism-bin constructs this type; it's an internal API boundary between bin and query).

P4-OBS-001: WriteToolInvalidationMap missing #[non_exhaustive]. Non-blocking per scope
(not in perimeter-violation gate, not a top-level re-export, only constructed by prism-bin).
Forward-compat can be addressed in PLUGIN-MIGRATION-001-C when the type stabilizes.

### validate_cross_composition Rule B call conditions

spec_parser.rs::parse() calls validate_cross_composition only when credential_refs.len() > 1.
add_sensor_spec.rs calls when !credential_refs.is_empty() (len >= 1).
Asymmetry is not a correctness bug:
- len == 0: valid (no auth configured; runtime fails if auth needed)
- len == 1: parse() skips (Rule B would pass; Rule A enforced by serde)
- len > 1: both paths call validate_cross_composition correctly
CLEAN.

### RwLock Choice for DYNAMIC_WRITE_TOOLS

RwLock (not OnceLock) is correct for a mutable-at-boot, read-only-at-query registry:
- Boot phase: multiple register_write_tool() calls need write guards.
- Query phase: reads via invalidation lookups need read guards (concurrent, non-blocking).
- OnceLock would not permit incremental registration during boot.
CLEAN.

---

## §3 BC-5.39.001 Convergence Declaration

Three consecutive CLEAN passes achieved:
- Pass-2 (pr-reviewer): CLEAN (0 blocking findings, 5 observations)
- Pass-3 (code-reviewer): CLEAN (0 findings)
- Pass-4 (security/edge-case): CLEAN (0 blocking findings, 1 observation)

BC-5.39.001 PR-LEVEL cascade converged. Per D-716 Option A: 3 consecutive CLEAN
passes at PR-LEVEL authorize squash-merge to develop.

Cumulative PR-LEVEL cascade:
- Pass-1: BLOCKED (2 HIGH — CI-incompatible test + SemVer gap) → FB-PR-1 closed both
- Pass-2: CLEAN (1/3)
- Pass-3: CLEAN (2/3)
- Pass-4: CLEAN (3/3 CONVERGED)

Total PR-LEVEL passes: 4. Fix-bursts: 1 (FB-PR-1).

LOCAL + PR-LEVEL combined: 16 LOCAL passes + 4 PR-LEVEL passes = 20 total adversarial passes.
