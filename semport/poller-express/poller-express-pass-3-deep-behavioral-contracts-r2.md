# Pass 3 Deep: Behavioral Contracts -- poller-express (Round 2)

## Audit of Round 1 Claims

### Hallucination Class: Over-extrapolated Lists

**Contract count:** Round 1 claimed 60 contracts. Verified: 8 + 11 + 5 + 6 + 9 + 9 + 9 + 2 + 1 = 60. Correct.

### Hallucination Class: Miscounted Enumerations

**BC-1.007 retry boundary:** Round 1 stated "retryCount > maxRetries (not >=)". Verified from code: `retryCount++` happens first, then `retryCount > c.cfg.Collector.MaxRetries` is checked. With MaxRetries=5, the collector performs 5 retry attempts (retryCount goes 1..5, all pass), and fails fatally on the 6th failure (retryCount=6 > 5). Error reports `attempts=5` (retryCount-1). This is correct as stated.

**BC-1.007 additional detail:** The guard `c.cfg.Collector.MaxRetries > 0` means MaxRetries=0 disables the retry limit entirely -- infinite retries. This was not explicitly stated in Round 1.

### Hallucination Class: Pattern Conflation

**BC-4.006 enrichPayload:** Round 1 stated "Not using struct marshaling for the outer wrapper." Verified: the code uses `buf.WriteString('{"data":')` followed by `enc.Encode(payload)`, then `buf.WriteString(',"xmp":')` followed by `enc.Encode(XMPMetadata{...})`. This is manual JSON construction, not `json.Marshal(EnrichedPayload{...})`. Correct.

**BC-5.009 readiness condition:** Round 1 stated "requires both alive AND ready." Code: `if !s.ready.Load() || !s.alive.Load()` returns 503. Correct -- both must be true for 200.

### Verification of Untested Gap Claims

1. **Alert collector retry loop (BC-1.007):** Confirmed no test file tests the retry loop directly for alerts. The asset_collector_test.go also does not test the retry loop. Gap confirmed.

2. **Alert state initialization (BC-1.008):** Confirmed no test. Asset collector equivalent IS tested (3 tests). Gap confirmed.

3. **Alert modification_date filter (BC-1.006):** Confirmed no test. Gap confirmed.

4. **Alert sort order (BC-1.002):** Confirmed no test. Gap confirmed.

5. **QueryFingerprint order-independence (BC-3.004):** Confirmed no test. Gap confirmed.

6. **Response size limit (BC-7.007):** Confirmed no test. Gap confirmed.

7. **Runner orchestration:** Confirmed no test file exists in `internal/app/runner/`. Gap confirmed.

8. **cookieTransport:** Confirmed no test. Gap confirmed.

9. **extractCustomerIDFromURL:** Confirmed no test. Gap confirmed.

10. **Graceful shutdown:** Confirmed no test. Gap confirmed.

---

## New Contracts Discovered in Round 2

### Subsystem 10: Runner Orchestration

### BC-10.001: Runner exits on first collector error

**Preconditions:** Both collectors running as goroutines

**Postconditions:**
- Select blocks on errCh (buffered size 2) and ctx.Done()
- First collector error received causes Execute() to proceed to shutdown
- The surviving collector is NOT explicitly cancelled (it continues running until process exits)
- context.Canceled errors are swallowed (Execute returns nil)
- Non-canceled errors are returned as-is

**Evidence:** `runner.go:162-193`
**Confidence:** HIGH (from code; no test)

### BC-10.002: Health server gets 5-second shutdown grace period

**Preconditions:** Execute() received an error or context cancellation

**Postconditions:**
- `healthServer.Shutdown()` called with 5s timeout context
- Shutdown errors for Canceled, DeadlineExceeded, and ErrServerClosed are silently ignored
- Other shutdown errors logged as warnings
- healthErrCh is drained (non-blocking select) -- if ListenAndServe returned an error other than ErrServerClosed, it is returned

**Evidence:** `runner.go:172-186`
**Confidence:** HIGH (from code; no test)

### BC-10.003: MaxRetries=0 means infinite retries

**Preconditions:** Config with MaxRetries=0

**Postconditions:**
- Guard `c.cfg.Collector.MaxRetries > 0` is false
- Retry count check is skipped
- Collector retries indefinitely until success or context cancellation

**Evidence:** `alert_collector.go:122`, `asset_collector.go:118`
**Confidence:** HIGH (from code; no test)

### BC-10.004: Customer ID auto-extraction from URL

**Preconditions:** ASSET_CUSTOMER_ID not set, base URL is `https://<subdomain>.cyberint.io`

**Postconditions:**
- Subdomain extracted as customer ID
- Skips "api" and "www" subdomains (returns empty)
- Requires >= 3 dot-separated parts after stripping protocol
- Returns empty string if URL doesn't match pattern

**Evidence:** `runner.go:223-239`
**Confidence:** HIGH (from code; no test)

### BC-10.005: Shared HTTP client with 30s timeout and cookie transport

**Preconditions:** Runner initializing

**Postconditions:**
- Single http.Client created with 30s timeout
- cookieTransport wraps http.DefaultTransport
- access_token cookie added to every request via RoundTrip
- Same client instance used by both alert (via cyberint.APIClient) and asset (via asset.Client) collectors

**Evidence:** `runner.go:59-65`, `runner.go:78-79`, `runner.go:120`
**Confidence:** HIGH (from code; no test)

### BC-10.006: Sink is optional -- nil sink means no forwarding

**Preconditions:** VECTOR_ENDPOINT not set

**Postconditions:**
- sinkSender remains nil
- Warning logged: "sink disabled; endpoint not configured"
- Collectors receive nil sink in Options
- collectOnce checks `if c.sink != nil` before calling Send
- Records are fetched, sorted, filtered, cursor advanced -- but NOT forwarded

**Evidence:** `runner.go:82-97`, `alert_collector.go:255`, `asset_collector.go:234`
**Confidence:** HIGH (from code; partially tested via asset collector tests with no sink)

### BC-10.007: Log level TRACE is aliased to DEBUG

**Preconditions:** POLLER_LOG_LEVEL set to "TRACE"

**Postconditions:** Logger level set to DebugLevel

**Evidence:** `runner.go:215-216`
**Confidence:** HIGH (from code; no test)

---

## Corrections to Round 1 Contracts

### BC-1.007 (amended): MaxRetries=0 disables retry limit

Added clarification: When `MaxRetries=0`, the guard `MaxRetries > 0` is false, so the retry exhaustion check is never reached. The collector retries indefinitely. This is a behavioral subtlety not captured in Round 1.

### BC-4.003 (clarification): EnrichedPayload uses manual JSON construction

The `enrichPayload` method does NOT use `json.Marshal` on an `EnrichedPayload` struct. Instead it manually writes `{"data":` + encoded payload + `,"xmp":` + encoded XMPMetadata + `}`. The `EnrichedPayload` struct exists but is only used for READING (in tests, via `json.Unmarshal`), never for writing. This explains why the struct has `json.RawMessage` for Data.

### BC-2.001 (clarification): Asset requests do not use Type/Status filters

The `collectOnce` method in `asset_collector.go:192-195` builds `GetAssetsRequest{CustomerID: ..., PageNumber: 1}` -- it does NOT set Type or Status filters. This means all asset types and statuses are fetched. The `GetAssetsRequest` struct supports these filters but they are unused in practice.

---

## Updated Contract Coverage Summary

| Subsystem | Total Contracts | HIGH Confidence | MEDIUM | LOW |
|-----------|----------------|-----------------|--------|-----|
| 1: Alert Collector | 8 | 7 | 0 | 0 |
| 2: Asset Collector | 11 | 11 | 0 | 0 |
| 3: State Management | 5 | 3 | 1 | 0 |
| 4: Sink Delivery | 6 | 6 | 0 | 0 |
| 5: Health Server | 9 | 9 | 0 | 0 |
| 6: Configuration | 9 | 7 | 0 | 0 |
| 7: Asset Client | 9 | 8 | 1 | 0 |
| 8: Profiling | 2 | 2 | 0 | 0 |
| 9: Utilities | 1 | 0 | 1 | 0 |
| 10: Runner Orchestration | 7 | 7 | 0 | 0 |
| **Total** | **67** | **60** | **3** | **0** |

### Updated Gaps List

Untested behaviors (confirmed in Round 2 audit):
1. Alert collector retry loop (no test)
2. Alert collector state initialization (no test; asset equivalent tested)
3. Alert modification_date filter construction (no test)
4. Alert sort order determinism (no test)
5. QueryFingerprint order-independence (no test)
6. Asset client 10 MiB response limit (no test)
7. Runner orchestration (no tests at all)
8. cookieTransport cookie injection (no test)
9. extractCustomerIDFromURL (no test)
10. Graceful shutdown sequence (no test)
11. MaxRetries=0 infinite retry behavior (no test)
12. TRACE->DEBUG log level alias (no test)
13. Surviving collector not cancelled when sibling fails (no test; design gap)

---

## Delta Summary
- New items added: 7 runner orchestration contracts (BC-10.001 through BC-10.007)
- Existing items refined: 3 corrections/clarifications to Round 1 contracts
- Remaining gaps: 13 untested behaviors cataloged (increased from 10 due to 3 new findings)

## Novelty Assessment
Novelty: NITPICK
The 7 new runner orchestration contracts describe the wiring and lifecycle management layer that was already implicitly understood from the broad sweep. The behavioral patterns (first-error-exits, shared client, optional sink) were described in the architecture section. The MaxRetries=0 infinite retry behavior and TRACE alias are edge cases. The critical discovery that the surviving collector is NOT cancelled when its sibling fails is a design observation, not a new behavioral contract -- it describes the absence of behavior. No new entities, subsystems, or fundamental behavioral patterns were discovered.

## Convergence Declaration
Pass 3 has converged -- findings are nitpicks, not gaps. The behavioral contracts are complete for specification purposes.

## State Checkpoint
```yaml
pass: 3
round: 2
status: complete
timestamp: 2026-04-13T00:00:00Z
novelty: NITPICK
```
