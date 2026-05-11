---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-05-10T19:45:00Z
phase: 5
inputs:
  - .factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md
input-hash: "f16cd17"
traces_to: prd.md
pass: 21
target_sha: 2fe48fd1
previous_review: ADR-023-pass-20.md
target_version: v1.15
findings_total: 3
findings_by_severity:
  critical: 0
  high: 1
  medium: 2
  low: 0
  obs: 0
residuals: 0
new: 3
streak: "0/3 RESET"
trajectory: "26→16→12→14→3→3→1→0→0→4→2→0→1→1→4→3→2→2→0→0→3"
verifications: 30+
---

# Adversarial Review: ADR-023 Plugin-Only Sensor Architecture (Pass 21)

## Finding ID Convention

Finding IDs use the format: `F-PASS21-<SEV>-<SEQ>`

- `F-PASS21`: Pass 21 prefix
- `<SEV>`: Severity abbreviation (`HIGH`, `MED`, `LOW`)
- `<SEQ>`: Three-digit sequence within the pass

## Part A — Fix Verification (pass >= 2 only)

All pass-20 findings verified. Pass-20 was CLEAN (zero findings). No residuals to verify.

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| Pass-20 | N/A — CLEAN | RESOLVED | Pass-20 surfaced zero findings across 25 verifications. Idempotency confirmed. |

## Part B — New Findings (or all findings for pass 1)

### CRITICAL

(none)

### HIGH

#### F-PASS21-HIGH-001: Numerical contradiction — "five" vs "four" hardcoded sensor auth modules

- **Severity:** HIGH
- **Category:** contradictions
- **Location:** L864, `### Status as of 2026-05-10` section
- **Description:** L864 states "The five hardcoded sensor auth modules" but Context L110-111
  states "four sensor-named Rust auth modules at `crates/prism-sensors/src/auth/`". The
  `mod.rs` file in `crates/prism-sensors/src/auth/` is the trait definition file (`SensorAuth`
  sealed trait, `private::Sealed`, `Credentials` newtype) — not a sensor auth module. Counting
  `mod.rs` as a fifth sensor auth module is factually incorrect.
- **Evidence:** Context L110-111: "four sensor-named Rust auth modules" (established ground
  truth). PLUGIN-AUDIT-001 (referenced L100-101): four sensor-named files catalogued —
  `armis.rs`, `claroty.rs`, `crowdstrike.rs`, `cyberint.rs`. `mod.rs` contains the sealed
  trait infrastructure, not sensor-specific authentication logic. Internal contradiction:
  Context says four, Status says five. The correct count is four.
- **Proposed Fix:** L864 "The five hardcoded sensor auth modules" → "The four hardcoded sensor
  auth modules".

### MEDIUM

#### F-PASS21-MED-001: C1 PREREQ-A crate enumeration cites two wrong crates

- **Severity:** MEDIUM
- **Category:** spec-fidelity
- **Location:** L521-522, `**C1 — SensorId newtype (PLUGIN-PREREQ-A):**` block
- **Description:** C1 lists the four crates containing `match SensorType::X` arms as
  `prism-sensors`, `prism-spec-engine`, `prism-query`, `prism-mcp`. Two of these are wrong.
  `prism-spec-engine` has zero `SensorType` references (confirmed in fix-burst-2 for
  F-CRIT-NEW-001-PASS2-RESIDUAL). `prism-mcp` is a 10-line stub with no dispatch logic
  (verified 2026-05-08). The correct four crates per PLUGIN-AUDIT-001 source-of-truth are
  `prism-core` (SensorType enum definition), `prism-sensors` (auth + adapter), `prism-query`
  (virtual field dispatch), and `prism-ocsf` (four per-sensor mapper modules in
  `crates/prism-ocsf/src/mappers/`).
- **Evidence:** Pass-2 fix record: "spec_parser.rs contains zero CustomAdapter references
  (grep confirmed)" — same property extends to SensorType. ARCH-INDEX AD-005 annotation:
  "prism-mcp is a 10-line stub (verified 2026-05-08); no rmcp dep in Cargo.toml". Rule 1
  body: "The four per-sensor mapper modules in `crates/prism-ocsf/src/mappers/` will be
  retired" — confirms prism-ocsf has SensorType-dependent dispatch that C1 must address.
- **Proposed Fix:** L521-522 replace `prism-sensors`, `prism-spec-engine`, `prism-query`,
  `prism-mcp` with `prism-core`, `prism-sensors`, `prism-query`, `prism-ocsf` per
  PLUGIN-AUDIT-001 source-of-truth crate enumeration.

#### F-PASS21-MED-002: ARCH-INDEX ADR Registry missing ADR-023 row (sibling-file gap)

- **Severity:** MEDIUM
- **Category:** missing-edge-cases
- **Location:** `.factory/specs/architecture/ARCH-INDEX.md`, ADR Registry table
- **Description:** ADR-023 has been COMMITTED status since v1.0 (2026-05-10). The ARCH-INDEX
  ADR Registry table runs ADR-001 through ADR-022. ADR-023 is absent after 21 adversarial
  passes. Every prior fix-burst applied changes to the ADR-023 body but never propagated
  ADR-023's existence to the sibling ARCH-INDEX file. This is a classic S-7.01 sibling-file
  partial-fix gap: a document is authored but its registry entry is never created.
- **Evidence:** ARCH-INDEX ADR Registry ends at ADR-022 (ACCEPTED v1.1, 2026-05-08). ADR-023
  was authored 2026-05-10 (same day, 21 passes ago). Any agent or reviewer reading ARCH-INDEX
  to enumerate all ADRs will miss ADR-023 — the most architecturally significant decision in
  the current sprint.
- **Proposed Fix:** Add row to ARCH-INDEX ADR Registry after ADR-022:
  `| ADR-023 | Plugin-Only Sensor Architecture — TOML Specs as Declarative Baseline, .prx WASM
  for Non-Declarative Cases, Retired CustomAdapter Rust Trait | COMMITTED v1.16 | 2026-05-10 |
  decisions/ADR-023-plugin-only-sensor-architecture.md |`

### LOW

(none)

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 1 |
| MEDIUM | 2 |
| LOW | 0 |

**Overall Assessment:** block
**Convergence:** FINDINGS_REMAIN — iterate with fix-burst-16 then pass-22
**Readiness:** Requires revision. Fix-burst-16 closes all 3 findings as ADR-023 v1.16.

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 21 |
| **New findings** | 3 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 3 / (3 + 0) = 1.0 |
| **Median severity** | MEDIUM (2 MED + 1 HIGH) |
| **Trajectory** | 26→16→12→14→3→3→1→0→0→4→2→0→1→1→4→3→2→2→0→0→3 |
| **Verdict** | FINDINGS_REMAIN |

Pass-21 demonstrates the "fresh-context compounding value" principle: each increase in rigor
(30+ verifications vs pass-20's 25) surfaces new defect axes. The HIGH finding exploits a
cross-section numerical comparison (Context vs Status) not included in any prior verification
checklist. The first MED finding extends a pass-2 fix (crate enumeration correction) to a
different section that cited the same wrong crates. The second MED finding is a pure
sibling-file registry check omitted across all 21 passes. Streak RESET 2/3 → 0/3 per HIGH
severity. Fix-burst-16 closes all 3; pass-22 targets streak 1/3.
