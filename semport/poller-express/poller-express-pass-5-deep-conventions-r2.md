# Pass 5 Deep: Convention & Pattern Catalog -- poller-express (Round 2)

## Audit of Round 1 Claims

### Claim: "CyberInt vs cyberint naming inconsistency across 7 error names"
Verified from `apperrors/errors.go`:
1. `ErrCyberIntConfigMissing` (unused)
2. `ErrCyberIntRequestBuild` (unused)
3. `ErrCyberIntRequestExec` (used)
4. `ErrCyberIntUnexpectedStatus` (unused)
5. `ErrCyberIntDecode` (unused)

Plus the collector references use `ErrCyberInt*` (capital I). Meanwhile the package is `cyberint` (lowercase), the API client is `cyberint.APIClient`, and the config struct is `CyberintConfig`.

Count: 5 error names with "CyberInt" (only 1 is actually used). The inconsistency is real but less impactful than stated because 4 of the 5 are dead code.

### Claim: "Options and AssetCollectorOptions are structurally identical"
Verified from source:
```
Options: Logger, Interval, Sink, HealthReporter  (alert_collector.go:24-35)
AssetCollectorOptions: Logger, Interval, Sink, HealthReporter  (asset_collector.go:39-48)
```
Field-for-field identical. Same types, same docstrings (with minor wording differences). Confirmed.

### Claim: "Unbounded rate limiter map"
Verified from `health/server.go:89-109`: The `limiters` map (`map[string]*rate.Limiter`) has `getLimiter()` that adds entries but no eviction function anywhere in the file. The map grows monotonically. Confirmed.

Severity: LOW in practice for this use case. Health endpoints are typically only hit by K8s kubelet (1-2 IPs) and monitoring systems (handful of IPs). But a port to a multi-tenant system or public-facing health endpoint would need eviction.

### Claim: "Asset client does not use sentinel errors"
Verified from `asset/client.go`: All 5 error returns use `fmt.Errorf("...: %w", err)` without any `apperrors.Err*` sentinels. Confirmed.

### Claim: "Two deferred close patterns coexist"
Verified:
1. `validate.Check(resp.Body.Close)` -- used in `alert_collector.go:220` and `asset/client.go:56`
2. Inline `defer func() { if cerr := resp.Body.Close(); ... }()` -- used in `sink/http_sender.go:105-109`

The sink uses inline logging; the collectors/client use `validate.Check`. The difference is the logging target: `validate.Check` uses `slog.Error`, inline uses `charmbracelet/log.Warn`.

---

## Detailed Collector Duplication Analysis

### Run() Method Comparison

Comparing `Collector.Run()` (alert_collector.go:86-154) with `AssetCollector.Run()` (asset_collector.go:82-150):

**Line-by-line identical sections** (modulo variable names):
- Reporter SetNotReady/defer (lines 87-89 / 83-85)
- initializeState call (lines 92-94 / 88-90)
- Reporter SetReady (lines 96-98 / 92-94)
- baseDelay/maxDelay initialization (lines 100-107 / 96-103)
- Ticker creation/defer (lines 109-110 / 105-106)
- retryCount/retryDelay initialization (lines 112-113 / 108-109)
- Entire retry loop structure (lines 115-153 / 111-149)

The ONLY differences in Run():
1. Log message: "collection attempt failed" vs "asset collection attempt failed"
2. Both call their respective `collectOnce(ctx)` method

**Verdict**: The Run() method is 95% copy-paste between the two collectors.

### initializeState() Comparison

Structurally identical. The only differences are:
1. `c.store.Load(ctx)` vs `c.store.LoadAsset(ctx)`
2. `state.PollState` vs `state.AssetPollState`
3. `state.Cursor` vs `state.AssetCursor`
4. `state.BatchReceipt` vs `state.AssetBatchReceipt`
5. `cyberint.CyberintTime{Time: ...}` vs plain `time.Time`
6. `c.store.Save(ctx, ...)` vs `c.store.SaveAsset(ctx, ...)`

**Verdict**: Type-level differences only. Generics could eliminate this duplication entirely.

### Function Scope Convention (Standalone vs Method)

| Function | Alert | Asset | Pattern |
|----------|-------|-------|---------|
| Run | Method on Collector | Method on AssetCollector | Same |
| initializeState | Method on Collector | Method on AssetCollector | Same |
| collectOnce | Method on Collector | Method on AssetCollector | Same |
| filterNew* | **Standalone function** | **Method on AssetCollector** | Different |
| isCursor/isAssetAhead | **Standalone function** | **Method on AssetCollector** | Different |
| ensureForwardProgress | **Standalone function** | **Method on AssetCollector** | Different |
| waitWithContext | Standalone function | (reuses from alert) | Same (shared) |

The alert collector defines cursor operations as standalone package-level functions. The asset collector defines them as methods on AssetCollector. This inconsistency is likely due to iterative development -- the alert collector was written first with standalone functions, then the asset collector was added with a preference for methods.

---

## Additional Convention Findings

### Import Alias Convention

| Package | Alias | File |
|---------|-------|------|
| `pkg/cyberint` | `cyberint` | alert_collector.go, runner.go |
| `pkg/validate` | `v` | alert_collector.go, asset/client.go |
| `internal/config` | `cfg` | main.go (import alias) |

The `v` alias for `pkg/validate` is terse and non-obvious. The `cyberint` alias for `pkg/cyberint` is redundant (same as package name) but explicit.

### Go Comment Convention

- All exported types and functions have GoDoc comments
- Comment format: `// TypeName does something.` (standard Go convention)
- Package comments use `// Package name provides...` format
- No `/* */` block comments in hand-written code

### Helm Template Convention

- All templates use `include` for helper functions (not `template`)
- Consistent use of `with` for optional blocks
- Consistent `nindent` usage for YAML indentation
- Labels follow Kubernetes recommended labels (`app.kubernetes.io/*`)

### CI Convention

- All workflows use SHA-pinned actions (e.g., `actions/checkout@sha256hash # v6`)
- All workflows use `step-security/harden-runner` as first step
- Self-hosted runners: `[self-hosted, Ubuntu, Common]` across all workflows
- Go version sourced from `go.mod` via `go-version-file` (single source of truth)

---

## Delta Summary
- New items added: Detailed Run() method comparison confirming 95% copy-paste, function scope convention inconsistency (standalone vs method), import alias conventions, Go comment conventions, Helm template conventions, CI conventions
- Existing items refined: CyberInt inconsistency narrowed from 7 to 1 actually-used error name, rate limiter unbounded map severity downgraded to LOW
- Remaining gaps: None significant

## Novelty Assessment
Novelty: NITPICK
The detailed Run() duplication analysis confirms what Round 1 already identified as "nearly identical structures." The function scope inconsistency (standalone vs method) is a style observation. The import aliases, Go comments, Helm templates, and CI conventions are standard practices that don't change how the system would be spec'd or ported. The CyberInt correction (7 -> 1 used) is a precision fix that reinforces the existing finding rather than discovering something new.

## Convergence Declaration
Pass 5 has converged -- findings are nitpicks, not gaps. The convention catalog is complete for specification purposes.

## State Checkpoint
```yaml
pass: 5
round: 2
status: complete
timestamp: 2026-04-14T00:05:00Z
novelty: NITPICK
```
