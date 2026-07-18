---
pass: pr-level-pass-3
story: AUDIT-COVERAGE-001
pr: 226
frozen_head: "0fbef7db"
namespace: F-AUD-PR4
date: 2026-07-18
clean_strict: false
clean_pr_merge: false
streak_before: 1
streak_after: 0
streak_reset_reason: "1 MEDIUM — ephemeral-port trap"
---

# Adversarial Review — F-AUD-PR4 (pr-level-pass-3)
**Pass:** PR-LEVEL streak pass 2 of 3 (attempt) · **Frozen HEAD:** `0fbef7db` · **PR:** #226 · **Date:** 2026-07-18

---

## Top-Line Counts

CRITICAL 0 | HIGH 0 | MEDIUM **1** | LOW 0 | OBS 0 | PROCESS-GAP 0

---

## Findings

### F-AUD-PR4-MED-001 — Audit-script port contract incoherent with demo-run.sh exports (ephemeral-port trap) — MEDIUM, confidence HIGH

**Route:** implementer

**Location:** `scripts/t13-preflight-audit.py` lines 148–149, 172–174; `demo-run.sh` lines 16, 165, 377–388; T13 capstone demo runbook (no `_PORT`/`t13-preflight`/bridging occurrence).

**Finding:** The audit script reads only `PRISM_THREATINTEL_PORT` and `PRISM_NVD_PORT` (with hard-coded defaults 54646/54647) and rebuilds `BASE_URL`s from them. It never reads `PRISM_*_BASE_URL`. However, `demo-run.sh` exports DTU ports as ephemeral (port=0 in demo.toml); operator-facing output emits only `PRISM_*_BASE_URL` full URLs — never bare port numbers. The T13 capstone demo runbook contains no bridging guidance and no occurrence of `_PORT` or `t13-preflight`.

**Consequence:** The defaults are always wrong for the natural copy-paste workflow. The `PRISM_*_BASE_URL` variables (the documented operator output) are silently discarded. All enrichment checks E1–E6, H19a/b, H20 FAIL with "connection refused" to dead ports. Fails loud (no false-green) but the documented happy path cannot be followed — this is the exact capstone-run workflow the tool serves.

**Severity rationale:** MEDIUM (not HIGH: fails loud, port digits visible in printed URL; not LOW: defaults always-wrong under standard use, companion contract ignored, no copy-pasteable path).

**Fix:** Honor `PRISM_*_BASE_URL` when present (fallback to PORT construction); correct docstring; correct PR body line 58 which propagates the misleading `_PORT` instruction.

---

## Per-Axis

**PR description:** Quantitative claims consistent; coverage matrix verified; checklist honest. Caveat: body line 58 propagates the misleading `_PORT` instruction (folded into MED-001).

**Script fail-loud:** SOLID.

**Grounding:** 54-tool catalog (tool_dispatch_tests.rs:493 asserts 54), taxonomy v2.56, EC-11-079 all grounded.

**POL-21/22:** No violation.

**AD-017:** OK.

**Merge-safety:** 2-file diff, no production Rust, 45/45 CI green.

**Documented-as-fixed regression check:** SEC-001/002, EC-11-079, provenance, diffstat/count corrections all HOLDING.

**Index/SAP probes:** N/A.

---

## Fresh Lenses

Sections B/E-F discriminate (E1 asserts Int64 type AND value ≥75). DTU fixture contracts match runbook (score≥75; CVSS 8.1 deterministic). Holdout-gate vs upstream: no contradiction. H23 runbook coherence: robust; runbook body uses only `_first` UDF forms → PASS. 44-commit provenance consistent.

---

## CI

45/45 pass @`0fbef7db`. Body "CI pending" = honest-lag under-claim.

---

## Novelty

MEDIUM — one genuinely new substantive finding in the fresh-lens target area (env-port plumbing); no re-tread.

---

## Verdict

CLEAN (strict): **no** (1 MED — resets streak 0/3) | CLEAN (PR-merge): **no** (MED is blocking)

**Recommendation:** implementer fix-burst (honor BASE_URL + docstring + body line 58); push resets streak; cascade restarts on new HEAD.

**Fix applied:** @`8d116f62` — BASE_URL > PORT > default precedence; 5-site TD-VSDD-060 sweep; docstring corrected; verified in both env modes (D-1861). Cascade restarted 0/3 on new frozen HEAD `8d116f62` pending CI.
