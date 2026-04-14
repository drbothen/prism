# Pass 1 Deep: Architecture -- Round 2

> Project: poller-bear
> Source: /Users/jmagady/Dev/prism/.references/poller-bear/
> Round: 2

---

## Hallucination Audit of Round 1

### Entry Point Duality: CONFIRMED
- `main.go` (root) at 37 lines: profiling.Start() -> runner.Execute() with pprof shutdown
- `cmd/collector/main.go` at 14 lines: runner.Execute() only, no pprof
- Dockerfile builds `./cmd/collector` -- confirmed: `go build -o /out/collector ./cmd/collector`
- Makefile builds `main.go` -- confirmed: `go build -ldflags="-s -w" -o build/$(BINARY_NAME) main.go`
- **Claim verified**: Docker images do NOT have pprof initialization.

### Optional Sink: CONFIRMED
- `runner.go` line 72-79: `if cfg.Sink.Endpoint != "" { alertSink, err = sink.NewHTTPSender(...) }`
- Default endpoint is `http://localhost:4413` (not empty) -- so in practice the sink is always created with default config
- `collector.go` line 194: `if c.sink != nil { ... }` -- nil sink check confirmed in all 9 collect* methods
- **Refinement**: The sink is architecturally optional, but operationally it is always present due to non-empty default.

### Health Probe Disabled by Default: CONFIRMED
- `values.yaml` lines 102-103: `livenessProbe.enabled: false`
- `values.yaml` lines 113-114: `readinessProbe.enabled: false`
- **Verified from template**: `{{- if and .Values.livenessProbe .Values.livenessProbe.enabled }}`

---

## New Findings: Collector Internal Architecture

### collectOnce() Sequencing

`collectOnce()` calls all 9 collect functions in a fixed order:
1. `collectAlerts`
2. `collectEvents`
3. `collectAuditLogs`
4. `collectDeviceAlertRelations`
5. `collectDeviceVulnerabilityRelations`
6. `collectServers`
7. `collectSites`
8. `collectDevices`
9. `collectVulnerabilities`

**Fail-fast behavior**: If ANY source fails, the entire `collectOnce()` returns immediately with the error. Subsequent sources are NOT attempted. This means:
- If alerts fail, events through vulnerabilities are all skipped
- The retry loop retries the entire `collectOnce()`, not individual sources
- After retry, the failed source is retried first (same fixed order), and all successful sources before it run again (but likely get empty batches due to persisted cursors)

**hasMore aggregation**: The return value is `alertsMore || eventsMore || ... || vulnerabilitiesMore`. If ANY source has more data, the loop continues immediately without waiting for the ticker.

### initializeState() Sequencing

Same fixed order as `collectOnce()`:
1. Alerts -> 2. Events -> 3. AuditLogs -> 4. DeviceAlertRelations -> 5. DeviceVulnerabilityRelations -> 6. Servers -> 7. Sites -> 8. Devices -> 9. Vulnerabilities

**Fail-fast on initialization too**: If fingerprint mismatch on alerts, events through vulnerabilities are never initialized.

### State Initialization Behavior per Source

Two categories of initial cursor:

**Timestamp-cursor sources** (5): Initialize with `Timestamp: c.cfg.Collector.InitialSince`
- Alerts, Events, AuditLogs, DeviceAlertRelations, DeviceVulnerabilityRelations
- `InitialSince` defaults to `time.Time{}` (zero time) -- meaning first run collects ALL historical data

**Offset-cursor sources** (4): Initialize with `Offset: 0`
- Servers, Sites, Devices, Vulnerabilities
- Always start from offset 0 on first run

### Run() Loop State Machine

```
                START
                  |
            SetNotReady
                  |
          initializeState
             /       \
          error      success
            |           |
         RETURN      SetReady
                        |
              +--> collectOnce()
              |     /        \
              |  error      success
              |    |           |
              | SetNotReady    | reset retryCount
              |    |           | SetReady
              | retryCount++   |
              |    |           +-- hasMore? --+
              |    |           |     YES      NO
              |    |           |      |        |
              |    |           |  continue   wait
              | maxRetries?   |              ticker/ctx
              |  YES    NO    |                |
              |   |      |    +--------<-------+
              | FATAL    |
              | RETURN   |
              |          |
              |   waitWithContext(retryDelay)
              |     /         \
              |  cancelled    timer
              |    |            |
              |  RETURN    retryDelay *= 2
              |            (cap at maxDelay)
              |                 |
              +--------<--------+
```

### Collector Struct Size

The Collector struct holds **18 fields**:
- 1 config (Config)
- 1 client (claroty.Client)
- 1 store (state.Store)
- 1 sink (sink.Sender)
- 1 reporter (health.Reporter)
- 1 logger (*log.Logger)
- 9 fingerprints (state.QueryFingerprint)
- 9 state objects (various PollState types, but only 9 stored in struct -- overlap with fingerprint count is coincidental; actually 9 states + 9 fingerprints + 6 other = 24 fields)

Recount from source:
`cfg, client, store, sink, reporter, logger, alertFingerprint, eventFingerprint, auditFingerprint, relationFingerprint, vulnFingerprint, interval, alertState, eventState, auditState, relationState, vulnState, serverState, serverFingerprint, siteState, siteFingerprint, deviceState, deviceFingerprint, vulnerabilityState, vulnerabilityFingerprint` = **25 fields**.

### Naming Inconsistency in Collector Fields

The fingerprint/state field naming is inconsistent:

| Source | State Field | Fingerprint Field |
|--------|-------------|-------------------|
| Alerts | `alertState` | `alertFingerprint` |
| Events | `eventState` | `eventFingerprint` |
| AuditLogs | `auditState` | `auditFingerprint` |
| DeviceAlertRelations | `relationState` | `relationFingerprint` |
| DeviceVulnRelations | `vulnState` | `vulnFingerprint` |
| Servers | `serverState` | `serverFingerprint` |
| Sites | `siteState` | `siteFingerprint` |
| Devices | `deviceState` | `deviceFingerprint` |
| Vulnerabilities | `vulnerabilityState` | `vulnerabilityFingerprint` |

Note: Relations use abbreviated names (`relationState`, `vulnState`) while standalone entities use full names. This is minor but contributes to the naming inconsistency identified in Pass 5.

---

## Architecture Summary: Data Lifecycle

```
1. STARTUP
   config.DefaultConfig() -> config.LoadFromEnvironment()
   -> claroty.NewHTTPClient()
   -> state.NewFileStore() or state.NewMemoryStore()
   -> sink.NewHTTPSender() (if endpoint non-empty)
   -> health.NewServer()
   -> collector.New()

2. INITIALIZATION
   For each of 9 sources:
     Load state from store
     -> If not found: bootstrap with initial cursor, save, done
     -> If found: verify fingerprint matches, done
     -> If fingerprint mismatch: FATAL ERROR

3. COLLECTION LOOP (infinite)
   collectOnce():
     For each of 9 sources (sequential, fail-fast):
       Fetch batch from Claroty (POST with cursor/offset)
       -> If empty: return hasMore=false
       Send each record to sink individually
       Verify forward progress
       Save new cursor + receipt to state
       -> return hasMore = (batchSize >= limit)
   
   If any error: retry with exponential backoff
   If success + hasMore: loop immediately
   If success + !hasMore: wait for 30s ticker

4. SHUTDOWN
   Context cancellation -> cancel in-flight HTTP -> 
   health server shutdown (5s) -> pprof shutdown (5s, root only)
```

---

## Delta Summary
- New items added: collectOnce() fail-fast sequencing, initializeState() sequencing, Run() state machine diagram, Collector struct field count (25), data lifecycle summary
- Existing items refined: Optional sink clarified (architecturally optional, operationally always present), initial cursor behavior (timestamp vs offset)
- Remaining gaps: None significant -- all architectural layers documented

## Novelty Assessment
Novelty: SUBSTANTIVE
The fail-fast behavior in collectOnce() is architecturally significant: a single source failure blocks all 9 sources. The state machine diagram reveals the exact retry/recovery flow. The initialization sequencing shows that timestamp-cursor sources use `InitialSince` (zero time = all historical data) while offset-cursor sources always start at 0. These findings change how the system would be specified.

## Convergence Declaration
Pass 1 architecture is approaching convergence. The major architectural patterns are fully documented. One more round could verify edge cases but would likely be NITPICK.

## State Checkpoint
```yaml
pass: 1
round: 2
status: complete
files_scanned: 28
timestamp: 2026-04-14T00:00:00Z
novelty: SUBSTANTIVE
```
