# Pass 5 Deep: Conventions -- Round 2

> Project: poller-bear
> Source: /Users/jmagady/Dev/prism/.references/poller-bear/
> Round: 2

---

## Hallucination Audit of Round 1

### C-1.2 Field Naming Inconsistency: CONFIRMED
Verified from source:
- `claroty.Alert` uses abbreviated names: `Name` (from `alert_name`), `Class` (from `alert_class`), `TypeName` (from `alert_type_name`)
- `claroty.DeviceAlertRelation` uses full prefixed names: `AlertName`, `AlertClass`, `AlertTypeName`, `DeviceName`, `DeviceCategory`
- This is intentional: relation structs prefix to disambiguate since they embed fields from multiple entities

### C-1.3 Env Var Prefix Inconsistency: CONFIRMED
- `POLLER_BEAR_LOG_LEVEL` (project prefix)
- `COLLECTOR_HEALTH_ADDR` (component prefix)
- `CLAROTY_*`, `VECTOR_*`, `XMP_*`, `OCSF_*`, `STATE_STORE_*` (domain prefixes)
- `ENABLE_PPROF`, `PPROF_ADDR` (feature prefixes)
- **Five different prefix schemes in one codebase**

### C-5.1 9x Repetition: CONFIRMED AND QUANTIFIED
From collector.go alone:
- 9 `initialize*State` functions (~35 lines each) = ~315 lines
- 9 `collect*` functions (~65 lines each) = ~585 lines
- 9 `ensure*ForwardProgress` functions (~12 lines each) = ~108 lines
- Total repetitive code in collector.go: ~1,008 lines out of 1,368 = **73.7%** of the file

### C-6.3 Duration Parsing Inconsistency: CONFIRMED
From `config.go`:
- `VECTOR_TIMEOUT_SECONDS` supports both `time.ParseDuration` and `strconv.Atoi` (dual parsing)
- No env var for `Collector.Interval`, `Collector.RetryBaseDelay`, `Collector.RetryMaxDelay`
- Helm values.yaml exposes `collector.interval`, `collector.retryBaseDelay`, `collector.retryMaxDelay` as template env vars (`COLLECTOR_INTERVAL`, `COLLECTOR_RETRY_BASE_DELAY`, `COLLECTOR_RETRY_MAX_DELAY`) but these are NOT handled in `config.go`

**NEW FINDING**: The Helm deployment template injects `COLLECTOR_INTERVAL`, `COLLECTOR_RETRY_BASE_DELAY`, `COLLECTOR_RETRY_MAX_DELAY`, and `COLLECTOR_MAX_RETRIES` as environment variables (lines 124-130 of deployment.yaml), but `config.go` does NOT read any of these. This means **Helm-configured collector values are silently ignored** -- the application always uses `DefaultConfig()` values for these fields.

---

## New Conventions from Collector Deep-Dive

### C-8: Cursor-to-State Mapping Convention

Each `collect*` function follows this exact pattern:

```go
func (c *Collector) collectX(ctx context.Context) (bool, error) {
    // 1. Build request from current state + fingerprint
    req := claroty.XRequest{
        Cursor: claroty.XCursor{...from c.xState.Cursor...},
        Limit:  c.xFingerprint.Limit,
        Fields: c.xFingerprint.Fields,
    }

    // 2. Fetch batch
    batch, err := c.client.FetchX(ctx, req)
    if err != nil { return false, err }

    // 3. Empty check
    if len(batch.Items) == 0 {
        c.logger.Info("no new X", ...)
        return false, nil
    }

    // 4. Sink delivery (nil-safe)
    if c.sink != nil {
        for i := range batch.Items {
            if err := c.sink.SendX(ctx, batch.Items[i]); err != nil {
                return false, fmt.Errorf("%w: %v", apperrors.ErrSinkDelivery, err)
            }
        }
    }

    // 5. Forward progress check
    newCursor := state.XCursor{...from batch.Last...}
    if err := ensureXForwardProgress(c.xState.Cursor, newCursor); err != nil {
        return false, err
    }

    // 6. Version increment
    newVersion := c.xState.Version + 1

    // 7. Build new state
    updated := state.XPollState{
        Cursor:    newCursor,
        Query:     c.xFingerprint,
        UpdatedAt: time.Now(),
        Version:   newVersion,
    }

    // 8. Build receipt
    receipt := state.XBatchReceipt{...}

    // 9. Persist state + receipt
    if err := c.store.SaveXState(ctx, updated, receipt); err != nil {
        return false, fmt.Errorf("%w: %v", apperrors.ErrCollectorStatePersist, err)
    }

    // 10. Update in-memory state
    c.xState = updated

    // 11. Log
    c.logger.Info("X batch processed", ...)

    // 12. Return hasMore
    return len(batch.Items) >= c.xFingerprint.Limit, nil
}
```

All 9 collect functions follow this exact 12-step pattern. The only variations are:
- Entity-specific field names in cursors and receipts
- AuditLog cursor offset uses `batch.Last.Offset + 1` (unique among all 9)
- Sink delivery uses `batch.Items[i]` with range-by-index (not range-by-value)

### C-9: Initialize Pattern

```go
func (c *Collector) initializeXState(ctx context.Context) error {
    xState, err := c.store.LoadXState(ctx)
    switch {
    case err == nil:
        if xState.Query.Hash != c.xFingerprint.Hash {
            return fmt.Errorf("%w: stored=%s current=%s",
                apperrors.ErrQueryFingerprintMismatch, ...)
        }
        c.xState = xState
        return nil
    case errors.Is(err, apperrors.ErrStateNotFound):
        initialCursor := state.XCursor{...}  // zero or InitialSince
        bootstrap := state.XPollState{
            Cursor: initialCursor, Query: c.xFingerprint,
            UpdatedAt: time.Now(), Version: 0,
        }
        receipt := state.XBatchReceipt{Version: 0, RequestHash: ..., Count: 0, ...}
        if err := c.store.SaveXState(ctx, bootstrap, receipt); err != nil {
            return fmt.Errorf("%w: %v", apperrors.ErrCollectorStatePersist, err)
        }
        c.xState = bootstrap
        return nil
    default:
        return fmt.Errorf("%w: %v", apperrors.ErrCollectorStateLoad, err)
    }
}
```

All 9 initialize functions follow this exact 3-case switch pattern.

### C-10: AlertStore Naming Exception

The AlertStore interface uses `Load`/`Save` (no prefix) while all other 8 sub-interfaces use `Load<Entity>State`/`Save<Entity>State`. This is because AlertStore was the first store interface created; the naming convention evolved with later sources but was not backported.

Confirmed from `state/store.go`:
- `AlertStore.Load()` vs `ServerStore.LoadServerState()`
- `AlertStore.Save()` vs `ServerStore.SaveServerState()`

---

## Helm-Config Mismatch Detail

The Helm deployment template sets these env vars that are **never read** by the application:

| Helm Value | Env Var Set in Template | Read by config.go? |
|------------|------------------------|---------------------|
| `collector.interval` | `COLLECTOR_INTERVAL` | NO |
| `collector.retryBaseDelay` | `COLLECTOR_RETRY_BASE_DELAY` | NO |
| `collector.retryMaxDelay` | `COLLECTOR_RETRY_MAX_DELAY` | NO |
| `collector.maxRetries` | `COLLECTOR_MAX_RETRIES` | NO |

These env vars appear in the deployment template (lines 124-130) but `config.go` only reads: `CLAROTY_*`, `VECTOR_*`, `XMP_*`, `OCSF_*`, `STATE_STORE_*`, `COLLECTOR_HEALTH_ADDR`, `POLLER_BEAR_LOG_LEVEL`.

This is a **bug or incomplete feature**: the Helm chart was updated to expose these values but the application code was never updated to consume them.

---

## Consistency Assessment Update

| Pattern | Consistency | Notes |
|---------|-------------|-------|
| 12-step collect pattern | 100% (9/9) | All 9 sources identical structure |
| 3-case initialize pattern | 100% (9/9) | All 9 sources identical structure |
| Sentinel error wrapping | 100% | All packages |
| Forward progress check placement | 100% | Always after sink delivery, before state save |
| Nil sink guard | 100% (9/9) | All collect functions check `c.sink != nil` |
| Range-by-index for sink delivery | 100% (9/9) | `for i := range batch.Items` |
| AlertStore naming exception | Inconsistent (1/9) | Only AlertStore lacks entity prefix |
| Env var prefix scheme | Inconsistent | 5 different prefix patterns |
| Helm-config alignment | Broken | 4 env vars set but never read |

---

## Delta Summary
- New items added: Helm-config mismatch (4 env vars set but never read), AlertStore naming exception, 12-step collect pattern formalized, 3-case initialize pattern formalized, AuditLog offset+1 uniqueness
- Existing items refined: 9x repetition quantified (73.7% of collector.go), env var prefix inconsistency counted (5 schemes)
- Remaining gaps: None significant

## Novelty Assessment
Novelty: SUBSTANTIVE
The Helm-config mismatch is a genuine **bug** -- operators setting `collector.interval` in Helm values would have no effect on the application. This is a model-changing finding. The AlertStore naming exception is a convention inconsistency that matters for spec crystallization. The formalized 12-step collect pattern documents the exact structure needed for generalization.

## Convergence Declaration
Pass 5 is approaching convergence. The Helm-config mismatch is the last major finding. Another round could investigate Python legacy conventions but would likely be NITPICK for the Go codebase analysis.

## State Checkpoint
```yaml
pass: 5
round: 2
status: complete
files_scanned: 30
timestamp: 2026-04-14T00:10:00Z
novelty: SUBSTANTIVE
```
