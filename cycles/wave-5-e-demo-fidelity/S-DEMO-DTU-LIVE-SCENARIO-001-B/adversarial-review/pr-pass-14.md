---
document_type: adversarial-review-pass
pass: 14
level: PR-LEVEL
story: S-DEMO-DTU-LIVE-SCENARIO-001-B
pr: 185
head: 7ddc0a51
timestamp: 2026-06-13T00:00:00Z
clean_strict: false
clean_pr_merge: true
streak_before: 1/3
streak_after: 0/3
findings_count: 1
finding_ids: [BPRL-P14-01]
producer: adversary
---

# PR-LEVEL Adversarial Review — Pass 14

**Story:** S-DEMO-DTU-LIVE-SCENARIO-001-B  
**PR:** #185  
**HEAD:** 7ddc0a51 (CODE UNCHANGED — no new commits since D-1118)  
**Pass:** 14  
**Date:** 2026-06-13

---

## Result

**CLEAN(strict): NO**  
**CLEAN(PR-merge): YES**  
**Streak: RESET 1/3 → 0/3**

---

## Finding: BPRL-P14-01 — HIGH — SPEC-ONLY

### Title

Self-contradiction in BC-2.06.020 PC-9 + implementer directive + AC-019: RNG range literal `0..100000` produces 5-digit values incompatible with the spec's own `^CVE-9999-\d{4}$` invariant

### Severity

HIGH (spec inconsistency; would cause a literal implementer-follow to fail TV-020-011 ~90% of the time)

### Surface

SPEC-ONLY — the shipped code (`gen_device_cves` in `prism-dtu-common/src/scenario/mod.rs`) already uses `0..10000` (4-digit range, producing values matching `\d{4}`). No code change is needed or made.

### Description

BC-2.06.020 v1.3 contains a self-contradiction across three parallel artifact surfaces:

1. **PC-9 (line ~281 of BC-2.06.020):** States baseline CVEs use `CVE-9999-{:04}` format, and the invariant section (`INV-CYBERINT-ALERT-CVE-CORRELATION-001`) states all synthetic CVEs MUST match `^CVE-9999-\d{4}$` (exactly 4 digits).

2. **Implementer directive (line ~297 of BC-2.06.020):** States the RNG range as `rng.gen_range(0..100000)` — a 5-digit upper-bound producing values from 0 to 99999. This would yield 5-digit zero-padded strings like `CVE-9999-99999` whenever the value >= 10000 (~90% of the time with uniform random), violating `\d{4}`.

3. **Story B AC-019 (line ~410):** Carries the same `0..100000` literal from the implementer directive, propagated as-is from the BC.

4. **TV-020-011:** Asserts the generated CVE ID matches `^CVE-9999-\d{4}$`. A literal implementer-follow of the `0..100000` directive would cause TV-020-011 to fail ~90% of the time (values >= 10000).

**The shipped code is correct:** `gen_device_cves` uses `0..10000` (exclusive upper bound 10000, yielding values 0–9999, all format `{:04}` zero-padded to exactly 4 digits, satisfying `\d{4}`). The code correctly implements the `\d{4}` invariant. The defect exists only in the spec's own implementer directive and AC-019 literal.

### Classification

SPEC-ONLY. No feature-branch changes needed. PR diff is unchanged at 7ddc0a51.

### Verification

- `grep -n '100000\|10000' crates/prism-dtu-common/src/scenario/mod.rs` confirms the shipped code uses `0..10000`.
- BC-2.06.020 PC-9 + invariant + TV-020-011 together mandate `\d{4}` (4 digits).
- The `0..100000` literal in the implementer directive and AC-019 is inconsistent with `\d{4}`.

---

## Closure

**SPEC-ONLY — closed in D-1120 burst (same burst as this pass report).**

- **PO:** BC-2.06.020 v1.3 → v1.4 — PC-9 (implementer directive) `0..100000` → `0..10000`; behavior UNCHANGED (code was already correct).
- **Story-writer:** Story B v2.11 → v2.12 — AC-019 literal `0..100000` → `0..10000`; BC-2.06.020 pin v1.3 → v1.4; no AC or RGT count change (19 ACs / 23 RGTs unchanged).
- **Story-writer:** PIVOT-003 v1.5 → v1.6 — BC-2.06.020 pin v1.3 → v1.4.
- **Feature branch HEAD UNCHANGED at 7ddc0a51 = remote.** No code commit. No push needed.

---

## Other Surfaces Verified

- All prior do-not-reflag items confirmed still closed (BPRL-P1-01 through BPRL-P13 cosmetic nit).
- SAP-1 (tracing emission catalog completeness): PASS — no new `event_type =` emissions in diff.
- SAP-2 (DTU↔TOML schema parity): N/A — no sensor TOML changes in this diff.
- Dedup: exactly 1 `_resolves_in_nvd` test in workspace (VP-020-K; demo-server; confirmed).
- VP-020-K genuine integration test: confirmed `NvdState::lookup_and_count` + `Some(record)` + `base_score >= 7.0` + `request_count >= 1`.
- VP-020-J subsumes removed membership guard: confirmed no coverage gap.
- BC-2.06.019 v1.7 Route Coverage Table (8 rows, EXHAUSTIVE): all route coverage rows unchanged from pass 13 verification.
- BC-2.06.020 v1.3 invariants (pre-fix): contradictory literal identified; all other invariants PASS.
- Story B v2.11 sync: RGT #22 crate demo-server correct; AC-019 body present; FSR row present.
- Cargo required-features: correct.

---

## Streak History

| Pass | Result | Streak |
|------|--------|--------|
| 13 | CLEAN(strict)=YES | 1/3 |
| 14 | CLEAN(strict)=NO — BPRL-P14-01 HIGH SPEC-ONLY | RESET 0/3 |

**NEXT: PR-LEVEL pass 15** at HEAD 7ddc0a51 (diff UNCHANGED — reuse /tmp/pr185-pass13.diff or `gh pr diff 185`; specs corrected by D-1120 burst; no CI push needed).

**Do-not-reflag addition (D-1120):** BPRL-P14-01 CLOSED — BC-2.06.020 v1.4 PC-9 implementer directive now reads `0..10000`; story B AC-019 RNG range literal is `0..10000`; the `\d{4}` regex invariant, TV-020-011, and shipped code are all consistent. DO NOT re-raise "RNG range `0..100000` contradicts `\d{4}` regex" or "AC-019 range literal inconsistent with format invariant" — CLOSED.
