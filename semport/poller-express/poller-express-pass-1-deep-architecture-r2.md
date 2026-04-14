# Pass 1 Deep: Architecture -- poller-express (Round 2)

## Audit of Round 1 Claims

### Claim: "5 unused sentinel errors"
Verified via Grep: `ErrCyberIntConfigMissing`, `ErrCyberIntRequestBuild`, `ErrCyberIntUnexpectedStatus`, `ErrCyberIntDecode`, and `ErrConfigLoad` are only referenced in their definitions in `apperrors/errors.go`. No other Go file wraps or matches them. Confirmed.

### Claim: "Cyberint and sink use DIFFERENT http.Client instances"
Verified:
- `runner.go:59-65`: Creates shared `http.Client{Timeout: 30s, Transport: cookieTransport}` 
- `runner.go:77-79`: Sets `cyberintConfig.HTTPClient = httpClient` and `asset.NewClient(baseURL, httpClient)`
- `sink/http_sender.go:76`: Creates its own `&http.Client{Timeout: timeout}` (default 15s)

Confirmed: Two separate clients with different timeouts and different transports (cookieTransport vs default).

### Claim: "Health server starts before collectors"
Verified from runner.go:
- Line 131-134: `go func() { healthErrCh <- healthServer.ListenAndServe() }()`
- Line 141-148: Alert collector goroutine
- Line 151-160: Asset collector goroutine

The health server goroutine starts first, but goroutine scheduling means there is no strict guarantee that `ListenAndServe` binds the port before collectors start their first cycle. In practice, the collectors' first action is `initializeState()`, which does not depend on health. The first `SetReady()` call happens after `initializeState()` succeeds.

### Claim: "Double-check locking in rate limiter"
Verified from `health/server.go:89-109`: Classic double-check locking with RLock/RUnlock, then Lock/check-again/Unlock. Correct.

### Claim: "logging inconsistency between charmbracelet/log and slog"
Verified:
- `pkg/validate/utils.go` uses `log/slog` (stdlib)
- All other files use `github.com/charmbracelet/log`
Confirmed. The validate package's `Check` function outputs through slog, while the rest of the app outputs through charmbracelet/log. In practice, both output to stdout, but with different formatting.

---

## Additional Architectural Details Not Covered in Round 1

### Collector Structural Comparison

Examining the two collector files side-by-side reveals the exact scope of duplication:

| Component | Alert Collector | Asset Collector | Identical? |
|-----------|----------------|-----------------|------------|
| Struct fields | 8 fields | 8 fields (same names, different types for client/store/state) | Structurally identical |
| Options type | `Options` | `AssetCollectorOptions` | Field-for-field identical |
| `Run()` retry loop | Lines 86-154 (69 lines) | Lines 82-150 (69 lines) | Character-for-character identical except `collectOnce` call |
| `initializeState()` | Lines 156-190 (35 lines) | Lines 152-186 (35 lines) | Same logic, different types (PollState vs AssetPollState) |
| `collectOnce()` | Lines 192-301 (110 lines) | Lines 188-280 (93 lines) | Similar structure, different API calls and cursor types |
| Sort comparison | `ModificationDate` + `RefId` | `Updated` + `ID` (numeric for sort) | Different |
| Filter function | `filterNewAlerts` (standalone) | `filterNewAssets` (method) | Different style |
| Cursor comparison | `isCursorAhead` (standalone) | `isAssetAhead` (method) | Different: alerts use standalone function, assets use method |
| Forward progress | `ensureForwardProgress` (standalone) | `ensureForwardProgress` (method with numeric fallback) | Different: asset has numeric fallback |
| `waitWithContext` | Defined in alert_collector.go | Reused from alert_collector.go | Shared (same package) |

Key divergences:
1. Asset sort uses numeric ID (`assets[i].ID < assets[j].ID`) but cursor comparison uses string (`GetID() > RecordID`)
2. Asset `ensureForwardProgress` has a numeric ParseInt fallback that alert version does not
3. Alert uses `modification_date` server-side filter; asset fetches everything
4. Alert pagination: `len(alerts) == pageSize`; Asset: `pageNumber * 1000 < totalAssets`
5. Alert filter is a standalone function accepting logger; asset filter is a method on AssetCollector

### Namespace Resolution Logic

The Helm `_helpers.tpl` namespace template has non-trivial logic worth documenting:

```
if Release.Namespace != "" AND (values.namespace == "" OR values.namespace == "poller-express") AND Release.Namespace != values.namespace:
    use Release.Namespace (prefer helm install -n flag)
else if values.namespace != "":
    use values.namespace
else if Release.Namespace != "":
    use Release.Namespace
else:
    use "poller-express" (hardcoded fallback)
```

This means `helm install -n custom-ns poller-express ./chart` correctly uses `custom-ns`, even though `values.yaml` defaults to `namespace: poller-express`.

### CI Test Values Anomaly

The `ci/test-values.yaml` file references `source.baseURL` and `source.apiKey` (lines 4-5), but the actual Helm chart values use `cyberint.baseURL` and `cyberint.apiKey`. This means the CI test values may not actually override the correct fields, potentially causing chart-testing to use empty credentials. The `lint-test.yml` workflow compounds this by creating test values with `armis.apiKey` (line 49-51), which is from a different project entirely. This appears to be a copy-paste error.

### Helm Probe Configuration

Both liveness and readiness probes are **disabled by default** in values.yaml (`enabled: false`). The tilt-values.yaml (dev) enables them. This means production deployments without explicit probe enabling have no K8s-level health monitoring -- the health server runs but K8s doesn't check it.

---

## Delta Summary
- New items added: Detailed collector structural comparison (exact line counts, 5 key divergences), Helm namespace resolution logic, CI test values anomaly, probe disabled-by-default confirmation
- Existing items refined: All 5 Round 1 claims audited and confirmed correct
- Remaining gaps: None significant

## Novelty Assessment
Novelty: NITPICK
The collector comparison provides useful detail for the port but does not change the architectural model -- the duplication was already identified in Round 1 and Pass 5. The CI test values anomaly is a bug report, not an architectural finding. The namespace logic is a Helm template detail. No new subsystems, layers, or architectural patterns were discovered. The probe disabled-by-default was already noted in Round 1.

## Convergence Declaration
Pass 1 has converged -- findings are nitpicks, not gaps. The architecture is complete for specification purposes.

## State Checkpoint
```yaml
pass: 1
round: 2
status: complete
timestamp: 2026-04-13T23:55:00Z
novelty: NITPICK
```
