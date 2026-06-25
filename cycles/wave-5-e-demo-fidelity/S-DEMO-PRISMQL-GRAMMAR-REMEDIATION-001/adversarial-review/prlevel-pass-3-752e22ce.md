---
document_type: adversarial-review
story: S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001
pass: prlevel-3
pr: "#203"
pr_head: "752e22ce"
result: NOT_CLEAN
clean_strict: false
clean_pr_merge: false
streak_before: 0
streak_after: 0
findings_total: 3
findings_med: 1
findings_obs: 2
date: 2026-06-25
state_decision: D-1349
---

# PR-LEVEL Adversarial Pass 3 — S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001

**PR HEAD:** `752e22ce`
**Date:** 2026-06-25
**CLEAN(strict):** NO
**CLEAN(PR-merge):** NO
**3-CLEAN streak:** RESET 0/3 on new FROZEN HEAD `3820268a`

---

## Summary

Pass 3 on PR HEAD `752e22ce` found 3 findings (1 MED + 2 OBS). All 3 CLOSED by implementer + product-owner before streak advance. New FROZEN PR HEAD after all closures = `3820268a`. 3-CLEAN streak RESET 0/3 on `3820268a`.

---

## Findings

### MED-1 — NYA handler doc correction over-generalized (HIGH-confidence; blocking)

**Severity:** MED (HIGH-confidence)
**Status:** CLOSED — commits `dbb05829` → `3820268a`

**Finding:** The OBS-1 fix from Pass 2 over-generalized. The corrected doc claim "no input processing occurs" is FALSE for 10 NYA handlers that call `validate_text_field` / `validate_id_field` (F-PR163 length-bound guard returning `-32602` on oversized input) BEFORE the `-32003` NYA fast-fail. Committed tests assert the `-32602` path. The doc comment for these handlers incorrectly stated "no input processing occurs" when in fact length-bound input validation does occur prior to the NYA return.

**Closure:** Implementer audited ALL NYA handlers. Corrected 10 handlers with input-processing guards to accurately state length-bound-then-NYA behavior:
`get_diagnostics`, `create_schedule`, `create_rule`, `create_case`, `configure_credential_source`, `delete_credential`, `list_alerts`, `delete_rule`, `get_case`, `update_case`. Parameter-free NYA stubs left as "no input processing occurs" — accurate. NOTE: `update_rule` does not exist; the real 4th validate handler was `create_case`. Code HEAD after MED-1 closure: `3820268a` (includes `dbb05829` MED-1 fix + `3820268a` OBS-2 closure).

---

### OBS-1 — E-QUERY-040 message hard-coded `| limit` even for `| tail` violations (LOW)

**Severity:** OBS (LOW)
**Status:** CLOSED — PO taxonomy refinement v2.00 + implementer `error.rs` update at commit `240f33f0`

**Finding:** `RedundantRowLimit` `Display` implementation in `prism-query/src/error.rs` hard-coded `"| limit"` in the error message even when the violation was triggered by a `| tail` pipe stage. E-QUERY-040 message format needed neutral wording covering both `PipeStage::Limit(_)` and `PipeStage::Tail(_)` FORBID-BOTH triggers.

**Closure:** Product-owner refined error-taxonomy.md v1.99→v2.00: E-QUERY-040 Message Format updated to neutral row-cap wording `"a row-capping \`| limit\`/\`| tail\` pipe stage (cap: {pipe_limit})"`. `{pipe_limit}` field definition updated. Firing-semantics predicate updated to `PipeStage::Limit(_) | PipeStage::Tail(_)`. Implementer updated `error.rs` `RedundantRowLimit` `Display` byte-for-byte to match taxonomy v2.00 neutral wording + updated test assertions. Commit `240f33f0`.

---

### OBS-2 — AC-017/018 demo tape depended on uncommitted `/tmp/run_ac017_018.sh` (OBS)

**Severity:** OBS
**Status:** CLOSED — commit `3820268a`

**Finding:** AC-017 and AC-018 demo tape referenced a driver script at `/tmp/run_ac017_018.sh` that was not committed to the evidence directory. This made the demo tape non-reproducible from a clean checkout.

**Closure:** Implementer committed the driver script into the evidence directory and fixed the tape path reference. Commit `3820268a`.

---

## Probes Passing (all other ACs/checks)

All other acceptance criteria and standing adversary probes PASS on `752e22ce`:
- AC-001..AC-016 grammar acceptance criteria: PASS
- AC-017/018 demo tape paths: PASS (after OBS-2 closure at `3820268a`)
- AC-019 BLOCKER-001 deferral (D-1326): DO-NOT-FLAG per standing exemption
- AC-020 runbook v1.4 satisfied: PASS
- AC-021..AC-027: PASS
- FORBID-BOTH / E-QUERY-040 trigger: PASS
- Temporal plain-string handling (D-1335): DO-NOT-FLAG
- E-QUERY-036/037 label distinction: PASS
- SAP-1 tracing emission catalog: PASS
- AD-017 credential redaction: PASS
- Non-exhaustive EXPECTED=87: PASS
- fmt-canonical: PASS
- `just check` EXIT=0 (4929 tests): PASS on `752e22ce`

---

## Closure Summary

| Finding | Severity | Closed By | Commit |
|---------|----------|-----------|--------|
| MED-1 NYA doc over-generalization | MED | implementer | `dbb05829` + `3820268a` |
| OBS-1 E-QUERY-040 `\| limit` hard-code | OBS | PO (taxonomy v2.00) + implementer | `240f33f0` |
| OBS-2 AC-017/018 driver script uncommitted | OBS | implementer | `3820268a` |

**New FROZEN PR HEAD after all closures:** `3820268a`
**just check on `3820268a`:** EXIT=0 (4929 tests)
**non-exhaustive:** 87
**fmt-canonical:** CLEAN
**3-CLEAN streak RESET:** 0/3 on `3820268a` (code HEAD moved by fix commits)
**CI:** re-runs on `3820268a` push (in progress)
