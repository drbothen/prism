# Demo Evidence — S-6.10: prism-dtu-armis

Story: **prism-dtu-armis: DTU for Armis Centrix API — L2 (stateful)**
Branch: `feature/S-6.10-dtu-armis`
Crate: `crates/prism-dtu-armis/`
POL-010 compliance: all evidence under `docs/demo-evidence/S-6.10/`

## Reproducing all evidence

```bash
cd /path/to/prism/.worktrees/S-6.10-armis
cargo test --features prism-dtu-armis/dtu -p prism-dtu-armis
```

Expected: 32 integration tests, all pass.

## File index

| File | AC | What it demonstrates |
|------|----|----------------------|
| `AC-1-aql-capture.md` | AC-1, EC-001, EC-004, EC-005 | AQL verbatim capture in GET and POST device queries; special chars; pagination edge |
| `AC-2-timestamp-fallback.md` | AC-2, EC-002 | d-001 null last_seen / non-null first_seen; risk endpoint shape; unknown device 404 |
| `AC-3-stateful-tag-add.md` | AC-3 | POST tag → 201; tag persists in subsequent GET; auth required |
| `AC-4-tag-delete.md` | AC-4, EC-003 | DELETE tag → 200 removed; tag absent from subsequent GET; never-added tag 404 |
| `AC-5-missing-bearer-403.md` | AC-5 | All vendor endpoints return 403 (not 401) without bearer; DTU endpoints bypass auth |
| `AC-6-rate-limit-and-malformed-response.md` | AC-6, EC-006 | FailureLayer rate-limit → 429; malformed_response mode → non-JSON body |
| `AC-7-reset-behavior.md` | AC-7 | reset() clears tag_store + aql_log; fixture data survives |
| `evidence-report.md` | All | Full AC/EC coverage matrix + green gate verification |
| `test-run.txt` | All | Raw `cargo test` transcript — 32 tests, all passed |

## Coverage summary

- ACs satisfied: 7 / 7
- ECs satisfied: 6 / 6
- Non-demo-able ACs: 0
- Total test functions: 32
- Failure paths covered: yes (every AC file includes error-path sequences)
