# Pass 2 Deep: Domain Model -- poller-cobra (Round 2)

> Convergence deepening round 2. Auditing Round 1 for hallucinations, hunting missed elements from Helm chart, Dockerfile, and deployment artifacts.

---

## Round 1 Hallucination Audit

### Over-extrapolated lists
- **alertToMap field count:** Round 1 claimed 31 named fields + overflow. Recount from api.go:204-275: id, composite_id, aggregate_id, cid (4) + timestamp, created_timestamp, updated_timestamp (3) + status, severity, severity_name, confidence (4) + name, display_name, description, type, product, platform (6) + tactic, tactic_id, technique, technique_id, objective (5) + agent_id (1) + cmdline, filename, filepath, sha256, md5 (5) + assigned_to_name, assigned_to_uuid (2) + resolution (1) + tags (1) = **32 named fields** + overflow map. Round 1 said 31. Corrected to 32.

### Miscounted enumerations
- **Sentinel errors:** Round 1 (Pass 3) counted 16. Recount from errors.go: ErrConfigValidationFailed, ErrStateNotFound, ErrQueryFingerprintMismatch, ErrCursorRegression, ErrCollectorRetriesExceeded, ErrCollectorStateLoad, ErrCollectorStatePersist, ErrSourceConfigMissing, ErrSourceRequestBuild, ErrSourceRequestExec, ErrSourceUnexpectedStatus, ErrSourceDecode, ErrSinkConfigMissing, ErrSinkRequestBuild, ErrSinkDelivery, ErrConfigLoad, ErrClientNotInitialized = **17 sentinel errors**. Round 1 (Pass 3) missed ErrConfigLoad in some counts. Corrected.

### Pattern conflation
- None found. The dual Client interface pattern (crowdstrike.Client vs collector.CrowdStrikeClient) was correctly identified as separate definitions.

### Basename conflation
- **Config confusion:** There are two distinct `Config` types: `crowdstrike.Config` (api client construction) and `config.Config` (application-level). Round 1 correctly distinguished these.

### Inflated metrics
- None found. Claims about unused interfaces (Record, Source) verified via grep.

---

## Newly Discovered Domain Elements

### Helm Chart Domain Constraints (deployment.yaml)

The Helm chart encodes several domain constraints not visible in Go source:

#### Migration Guards (deployment.yaml:1-16)

5 deprecated values that trigger hard failure if present:
1. `crowdstrike.apiKey` -- removed in v0.3.0
2. `crowdstrike.apiKeySecretName` -- removed in v0.3.0
3. `crowdstrike.apiKeySecretKey` -- removed in v0.3.0
4. `crowdstrike.baseURL` -- removed in v0.3.0
5. `crowdstrike.timeout` -- removed in v0.3.0

These correspond to the deprecated `BaseURL` and `APIKey` fields in CrowdStrikeConfig.

#### Credential Resolution Priority (deployment.yaml:17-25)

Helm template enforces: either `existingSecret` OR `clientId`+`clientSecret` must be provided. This is a deployment-time validation layer above the application-level config validation.

Sink credentials have a 3-level priority: `existingSecret` > `secretName` > plaintext values.

#### Environment Variables NOT Exposed in Helm values.yaml

The following env vars are only settable via `extraEnv`:
- `CROWDSTRIKE_REGION` (defaults to "us-1" in Go)
- `CROWDSTRIKE_DATA_SOURCE` (defaults to "alerts" in Go)
- `CROWDSTRIKE_LIMIT` (defaults to 100 in Go)
- `CROWDSTRIKE_FILTER` (defaults to "" in Go)
- `COLLECTOR_INTERVAL` (set in values.yaml as `collector.interval` but NOT wired to env var in deployment.yaml -- **BUG**)
- `COLLECTOR_MAX_RETRIES` (not exposed)
- `COLLECTOR_RETRY_BASE_DELAY` (not exposed)
- `COLLECTOR_RETRY_MAX_DELAY` (not exposed)
- `HEALTH_ADDR` (not exposed -- hardcoded via containerPort)
- `ENABLE_PPROF` / `PPROF_ADDR` (not exposed)

**Finding:** `collector.interval` appears in values.yaml but the deployment template never sets `COLLECTOR_INTERVAL` as an env var. The value in values.yaml is dead configuration. The actual interval always uses the Go default of 30s unless overridden via `extraEnv`.

### Deployment Domain Model

#### Container Identity
- Image: `docker.cloudsmith.io/1898-and-co/poller-cobra/poller-cobra`
- Base: `gcr.io/distroless/static-debian12:nonroot`
- User: nonroot (UID 65532, GID 65532)
- Build: Multi-stage, CGO_ENABLED=0, static binary, trimpath, stripped symbols
- Entrypoint: `/app/collector`

#### Security Context (values.yaml:92-108)
- Pod: runAsNonRoot, runAsUser/Group/fsGroup=65532, seccomp RuntimeDefault
- Container: allowPrivilegeEscalation=false, drop ALL capabilities, readOnlyRootFilesystem

#### Storage
- PVC: 100Mi, ReadWriteOnce, mounted at `/var/lib/poller-cobra`
- fsGroup=65532 ensures PVC is writable by nonroot user

#### Probes (values.yaml:110-131)
- Liveness: disabled by default. /health on http port. 10s initial delay, 10s period, 3 failures.
- Readiness: disabled by default. /ready on http port. 5s initial delay, 10s period, 3 failures.

Both probes are disabled by default, meaning Kubernetes won't restart the pod on health check failure or remove it from service endpoints on readiness failure.

#### RBAC (values.yaml:157-163)
- Role: get/list configmaps+secrets, watch secrets
- Purpose: Credential rotation via K8s secret watches

#### Service
- ClusterIP on port 7322

### Makefile Domain Artifacts

- `make vector` -- runs Vector with local config and `scripts/setup.sh` environment
- `make run` -- runs collector with `scripts/setup.sh` environment
- `make lint` -- uses golangci-lint v2 via tools module
- `make vuln` -- uses govulncheck via tools module

### tools/tools.go

<The tools module imports golangci-lint and govulncheck as development dependencies.>

### CrowdStrikeConfig Deprecated Fields

The `BaseURL` and `APIKey` fields (config.go:103-104) are never read by any code path in LoadFromEnvironment or anywhere else. They exist purely for backward compatibility with earlier config versions. The Helm migration guards enforce that v0.2.0 chart values (which used these) are not accidentally used with v0.3.0.

---

## Corrected Entity Count

| Category | Count | Items |
|----------|-------|-------|
| Core domain entities | 3 | Alert, Detection (stub), Host (stub) |
| State entities | 3 | PollState, Cursor, QueryFingerprint |
| Audit entities | 1 | BatchReceipt |
| Transport entities | 2 | EnrichedPayload, XMPMetadata |
| Config aggregates | 7 | Config, CrowdStrikeConfig, CollectorConfig, SinkConfig, LoggingConfig, XMPConfig, StateConfig |
| Infrastructure entities | 3 | crowdstrike.Config, RateLimitConfig, StoreType |
| Record adapters (unused) | 3 | AlertRecord, DetectionRecord, HostRecord |
| Interfaces | 6 | Record (unused), Client, CrowdStrikeClient, Sender, Store, Reporter |
| Service types | 5 | HTTPClient, Source (unused), Collector, AlertCollector, HTTPSender |
| Infrastructure types | 2 | MemoryStore, health.Server |
| **Total** | **35** | |

---

## Corrected Bounded Context Map

No changes from Round 1. The bounded contexts are accurately mapped:
- Source Context (crowdstrike) -- includes unused Source/Record abstractions
- Collection Context (collector) -- core orchestration
- State Context (state) -- persistence abstractions
- Sink Context (sink) -- downstream delivery
- Health Context (health) -- operational readiness
- Config Context (config) -- runtime configuration
- Error Context (apperrors) -- sentinel errors
- Profiling Context (profiling) -- opt-in diagnostics
- Runner Context (runner) -- application wiring
- Deployment Context (Helm) -- K8s deployment constraints

---

## Delta Summary
- New items added: 1 (deployment domain model with Helm constraints, migration guards, security context, probe defaults)
- Existing items refined: 2 (alertToMap field count corrected 31->32, sentinel error count corrected 16->17)
- Remaining gaps: None significant. The `scripts/setup.sh` was not read but is only used for local dev env setup.

## Novelty Assessment
Novelty: NITPICK
The Helm chart findings (dead `collector.interval` config, disabled probes, migration guards) are deployment-level refinements that do not change the application domain model. The field count corrections (31->32, 16->17) are minor arithmetic fixes. No new entities, relationships, or state machines were discovered. Removing this round's findings would not change how you'd spec the system.

## Convergence Declaration
Pass 2 has converged -- findings are nitpicks, not gaps. The domain model is complete across all Go source files, test files, and deployment artifacts.

## State Checkpoint
```yaml
pass: 2
round: 2
status: complete
files_scanned: 18 (Go) + 10 (Helm) + 1 (Dockerfile) + 1 (Makefile)
timestamp: 2026-04-13T00:00:00Z
novelty: NITPICK
```
