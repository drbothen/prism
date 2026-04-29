# Demo Evidence Report — S-3.7.05

**Story:** S-3.7.05 — CrowdStrike fixture generator (all 8 archetypes, 2-step pagination, OAuth2)
**Impl SHA:** a0f0c5ac
**Recorded:** 2026-04-28
**Tool:** VHS

## Coverage

| Recording | Acceptance Criteria | BCs / VPs | Result |
|-----------|---------------------|-----------|--------|
| AC-001-all-37-tests-green | All 37 tests GREEN (BC-3.4.001-004, 8 archetypes, 2-step pagination, OAuth2) | BC-3.4.001, BC-3.4.002, BC-3.4.003, BC-3.4.004 / VP-108, VP-112, VP-113, VP-114, VP-119, VP-120, VP-121 | PASS |
| AC-002-quirks-pagination-oauth2 | CrowdStrike-specific quirks: 2-step IdPage→detail join + OAuth2 token fixture fields | BC-3.4.002 / VP-112, VP-113 | PASS |

## Recordings

### AC-001 — All 37 tests GREEN

- Tape: `docs/demo-evidence/S-3.7.05/AC-001-all-37-tests-green.tape`
- GIF: `docs/demo-evidence/S-3.7.05/AC-001-all-37-tests-green.gif` (477 KB)
- WEBM: `docs/demo-evidence/S-3.7.05/AC-001-all-37-tests-green.webm` (794 KB)

Command demonstrated:
```
cargo test -p prism-dtu-crowdstrike --features fixture-gen --test bc_3_4_crowdstrike_generator
```
Result: `test result: ok. 37 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`

### AC-002 — 2-step pagination + OAuth2 quirks

- Tape: `docs/demo-evidence/S-3.7.05/AC-002-quirks-pagination-oauth2.tape`
- GIF: `docs/demo-evidence/S-3.7.05/AC-002-quirks-pagination-oauth2.gif` (146 KB)
- WEBM: `docs/demo-evidence/S-3.7.05/AC-002-quirks-pagination-oauth2.webm` (326 KB)

Command demonstrated:
```
cargo test -p prism-dtu-crowdstrike --features fixture-gen --test bc_3_4_crowdstrike_generator -- two_step oauth2 --nocapture
```
Tests covered: `test_bc_3_4_002_ac_002_two_step_pagination_id_pages_precede_detail` + `test_bc_3_4_002_ac_003_oauth2_record_shape_matches_types_rs`
Result: `test result: ok. 2 passed; 0 failed`

## Notes

- No fixture pollution: recordings run against the test binary, no fixture files written to disk.
- All recordings <= 30 seconds as required.
- Error paths (schema drift, auth outage 401) exercised within the bc_3_4_crowdstrike_generator test suite (covered by AC-001 full run).
