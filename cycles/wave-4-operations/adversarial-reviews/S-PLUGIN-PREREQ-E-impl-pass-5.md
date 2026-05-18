---
document_type: adversarial-review
producer: adversary
pass: 5
cascade_scope: LOCAL implementation
story_id: S-PLUGIN-PREREQ-E
diff_head: 9e7c3d8e
diff_base_to_develop: a5ab742c
factory_artifacts_head: b8a527fc
version: "1.0"
timestamp: 2026-05-18T05:00:00Z
verdict: BLOCKED
streak_before: "0/3"
streak_after: "0/3"
finding_counts:
  critical: 1
  important: 1
  suggestion: 1
  observation: 2
  process_gap: 0
fb_impl_3_closures:
  verified: 3
  defective: 1  # BC-2.16.002 intro count drift (F-LP-IMPL-P5-003)
  partial: 1    # Rule C structurally dead in keyring path (F-LP-IMPL-P5-001)
---

# S-PLUGIN-PREREQ-E Impl-Cascade Adversary Pass 5

**Verdict: BLOCKED** — 1 CRITICAL + 1 IMPORTANT + 1 SUGGESTION + 2 OBSERVATIONS

Pass 5 reviewed feature/S-PLUGIN-PREREQ-E at HEAD `9e7c3d8e` (2 new commits beyond
`8e4df5bf`: FB-IMPL-3 implementer at `253d9e50` + `db16f906`... wait — pass 5 was run
against `9e7c3d8e`, the FB-IMPL-3 feature HEAD per frontmatter). Fresh-context adversarial
review against the full diff `a5ab742c..9e7c3d8e`.

---

## Finding F-LP-IMPL-P5-001 — CRITICAL: Rule C Structurally Dead in Production Keyring Path (3rd-Iteration Paper-Fix Lineage)

**Severity:** CRITICAL (P0 security contract)
**Finding ID:** F-LP-IMPL-P5-001
**Lineage:** F-003 (pass-1) → F-P2-001 (pass-2, paper-fix) → F-P4-001 (pass-4, argument-semantic-alias) → F-LP-IMPL-P5-001 (pass-5, backend-no-op)

### Description

After FB-IMPL-3 at `9e7c3d8e`, `CredentialRefProbe::probe()` has been extended to return
`Result<Option<String>, BootError>` and the `step5_init_credential_store_with_probe` gate
correctly executes `if let Some(actual_shape) = actual_shape_opt { ... }`. However:

`KeyringCredentialProbe::probe()` returns `Ok(None)` unconditionally. The keyring backend
stores only credential values — no auth_type metadata. The Rule C gate (`if let Some(...)`)
is structurally correct code that is unreachable in production: `KeyringCredentialProbe` is
the only production `CredentialRefProbe` implementor; `ShapedProbe` is test-only.

This creates a semantic gap between the spec claim and the runtime behavior:
- ADR-026 §D3 (as written at this HEAD): "The resolved credential type must structurally
  match the spec's auth_type variant. Mismatches are rejected at credential-resolution time,
  before any HTTP request." — no backend qualification.
- BC-2.01.016 §Error Cases E-SPEC-014: same unconditional mandate.
- Implementation: Rule C fires only through `ShapedProbe` (test fixture), never through
  `KeyringCredentialProbe` (production).

This is not a code bug — the `Option<String>` shape extension and the gate are correct. The
defect is a spec/story/implementation coherence gap: ADR-026 §D3 and BC-2.01.016
§E-SPEC-014 read as unconditional production mandates, but the implementation (following
story S-PLUGIN-PREREQ-E `risk_mitigations` line 67) scopes Rule C to test-fixture coverage.

### Root Cause Analysis

The story spec at risk_mitigations line 67 states: "AC-1..3c: SensorAuth unsealing is pure
deletion + per-test-fixture credential-validation coverage." This establishes test-fixture
scope for Rule C. ADR-026 §D3 and BC-2.01.016 §E-SPEC-014 do not contain this
qualification. Three artifacts at the same precedence level disagree on scope.

### Escalation Required

This requires architect adjudication per CLAUDE.md Source-of-Truth Precedence:
- Option A: Extend keyring backend to store auth_type metadata NOW (in PREREQ-E scope)
- Option B: Amend ADR-026 §D3 + BC-2.01.016 §E-SPEC-014 to scope Rule C to backends with
  shape metadata; defer keyring production enforcement to PLUGIN-MIGRATION-001-A
- Option C: Amend S-PLUGIN-PREREQ-E to remove the "per-test-fixture" carveout (forces
  Option A implementer work)

Route to orchestrator → architect adjudication before pass-6.

**Status:** ESCALATED to architect D-706; Option B chosen; ADR-026-AMENDMENT committed at
SHA 4dd97f14; F-LP-IMPL-P5-001 closed by FB-IMPL-4 (D-707).

---

## Finding F-LP-IMPL-P5-002 — IMPORTANT: `unregister_plugin` Doc-vs-Code Discrepancy on CAS Claim

**Severity:** IMPORTANT
**Finding ID:** F-LP-IMPL-P5-002

### Description

`PluginRuntime::unregister_plugin` doc comment states: "Uses compare-and-swap (CAS)
semantics to remove the plugin atomically." The implementation uses a `write()` lock on a
`RwLock<HashMap<...>>` — this is a mutual exclusion write lock, not a CAS operation. CAS is
a specific non-blocking atomic primitive (CPU-level compare-exchange). The doc claim is
misleading and technically incorrect: the operation is mutually exclusive write-lock removal,
not CAS.

In a production multi-threaded environment, this is not a correctness bug — the write lock
ensures atomicity. But the doc comment's "CAS" claim could mislead future implementers or
auditors who expect non-blocking behavior or spin-retry semantics.

### Evidence

`crates/prism-spec-engine/src/plugin/runtime.rs` (at `9e7c3d8e`):
`unregister_plugin` acquires `self.plugins.write().unwrap()` and calls `.remove()`. This is
straightforward `RwLock` write-lock removal. No atomic compare-exchange involved.

### Required Fix

Update `unregister_plugin` doc comment to accurately describe the operation:
"Uses a write lock to atomically remove the plugin from the registry (load → clone → store
single-threaded pattern via `RwLock`). Returns true if the plugin was registered, false if
it was not found."

**Status:** Closed by implementer at feature@`db16f906`. Doc updated to describe
load→clone→store single-threaded pattern via RwLock. F-LP-IMPL-P5-002 closed.

---

## Finding F-LP-IMPL-P5-003 — SUGGESTION: BC-2.16.002 Intro Catalog Count "33" Stale; Body Has 34

**Severity:** SUGGESTION (state-manager domain)
**Finding ID:** F-LP-IMPL-P5-003

### Description

BC-2.16.002 §Postconditions Canonical Structured Event Catalog intro line reads:
"The catalog currently contains **33** structured events."

However, the catalog table body has 34 rows — row 34 `plugin_registration_rolled_back` was
added by FB-IMPL-3 (implementer + state-manager, `9e7c3d8e` feature HEAD, factory SHA
`b8a527fc`). The state-manager updated BC-2.16.002 frontmatter `version: 1.32` and the
changelog row for v1.32 correctly cites "Catalog count 33→34" but the intro LINE at §
Postconditions body was not updated from 33 to 34.

This is a TD-VSDD-060 sibling-site sweep gap: the state-manager updated the frontmatter
and changelog but missed the one in-body count reference.

### Required Fix

BC-2.16.002 §Postconditions intro: `33` → `34`.
State-manager applies this as part of the FB-IMPL-4 D-707 burst (F-LP-IMPL-P5-003 in scope).

**Status:** Closed by state-manager in FB-IMPL-4 D-707 (this burst).

---

## Observation F-LP-IMPL-P5-OBS-001: HS-PREREQ-E-001 Holdout Scenario Coverage Completeness

**Severity:** OBSERVATION (process gap — cycle-close)
**Finding ID:** F-LP-IMPL-P5-OBS-001

### Description

HS-PREREQ-E-001 holdout scenarios covering Rule C validation include scenarios exercising
`ShapedProbe` in isolation. Given that `KeyringCredentialProbe::probe()` returns `Ok(None)`
unconditionally in production, there is no holdout scenario that exercises the production
keyring path end-to-end for a Rule C mismatch case (i.e., a sensor spec with
`auth_type = "bearer_static"` registered against a credential that was originally intended
for `oauth2_client_credentials` in a real keyring, without `ShapedProbe` injection).

This is not a blocking defect (the holdout scenarios correctly cover the Rule C code path
via `ShapedProbe`; the production keyring gap is now documented in ADR-026 §D3 per D-706).
However, when PLUGIN-MIGRATION-001-A ships the credential metadata sidecar, the holdout
suite should be extended to cover the production Rule C path.

**Disposition:** Cycle-close deferred to PLUGIN-MIGRATION-001-A per S-7.02.

---

## Observation F-LP-IMPL-P5-OBS-002: `PluginLoadResult` Rustdoc Minor Clarification Opportunity

**Severity:** OBSERVATION (non-blocking documentation quality)
**Finding ID:** F-LP-IMPL-P5-OBS-002

### Description

`PluginLoadResult` enum variants in `crates/prism-spec-engine/src/plugin/runtime.rs` have
minimal doc comments. The `Rollback` variant doc says "Plugin was loaded but then rolled back
due to a registration failure." The cause of the registration failure (specifically that
`register_write_tool` returned `DuplicateWriteToolRegistration`) is not spelled out. This is
an observability minor gap — future developers debugging a `Rollback` result would benefit
from knowing the precise trigger condition.

This is a documentation-quality observation, not a correctness defect.

**Disposition:** Cycle-close deferred per S-7.02. The behavior is correctly catalogued in
BC-2.16.002 row 34 `plugin_registration_rolled_back` + BC-2.16.012 EC-016-012-004. A future
documentation-sweep story could improve the rustdoc.

---

## FB-IMPL-3 Closure Verification Summary

| Finding | Pass-4 Source | Status at Pass-5 |
|---------|---------------|------------------|
| F-P4-001 Rule C dead-code | Both callsites aliased auth_type | PARTIAL CLOSE — code structure correct (`Option<String>` shape return + `if let Some(...)` gate); underlying gap is backend limitation, not code logic. D-706 architect adjudication required. |
| F-P4-002 Silent partial failure | register_write_tool failure kept plugin loaded | VERIFIED CLOSED — deregister_write_tools_for_plugin + unregister_plugin + plugin_registration_rolled_back ERROR event all load-bearing. |
| BC-2.16.002 row 34 catalog addition | Missing catalog entry | VERIFIED CLOSED — row 34 `plugin_registration_rolled_back` correct. DEFECT: intro count not updated (F-LP-IMPL-P5-003). |

---

## Streak Summary

- Pass-5 verdict: BLOCKED (1 CRIT + 1 IMP + 1 SUG + 2 OBS)
- Streak before: 0/3
- Streak after: 0/3 (BLOCKED resets; was 0/3 coming in after FB-IMPL-3)
- Next: FB-IMPL-4 (architect D-706 Option B adjudication + state-manager spec amendments D-707)
- After FB-IMPL-4: adversary pass-6 fresh-context against feature HEAD db16f906
