# Pass 1 Deep: Architecture -- Round 1

> Project: poller-bear
> Source: /Users/jmagady/Dev/prism/.references/poller-bear/
> Round: 1

---

## Audit of Broad Sweep Architecture Claims

### Entry Point Duality
The broad sweep identified two entry points but understated the difference:
- `main.go` (root): Initializes pprof server first, then calls `runner.Execute`, handles pprof shutdown with 5s timeout, exits with code 0 or 1.
- `cmd/collector/main.go`: Calls `runner.Execute` directly with no pprof setup. Just `os.Exit(1)` on error.
- **Architecture implication**: The root `main.go` is the production entry point (used in `make build`). The `cmd/collector` entry point is used in the Dockerfile (`go build -o /out/collector ./cmd/collector`). This means **Docker images do NOT have pprof support** unless `ENABLE_PPROF` is added to `cmd/collector/main.go`.

This is a **design inconsistency**: the Dockerfile builds `cmd/collector` which lacks pprof initialization, but the Makefile builds from `main.go` which includes pprof. The pprof documentation and environment variables are available in both, but only the Makefile binary actually starts the pprof server.

### Runner Orchestration Detail

The `runner.Execute` function is the true orchestration center. Its exact wiring sequence:

1. Nil-guard on context (defensive)
2. `signal.NotifyContext(ctx, SIGTERM, SIGINT)` -- creates cancellable context
3. Create JSON logger with timestamps
4. `config.DefaultConfig()` -> `config.LoadFromEnvironment(cfg)` -- overlay env vars
5. Parse log level (supports INFO, DEBUG, TRACE; TRACE maps to DEBUG)
6. `createStore(cfg.State, logger)` -- switch on `StoreTypeFile` / `StoreTypeMemory`
7. `claroty.NewHTTPClient(Config{BaseURL, Token, Timeout: 30s, Logger})`
8. Conditional sink creation: **if `cfg.Sink.Endpoint != ""` then create HTTPSender, else alertSink is nil**
9. `health.NewServer(cfg.Collector.HealthAddr)` -- always created
10. `collector.New(cfg, clarotyClient, store, opts)` -- wires everything
11. Start health server in goroutine
12. `c.Run(ctx)` -- blocking main loop
13. On context.Canceled: log shutdown, proceed to cleanup
14. On other error: shutdown health server, return error
15. Health server graceful shutdown with 5s timeout
16. Drain health server error channel

**New finding**: The sink is **optional**. If `VECTOR_ENDPOINT` is empty (which it isn't by default -- default is `http://localhost:4413`), the sink will be nil. The collector must handle nil sink gracefully.

### Health Server Architecture

The health server is more nuanced than the broad sweep described:

- `/health` and `/live` are **aliases** -- both call `handleLiveness`
- `/ready` checks **both** `ready` AND `alive` flags -- readiness implies liveness
- `ReadHeaderTimeout: 10s` is set on the HTTP server (Slowloris protection)
- `alive` starts `true`, `ready` starts `false`
- `Shutdown()` sets `alive` to `false` BEFORE calling `httpServer.Shutdown()` -- this means in-flight readiness checks will fail during shutdown window

### Profiling Server Architecture

Not covered in broad sweep's architecture section:

- Gated by `ENABLE_PPROF` env var (must be parseable by `strconv.ParseBool`)
- Default address: `localhost:3030` (loopback only)
- Non-loopback binding triggers a warning log
- `/debug/pprof/cmdline` is **explicitly blocked** (returns 404) to prevent exposing process arguments -- security hardening
- Eager listener binding: the TCP listener is opened before starting the goroutine, so bind failures are caught synchronously
- HTTP server has restrictive timeouts: Read 10s, Write 120s (for profiling), ReadHeader 5s, Idle 60s

### Transport Layer Architecture

Broad sweep listed transport settings but missed architectural significance:

- `transport.NewTransport` is used by **both** the Claroty client (via `claroty.NewHTTPClient`) and the sink (via `sink.NewHTTPSender`)
- Both clients use `transport.DefaultConfig()` -- they share the same connection pool settings
- The `Config.IsZero()` method exists but is never called in production code (utility for testing)
- `Config.Validate()` enforces non-negative values on all numeric settings
- `InsecureSkipVerify` is configurable in the transport but **not exposed via environment variables** -- it requires code changes to enable

### Deployment Architecture

The broad sweep's Kubernetes section was accurate but missed several details from the Helm chart:

**Security posture:**
- Container runs as UID 65532 (distroless nonroot)
- `fsGroup: 65532` on pod security context -- ensures PVC is writable
- `readOnlyRootFilesystem: true`
- `allowPrivilegeEscalation: false`
- `capabilities.drop: ALL`
- RBAC is **disabled by default** (`rbac.create: false`) but templates exist for configmaps/secrets read access

**Health probes are disabled by default:**
- Both `livenessProbe.enabled` and `readinessProbe.enabled` default to `false`
- This means Kubernetes does not monitor the collector's health unless explicitly enabled
- The health server runs regardless (always started in runner.go)

**Secret management flexibility:**
- Claroty API key supports 4 methods: `existingSecret`, `apiKeySecretName+apiKeySecretKey`, `apiKey` (plaintext in values), or empty
- Sink credentials support 3 methods: `existingSecret`, `secretName`, or plaintext values
- The template uses a priority cascade with if/else-if chains

**Persistence:**
- PVC defaults to 100Mi, ReadWriteOnce
- State file mounts at `/var/lib/poller-bear/state.json`
- `existingClaim` support for bring-your-own PVC

---

## Revised Component Diagram

```
                        Docker Entry Point
                              |
                    cmd/collector/main.go
                              |
                        runner.Execute(ctx)
                              |
            +---------+-------+-------+---------+
            |         |       |       |         |
     config.Load  state.New  claroty  sink.New  health.New
            |         |    .NewHTTP   (optional)    |
            |         |    Client()       |         |
            |    +----+----+    +---------+    +----+
            |    |File|Mem |    |HTTPSender|    |Server|
            |    +----+----+    +----------+    +------+
            |         |              |               |
            v         v              v               v
                collector.New(cfg, client, store, opts)
                              |
                        collector.Run(ctx)
                              |
                  +--- collectOnce() <---+
                  |          |           |
                  |    [9 sources]       |
                  |          |      (ticker 30s or
                  |    sink.Send*   immediate if
                  |    state.Save*  hasMore=true)
                  |          |           |
                  +----------+-----------+
                             |
                    retry with exponential
                    backoff on error
```

**Makefile Entry Point** (different path):
```
main.go -> profiling.Start() -> runner.Execute(ctx)
                                    (pprof available)
```

---

## CI/CD Architecture

```
                    Push to main
                         |
          +--------------+---------------+
          |              |               |
    collector-tests   build.yml    security-scan
    (go test ./...)   (Docker)    (gosec+govulncheck
          |              |         +staticcheck)
          |         Build Image         |
          |              |              |
          |         Push to         (informational
          |        Cloudsmith        only, non-blocking)
          |              |
          +--------------+
                         |
                    PR triggers:
                    + lint-test (Helm chart only)
                    + collector-tests
                    + security-scan
                    + build (no push)
```

Key observations:
- Self-hosted runners (`[self-hosted, Ubuntu, Common]`) for all workflows
- `step-security/harden-runner` with egress audit on every job
- Docker base image pulled from ECR mirror (`830652780623.dkr.ecr.us-east-1.amazonaws.com/docker-hub/library/golang`)
- Image pushed to Cloudsmith registry (`docker.cloudsmith.io/1898-and-co/poller-bear/poller-bear`)
- Security scans run on daily cron (`0 6 * * *`) in addition to PR/push triggers

---

## Cross-Cutting Concerns

### Logging Architecture
- `charmbracelet/log` v0.4.2 with JSON formatter
- Timestamps always enabled (`ReportTimestamp: true`)
- Log level configurable via `POLLER_BEAR_LOG_LEVEL` (INFO, DEBUG, TRACE -> maps to DEBUG)
- Logger is dependency-injected via constructor params (claroty client, sink, and via collector Options)
- Profiling server uses package-level `log.Info/Error/Warn` (not injected)

### Error Architecture
- 15 sentinel errors in `apperrors` package (not 13 as broad sweep stated)
- All use `fmt.Errorf("%w: ...")` wrapping pattern
- `ErrCursorRegression` is defined but **never used** (dead sentinel, confirmed in Pass 2 R2)
- Error classification allows `errors.Is()` matching at any call depth

---

## Delta Summary
- New items added: 2 entry point paths documented with behavioral differences, pprof cmdline blocking, health probe disabled-by-default finding, sink optional finding, CI architecture with ECR mirror + Cloudsmith, 4 Claroty secret methods
- Existing items refined: sentinel error count (13 -> 15), transport shared between both clients, health readiness double-checks alive flag
- Remaining gaps: Internal structure of collector.go (Run loop, collectOnce orchestration of 9 sources), exact initialization sequence of 9 data sources within collector.New

## Novelty Assessment
Novelty: SUBSTANTIVE
The entry point duality (Docker builds cmd/collector without pprof while Makefile builds main.go with pprof) is an architectural design inconsistency that changes how we would spec the system. The disabled-by-default health probes are a significant deployment architecture detail. The optional sink is a behavioral discovery.

## Convergence Declaration
Another round needed -- collector internal architecture (Run loop, collectOnce sequencing, initialization) not yet deep-dived.

## State Checkpoint
```yaml
pass: 1
round: 1
status: complete
files_scanned: 25
timestamp: 2026-04-13T23:35:00Z
novelty: SUBSTANTIVE
next_pass: 1-r2
```
