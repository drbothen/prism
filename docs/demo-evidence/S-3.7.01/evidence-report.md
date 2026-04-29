# Demo Evidence Report — S-3.7.01

**Story:** Archetype catalog + GenOpts API (`prism-dtu-common` generator module, D-056)
**Impl SHA:** `250a2609`
**Recorded:** 2026-04-28
**Tests:** 39/39 GREEN (workspace 1522/1522 unchanged)

---

## Coverage Map

### Group 1 — Archetype Catalog + GenOpts Shape

| BC | VP | AC | Recording |
|----|----|----|-----------|
| BC-3.4.003 | VP-115 | AC-001: 8 archetypes present, no duplicates | [BC-3.4.003-VP-115-archetype-tests-green.gif](BC-3.4.003-VP-115-archetype-tests-green.gif) |
| BC-3.4.003 | VP-115 | AC-005: `default_page_size` non-zero for all sensors | same |
| BC-3.4.001 | — | AC-002: `GenOpts` defaults (seed=42, scale=1, epoch, overrides=null) | same |
| BC-3.4.001 | — | AC-002: `GenOpts::new` validates scale (rejects zero/negative/nan/inf) | same |
| BC-3.4.001 | — | AC-004: `apply_overrides` merges/removes keys deterministically | same |
| BC-3.4.001 | — | AC-006: `FixtureSet` fields accessible; provenance schema_valid logic | same |

**Recording:** `BC-3.4.003-VP-115-archetype-tests-green.gif` (288 KB)
**Tape:** `BC-3.4.003-VP-115-archetype-tests-green.tape`
**Command:** `cargo test -p prism-dtu-common --features fixture-gen --test bc_3_4_001_003_archetype_genopts`
**Result:** 39 tests, 0 failures

---

### Group 2 — Generator Determinism

| BC | VP | Property | Recording |
|----|----|----------|-----------|
| BC-3.4.001 | VP-108 | Same seed+org_id produces byte-identical RNG stream | [BC-3.4.001-VP-108-111-116-117-determinism.gif](BC-3.4.001-VP-108-111-116-117-determinism.gif) |
| BC-3.4.001 | VP-111 | Distinct seeds produce distinct first-draw values | same |
| BC-3.4.001 | VP-116 | Distinct org_ids produce distinct RNG streams | same |
| BC-3.4.001 | VP-117 | `seeded_rng` callable only under `fixture-gen` feature; invariant documented | same |

**Recording:** `BC-3.4.001-VP-108-111-116-117-determinism.gif` (181 KB)
**Tape:** `BC-3.4.001-VP-108-111-116-117-determinism.tape`
**Command:** `cargo test -p prism-dtu-common --features fixture-gen --test bc_3_4_001_003_archetype_genopts -- vp_108 vp_111 vp_116 vp_117`
**Result:** 8 tests (all VP-named tests for 108/111/116/117), 0 failures

---

## Error Path Coverage

| Scenario | Covered By |
|----------|-----------|
| `GenOpts::new` rejects `scale=0` | AC-002 tests in Group 1 recording |
| `GenOpts::new` rejects negative scale | AC-002 tests in Group 1 recording |
| `GenOpts::new` rejects NaN scale | AC-002 tests in Group 1 recording |
| `GenOpts::new` rejects infinite scale | AC-002 tests in Group 1 recording |
| `provenance.schema_valid = false` on schema drift | AC-006 test in Group 1 recording |

---

## Files

| File | Size |
|------|------|
| `BC-3.4.003-VP-115-archetype-tests-green.gif` | 288 KB |
| `BC-3.4.003-VP-115-archetype-tests-green.webm` | 589 KB |
| `BC-3.4.003-VP-115-archetype-tests-green.tape` | 634 B |
| `BC-3.4.001-VP-108-111-116-117-determinism.gif` | 181 KB |
| `BC-3.4.001-VP-108-111-116-117-determinism.webm` | 366 KB |
| `BC-3.4.001-VP-108-111-116-117-determinism.tape` | 699 B |
