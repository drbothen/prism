# Pass 1 Deep: Architecture -- Round 3

> Project: poller-bear
> Source: /Users/jmagady/Dev/prism/.references/poller-bear/
> Round: 3

---

## Hallucination Audit of Round 2

### Collector Struct Field Count: CONFIRMED
Round 2 said 25 fields. Recount from source (lines 38-63): cfg, client, store, sink, reporter, logger, alertFingerprint, eventFingerprint, auditFingerprint, relationFingerprint, vulnFingerprint, interval, alertState, eventState, auditState, relationState, vulnState, serverState, serverFingerprint, siteState, siteFingerprint, deviceState, deviceFingerprint, vulnerabilityState, vulnerabilityFingerprint = **25 confirmed**.

### Fail-Fast in collectOnce(): CONFIRMED
Lines 804-851 show sequential calls with immediate error return.

### InitialSince for timestamp-cursor sources: CONFIRMED
- `initializeAlertState`: `state.AlertCursor{Timestamp: c.cfg.Collector.InitialSince}` (line 742)
- `initializeEventState`: `state.EventCursor{Timestamp: c.cfg.Collector.InitialSince}` (line 778)
- `initializeAuditLogState`: `state.AuditLogCursor{Timestamp: c.cfg.Collector.InitialSince, Offset: 0}` (line 148)
- `initializeDeviceAlertRelationState`: `state.DeviceAlertRelationCursor{Timestamp: c.cfg.Collector.InitialSince}` (line 706)

Offset-cursor sources: Servers (line 77), Sites (line 112), Devices (line 344): `Offset: 0`

---

## New Findings: FileStore Concurrency Architecture

### RWMutex Pattern
- `FileStore` uses `sync.RWMutex` for thread safety
- All Load operations use `RLock()`/`RUnlock()` (read lock)
- All Save operations use `Lock()`/`Unlock()` (write lock)
- `persist()` is called within the write lock -- atomic file write happens under lock
- **Single-writer assumption**: The collector calls `collectOnce()` sequentially, so there is never concurrent write contention. The mutex is defensive for potential future concurrent use.

### File State Structure
`fileState` struct holds 9 poll state pointers (nullable via `omitempty`) + 9 receipt slices + 1 `LastUpdated` timestamp = **19 fields**. All serialized to a single JSON file.

### Generic trimReceipts Function
```go
func trimReceipts[T any](receipts []T, limit int) []T
```
The codebase uses Go generics for receipt trimming -- the **only** use of generics in the entire codebase. This is architecturally significant because it demonstrates that the team is aware of generics and could have used them to eliminate the 9x repetition pattern but chose not to.

---

## Architecture Completeness Check

All architectural layers are now documented:

| Layer | Files | Deep-Dived? | Coverage |
|-------|-------|-------------|----------|
| Entry | main.go, cmd/collector/main.go | YES | Complete |
| Orchestration | runner.go | YES | Complete |
| Collection | collector.go | YES | Complete (Run, collectOnce, 9 collect*, 9 initialize*, 9 ensureForwardProgress) |
| API Client | api.go, http_client.go | YES (via Pass 2/3) | Complete |
| State | store.go, file_store.go, memory_store.go | YES | Complete (types, interfaces, persistence, generics) |
| Delivery | sink.go, http_sender.go | YES | Complete |
| Normalization | detection_finding.go, severity.go, config.go | YES | Complete |
| Transport | http.go | YES | Complete |
| Health | server.go | YES | Complete |
| Profiling | pprof.go | YES | Complete |
| Errors | errors.go | YES | Complete |
| Config | config.go | YES | Complete |
| Deployment | Dockerfile, Helm chart | YES | Complete |
| CI/CD | 6 workflows | YES | Complete |

No undiscovered subsystems or layers remain.

---

## Delta Summary
- New items added: FileStore concurrency architecture (RWMutex pattern), generics usage (trimReceipts only), fileState structure (19 fields)
- Existing items refined: None
- Remaining gaps: None

## Novelty Assessment
Novelty: NITPICK
The RWMutex pattern and generics observation are refinements to known patterns. The fileState structure is a detail that does not change how the system would be specified. All architectural layers are now fully documented.

## Convergence Declaration
Pass 1 has converged -- findings are nitpicks, not gaps. The architecture is completely documented across all layers.

## State Checkpoint
```yaml
pass: 1
round: 3
status: complete
files_scanned: 35
timestamp: 2026-04-14T00:15:00Z
novelty: NITPICK
```
