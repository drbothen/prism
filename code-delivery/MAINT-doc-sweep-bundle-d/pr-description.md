## Summary

Doc-comment cleanup sweep (Bundle D) from the workspace audit (`workspace-audit-2026-05-08.md`, 53 findings). This PR addresses 9 audit findings across 10 files: stale STUB headers where code is real, test docstring polarity drift (comments claimed tests MUST FAIL but tests actually pass), stale VP-025 cache_key proof comments, and a missing README. No logic changes — all code semantics are preserved.

Factory-artifacts side already landed at `543bd759` (D-302 in `.factory/STATE.md`). This PR is the develop-branch code-side counterpart.

## Files changed

- **`crates/prism-credentials/src/trait_.rs`** — removed STUB header; doc-comment now describes the real `CredentialProvider` trait (F-AUD-D1-13)
- **`crates/prism-security/src/content_hash.rs`** — removed STUB header; doc-comment describes actual blake3 content-hash implementation (F-AUD-D1-15)
- **`crates/prism-security/src/risk_tier.rs`** — removed STUB header; doc-comment describes real risk-tier classification logic (F-AUD-D1-16)
- **`crates/prism-audit/src/audit_emitter.rs`** — removed STUB header; doc-comment matches actual audit event emission implementation
- **`crates/prism-sensors/src/registry.rs`** — removed STUB header; doc-comment matches real sensor registry
- **`crates/prism-credentials/tests/bc_3_2_002_org_id_namespace.rs`** — corrected polarity-drift docstring (F-AUD-D3-02); test passes, comment no longer claims it MUST FAIL
- **`crates/prism-dtu-cyberint/tests/multi_tenant.rs`** — corrected polarity-drift docstring (F-AUD-D3-03); test passes, comment no longer claims it MUST FAIL
- **`crates/prism-core/tests/bc_3_1_001_org_id.rs`** — corrected polarity-drift docstring (F-AUD-D3-05); test passes, comment no longer claims it MUST FAIL
- **`crates/prism-query/src/proofs/vp025_cache_key.rs`** — updated stale VP-025 cache_key proof comments (F-AUD-D5-08, F-AUD-D5-09, F-AUD-D7-03) to reflect current Kani verification logic
- **`README.md`** — backfilled from stub (F-AUD-D8-02)

## Traceability

- **D-302**: `.factory/STATE.md` decision log (factory-artifacts side at `543bd759`)
- **Audit source**: `.factory/cycles/wave-4-operations/workspace-audit-2026-05-08.md` (53 findings, cb40bd00)
- **Findings addressed**: F-AUD-D1-13, F-AUD-D1-15, F-AUD-D1-16, F-AUD-D3-02, F-AUD-D3-03, F-AUD-D3-05, F-AUD-D5-08, F-AUD-D5-09, F-AUD-D7-03, F-AUD-D8-02

## Test plan

- [ ] `just check` passing locally (pre-push hook gate — resolves Open Questions Q6 and Q7)
- [ ] CI green (fmt + clippy + nextest + doctests + crate-layout)
- [ ] No logic changes — all test semantics preserved; only doc-comment text changed
- [ ] Doc-comments now match actual code behavior (stale STUB headers removed)
- [ ] Test polarity-drift docstrings corrected — test execution unchanged
- [ ] VP-025 proof comments accurate — proof logic unchanged
