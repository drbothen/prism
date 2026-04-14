# Pass 1 Deep: Architecture -- poller-cobra (Round 2)

> Convergence deepening round 2. Hallucination audit + cross-reference with PROFILING_FINDINGS.md.

---

## Round 1 Hallucination Audit

### Health Server Shutdown Gap

**Round 1 claimed:** Health server is never gracefully shut down.
**Verified:** runner.go:111-116 starts `healthServer.ListenAndServe()` in a goroutine. No call to `healthServer.Shutdown()` exists anywhere in runner.go or main.go. PROFILING_FINDINGS.md Finding #9 independently confirms this. **Claim is correct.**

### Dead Code in source.go

**Round 1 claimed:** source.go is dead code (183 lines), never called by runner or collector.
**Verified:** Searched for `NewSource`, `NewSourceFromEnv`, `FetchRecords`, `Source{` in all files outside source.go and its README. None found in any Go source file. The `crowdstrike/README.md` shows example usage but the actual codebase never calls it. **Claim is correct.**

### Logging Level Bug

**Round 1 claimed:** WARN/ERROR/FATAL are unreachable despite being accepted by validation.
**Verified:** config.go:430-438 accepts DEBUG/INFO/WARN/ERROR/FATAL. runner.go:131-141 `parseLogLevel` has cases for "", "INFO", "DEBUG", "TRACE" only. Default returns error, triggering fallback to INFO. PROFILING_FINDINGS.md Finding #10 independently confirms this. **Claim is correct.**

### Connection Pool Impact

**Round 1 claimed:** Response body not drained on success path prevents connection reuse.
**Verified:** http_sender.go:105-118. On success path (status < 400), execution reaches line 117 (log + return nil). The deferred body close at line 105 fires, but body was never read. PROFILING_FINDINGS.md Finding #3 independently confirms this and provides the `io.Copy(io.Discard, resp.Body)` fix. **Claim is correct.**

---

## Cross-Reference with PROFILING_FINDINGS.md

PROFILING_FINDINGS.md contains 12 findings. Mapping to architecture:

| PROFILING Finding | Already in our analysis? | Category |
|-------------------|--------------------------|----------|
| #1 FileStore not implemented | Yes (broad sweep, Pass 4 R-010) | Reliability |
| #2 Single-record sink delivery | Yes (broad sweep, Pass 4 P-002) | Performance |
| #3 No response body draining | Yes (Pass 1 R1, Pass 4 P-009) | Performance |
| #4 Rate limiter memory growth | Yes (broad sweep, Pass 4 R-011) | Memory |
| #5 Test coverage ~10% | Yes (Pass 5 R1) | Quality |
| #6 Ping does not verify API | **STALE** -- code now has real Ping | N/A |
| #7 Double JSON serialization | Yes (Pass 3 R2 verified as correct by design) | Performance |
| #8 alertToMap allocation | Not previously in our analysis | Performance |
| #9 Health server not shut down | Yes (Pass 1 R1) | Reliability |
| #10 parseLogLevel bug | Yes (Pass 1 R1, Pass 4 O-002) | Correctness |
| #11 Stub methods | Yes (broad sweep) | Completeness |
| #12 Deprecated config fields | Yes (Pass 2 R1) | Maintenance |

**New from PROFILING_FINDINGS.md:** Finding #8 (alertToMap allocation pattern) provides a specific performance insight: `make(map[string]interface{})` without size hint for ~32 keys causes ~3,000 small allocations per 100-alert batch. This was not previously documented in our architecture analysis.

### PROFILING_FINDINGS.md Accuracy Assessment

| Finding | Current Accuracy |
|---------|-----------------|
| #1-5 | Accurate |
| #6 | **STALE** -- Ping now makes real API call |
| #7 | Technically accurate but overstated (RawMessage avoids re-encoding) |
| #8-12 | Accurate |

---

## Architecture Refinement: Performance Hot Path

From PROFILING_FINDINGS.md "Performance Hot Path Summary":

```
FetchAlerts (HTTP to CrowdStrike)      ~200-500ms per call
  -> alertToMap x N                     ~3000 allocations per 100 alerts
    -> sort + filter                    negligible
      -> Send x N (sequential HTTP)     ~500-2000ms for 100 alerts
        -> enrichPayload (2x marshal)   measurable CPU per record
          -> Save (MemoryStore)          negligible (in-memory)
```

The two dominant bottlenecks are:
1. **Sequential sink delivery** (100 HTTP round-trips)
2. **Missing connection reuse** (no body draining)

This hot path analysis is relevant for the Rust rewrite's architecture decisions.

---

## Final Architecture Assessment

The poller-cobra architecture is a straightforward single-process polling loop with clean interface boundaries. Key characteristics:

1. **Clean separation of concerns** -- each package has one responsibility
2. **Interface-based dependency injection** -- testable but only partially tested
3. **Dead code exists** -- source.go represents an abandoned abstraction layer
4. **Infrastructure gaps** -- FileStore, health server shutdown, log level parsing
5. **Performance design** -- adequate for low-medium volume, bottleneck at high volume due to single-record delivery
6. **Security hardened** -- distroless, nonroot, capabilities dropped, secrets via file mounts

---

## Delta Summary
- New items added: 1 (alertToMap allocation pattern from PROFILING_FINDINGS.md)
- Existing items refined: 0 (all R1 claims verified correct)
- Remaining gaps: None

## Novelty Assessment
Novelty: NITPICK
The alertToMap allocation pattern (Finding #8 from PROFILING_FINDINGS.md) is a micro-optimization detail, not an architectural pattern change. All R1 claims were independently verified by PROFILING_FINDINGS.md, confirming convergence. The hot path analysis is a useful summary but contains no new information beyond what was already documented across Pass 1 R1 and Pass 4 R1. Removing this round's findings would not change how you'd spec the system.

## Convergence Declaration
Pass 1 has converged -- findings are nitpicks, not gaps. The architecture model is complete and cross-validated against PROFILING_FINDINGS.md.

## State Checkpoint
```yaml
pass: 1
round: 2
status: complete
files_scanned: all
timestamp: 2026-04-13T00:00:00Z
novelty: NITPICK
```
