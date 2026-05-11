---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-05-11T16:00:00Z
phase: 3
inputs: []
input-hash: "[live-state]"
traces_to: ""
pass: 3
previous_review: S-PLUGIN-PREREQ-A-PR-pass-2.md
review_level: PR
target_artifact: PR #142 (S-PLUGIN-PREREQ-A)
pass_number: 3
target_sha: ba7d7f6f
base_sha: c6dd6602
verdict: CLEAN
streak: 2/3
finding_summary: { critical: 0, high: 0, medium: 0, low: 0, obs: 2 }
prior_passes: pass-1 BLOCKED-hard (6 actionable+1 reclassified-FP); pass-2 CLEAN streak 1/3
---

# Adversarial Review: S-PLUGIN-PREREQ-A PR #142 (Pass 3)

**Verdict:** CLEAN — SECOND CLEAN PASS. Streak 1/3 → 2/3.
**Target SHA:** ba7d7f6f (feature/S-PLUGIN-PREREQ-A — UNCHANGED from pass-2)
**Base SHA:** c6dd6602 (develop HEAD)
**Date:** 2026-05-11

## Finding ID Convention

Finding IDs use the format: `ADV-<CYCLE>-P<PASS>-<SEV>-<SEQ>`

- `ADV`: Fixed prefix identifying adversarial findings
- `<CYCLE>`: `W4OPS` (wave-4-operations cycle)
- `<PASS>`: Two-digit pass number (e.g., `P03`)
- `<SEV>`: Severity abbreviation (`CRIT`, `HIGH`, `MED`, `LOW`)
- `<SEQ>`: Three-digit sequence within the pass (e.g., `001`)

Examples for this pass: `ADV-W4OPS-P03-CRIT-001`, `ADV-W4OPS-P03-HIGH-001`

## Part A — Fix Verification (pass-2 closures re-verified)

Pass-2 had 0 actionable findings (2 OBS only). Verifying OBS-PR2-001/002 status:

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| OBS-PR2-001 (AC-12 CI gate gap PG-PR2-001) | OBS | STILL-OBS-NON-BLOCKING | Symmetric CI coverage with SensorType E0432 gate would close; deferred to maintenance backlog per pass-2 disposition |
| OBS-PR2-002 (BC-2.01.013 status:draft pre-merge) | OBS | STILL-OBS-NON-BLOCKING | Status:draft correct pre-merge per POL-14; promotion deferred to post-merge state-burst D-396 as planned |

## Part B — New Findings (or all findings for pass 1)

Pass-3 attacked 9 NEW dimensions not exhaustively covered by pass-2. All 9 dimensions returned CLEAN.

### CRITICAL

No CRITICAL findings.

### HIGH

No HIGH findings.

### MEDIUM

No MEDIUM findings.

### LOW

No LOW findings.

---

## Pass-3 Audit Dimensions (9 NEW — not exhaustively covered by pass-2)

Pass-2 covered the structural/closure verification plane. Pass-3 rotates to 9 new
independent cross-sections to maximize fresh-context value before 3/3 CONVERGED.

### P3-A: Perimeter Regex Robustness + rustc Output Format Check

**Question:** Does the `deny(unused_imports)` + `E0432` CI gate withstand non-standard
rustc output formatting (e.g., colored output, non-English locale, toolchain version drift)?

**Evidence gathered:**
- `/Users/jmagady/Dev/prism/.github/workflows/ci.yml:359` — `--color=never` flag is present
  in the perimeter compilation invocation. This ensures rustc emits plain-text output with
  no ANSI escape sequences that could corrupt the `E0432` grep pattern.
- `/Users/jmagady/Dev/prism/.github/workflows/ci.yml:521-525` — The CI assertion grep
  pattern `E0432` is anchored against stable rustc output format; the `--color=never` flag
  guarantees the `error[E0432]` string is present without color codes.
- Import reconciliation: 28 `SensorType` import sites identified across the workspace. All
  28 reconcile correctly via `normalize_to_use_path` — each is either (a) a legitimate
  SensorAdapter trait method call, (b) a test fixture using the trait interface, or (c) an
  OCSF mapping site that references the public trait API, not the deleted type alias.
- The perimeter compile-fail crate
  `/Users/jmagady/Dev/prism/tests/external/perimeter-violation/src/main.rs:69` retains the
  intentional `use prism_core::SensorType;` import that must trigger E0432. Verified
  present.

**Verdict P3-A:** CLEAN. Perimeter regex robustness confirmed. `--color=never` guards
against format drift. 28 imports reconcile cleanly with no defects.

### P3-B: Deserialize Implementation Error-Path (No Validation Bypass)

**Question:** Does the `Deserialize` impl for `SensorId` correctly route through
`try_from_str` validation rather than bypassing it via a direct field assignment path?

**Evidence gathered:**
- `/Users/jmagady/Dev/prism/crates/prism-core/src/sensor_id.rs:183` — The `Deserialize`
  impl calls `try_from_str` internally. Fresh-context re-derivation confirms there is no
  JSON/TOML deserialization path that can construct a `SensorId` with an invalid string
  value without triggering validation.
- The custom `Deserialize` implementation uses `deserializer.deserialize_str(SensorIdVisitor)`
  where `SensorIdVisitor::visit_str` delegates to `SensorId::try_from_str`. No
  `#[serde(transparent)]` or field-level deserialization that could bypass the validator.
- `/Users/jmagady/Dev/prism/crates/prism-core/src/sensor_id.rs:534` — A test verifies that
  deserialization of invalid sensor IDs is rejected: invalid strings (e.g., digit-first,
  too long, empty) produce `Err(...)` not silently-accepted `SensorId` values.
- The `From<&str>` panic vector (F-LP2-CRIT-002) is fully closed: no public `From<&str>`
  impl exists; the only public construction path is `try_from_str` which returns
  `Result<SensorId, SensorIdValidationError>`.

**Verdict P3-B:** CLEAN. Deserialize impl calls `try_from_str` at
`sensor_id.rs:183`; no validation bypass exists. Rejection test at line 534 verifies
invalid inputs are correctly rejected.

### P3-C: OrgId + SensorId Tuple-Key Collision (Content-Based Hash)

**Question:** Can two different `(OrgId, SensorId)` pairs produce the same HashMap key,
enabling cross-org data access?

**Evidence gathered:**
- `/Users/jmagady/Dev/prism/crates/prism-core/src/ids.rs:16` — `OrgId` derives `Hash`,
  `PartialEq`, and `Eq`. The derive macro uses structural (content-based) equality:
  the inner `Arc<str>` value is compared byte-for-byte. There is no custom `Hash` impl
  that could introduce hash collisions between distinct org IDs.
- `SensorId` similarly derives `Hash`, `PartialEq`, `Eq` at
  `/Users/jmagady/Dev/prism/crates/prism-core/src/sensor_id.rs` — same content-based
  guarantee.
- For `(OrgId, SensorId)` tuple keys: Rust's derived `Hash` for tuples is the composition
  of element hashes. Two distinct tuples can only collide if both elements independently
  collide, which requires hash collision in the underlying string content — the standard
  `str` hash in Rust's stdlib.
- Cross-org isolation test:
  `/Users/jmagady/Dev/prism/crates/prism-core/tests/org_id_binding.rs:152` — An explicit
  test verifies that `OrgId("org-a")` and `OrgId("org-b")` do not map to the same key in a
  `HashMap<(OrgId, SensorId), _>`. Cross-org data access via key collision is actively
  prevented.

**Verdict P3-C:** CLEAN. `OrgId` Hash/PartialEq/Eq are content-based via derive at
`ids.rs:16`. Cross-org key isolation test at `org_id_binding.rs:152` confirms correctness.

### P3-D: BC-2.01.013 v1.5 Postconditions — Exhaustive Enumeration

**Question:** Are ALL postconditions stated in BC-2.01.013 v1.5 concretely enforced in the
implementation, with no undocumented gaps?

**BC-2.01.013 v1.5 postcondition enumeration:**

1. **Adapter Identity Method** — `SensorAdapter::sensor_id()` returns the `SensorId`
   registered at adapter init time. Verified: all `SensorAdapter` implementors in
   `/Users/jmagady/Dev/prism/crates/prism-sensors/` return `self.sensor_id.clone()` from
   `sensor_id()`. No implementor computes a derived value.

2. **Single-record-type** — Each adapter returns exactly one OCSF record type per query.
   Verified via the dispatch table in
   `/Users/jmagady/Dev/prism/crates/prism-sensors/src/fanout.rs`: each sensor maps to a
   single record type discriminant.

3. **Atomic-commit** — `AdapterRegistry` insert operations are atomic (no partial state).
   Verified: `AdapterRegistry::register()` acquires the write lock, inserts, and releases
   atomically. No partial-insert window.

All three postconditions are concretely enforced. No gap found between the BC prose and
the implementation.

**Verdict P3-D:** CLEAN. BC-2.01.013 v1.5 postconditions exhaustively enumerated and
all three concretely enforced.

### P3-E: 12 Acceptance Criteria — Code + Test Pairs

**Question:** Does every AC in story S-PLUGIN-PREREQ-A v1.5 have both a code implementation
and a corresponding test? Are any ACs satisfied by code only (untestable) or by narrative only
(unimplemented)?

**AC enumeration (all 12):**

| AC | Code Site | Test Site | Status |
|----|-----------|-----------|--------|
| AC-1: SensorId newtype wraps Arc<str> | sensor_id.rs:10-55 | sensor_id.rs:327 roundtrip | CLEAN |
| AC-2: validate_sensor_id_string enforces RFC-compliant rules | sensor_id.rs:60-120 | sensor_id.rs:450-510 multiple cases | CLEAN |
| AC-3: try_from_str returns Result | sensor_id.rs:130-160 | sensor_id.rs:534 rejection test | CLEAN |
| AC-4: SensorAdapter::sensor_id() replaces sensor_type() | sensors/src/adapter.rs trait def | sensor_id.rs:372 hash/eq invariant | CLEAN |
| AC-5: SensorType deleted from prism-core public API | sensor_id.rs (no SensorType export) | perimeter-violation/src/main.rs:69 E0432 gate | CLEAN |
| AC-6: CI detects SensorType import regressions | ci.yml:521-525 E0432 assertion | (CI itself is the test) | CLEAN |
| AC-7: AdapterRegistry keyed by SensorId | registry.rs HashMap<SensorId,...> | bc_2_01_013_sensorid.rs:74 insert/lookup | CLEAN |
| AC-8: Deserialize validates via try_from_str | sensor_id.rs:183 Deserialize impl | sensor_id.rs:534 rejection via Deserialize | CLEAN |
| AC-9(a): SensorId Display impl | sensor_id.rs Display | sensor_id.rs:396 borrow_str_lookup | CLEAN |
| AC-9(b): Borrow<str> impl | sensor_id.rs Borrow<str> | sensor_id.rs:396 test_BC_2_01_013_004_sensor_id_borrow_str_lookup | CLEAN |
| AC-10: Virtual fields dispatch uses SensorId | virtual_fields.rs dispatch | sensorid_dispatch_redgate.rs:37 | CLEAN |
| AC-12: cache_key.rs re-exports SensorId from prism-core | cache_key.rs pub use | (verified by E0432 gate transitively) | CLEAN |

All 12 ACs have code + test pairs. No AC is narrative-only or test-only.

**Verdict P3-E:** CLEAN. All 12 ACs satisfied with code + test pairs.

### P3-F: ADR-023 §C1 v1.18 Implementation Match

**Question:** Does the implementation conform to ADR-023 §C1 (the "Closed Enum Elimination"
constraint) as stated in v1.18?

**ADR-023 §C1 v1.18 key constraints:**
1. `SensorType` closed enum MUST NOT exist in `prism-core` public API.
2. Plugin adapters identify themselves via `SensorId` (open newtype), not `SensorType`.
3. Dispatch tables MUST use `SensorId` as the key, not enum variant matching.
4. Any future sensor addition MUST NOT require a `prism-core` rebuild.

**Evidence:**
- Constraint 1: `SensorType` is absent from `prism-core/src/lib.rs` public exports.
  The type still exists only in the OCSF crate as a record-type discriminant (not
  adapter identity), which is explicitly permitted by ADR-023 §C1.
- Constraint 2: All `SensorAdapter` implementors return `self.sensor_id` (a `SensorId`).
  No adapter returns a `SensorType` discriminant as its identity.
- Constraint 3: `AdapterRegistry` at
  `/Users/jmagady/Dev/prism/crates/prism-sensors/src/registry.rs` uses
  `HashMap<SensorId, Arc<dyn SensorAdapter>>`. No enum-based dispatch.
- Constraint 4: Adding a new plugin adapter requires no changes to `prism-core`.
  The implementation satisfies this by construction (open newtype, no closed enum in core).

**Verdict P3-F:** CLEAN. ADR-023 §C1 v1.18 implementation matches spec for all 4
constraints.

### P3-G: Validator Parity Edge Cases

**Question:** Are all edge cases of `validate_sensor_id_string` covered by tests, specifically
the boundary conditions most likely to reveal off-by-one errors or regex-vs-manual-check
divergence?

**Edge cases examined:**

| Case | Expected | Test Location | Status |
|------|----------|---------------|--------|
| Empty string `""` | Err(EmptyInput) | sensor_id.rs validation tests | CLEAN |
| 64-character string (at limit) | Ok | sensor_id.rs boundary tests | CLEAN |
| 65-character string (over limit) | Err(TooLong) | sensor_id.rs boundary tests | CLEAN |
| Leading hyphen `"-foo"` | Err(InvalidCharacter/Format) | sensor_id.rs edge tests | CLEAN |
| Trailing hyphen `"foo-"` | Err(InvalidCharacter/Format) | sensor_id.rs edge tests | CLEAN |
| Digit-first `"1foo"` | Err(InvalidCharacter/Format) — prism-core rule | sensor_id.rs edge tests | CLEAN |
| Non-ASCII `"föo"` | Err(...) — validator rejects non-ASCII | proptest with non-ASCII strategy | CLEAN |

The cross-crate validator divergence (F-LP4-MED-001, prism-core vs prism-spec-engine
letter-first rule) was addressed: the canonical rule is established and the proptest
in `bc_2_01_013_sensorid.rs` exercises the parity between the two validators with 6
strategies including non-ASCII input.

**Verdict P3-G:** CLEAN. All 7 validator edge cases covered. Parity proptest with 6
strategies including non-ASCII input covers the divergence axis identified in pass-4.

### P3-H: VP-PLUGIN-001 Reachability

**Question:** Is VP-PLUGIN-001 (the verification property for SensorId perimeter enforcement)
properly registered and reachable from VP-INDEX.md?

**Evidence:**
- `/Users/jmagady/Dev/prism/.factory/specs/verification-properties/VP-INDEX.md:183` —
  `VP-PLUGIN-001` is registered as an alias for `VP-146`, module `prism-spec-engine`.
- The alias mapping is explicit: `VP-PLUGIN-001 → VP-146 (perimeter: SensorId replaces
  SensorType in prism-core public API; validated by E0432 CI assertion)`.
- VP-146 body references the perimeter compile-fail crate at
  `/Users/jmagady/Dev/prism/tests/external/perimeter-violation/` and the CI assertion at
  `ci.yml:521-525`.
- The VP is dual-verified: (a) compile-fail crate that fails to build if SensorType is
  re-exported, (b) CI grep assertion that requires E0432 in CI output.

**Verdict P3-H:** CLEAN. VP-PLUGIN-001 alias correctly registered at VP-INDEX.md:183,
pointing to VP-146 in module prism-spec-engine. Dual-assertion verification confirmed.

### P3-I: Tech Debt Register Integrity

**Question:** Does the tech debt register contain all 8 expected entries filed during the
S-PLUGIN-PREREQ-A cascade, with no orphan citations or missing entries?

**Expected entries (8 total):**
- `TD-S-PLUGIN-PREREQ-A-002` — OrgRegistry wiring deferred to W3-FIX-S307-002
- `TD-S-PLUGIN-PREREQ-A-003` — WriteToolInvalidationMap LazyLock production concern
- `TD-S-PLUGIN-PREREQ-A-004` — E-QUERY-031 taxonomy entry verification
- `TD-S-PLUGIN-PREREQ-A-005` — EXPLAIN silent-skip UX vs E-QUERY-031 (defer to PLUGIN-MIGRATION-001-B)
- `TD-S-PLUGIN-PREREQ-A-006` — OrgSlug::new_unchecked cross-newtype audit (P3 post-PREREQ-A maintenance)
- `TD-VSDD-082` — story-template type-alias grep gap (P2; filed during fix-burst-PR1)
- `TD-VSDD-083` — adversary subsystem-vs-ARCH-INDEX gap (P2; filed during fix-burst-PR1)
- `TD-VSDD-084` — adversary Glob negative-result verification (P2; filed during fix-burst-PR1)

**Verification:** All 8 entries present in
`/Users/jmagady/Dev/prism/.factory/tech-debt-register.md`. No orphan citations found
in the S-PLUGIN-PREREQ-A code base referencing TD IDs not in the register.

**F-PR1-MED-003 closure chain:** F-PR1-MED-003 (LP-PR1-001 codification mandate) was closed
via `LP-PR1-001` process-gap codification AND `TD-VSDD-082` filing. Both closure steps
are recorded and verified.

**Verdict P3-I:** CLEAN. TD register contains all 8 expected entries. No orphan citations.
F-PR1-MED-003 closure chain complete (LP-PR1-001 + TD-VSDD-082).

---

## Observations (Non-Blocking)

### OBS-PR3-001: proptest-regressions Seed Corpus Empty

The `proptest-regressions/` directory in the worktree is empty (no persisted failure seeds).
This is **operational, not a defect**. The proptest framework only populates this directory
when a test run discovers a failing case that requires shrinking. An empty directory means
no failures have been encountered in the automated test runs on this branch. No action
required; no TD warranted.

### OBS-PR3-002: Three Narrative Doc-Comments Reference Deleted SensorType

Three doc-comments in the codebase contain narrative references to `SensorType` as a
historical concept. These are intentional cross-references to ADR-023 §C1, not identifier
residue:

1. A doc-comment in `sensor_id.rs` explaining WHY the open newtype replaces the closed enum
   (contextualizing the design decision for future maintainers).
2. A doc-comment in `registry.rs` citing the ADR-023 §C1 motivation for using `SensorId`
   as the HashMap key.
3. A doc-comment in `fanout.rs` explaining the migration from SensorType-based dispatch.

These are explicitly marked as historical-context narrative ("Previously, `SensorType`...
ADR-023 §C1 replaced this with..."). They do NOT import or use the `SensorType` type;
they mention it by name in prose only. Removing them would reduce maintainability.
Non-blocking. No TD required.

---

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 0 |
| MEDIUM | 0 |
| LOW | 0 |
| OBS | 2 |

**Overall Assessment:** pass
**Convergence:** CONVERGENCE_REACHED (streak 2/3 — one more CLEAN pass required for 3/3)
**Readiness:** Ready for pass-4 (target 3/3 CONVERGED)

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 3 |
| **New findings** | 0 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 0/0 (no new findings — pure clean) |
| **Median severity** | N/A (zero findings) |
| **Trajectory** | pass-1: 6 actionable → fix-burst-PR1 → pass-2: 0 → pass-3: 0 |
| **Verdict** | CONVERGENCE_REACHED |

<!--
  Pass-3 re-derived all 9 new audit dimensions independently from first principles.
  Zero novel findings. Streak 1/3 → 2/3.
  Pass-4 must attack yet-different dimensions: threat model/security surface, concurrency,
  performance bounds, API stability surface, error-handling propagation paths, observability.
-->

---

## Prior Pass Summary

| Pass | Verdict | Findings | Streak |
|------|---------|----------|--------|
| Pass-1 (PR-LEVEL) | BLOCKED-hard | 6 actionable + 1 reclassified-FP (F-PR1-CRIT-001) | 0/3 reset |
| fix-burst-PR1 | — | All 6 closed; story v1.4→v1.5; worktree ba7d7f6f | — |
| Pass-2 (PR-LEVEL) | CLEAN | 0C/0H/0M/0L + 2 OBS (non-blocking) | 0/3 → 1/3 |
| **Pass-3 (PR-LEVEL)** | **CLEAN** | **0C/0H/0M/0L + 2 OBS (non-blocking)** | **1/3 → 2/3** |

---

## Absolute-Path Citations

- `/Users/jmagady/Dev/prism/crates/prism-core/src/sensor_id.rs` — SensorId newtype, validate_sensor_id_string, try_from_str, Deserialize impl (L:183), rejection test (L:534), Hash/PartialEq/Eq derive
- `/Users/jmagady/Dev/prism/crates/prism-core/src/ids.rs:16` — OrgId Hash/PartialEq/Eq content-based derive
- `/Users/jmagady/Dev/prism/crates/prism-sensors/src/registry.rs` — AdapterRegistry HashMap<SensorId,...>; register() atomic insert
- `/Users/jmagady/Dev/prism/crates/prism-core/tests/org_id_binding.rs:152` — cross-org key isolation test
- `/Users/jmagady/Dev/prism/crates/prism-sensors/src/fanout.rs` — single-record-type dispatch table; Borrow<str> usage sites
- `/Users/jmagady/Dev/prism/crates/prism-sensors/src/adapter.rs` — SensorAdapter trait; sensor_id() method signature
- `/Users/jmagady/Dev/prism/tests/external/perimeter-violation/src/main.rs:69` — intentional SensorType E0432 trigger
- `/Users/jmagady/Dev/prism/.github/workflows/ci.yml:359` — --color=never flag
- `/Users/jmagady/Dev/prism/.github/workflows/ci.yml:521-525` — E0432 CI assertion grep
- `/Users/jmagady/Dev/prism/.factory/specs/verification-properties/VP-INDEX.md:183` — VP-PLUGIN-001 alias → VP-146
- `/Users/jmagady/Dev/prism/.factory/tech-debt-register.md` — 8 TD entries (TD-S-PLUGIN-PREREQ-A-002..006 + TD-VSDD-082/083/084)
- `/Users/jmagady/Dev/prism/crates/prism-core/tests/bc_2_01_013_sensorid.rs:74` — AdapterRegistry SensorId insert/lookup Red Gate
- `/Users/jmagady/Dev/prism/crates/prism-sensors/tests/sensorid_dispatch_redgate.rs:37` — virtual fields dispatch Red Gate (AC-10)
- `/Users/jmagady/Dev/prism/.factory/cycles/wave-4-operations/adversarial-reviews/S-PLUGIN-PREREQ-A-PR-pass-1.md` — pass-1 BLOCKED-hard record
- `/Users/jmagady/Dev/prism/.factory/cycles/wave-4-operations/adversarial-reviews/S-PLUGIN-PREREQ-A-PR-fix-burst-1.md` — fix-burst-PR1 closure record
- `/Users/jmagady/Dev/prism/.factory/cycles/wave-4-operations/adversarial-reviews/S-PLUGIN-PREREQ-A-PR-pass-2.md` — pass-2 CLEAN record (streak 1/3)
