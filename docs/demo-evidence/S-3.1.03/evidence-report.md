# Evidence Report — S-3.1.03

**Story:** S-3.1.03 — prism-core: implement OrgRegistry (bijective BiMap, resolve/slug_for/register)
**Implementation commit:** a6e95388
**Behavioral contracts:** BC-3.1.001, BC-3.1.003, BC-3.1.004
**Recording toolchain:** VHS 0.10.0 / FiraCode Nerd Font Mono / Dracula theme
**Recorded:** 2026-04-29

---

## Coverage Map

| Recording | AC(s) | BC Anchor(s) | Success Path | Error Path |
|-----------|-------|--------------|-------------|------------|
| AC-001-all-35-tests-green | AC-1 through AC-8 | BC-3.1.001, BC-3.1.003, BC-3.1.004 | 35/35 tests pass | N/A (all green = no failures) |
| AC-002-bijection-conflicts | AC-6, AC-7, AC-8 | BC-3.1.003, BC-3.1.004 | 3 conflict variant tests pass | SlugConflict / IdConflict / idempotent-Ok exercised |

---

## AC-001 — All 35 OrgRegistry Tests GREEN

**Command demonstrated:**
```
cargo test -p prism-core --test bc_3_1_003_org_registry
```

**Result:** `test result: ok. 35 passed; 0 failed; 0 ignored`

**Acceptance criteria covered:**
- AC-1 (BC-3.1.001 postcondition 1): round-trip resolve/slug_for consistency
- AC-2 (BC-3.1.001 postcondition 2): resolve unknown slug returns None without side effects
- AC-3 (BC-3.1.001 invariant 4): no filesystem I/O on hot path
- AC-4 (BC-3.1.003 postcondition 1): both directions populated after register
- AC-5 (BC-3.1.003 invariant 1): forward_len == reverse_len (proptest, 1000 cases)
- AC-6 (BC-3.1.004 postcondition 2): SlugConflict returned and registry unchanged
- AC-7 (BC-3.1.004 postcondition 3): IdConflict returned and registry unchanged
- AC-8 (BC-3.1.004 postcondition 4): idempotent re-registration returns Ok

**Artifacts:**
- [AC-001-all-35-tests-green.tape](AC-001-all-35-tests-green.tape)
- [AC-001-all-35-tests-green.gif](AC-001-all-35-tests-green.gif)
- [AC-001-all-35-tests-green.webm](AC-001-all-35-tests-green.webm)

---

## AC-002 — Bijection Conflict Variants (3 RegistrationError paths)

**Command demonstrated:**
```
cargo test -p prism-core --test bc_3_1_003_org_registry test_BC_3_1_004_tv -- --nocapture
```

**Result:** `test result: ok. 3 passed; 0 failed`

**Tests run (each maps to a RegistrationError variant):**
- `test_BC_3_1_004_tv_01_slug_conflict_error_fields` — `RegistrationError::SlugConflict { slug, existing_id, attempted_id }`
- `test_BC_3_1_004_tv_02_id_conflict_error_fields` — `RegistrationError::IdConflict { id, existing_slug, attempted_slug }`
- `test_BC_3_1_004_tv_03_no_partial_state_after_rejection` — registry unchanged after any conflict

**Acceptance criteria covered:**
- AC-6 (BC-3.1.004 postcondition 2): SlugConflict fields correct, registry intact
- AC-7 (BC-3.1.004 postcondition 3): IdConflict fields correct, registry intact
- AC-8 (BC-3.1.004 postcondition 4): idempotent Ok for exact duplicate

**Artifacts:**
- [AC-002-bijection-conflicts.tape](AC-002-bijection-conflicts.tape)
- [AC-002-bijection-conflicts.gif](AC-002-bijection-conflicts.gif)
- [AC-002-bijection-conflicts.webm](AC-002-bijection-conflicts.webm)

---

## Full AC Coverage

| AC | Description | Recording | Status |
|----|-------------|-----------|--------|
| AC-1 | Round-trip slug/id consistency | AC-001 | PASS |
| AC-2 | resolve unknown returns None | AC-001 | PASS |
| AC-3 | No I/O on hot path | AC-001 | PASS |
| AC-4 | Both directions populated after register | AC-001 | PASS |
| AC-5 | forward_len == reverse_len (proptest) | AC-001 | PASS |
| AC-6 | SlugConflict returned, registry unchanged | AC-001, AC-002 | PASS |
| AC-7 | IdConflict returned, registry unchanged | AC-001, AC-002 | PASS |
| AC-8 | Idempotent re-registration is Ok | AC-001, AC-002 | PASS |

All 8 acceptance criteria demonstrated. All 35 tests GREEN.
