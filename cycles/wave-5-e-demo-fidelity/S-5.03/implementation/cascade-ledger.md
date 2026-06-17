# S-5.03 LOCAL Cascade Ledger

Story: S-5.03 Resources and Prompts (prism-mcp)
Cycle: wave-5-e-demo-fidelity
Streak target: 3-CLEAN (strict per BC-5.39.001 + D-779)

## Cascade History

### Pass 1 (2026-06-17) — FINDINGS

| ID | Severity | Summary |
|----|----------|---------|
| F-001 | HIGH | Tautological tests — tests verify inputs not outputs |
| F-002 | HIGH | DI-008 client-scoping not wired (BC-2.10.008 v1.8) |
| F-003 | MED | VP-050 vacuous — property not exercised |
| F-004 | LOW | Reachable false-positive — branch unreachable |
| F-005 | HIGH | AC-9 unwired (api_base_url field missing) |
| F-006 | MED | Mutex used where RwLock required |
| F-007 | MED | Non-exhaustive registration missing |

Fix IN PROGRESS. Streak 0/3.

### AC-10 Relocation Note

AC-10 (unregistered_table_queries) relocated from S-5.03 to S-5.08 per PO decision B (D-1212).
BC-2.08.009 v1.4 is the owning contract. S-5.03 was previously tasked with this AC.
DO-NOT-REFLAG in S-5.03 adversary passes.
