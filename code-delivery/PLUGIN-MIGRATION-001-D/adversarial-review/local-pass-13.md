---
document_type: adversarial-review-pass
story_id: PLUGIN-MIGRATION-001-D
pass_number: 13
pass_scope: LOCAL-SPEC-LEVEL
pass_date: 2026-05-20
adversary_model: claude-opus-4-7 (1M context); fresh-context
streak_before: 0/3
streak_after: 0/3
findings_summary: "1 HIGH + 2 MED + 1 LOW + 0 OBS"
checkpoint_status: BLOCKED-soft
---

# Pass-13 Adversarial Review

## Scope

LOCAL spec-level fresh-context verification post-FB-IMPL-P12 (D-746). Independent grep across all 14 primary artifacts + ADR-026 cross-check + Red Gate test code witness.

## Findings

### F-LP13-HIGH-001 — Spec-vs-Spec Contradiction: ADR-026 §D3 mandates strings ADR-028 §D2 calls "latent label bugs"

NOVEL 6th coherence-axis class. ADR-026 §D3 (ACTIVE, shipped 2026-05-19 via PR #151) mandates auth_type_name() returns: cyberint=bearer_static, claroty=cookie_roundtrip, armis=api_key. ADR-028 §D2 (PROPOSED v1.4) declares these "latent label bugs". Live code at `crates/prism-sensors/src/auth/{cyberint,claroty,armis}.rs` returns ADR-026 values; Red Gate test `test_BC_2_01_016_003_four_auth_impls_minimal_diff_post_unsealing` at `mod.rs:158-200` ASSERTS them. No Supersedes linkage between ADR-026 and ADR-028.

Severity HIGH (potentially CRITICAL): implementer following BC-2.16.013 will author TOML with auth_type that runtime resolution will silently mismatch. validate_cross_composition only checks enumerated-set membership, not impl resolution.

Routing: architect adjudication required.

### F-LP13-MED-001 — AC-001 hard-asserts tables.len() == 3 but Task 3 says incidents "may be omitted"

Story line 248 asserts `tables.len() == 3`; Task 3 lines 671-673 says "may omit or include as placeholder". Non-deterministic. HS-018-03 expects 3 tables registered.

Routing: story-writer.

### F-LP13-MED-002 — HS-018 has 3 sub-scenarios but RG-09 covers only HS-018-01

HS-018-02 (case-mismatch CrowdStrike vs crowdstrike) is distinct P0 sub-scenario per HOLDOUT-INDEX:230 but not anchored to any RG test.

Routing: product-owner (anchor or clarify); possibly story-writer (new RG).

### F-LP13-LOW-001 [process-gap] — TS-PLUGIN-PARITY-001 lacks modified field

POL-27 extension proposed pass-12 should flag missing `modified:` on changelog-bearing documents, not just stale `modified:` values. The audit must scope to MISSING field too.

Routing: product-owner (add field); orchestrator (policy refinement at codification).

## Cumulative-closure durability verification

All pass-1..12 closures durable except P2 F-001 (auth_type swap overturned by P4/P5 and now creates F-LP13-HIGH-001 chain).

## Phase verification summary

A FAIL (F-LP13-MED-001 AC vs Task) / B FAIL (F-LP13-HIGH-001 ADR vs ADR vs code) / C FAIL (F-LP13-MED-002 HS coverage) / D PASS / E PASS / F PASS modulo F-LP13-LOW-001 missing-field gap / G PASS / H PASS / I PASS / J PASS / K PASS

## Verdict

BLOCKED-soft — 1 HIGH + 2 MED + 1 LOW process-gap. Streak resets 0/3 → 0/3.

## Streak Update

- streak_before: 0/3
- streak_after: 0/3
- next_action: orchestrator surfaces F-LP13-HIGH-001 to user for ADR adjudication (Path A/B/C). User locked Path A. FB-IMPL-P13 dispatches architect (ADR supersession) + PO (BC/HS/TS) + SW (story).

## Novelty Assessment

HIGH — 6th distinct novel axis: inter-ADR contradiction with shipped+tested code witness.
