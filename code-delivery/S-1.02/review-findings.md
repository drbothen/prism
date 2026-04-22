# Review Findings — S-1.02: Entity Types and State Machines

**PR:** #17
**Branch:** feature/S-1.02-entity-types
**Base:** develop

## Convergence Tracking

| Cycle | Findings | Blocking | Fixed | Remaining | Verdict |
|-------|----------|----------|-------|-----------|---------|
| 1 | 5 | 0 | 0 | 0 | APPROVE |

## Finding Detail

| ID | Finding | Severity | Category | Status | Notes |
|----|---------|----------|----------|--------|-------|
| F1 | `proof_exactly_12_transitions` name misleading — proof verifies correctness against VALID_TRANSITIONS table, not count; count is in runtime test `vp005_exactly_12_valid_transitions` | suggestion | documentation | deferred-tech-debt | Not blocking; property is correctly verified |
| F3 | `CursorId(pub u64)` inner field is public — callers can construct `CursorId(n)` without registry, bypassing allocation tracking | suggestion | API design | deferred-tech-debt | Not exploitable at current layer; S-3.05 async wrapper should enforce |
| F4 | ID newtypes (`AlertId`, `CaseId`, `RuleId`, `ScheduleId`) have `pub Uuid` inner — diverges from S-1.01 `TenantId` private-inner convention | suggestion | consistency | deferred-tech-debt | No security consequence at this layer |
| F5 | VP-011 Kani proof `proof_path_traversal_rejected` uses concrete string inputs rather than symbolic — proof name overstates formal coverage | important | proof methodology | deferred-tech-debt | Runtime tests cover additional inputs; Kani limitation documented in proof comment |
| F6 | `MockStorageEngine::put_batch` uses pre-check-then-abort rather than write-and-rollback — atomicity property passes trivially (no partial writes) | suggestion | test fidelity | deferred-tech-debt | Mock spec allows this; VP-055 property is satisfied; RocksDB implementation (S-2.01) will test true rollback |

## Security Review

| Component | Findings | Notes |
|-----------|----------|-------|
| CredentialName | 0 | All 4 patterns rejected; input not echoed in error messages |
| CaseStatus state machine | 0 | VALID_TRANSITIONS const is single source of truth; valid_transitions() consistent |
| CursorRegistry | 0 | Cap check `>= 200` correct; release semantics correct |
| MockStorageEngine | 0 | Domain isolation via BTreeMap key; atomicity via pre-check |
| VP-057 crash recovery | 0 | Pure function; threshold `>= 2` encodes `+1 >= 3` correctly; idempotent |

## Resolution

All 5 findings are tech-debt deferrals. 0 blocking findings. PR approved for merge after CI passes.

### Tech-debt items to register

- F1: Rename/clarify Kani proof name or add comment (cosmetic)
- F3: Consider making `CursorId` inner field private, adding `fn as_u64()` accessor (S-3.05 scope)
- F4: Make ID newtype inner fields private for consistency with TenantId (S-2.x scope)
- F5: Document VP-011 limitation explicitly in Phase 5 Kani schedule (Phase 5 scope)
- F6: VP-055 proptest could use a write-and-rollback mock for stronger coverage (S-2.01 scope)
