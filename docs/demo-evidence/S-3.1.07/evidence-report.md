# Demo Evidence Report — S-3.1.07

**Story:** prism-audit org fields + SHA-256 aql_hash
**Impl SHA:** 95ecb88f
**Recorded:** 2026-04-29
**Toolchain:** VHS 0.10.0 / FiraCode Nerd Font Mono / Dracula theme

---

## Coverage Map

| AC | Description | Success Path | Error Path | Files |
|----|-------------|-------------|------------|-------|
| AC-001 | All 25 audit tests GREEN | recorded | N/A — test suite pass/fail is binary | AC-001-all-25-tests-green.{tape,gif,webm} |
| AC-002 | aql_hash determinism (14 focused tests) | recorded | N/A — determinism verified by distinct-hash tests | AC-002-aql-hash.{tape,gif,webm} |

---

## AC-001 — All 25 Audit Tests GREEN

**Command demonstrated:** `cargo test -p prism-audit 2>&1 | tail -30`

**Result:** `test result: ok. 138 passed; 0 failed; 0 ignored`

Note: The full prism-audit suite runs 138 tests (includes all BC modules). The 25
tests specifically for S-3.1.07 (org_id/org_slug fields + aql_hash) are a subset
confirmed passing. The tail-30 view shows the final summary line green.

Recordings:
- [AC-001-all-25-tests-green.gif](AC-001-all-25-tests-green.gif)
- [AC-001-all-25-tests-green.webm](AC-001-all-25-tests-green.webm)
- [AC-001-all-25-tests-green.tape](AC-001-all-25-tests-green.tape)

---

## AC-002 — aql_hash Determinism

**Command demonstrated:** `cargo test -p prism-audit test_BC_3_1_002_aql_hash -- --nocapture | head -40`

**Result:** 14 focused aql_hash tests pass, covering:
- SHA-256 output is 64-char lowercase hex
- Empty query returns SHA-256 of empty string
- Same input always produces same hash (determinism)
- Distinct inputs produce distinct hashes (collision resistance)
- Single-byte change produces different hash (avalanche)
- Unicode queries produce valid 64-char hex
- Very long queries do not panic

Recordings:
- [AC-002-aql-hash.gif](AC-002-aql-hash.gif)
- [AC-002-aql-hash.webm](AC-002-aql-hash.webm)
- [AC-002-aql-hash.tape](AC-002-aql-hash.tape)

---

## Error Path Note

Both ACs are verified by deterministic test suites. Error paths are embedded in the
test names themselves (e.g., `test_BC_3_1_002_aql_hash_distinct_inputs_produce_distinct_hashes`,
`test_BC_3_1_002_aql_hash_single_byte_change_produces_different_hash`). No separate
error-path recording is warranted — the failure case is a test failure (red output),
which is not the state under demo since implementation is correct.
