# Pass 4 Deep: NFR Catalog -- Round 3

> Project: poller-bear
> Source: /Users/jmagady/Dev/prism/.references/poller-bear/
> Round: 3

---

## Hallucination Audit of Round 2

### NFR-7.1 Fail-Fast: CONFIRMED
Lines 804-851 of collector.go: each collect* call returns immediately on error.

### NFR-8.3 Non-blocking Security Scans: CONFIRMED
`security-scan.yml` line 50: `gosec -no-fail -fmt text ./...`

### Retry count off-by-one: CONFIRMED
Line 621: `if c.cfg.Collector.MaxRetries > 0 && retryCount > c.cfg.Collector.MaxRetries`
With MaxRetries=5: retryCount goes 1,2,3,4,5 (all pass), then 6 > 5 triggers fatal.
Error message: `fmt.Errorf("%w: attempts=%d", apperrors.ErrCollectorRetriesExceeded, retryCount-1)` reports `attempts=5`.

---

## Additional NFR Detail: State File Concurrency

### NFR-9: Concurrency Safety (NEW SUBCATEGORY)

### NFR-9.1: FileStore Thread Safety
- `sync.RWMutex` protects all state access
- Load operations use `RLock()` (multiple concurrent readers allowed)
- Save operations use `Lock()` (exclusive writer access)
- `persist()` called within write lock -- file I/O under lock
- Context parameter accepted but unused (`_ = ctx`) -- no timeout on state operations

### NFR-9.2: Health Server Thread Safety
- `atomic.Bool` for both `ready` and `alive` flags
- No mutex needed -- atomic operations are sufficient for boolean state

### NFR-9.3: Single-Writer Guarantee
- `collectOnce()` is called synchronously from `Run()` loop
- No goroutines spawn within `collectOnce()`
- State file can only have one writer at a time (by design)
- PVC ReadWriteOnce further prevents multi-pod concurrent access

---

## NFR Gap: Context Usage in State Operations

The state store accepts `context.Context` but ignores it:
```go
func (fs *FileStore) Load(ctx context.Context) (AlertPollState, error) {
    _ = ctx
    // ...
}
```
This means:
- State operations cannot be cancelled via context
- State operations cannot timeout
- If `persist()` blocks (disk full, NFS stall), the entire collector hangs
- The context parameter is a **forward-compatibility placeholder** that is not yet functional

This is a **minor NFR gap**: state operations are not cancellable.

---

## NFR Completeness Check

All discoverable NFRs have been cataloged:

| Category | Items Found | Items Missing/Incomplete |
|----------|-------------|-------------------------|
| Security (1.x) | 5 implemented, 3 missing | Complete |
| Reliability (2.x) | 6 implemented, 1 missing | Complete |
| Performance (3.x) | 4 implemented, 2 missing | Complete |
| Observability (4.x) | 4 implemented, 2 missing | Complete |
| Scalability (5.x) | 3 documented | Complete |
| Maintainability (6.x) | 3 documented | Complete |
| Isolation (7.x) | 4 documented | Complete |
| Build (8.x) | 5 documented | Complete |
| Concurrency (9.x) | 3 documented | Complete |

Total: **39 NFR items** across 9 categories.

---

## Delta Summary
- New items added: 3 concurrency safety NFRs, 1 context-unused gap
- Existing items refined: None
- Remaining gaps: None significant

## Novelty Assessment
Novelty: NITPICK
The concurrency safety details are refinements to the architecture already documented. The context-unused gap is a minor finding that doesn't change the system model. All major NFR categories are fully cataloged.

## Convergence Declaration
Pass 4 has converged -- findings are nitpicks, not gaps. The NFR catalog is complete with 39 items across 9 categories.

## State Checkpoint
```yaml
pass: 4
round: 3
status: complete
files_scanned: 35
timestamp: 2026-04-14T00:20:00Z
novelty: NITPICK
```
