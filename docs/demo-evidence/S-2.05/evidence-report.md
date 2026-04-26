# Demo Evidence Report — S-2.05: prism-audit Specialized Audit Events

**Story:** S-2.05  
**Branch:** feature/S-2.05-audit-events  
**Recorded:** 2026-04-25  
**Tool:** VHS 0.10.0  
**Test suite:** `cargo test -p prism-audit specialized_event_tests` → 35 passed / 0 failed  

---

## Healthy TDD Disclosure

S-2.05 followed proper Red Gate discipline: 7 `todo!()` bodies in production drove 19 RED
tests; 16 GREEN-BY-DESIGN tests covered pure-data assertions (outcome→log_level mappings,
struct serde delegation). RED ratio 54.3% well above 50% threshold (Layer 2 Red Gate density
check satisfied — first story since prevention layers conceptualized). All 4 BC implementations
landed in single squash commit at `4cf612fc`.

---

## Coverage Map

| AC | Acceptance Criterion | BC | Test Filter | Demo File | Result |
|----|---------------------|----|-------------|-----------|--------|
| AC-1 | `emit_credential_event()` records `credential_name`, `access_type`, requesting context; serialized JSON contains no credential value field | BC-2.05.005 | `BC_2_05_005` (6 tests) | [ac-1-credential-access-event.gif](ac-1-credential-access-event.gif) | PASS |
| AC-2 | `to_vector_json()` produces JSON with `@timestamp`, `host`, `service: "prism"`, `log.level`; parameters as JSON string; read-only | BC-2.05.007 | `BC_2_05_007` (13 tests) | [ac-2-vector-pipeline-format.gif](ac-2-vector-pipeline-format.gif) | PASS |
| AC-3 | `emit_flag_eval()` for `"sensors.crowdstrike.write"` records full `resolution_trace`; empty trace emitted without panic | BC-2.05.009 | `BC_2_05_009` (6 tests) | [ac-3-flag-evaluation-write-only.gif](ac-3-flag-evaluation-write-only.gif) | PASS |
| AC-4 | `emit_token_generated/consumed/expired()` all succeed; `Generated`, `Consumed`, `Expired` are distinct serialized values; `token_id` excluded from `action_summary` | BC-2.05.010 | `BC_2_05_010` (10 tests) | [ac-4-token-lifecycle-events.gif](ac-4-token-lifecycle-events.gif) | PASS |

---

## Test Breakdown by BC

| BC Module | Tests | RED at Gate | GREEN-BY-DESIGN | Notes |
|-----------|-------|-------------|-----------------|-------|
| BC-2.05.005 | 6 | 2 | 4 | `emit_credential_event` + `emit_not_found` were RED; struct-shape/serde tests GREEN |
| BC-2.05.007 | 13 | 7 | 6 (outcome_to_log_level, result) | `to_vector_json`, `resolve_host` paths were RED; mapping helpers GREEN |
| BC-2.05.009 | 6 | 2 | 4 | `emit_flag_eval` (with/without trace) RED; serde struct tests GREEN |
| BC-2.05.010 | 10 | 3 | 7 | `emit_token_generated/consumed/expired` were RED; enum/struct serde tests GREEN |
| **Total** | **35** | **~19** | **~16** | **RED ratio: 54.3% (above 50% gate)** |

---

## Recordings

All recordings produced with VHS 0.10.0 using FiraCode Nerd Font Mono, Dracula theme,
1000x600 viewport. Each recording runs `cargo test -p prism-audit` filtered to the
relevant BC test module.

| File | Size | AC |
|------|------|----|
| `ac-1-credential-access-event.gif` | 139 KB | AC-1 |
| `ac-2-vector-pipeline-format.gif` | 204 KB | AC-2 |
| `ac-3-flag-evaluation-write-only.gif` | 142 KB | AC-3 |
| `ac-4-token-lifecycle-events.gif` | 188 KB | AC-4 |

Total GIF size: ~673 KB

---

## Workspace Regression Check

Workspace test suite (`cargo test --workspace`) verified 1276 PASS / 0 FAIL / 4 IGN on
commit `4cf612fc` (impl complete) prior to demo recording. Demo commit adds only
`docs/demo-evidence/S-2.05/` files — no source changes.
