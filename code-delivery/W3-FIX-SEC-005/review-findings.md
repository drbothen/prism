# Review Findings: W3-FIX-SEC-005

**Story:** 5-DTU admin-token uniformity — constant-time comparison + post_reset gate
**PR:** #125 — fix/W3-FIX-SEC-005-dtu-admin-token-uniformity
**Base:** develop @ e4be29ae
**Review cycles:** 2
**Final verdict:** APPROVE (cycle 2)

---

## Convergence Table

| Cycle | Findings | Blocking | Fixed | Remaining |
|-------|----------|----------|-------|-----------|
| 1 | 3 | 1 | 1 | 0 |
| 2 | 0 | 0 | 0 | 0 → APPROVE |

---

## Cycle 1 Findings

### R1-001 — BLOCKING (resolved at fc467937)

**Category:** Code fix (CWE-208 fix incomplete)
**File:** `crates/prism-dtu-threatintel/src/routes/lookup.rs:308`
**Root cause:** ThreatIntel's `configure` handler lives in `lookup.rs`, not `dtu.rs`. The implementer applied `ct_eq` to `dtu.rs` only, missing the actual configure handler.
**Resolution:** `subtle::ConstantTimeEq` import added + `!=` replaced with `ct_eq` on byte slices at `lookup.rs:308`. Zero residual `!=` comparisons confirmed by grep.
**Routed to:** implementer (self-fix applied directly)
**Commit:** fc467937

### R1-002 — SUGGESTION (deferred)

**Category:** AC-006 deviation — `subtle = "2"` direct pin vs `subtle = { workspace = true }`
**Resolution:** Deferred — functional behavior identical, no security impact. Follow-up normalization acceptable.

### R1-003 through R1-006 — PASS (no action required)

- R1-003: All 4 other DTUs ct_eq verified — PASS
- R1-004: Cyberint gate before org-id branch — PASS
- R1-005: 9 new test files present and correct — PASS
- R1-006: Existing test fixtures updated to supply X-Admin-Token — PASS

---

## Cycle 2 Verification

- R1-001 fix confirmed: `lookup.rs` has `ct_eq` at line ~315
- grep across all 5 DTU crates: **0 residual `!=` comparisons**
- All 10 fix sites use `ct_eq`: cyberint(2), jira(2), nvd(2), pagerduty(2), threatintel(2)
- threatintel tests after fix: td_wv0_07(3/3 PASS), td_wv0_08(3/3 PASS)
- Pre-push hook (full workspace): PASS (181s)
- APPROVE issued

---

## AC Verification Summary (final)

| AC | Description | Status |
|----|-------------|--------|
| AC-001 | 0 residual `!=` in all 5 DTU post_configure | PASS (after R1-001 fix) |
| AC-002 | post_reset gate is first check in all 5 DTUs | PASS |
| AC-003 | correct token → 200 for all 5 DTUs | PASS (15/15 td_wv0_08) |
| AC-004 | 9 new test files present | PASS |
| AC-005 | workspace check passes | PASS |
| AC-006 | subtle dep in all 5 crates | PASS* (direct pin, functional equiv.) |

---

## Branch State at APPROVE

- Commits (newest first):
  - fc467937 fix(W3-FIX-SEC-005): apply ct_eq to ThreatIntel configure handler in lookup.rs (R1-001)
  - 2b62f313 chore(W3-FIX-SEC-005): demo evidence per POL-010 AC-001..006
  - 3f371eb4 fix(W3-FIX-SEC-005): admin-token uniformity across 5 DTU clones — ct_eq + post_reset gate (CR-021/022)
  - 6b93086c chore(W3-FIX-SEC-005): update Cargo.lock for subtle = "2" in 5 DTU crates
  - f30f9789 test(W3-FIX-SEC-005): red gate — 9 admin-token regression tests across 5 DTU clones
- Base: e4be29ae (develop)
- 5 commits ahead of develop
