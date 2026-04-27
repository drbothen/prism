---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: general-purpose-as-adversary
timestamp: 2026-04-27T01:50:00
phase: 5
inputs: ["crates/**/*.rs", "tests/**/*.rs", ".factory/specs/architecture/decisions/ADR-005-aql-injection-mitigation.md", ".factory/specs/behavioral-contracts/BC-2.05.010-confirmation-token-audit.md", ".factory/cycles/phase-3-dtu-wave-2/mutation-*.log"]
input-hash: "e2f206a"
traces_to: ".factory/specs/prd.md"
pass: 7
previous_review: "pass-6.md"
adversary: general-purpose-as-adversary (TD-VSDD-005 workaround)
develop_sha: e2f206af
date: 2026-04-27
verdict: OPEN
critical_count: 0
high_count: 3
medium_count: 4
low_count: 3
---

# Adversarial Review: Prism Wave 2 Integration Gate (Pass 7)

## Pass 7 Verdict: OPEN

**Critical:** 0
**High:** 3
**Medium:** 4 (TD)
**Low:** 3 (note)

**Cycle-closing assessment (S-7.02):** 1 of the 3 HIGH findings is a process-gap (BC vs AC contradiction unresolved through 6 passes + 4 fix-PRs); requires follow-up TD entry on spec-consistency-validation gate. The remaining HIGH items are implementation defects in W2-FIX-H/W2-FIX-I that prior passes missed because they treated the fix-PR additions as additive ("does it persist?") rather than integrative ("does the persisted shape obey the BC?").

**Fit to close:** NO — 3 HIGH blockers must be resolved before Wave 2 close.

**Top-3 most concerning findings:**
1. ADV-W2GATE-P07-HIGH-001 — `emit_token_generated` persists `token_id` in the issuance audit entry, contradicting BC-2.05.010 canonical test-vector ("Token ID in Entry?" = "No" for issuance). W2-FIX-H added persistence without recognizing the BC postcondition.
2. ADV-W2GATE-P07-HIGH-002 — Armis AQL allowlist validator's `select`-keyword check only inspects the FIRST occurrence of "select" via `find()`. Adversarial input `in:devices selected:y or select:x` bypasses the check because the first match (`selected`) fails its word-boundary heuristic, and subsequent occurrences are never re-examined.
3. ADV-W2GATE-P07-HIGH-003 — Test `test_BC_2_05_010_token_id_excluded_from_result_summary_level_detail` is a tautology that masks finding #1.

---

## Finding ID Convention

Finding IDs use the format: `ADV-W2GATE-P07-<SEV>-<SEQ>` per the project's adversarial-review-template convention. Cycle prefix `W2GATE` corresponds to phase-3-dtu-wave-2 integration gate.

---

## Verification context

| Check | Result |
|-------|--------|
| `git rev-parse HEAD` | `e2f206af` (W2-FIX-J merged, target SHA) ✓ |
| `cargo test --workspace` | exit 0; 1498 tests passing (matches baseline) ✓ |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 ✓ |
| `cargo fmt --all --check` | exit 0 ✓ |
| `cargo deny check` | "advisories ok, bans ok, licenses ok, sources ok" ✓ |
| `cargo audit` | exit 0 with 3 unmaintained-warnings (bincode, instant, rustls-pemfile) — see ADV-W2GATE-P07-MED-003 |
| `cargo semver-checks` | not run (no published baselines) |

---

## Part A — Fix Verification (pass >= 2)

Note: per Pass 7 charter, prior pass-N reports were NOT read (fresh-context discipline). Fix verification is conducted by examining the merged fix-PR artifacts (W2-FIX-G state-mgr, W2-FIX-H #68, W2-FIX-I #69, W2-FIX-J #70) directly via `gh pr diff` and source inspection.

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| W2-FIX-H emitter persistence (WGC-W2-001) | HIGH | RESOLVED | All 5 emitters call `append_audit_entry` with `<B: RocksStorageBackend>` generic; tests `test_WGC_W2_001_*_persists_to_backend` exercise real persistence count |
| W2-FIX-H evict_expired backend.scan (WGC-W2-002) | HIGH | PARTIALLY_RESOLVED | Backend scan added at event_buffer.rs:370; see ADV-W2GATE-P07-MED-001 for asymmetric error-handling residue |
| W2-FIX-I SecretString bearer tokens (WGS-W2-002) | HIGH | RESOLVED with caveat | Type-system enforcement in place, Debug correctly redacts; see ADV-W2GATE-P07-MED-004 for parser-side plaintext residue |
| W2-FIX-I Armis AQL validator (WGS-W2-001) | HIGH | UNRESOLVED | See ADV-W2GATE-P07-HIGH-002 — `select`-keyword check is bypassable |
| W2-FIX-J MockStorageEngine cfg-gating | HIGH | RESOLVED | `#[cfg(any(test, feature = "test-utils"))]` correctly applied |

---

## Part B — New Findings

### CRITICAL

(none)

### HIGH

#### ADV-W2GATE-P07-HIGH-001: emit_token_generated persists token_id in issuance audit entry — direct BC-2.05.010 violation
- **Severity:** HIGH
- **Category:** spec-fidelity / contradictions / [process-gap]
- **Location:** `crates/prism-audit/src/token_events.rs:115-165` (emit_token_generated) and `:264-311` (emit_token_expired)
- **Description:** The persisted `AuditEntry.payload["parameters"]` JSON contains `token_lifecycle_detail.token_id` for both `Generated` and `Expired` events, contradicting BC-2.05.010 canonical-test-vector "Token ID in Entry? = No" for issuance.
- **Evidence:** `BC-2.05.010-confirmation-token-audit.md` line 70: "Token issued | `confirmation_token_issued` + `action_summary` + expiry | **No**" for column "Token ID in Entry?". Same "No" for `token_expired` (line 72). The W2-FIX-H implementation:

```rust
// token_events.rs:122-127
let detail = TokenLifecycleDetail {
    token_id: token_id.to_owned(),       // <- embedded into detail
    event_type: TokenEvent::Generated,
    action_summary: action_summary.to_owned(),
    expiry_time: expiry,
};
let parameters = serde_json::json!({
    "token_lifecycle_detail": detail_to_json(&detail) ...   // <- serialized verbatim
});
// :155
payload.insert("parameters".to_owned(), parameters.to_string());   // <- persisted
```

S-2.05 AC-4 (line 153-155) actually contradicts BC-2.05.010 — it says "the audit entry contains `token_id`" — so a spec consistency gap is also exposed: the AC and the BC disagree, and 6 prior passes plus the fix-PR did not detect the conflict.
- **Proposed Fix:** Resolve the BC-vs-AC contradiction (likely BC wins because token-ID-in-issuance is a known forensic anti-pattern per BC's stated rationale "to prevent correlation by log readers"). Then either (a) drop `token_id` from `TokenLifecycleDetail` for `Generated`/`Expired` variants (skip-serializing-if), or (b) emit a redacted detail for issuance that omits the field. Update `test_BC_2_05_010_token_id_excluded_from_result_summary_level_detail` to actually inspect the persisted MemBackend bytes and assert "token_id" is absent.
- **Process-gap:** [process-gap] — neither the spec-validate-consistency skill, the implementation-readiness skill, nor any of the 6 adversarial passes flagged the BC-vs-AC contradiction. Add a check to `validate-consistency` that compares BC-canonical-test-vectors against story ACs.

#### ADV-W2GATE-P07-HIGH-002: Armis AQL `select`-keyword validator only checks first occurrence — bypassable via benign-prefix decoy
- **Severity:** HIGH
- **Category:** security-surface
- **Location:** `crates/prism-sensors/src/auth/armis.rs:197-223`
- **Description:** The validator only inspects the first `select` occurrence; if that occurrence falls inside a benign substring (e.g., `selected`), the function returns `Ok(())` without re-examining subsequent matches.
- **Evidence:**

```rust
if lower_remainder.contains("select") {
    let select_re = lower_remainder.find("select");          // FIRST occurrence only
    if let Some(pos) = select_re {
        let prev_ok = ...;   // word-boundary before
        let next_ok = ...;   // word-boundary after
        if prev_ok && next_ok { return Err(...); }
        // NOTE: if the first match fails the boundary check, fall through to OK
        //       — subsequent `select` occurrences are NEVER re-examined
    }
}
```

Adversarial input that passes: `in:devices selected:y or select:x`. After stripping `in:`, remainder = `"devices selected:y or select:x"`. `find("select")` returns offset 8 (inside `selected`). At that position: `prev_ok=true` (preceded by space), `next_ok=false` (followed by `'e'` which is alphanumeric). The check falls through to `Ok(())`, but the second occurrence at offset 23 (`select:x`) is a real keyword followed by `:` (non-alphanumeric → next_ok=true) preceded by space (prev_ok=true) — that one WOULD be rejected if examined, but it never is. ADR-005 explicitly enumerates "select" in §"Rejected constructs"; the implementation does not honour the rule for non-first occurrences.

Additionally, the validator's quote-injection check (lines 230-263) covers ONLY double-quotes. Single-quote-based payloads (`name:'a'='a'`) bypass the equality-quote and quote-equality patterns entirely. Whether Armis itself accepts single-quoted values is sensor-specific, but ADR-005's "Unbalanced quotes" rule is half-implemented.
- **Proposed Fix:** Loop over ALL occurrences of `select` (e.g., iterate via `match_indices`) and reject if any one passes the word-boundary check. Extend quote-balance and equality-quote / quote-equality checks to cover single-quotes (or document explicitly in ADR-005 why single-quotes are exempt).

#### ADV-W2GATE-P07-HIGH-003: test_BC_2_05_010_token_id_excluded — tautology that masks ADV-W2GATE-P07-HIGH-001
- **Severity:** HIGH
- **Category:** verification-gaps / coverage-gap
- **Location:** `crates/prism-audit/src/tests/specialized_event_tests.rs:925-946`
- **Description:** Test name claims to verify BC-2.05.010 token-ID-exclusion postcondition, but neither assertion exercises the persistence path or inspects the actual `AuditEntry.parameters` bytes.
- **Evidence:**

```rust
fn test_BC_2_05_010_token_id_excluded_from_result_summary_level_detail() {
    let token_id = "tok-secret-001";
    let action_summary = "isolate host acme-ws-01";
    let detail = TokenLifecycleDetail {
        token_id: token_id.to_owned(),
        action_summary: action_summary.to_owned(),
        ...
    };
    // assertion 1: action_summary.contains(token_id) — trivially false because the
    //              hardcoded action_summary literal does not contain "tok-secret-001"
    assert!(!detail.action_summary.contains(token_id), "...");
    // assertion 2: detail.token_id == token_id — pure tautology (constructed value
    //              equals the input we constructed it from)
    assert_eq!(detail.token_id, token_id, "...");
}
```

The test would pass even if `emit_token_generated` printed the token_id to every log line and embedded it everywhere in the entry. The companion test `test_BC_2_05_010_token_event_context_carries_required_fields` (lines 953-963) is a similar self-truth: constructs the struct and asserts every field equals its input.
- **Proposed Fix:** Replace with a real assertion that calls `emit_token_generated` against a `MemBackend`, scans the audit_buffer CF, and asserts the persisted JSON does NOT contain `"token_id"` for the issuance entry.
- **Process-gap:** [process-gap] — assertion content is not validated by any skill. Suggest adding a tautology-detector to the `convergence-check` skill: flag tests where every `assert*!` references only locals constructed from test-literal inputs.

### MEDIUM

#### ADV-W2GATE-P07-MED-001: evict_expired backend-error path silently swallows individual remove() failures
- **Severity:** MEDIUM
- **Category:** silent-failure / code-quality
- **Location:** `crates/prism-sensors/src/event_buffer.rs:411-423`
- **Description:** Asymmetric error-handling between cache-eviction (fail-fast) and backend-eviction (fail-soft) loops; failed backend removes are swallowed but `total_deleted` and `known_prefixes` mutate as if success.
- **Evidence:**

```rust
match self.backend.remove(StorageDomain::EventBuffer, key) {
    Ok(()) => backend_deleted += 1,
    Err(e) => {
        tracing::warn!(...);   // swallowed
    }
}
```

The cache-eviction loop (lines 357-364) propagates errors via `?`, but the backend-eviction loop only emits `tracing::warn!` and continues. A failing backend `remove` corrupts the `known_prefixes` retain at line 437-442 (which only consults the cache, so a partial-failure leaves orphaned data the prefix set has forgotten about), and `total_deleted` is reported as if eviction succeeded. The W2-P1-A-004 doc-comment accepts "may persist until next eviction cycle" for the cache-failure case but the backend-failure case introduces a different state-divergence not covered by that note.
- **Proposed Fix:** Either (a) propagate the first backend-remove error as the cache path does, OR (b) document the asymmetry explicitly and ensure `known_prefixes` reflects backend reality (re-scan after eviction). The dead `let _ = start_key;` on line 430 should also be removed.

#### ADV-W2GATE-P07-MED-002: retry_forward_entry is a no-op stub with delay_secs that never sleeps
- **Severity:** MEDIUM
- **Category:** code-quality / missing-edge-cases
- **Location:** `crates/prism-storage/src/audit_buffer.rs:136-167`
- **Description:** Function loops `RETRY_MAX_ATTEMPTS` times with hardcoded `Err("not yet wired")`, mutates `delay_secs` for exponential backoff but never invokes any sleep; if wired, would burn 10 retry slots in microseconds and always return `PrismError::Internal`.
- **Evidence:**

```rust
#[allow(dead_code)]
pub(crate) fn retry_forward_entry(entry: &AuditEntry) -> Result<(), PrismError> {
    let mut delay_secs = RETRY_BASE_DELAY_SECS;
    for attempt in 1..=RETRY_MAX_ATTEMPTS {
        let forward_result: Result<(), String> = Err("not yet wired".to_string());
        match forward_result {
            Ok(()) => return Ok(()),
            Err(e) => {
                if attempt == RETRY_MAX_ATTEMPTS { ... return Err(...); }
                delay_secs = (delay_secs * RETRY_MULTIPLIER as u64).min(RETRY_MAX_DELAY_SECS);
            }   // no sleep call
        }
    }
}
```

Currently uncalled, but the function is `pub(crate)` and the doc-comment promises exponential backoff that does not exist.
- **Proposed Fix:** Either (a) delete until needed, OR (b) actually implement the forwarding shim and the sleep/yield.

#### ADV-W2GATE-P07-MED-003: 3 new RUSTSEC unmaintained-warnings not tracked in tech-debt-register
- **Severity:** MEDIUM
- **Category:** code-quality / coverage-gap
- **Location:** `.factory/tech-debt-register.md` (no entries for any of the 3) plus `cargo audit` output
- **Description:** Three unmaintained-class advisories surfaced by `cargo audit`, not tracked in the TD register. Bincode is on the critical path for SOC2 audit persistence.
- **Evidence:**
  - RUSTSEC-2025-0141 — `bincode 2.0.1` is unmaintained (used by prism-storage, prism-sensors, prism-query, prism-audit, prism-spec-engine — 5 in-scope crates including the audit-buffer encoder)
  - RUSTSEC-2024-0384 — `instant 0.1.13` unmaintained (transitive)
  - RUSTSEC-2025-0134 — `rustls-pemfile` unmaintained (transitive)
- **Proposed Fix:** TD entries for each. The bincode case in particular merits a "decide" note: stay-on-2.x with monitoring, OR pre-emptively migrate to `rmp-serde` / `bitcode` for the audit-buffer encoder. cargo audit should be wired into CI with explicit allow-list rather than implicit.

#### ADV-W2GATE-P07-MED-004: emit_credential_event token_str.clone() creates extra plaintext heap copy
- **Severity:** MEDIUM
- **Category:** security-surface
- **Location:** `crates/prism-sensors/src/auth/crowdstrike.rs:200-220`
- **Description:** OAuth2 token plaintext lives in `serde_json::Value` parse tree (no zeroing on drop) AND is `.clone()`-ed gratuitously before being wrapped in `SecretString`.
- **Evidence:**

```rust
let token_str = json.get("access_token") ... .to_string();   // plaintext heap copy 1
let token = SecretString::new(token_str.clone());            // plaintext heap copy 2 (clone)
let cached = CachedToken {
    token: SecretString::new(token_str),                     // moves the original
    ...
};
```

Both clones are eventually wrapped in `SecretString` and zeroed-on-drop, BUT the underlying parsed `json: serde_json::Value` (line 187) also contains the plaintext `access_token` string and is dropped without zeroing (`String` `Drop` deallocates without overwriting). For a defense-in-depth CWE-312 posture, the parser-side plaintext is the weak link. The `.clone()` on line 210 is gratuitous — `token_str` could be moved once, the `SecretString` cloned via its `clone()` impl.
- **Proposed Fix:** Either (a) parse the OAuth response without serde_json (manual JSON for just `access_token` + `expires_in`, into pre-pinned-and-zeroed buffer), OR (b) accept the parser leak as out-of-scope and document in TD-W2-FIX-I-002. Also remove the gratuitous `.clone()`.

### LOW

#### ADV-W2GATE-P07-LOW-001: emit_credential_event/flag_eval lose parent invocation trace_id in persisted form
- **Severity:** LOW
- **Category:** ambiguous-language / verification-gaps
- **Location:** `crates/prism-audit/src/flag_events.rs:121-145`, `token_events.rs:115-165`
- **Description:** Parent `ctx.trace_id` is logged via `tracing::info!` but not embedded in the persisted `AuditEntry`; durable forensic linkage to parent invocation is absent for flag and token paths.
- **Evidence:** `FlagEvalContext.trace_id` and `TokenEventContext` are passed to the emitter and emitted via `tracing::info!(...)`, but the persisted `AuditEntry.trace_id` is set to a NEW `Uuid::now_v7()` (line 133) with no link back. BC-2.05.010 says "All token lifecycle events include the `client_id`, `sensor`, and `tool_name` of the original write operation, **enabling forensic reconstruction of the full two-step write flow from the audit trail**." Without the parent trace_id in the persisted entry, that forensic claim is conditional on Vector forwarding being intact. Credential events DO embed `requesting_context` correctly — only flag_eval and token paths lose it.
- **Proposed Fix:** Add a `parent_trace_id` field to `FlagEvalDetail` and `TokenLifecycleDetail` (or surface `ctx.trace_id` directly in the parameters JSON). Probably folds into the ADV-W2GATE-P07-HIGH-001 fix-PR.

#### ADV-W2GATE-P07-LOW-002: Dead-code in audit_emitter Err branch (refactor cruft)
- **Severity:** LOW
- **Category:** code-quality
- **Location:** `crates/prism-audit/src/audit_emitter.rs:215-238`
- **Description:** The `Err(ref e)` path computes `outcome`/`result_summary`/`error_code` locals, discards them via `let _ = (...)`, then re-derives the same values inside a tuple constructor — wasted work, no functional impact.
- **Evidence:**

```rust
Err(ref e) => {
    let outcome = AuditOutcome::Failure { error_code: e.to_string() };  // computed
    let result_summary = format!("error: {e}");                          // computed
    let error_code = Some(e.to_string());                                // computed
    let _ = (outcome.clone(), result_summary.clone(), error_code.clone()); // discarded
    // re-derived in tuple constructor below
    ( AuditedResponse { outcome: AuditOutcome::Failure { ... }, ... },
      AuditOutcome::Failure { ... }, format!("error: {e}"), Some(e.to_string()) )
}
```

- **Proposed Fix:** Cleanup opportunity in next touch.

#### ADV-W2GATE-P07-LOW-003: 3 of 6 BC-2.05.010 token-rejection events lack dedicated emitters
- **Severity:** LOW
- **Category:** missing-edge-cases / coverage-gap
- **Location:** `crates/prism-audit/src/token_events.rs` (only 3 of 6 emitters present)
- **Description:** BC-2.05.010 enumerates 6 result_summary types; only 3 emitter functions exist (`emit_token_generated`, `emit_token_consumed`, `emit_token_expired`). The TokenEvent enum at line 27-42 includes the 3 missing variants (NotFound, HashMismatch, AlreadyConsumed) but no public function persists them.
- **Evidence:** `grep -n "emit_token" /Users/jmagady/Dev/prism/crates/prism-audit/src/token_events.rs` shows only the 3 emitters; BC-2.05.010 lines 55-56, 73-75 enumerate `token_not_found`, `action_hash_mismatch`, and `token_already_consumed`. If those rejection paths flow through `audit_emitter::AuditEmitter` instead, that may be acceptable, but I did not find evidence of that routing.
- **Proposed Fix:** Verify whether the 3 rejection events are emitted via the general AuditEmitter path (acceptable) or are entirely missing (must-fix in S-3.x). If the former, add a doc comment to `token_events.rs` cross-referencing the path.

---

## Areas confirmed clean

| Area | Verdict | Evidence |
|------|---------|----------|
| W2-FIX-G state-manager (factory artifacts) | not in code-audit scope | n/a |
| W2-FIX-H emit_credential_event persistence + generic threading | clean — append_audit_entry called, generic `<B: RocksStorageBackend>` consistent across 5 emitters | manual diff of PR #68; tests `test_WGC_W2_001_*_persists_to_backend` exercise real persistence count |
| W2-FIX-H evict_expired backend.scan added | partially clean — see ADV-W2GATE-P07-MED-001 for asymmetric error handling | event_buffer.rs:370-424 |
| W2-FIX-I SecretString discipline (3 sites: armis/claroty/crowdstrike) | clean except ADV-W2GATE-P07-MED-004 — type-system enforcement in place, Debug correctly redacts, expose_secret only at HTTP-header injection | crowdstrike.rs:81-88, armis.rs:310-317, claroty.rs |
| W2-FIX-I AQL validator placement vs HTTP semaphore | clean — `build_aql` (validation) at armis.rs:525 runs BEFORE `acquire_http_permit` at :528 | line ordering verified |
| W2-FIX-I AQL validator allowlist completeness | NOT clean — see ADV-W2GATE-P07-HIGH-002 | n/a |
| W2-FIX-J MockStorageEngine cfg-gating | clean — `#[cfg(any(test, feature = "test-utils"))]` on `mod mock` and `pub use mock::MockStorageEngine`, `proofs/storage_batch.rs` test body inside `#[cfg(test)] mod tests`, dev-dep feature wired in workspace Cargo.toml | lib.rs:32-56; storage_batch.rs:12 |
| init_registry SecretString discipline | clean — both `claroty_token` and `armis_token` are `SecretString` parameters; only test callers exist (no plain-String slip) | lib.rs:98-115, grep all callers |
| API-surface hygiene beyond MockStorageEngine | clean — all `pub mod tests` have `#[cfg(test)]`; `pub use test_utils::*` in prism-dtu-common is gated by crate-level `#![cfg(any(test, feature = "dtu"))]` | grep all `pub mod` and `pub use` |
| AC-5a/5b split documentation in S-2.08 + S-3.02 inheritance | clean — split annotated in S-2.08 (lines 197-216, changelog v1.8) and inherited as AC-9 in S-3.02 (lines 234-269, changelog v1.7) | grep verified |
| Workspace test count | 1498 (matches post-W2-FIX-J baseline) | `cargo test --workspace` total |
| cargo clippy / fmt / deny | all green | exit 0 across 3 |
| DTU clones 0% caught mutation rate | acknowledged settled per TD-W2-MUTATE-005; not re-litigated | mutation logs |
| prism-audit mutation rate | 80% caught (20/25 viable) — acceptable for in-scope crate | mutation-prism-audit.log |

---

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 3 |
| MEDIUM | 4 |
| LOW | 3 |

**Overall Assessment:** block
**Convergence:** findings remain — iterate (3 HIGH blockers)
**Readiness:** requires revision before Wave 2 close

---

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 7 |
| **New findings** | 10 (all 10 findings are net-new — Pass 7 was conducted in fresh-context with no read of pass-1..pass-6 reports) |
| **Duplicate/variant findings** | 0 (none of the 10 are restatements of prior pass findings, by adversary discipline) |
| **Novelty score** | 1.00 (10 / (10 + 0)) |
| **Median severity** | 3.0 (MEDIUM) — 3 HIGH (sev 4), 4 MEDIUM (sev 3), 3 LOW (sev 2); median across 10 = 3 |
| **Trajectory** | unknown (Pass 7 did not read prior passes; orchestrator may correlate. Prior passes per prompt: Pass 4 + Pass 6 were "clean" with 4 fix-PRs G/H/I/J merged after) |
| **Verdict** | FINDINGS_REMAIN — Pass 7 surfaced 3 net-new HIGH findings unaddressed by W2-FIX-G/H/I/J |

The novelty score of 1.00 reflects that fresh-context audit found defects orthogonal to the surface that prior passes + fix-PRs converged on. This is a known failure-mode of pass-by-pass refinement: each pass narrows on the previous pass's findings, but cross-cutting contract concerns (BC-vs-AC contradictions, multi-occurrence regex bypasses, test-tautologies) survive because they're not on any pass's incremental delta.

---

## Cycle-closing assessment notes

The 6+1 adversarial cycle did its job for surface-level defects (persistence-call presence, cfg-gating, type-signature enforcement) but missed the contract-level question about what gets persisted and whether the persisted shape matches the BC's canonical test-vector. The `test_BC_2_05_010_token_id_excluded` tautology is the smoking gun: a test named for a BC postcondition that does NOT exercise the postcondition surface. This is a class of defect (named-after-spec, doesn't-verify-spec) that the convergence-check skill should catch.

ADR-005 (AQL injection mitigation), TD-VSDD-005, and TD-W2-MUTATE-005 are settled-decisions per the prompt and were not critiqued. ADV-W2GATE-P07-HIGH-002 is critique of the IMPLEMENTATION of ADR-005, not of the ADR itself.

## Appendix: input-hash chain confirmation

- Audit conducted at `e2f206af` (`fix(W2-FIX-J)`, PR #70)
- 4 fix-PRs accounted for: G (state-mgr only, no PR), H (#68), I (#69), J (#70)
- All Wave 2 stories (S-2.01–S-2.08, S-6.11–S-6.13) merged on develop
- Pass 1-6 reports were NOT read (fresh-context discipline)
