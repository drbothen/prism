---
document_type: architect-adjudication
adjudication_id: FB-PR-1-error-taxonomy-test-relocation
producer: architect
timestamp: "2026-05-19T00:00:00Z"
story: S-PLUGIN-PREREQ-E
pr: 151
fix_burst: FB-PR-1
related_bc: BC-2.16.011
related_ac: AC-11
inputs:
  - crates/prism-spec-engine/tests/error_taxonomy_annotation.rs
  - .factory/specs/prd-supplements/error-taxonomy.md
  - .factory/specs/behavioral-contracts/BC-2.16.011-customadapter-rust-trait-retirement.md
status: decided
---

# FB-PR-1 Architect Adjudication — error-taxonomy-test Relocation

## Decision

**Option 1:** Drop sub-assertion A (the `.factory/` filesystem read) from the Rust test.
Relocate the spec-governance invariant to a new `.factory/hooks/validate-error-taxonomy-retirement-annotations.sh`
validator. Sub-assertion B (the construction-site grep) remains in the Rust test unchanged.

## Rationale

### Why not Option 2 (snapshot fixture)

A committed snapshot at `crates/prism-spec-engine/tests/fixtures/error-taxonomy-snapshot.md`
would introduce a perpetual drift surface: every future change to `error-taxonomy.md`
(version bumps, new rows, row-level edits) would require a parallel update to the fixture.
The production-grade principle requires that invariants not create hidden maintenance landmines.
Snapshot drift is a class-B paper-fix: the test passes locally because the maintainer remembered
to update it, but CI catches only whether the two files byte-match, not whether the annotation
semantics are correct. This is strictly worse enforcement than the hook pattern, at higher cost.

### Why not Option 3 (skip-when-missing)

Rejected per CLAUDE.md Production-Grade Default, §Anti-pattern table row "Skip-when-missing."
No further analysis required.

### Why Option 1 is correct

The root cause of the CI failure is a category error in test-placement, not a weakness in the
invariant itself. Sub-assertion A enforces a spec-governance invariant ("`error-taxonomy.md`
contains a specific annotation string"). Spec-governance invariants have the wrong home in a
compiled test binary: they require filesystem access to a file that is not part of the source
tree checked out by CI.

ADR-027 §Decision establishes that the operational deletion mandate lives in `.factory/` spec
artifacts. BC-2.16.011 §Error Cases and §AC-11 both describe the invariant as a documentation
annotation on an error-taxonomy row — not as a runtime code property. The correct enforcement
layer for "a `.factory/` spec file contains a required annotation" is the `.factory/` hook
chain, not the Rust test suite.

Sub-assertion B ("no construction sites in `src/`") IS a code-side invariant — it asserts
that no live Rust source constructs `ESpec008`. That assertion belongs in a Rust test (or
compile-fail gate), because it operates on the source tree that CI checks out. Sub-assertion B
is correct-placement and must stay in the Rust test exactly as written.

Splitting the two sub-assertions along the code/spec boundary is clean, does not weaken either
invariant, and eliminates the CI gap without introducing a snapshot drift surface.

## Spec-Governance Invariant Relocation

The "BC-2.16.011 AC-11 retirement annotation in error-taxonomy.md" invariant re-enforces at:

**File:** `.factory/hooks/validate-error-taxonomy-retirement-annotations.sh`

**Trigger:** The hook is called from the `.factory/` pre-commit chain. Specifically, it should
be invoked in the same position as `verify-sha-currency.sh` — as a state-manager burst hygiene
gate. The state-manager dispatches it before any `.factory/` commit that touches
`specs/prd-supplements/error-taxonomy.md`. Additionally, it should run unconditionally on the
wave-gate check for wave-0-plugin-prereqs to verify PREREQ-E acceptance criteria.

**Check logic:**

```bash
#!/usr/bin/env bash
# validate-error-taxonomy-retirement-annotations.sh
# Verifies that all error codes declared RETIRED in error-taxonomy.md carry
# the required back-pointer fields: story ID, ADR reference.
#
# Current invariants enforced:
#   E-SPEC-008 row must contain:
#     - "RETIRED in S-PLUGIN-PREREQ-E"    (BC-2.16.011 AC-11)
#     - "ADR-027"                          (BC-2.16.011 AC-11 back-pointer)
#
# Exit codes:
#   0 — PASS
#   1 — FAIL (annotation absent or malformed)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FACTORY_DIR="$(dirname "$SCRIPT_DIR")"
TAXONOMY="$FACTORY_DIR/specs/prd-supplements/error-taxonomy.md"

if [ ! -f "$TAXONOMY" ]; then
  echo "FAIL: error-taxonomy.md not found at $TAXONOMY"
  exit 1
fi

FAIL=0

# E-SPEC-008 retirement annotation (BC-2.16.011 AC-11)
if ! grep -q "RETIRED in S-PLUGIN-PREREQ-E" "$TAXONOMY"; then
  echo "FAIL: BC-2.16.011 AC-11 — error-taxonomy.md missing 'RETIRED in S-PLUGIN-PREREQ-E' in E-SPEC-008 row"
  FAIL=1
fi

if ! grep -q "ADR-027" "$TAXONOMY"; then
  echo "FAIL: BC-2.16.011 AC-11 — error-taxonomy.md missing 'ADR-027' back-pointer in E-SPEC-008 retirement annotation"
  FAIL=1
fi

if [ "$FAIL" -eq 0 ]; then
  echo "PASS: error-taxonomy.md retirement annotations verified"
fi

exit "$FAIL"
```

**Registration:** The hook must be registered in the factory-dispatcher hooks-registry.toml
under a `pre-commit` trigger scoped to `.factory/specs/prd-supplements/error-taxonomy.md`
changes, AND as a standalone invocable script for wave-gate verification. The naming
convention matches the existing `verify-sha-currency.sh` pattern in `.factory/hooks/`.

**Running manually:**

```bash
bash .factory/hooks/validate-error-taxonomy-retirement-annotations.sh
```

Expected: exit 0 with `PASS: error-taxonomy.md retirement annotations verified`.

**Extensibility:** When future stories retire additional error codes, implementer adds a new
`grep -q` block to this script for each new retired code's required annotation strings.
The script accumulates invariants over time; it is NOT story-scoped. Each new invariant
block cites the BC and AC that mandates it (same pattern as the E-SPEC-008 block above).

## Implementer Instructions

### Change 1: Remove sub-assertion A from the Rust test

In `crates/prism-spec-engine/tests/error_taxonomy_annotation.rs`:

- Delete the `workspace_root()` helper function entirely (lines 29–36).
- Delete the entire "Sub-assertion A" block (lines 63–91 inclusive): the `taxonomy_path`
  binding, the `std::fs::read_to_string` call, and the two `assert!` calls checking for
  `"RETIRED in S-PLUGIN-PREREQ-E"` and `"ADR-027"`.
- Update the module-level doc comment table at line 6 to describe what the test now covers:
  sub-assertion B only (construction-site grep). Remove the word "annotation" from the
  failure mode column since that clause is now enforced by the hook.
- Update the function-level doc comment (lines 44–56) to remove the "Sub-assertion A"
  paragraph and update the description to reflect that only sub-assertion B remains.
- The `collect_e_spec_008_hits` helper function (lines 143–173) is UNCHANGED.
- The sub-assertion B block (lines 100–138) is UNCHANGED.
- The test still needs `use std::path::PathBuf;` at line 10 — keep that import.
  The `workspace_root()` call at line 59 is replaced by inline workspace root
  resolution using only the `crates_dir` needed for sub-assertion B:
  ```rust
  let crates_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
      .parent()  // crates/
      .expect("crates/ parent must exist")
      .parent()  // workspace root
      .expect("workspace root must exist")
      .join("crates");
  ```

### Change 2: Create the hook script

Create `.factory/hooks/validate-error-taxonomy-retirement-annotations.sh` with the
check logic specified above in "Spec-Governance Invariant Relocation". Make it executable
(`chmod +x`).

### Change 3: Update BC-2.16.011 AC-11 description (product-owner scope — route separately)

The AC-11 description in `S-PLUGIN-PREREQ-E.md` and in BC-2.16.011 §Error Cases currently
states that `test_BC_2_16_011_e_spec_008_retired_annotation` asserts the retirement annotation
in error-taxonomy.md. After this change, the Rust test only asserts the construction-site
absence. The annotation assertion moves to the hook.

The AC-11 text needs a one-sentence update to state: "The retirement annotation invariant
('RETIRED in S-PLUGIN-PREREQ-E' + 'ADR-027' present in the E-SPEC-008 row) is enforced by
`.factory/hooks/validate-error-taxonomy-retirement-annotations.sh`; the Red Gate test
`test_BC_2_16_011_e_spec_008_retired_annotation` enforces the code-side construction-site
absence gate."

This is product-owner scope. Route to product-owner for the BC and story text updates.
The implementer handles Changes 1 and 2 only.

### Change 4: Verify the hook passes locally

Before pushing, run:

```bash
bash .factory/hooks/validate-error-taxonomy-retirement-annotations.sh
```

This confirms the annotation is already present in the canonical `error-taxonomy.md` (which
it is, per v1.26 commit per FB-IMPL-6). If it fails, error-taxonomy.md has regressed
— investigate before pushing.

## Sibling-Sweep Required

TD-VSDD-060 sibling-sweep obligations triggered by this change:

1. **`workspace_root()` helper removal:** The function is defined and used only within
   `error_taxonomy_annotation.rs`. No other test file in `crates/prism-spec-engine/tests/`
   should share this helper (it is not in a `util.rs` or shared module). Implementer must
   grep `crates/prism-spec-engine/tests/` for `workspace_root` to confirm no sibling test
   imports it before removing.

2. **AC-11 Red Gate test 14 description in story frontmatter:** The frontmatter block at story
   line 72 describes AC-11's mitigation as asserting annotation presence. After Change 3
   above (product-owner scope), this line must be updated. Product-owner sweeps story line 72,
   BC-2.16.011 §Error Cases row for E-SPEC-008, and any other places AC-11's enforcement
   mechanism is described.

3. **No other test files reference `.factory/specs/prd-supplements/error-taxonomy.md`
   via filesystem path:** Implementer greps `crates/` for the string `error-taxonomy.md`
   in Rust source (excluding comments) to confirm no sibling test will suffer the same
   CI failure after this change. If other tests are found, they are candidates for the
   same relocation pattern and should be flagged to the orchestrator.
