# Coverage Audit: poller-express

## Method

1. Full source tree inventoried (319 files, 33 directories)
2. Each package/directory grep-searched against all 13 prior analysis files
3. Coverage matrix built: package x pass -> covered (yes/partial/no)
4. Blind spots identified and source files read for substantive gaps
5. Audit contracts (BC-AUDIT-NNN) extracted for uncovered behaviors

---

## Coverage Matrix

| Package/Directory | Pass 0 | Pass 1 | Pass 2 | Pass 3 | Pass 4 | Pass 5 | Verdict |
|-------------------|--------|--------|--------|--------|--------|--------|---------|
| `cmd/collector/` | YES | YES | -- | YES | -- | -- | COVERED |
| `internal/app/runner/` | YES | YES | YES | YES | YES | YES | COVERED |
| `internal/apperrors/` | YES | YES | -- | YES | YES | YES | COVERED |
| `internal/asset/` | YES | YES | YES | YES | YES | YES | COVERED |
| `internal/collector/` | YES | YES | YES | YES | YES | YES | COVERED |
| `internal/config/` | YES | YES | YES | YES | YES | YES | COVERED |
| `internal/health/` | YES | YES | YES | YES | YES | YES | COVERED |
| `internal/profiling/` | YES | YES | -- | YES | YES | -- | COVERED |
| `internal/sink/` | YES | YES | YES | YES | YES | YES | COVERED |
| `internal/state/` | YES | YES | YES | YES | YES | YES | COVERED |
| `pkg/cyberint/` (generated models) | YES | PARTIAL | YES | -- | -- | -- | COVERED (models are opaque pass-through) |
| `pkg/cyberint/client.go` | PARTIAL | YES | -- | -- | -- | -- | **PARTIAL** |
| `pkg/cyberint/client_test.go` | NO | NO | NO | NO | NO | NO | **BLIND SPOT** |
| `pkg/cyberint/api_public.go` | PARTIAL | NO | NO | NO | NO | NO | **BLIND SPOT** |
| `pkg/cyberint/response.go` | NO | NO | NO | NO | NO | NO | **BLIND SPOT** |
| `pkg/cyberint/utils.go` | NO | NO | NO | NO | NO | NO | **BLIND SPOT** |
| `pkg/cyberint/configuration.go` | PARTIAL | NO | NO | NO | NO | NO | **BLIND SPOT** |
| `pkg/cyberint/test/` | NO | NO | NO | NO | NO | NO | **BLIND SPOT** |
| `pkg/validate/` | YES | YES | -- | YES | -- | YES | COVERED |
| `deploy/helm/` | YES | YES | -- | -- | YES | -- | COVERED |
| `.github/workflows/` | YES | -- | -- | -- | YES | YES | COVERED |
| `docs/specs/` (4 API spec files) | NO | NO | NO | NO | NO | NO | **BLIND SPOT** |
| `docs/PROFILING.md` | NO | NO | NO | NO | NO | NO | **BLIND SPOT** |
| `docs/*.md` (100+ generated docs) | NO | NO | PARTIAL | NO | NO | NO | Low-priority (generated) |
| `scripts/pprof-harness.sh` | NO | NO | NO | NO | PARTIAL | NO | **PARTIAL** |
| `scripts/setup.sh` | NO | NO | NO | NO | NO | NO | **BLIND SPOT** |
| `Makefile` | PARTIAL | NO | NO | NO | NO | NO | **PARTIAL** |
| `Dockerfile` | YES | YES | -- | -- | YES | -- | COVERED |
| `INGESTION.md` | NO | NO | NO | NO | NO | NO | Not analysis; copy of broad sweep |
| `CLAUDE.md` | PARTIAL | -- | -- | -- | -- | -- | Reference doc, not source |
| `vector.yaml` | PARTIAL | NO | NO | NO | NO | NO | **PARTIAL** |
| `collector` (binary) | NO | NO | NO | NO | NO | NO | **BLIND SPOT** (build artifact) |
| `tools/` | YES | -- | -- | -- | -- | YES | COVERED |

---

## Blind Spot Analysis

### BS-001: Generated Client Infrastructure (pkg/cyberint/*.go non-model files)

**Files**: `client.go`, `client_test.go`, `api_public.go`, `configuration.go`, `response.go`, `utils.go`

**Prior coverage**: Mentioned as "generated, do not edit" but internal structure never analyzed.

**Findings from source review**:

1. **10 API methods available, only 1 used**: The generated `PublicAPI` interface exposes 10 methods:
   - `GetAlertsApiV1AlertsPost` -- **USED** by alert collector
   - `GetFullAlertByRefIdApiV1AlertsAlertRefIdGet` -- NOT used
   - `GetAlertMetadataApiV1AlertsMetadataGet` -- NOT used
   - `GetAlertAnalysisReportApiV1AlertsAlertRefIdAnalysisReportGet` -- NOT used
   - `GetAlertAttachmentApiV1AlertsAlertRefIdAttachmentsAttachmentIdGet` -- NOT used
   - `GetAlertIndicatorApiV1AlertsAlertRefIdIndicatorsIndicatorIdGet` -- NOT used
   - `UpdateAlertsStatusApiV1AlertsStatusPut` -- NOT used
   - `ExternalGetCurrentRisksApiV1AnalyticsEnvironmentRisksCurrentGet` -- NOT used
   - `ExternalGetMonthlyRisksApiV1AnalyticsEnvironmentRisksMonthlyGet` -- NOT used
   - `ExternalGetOverallScoreOvertimeApiV1AnalyticsEnvironmentOverallScorePerMonthGet` -- NOT used

   **Implication for port**: Only `GetAlertsApiV1AlertsPost` needs implementation. The other 9 methods are dead code from the generated client.

2. **NullableTime.UnmarshalJSON delegates to CyberintTime**: The generated `utils.go` file's `NullableTime.UnmarshalJSON` method (lines 327-345) was modified to delegate to `CyberintTime` for parsing. This is a hand-written modification inside the generated code, in addition to `cyberint_time.go`. This means there are actually **two** hand-written files in `pkg/cyberint/`, plus one hand-written modification to a generated file.

3. **Nullable type system**: `utils.go` defines `Nullable{Bool,Int,Int32,Int64,Float32,Float64,String,Time}` wrappers using `value *T` + `isSet bool` pattern. These handle JSON `null` vs absent field distinction. The `NullableAlertClosureReason` mentioned in the Alert model is a separate generated type using this same pattern.

4. **Strict JSON decoder**: `utils.go` provides `newStrictDecoder` which uses `DisallowUnknownFields()`. This means API responses with unexpected fields will fail to parse. This could be a forward-compatibility risk if Cyberint adds new fields.

5. **Configuration.Servers default**: `configuration.go` has a single server entry with URL `/alert`. This means when `runner.go` sets `cfg.Servers[0].URL = baseURL + "/alert"`, it overwrites the generated default. The path `/alert` is baked into the server URL, and then `api_public.go` appends `/api/v1/alerts` to it. So the full path is `{baseURL}/alert/api/v1/alerts`.

6. **client_test.go is hand-written**: Despite living in the generated package, `client_test.go` contains hand-written tests (no "DO NOT EDIT" header, uses `httptest`, has 15 test functions). It tests:
   - Client construction (3 tests)
   - Configuration (2 tests)
   - GetAlerts success, empty, missing request (3 tests)
   - GetFullAlertByRefId including special characters (2 tests)
   - Error responses: 422, 500, 404 (3 tests)
   - Network error and context cancellation (2 tests)
   - Default headers and user agent (2 tests)
   - Metadata and indicator endpoints (2 tests)
   - Attachment download (1 test)
   - Analytics/risks endpoint (1 test)
   - GenericOpenAPIError (1 test)

   This was completely missed by all prior passes.

### BS-002: docs/specs/ -- Four OpenAPI Spec Files

**Files**: `alert_api_specs.json` (166KB), `asm_assets_api_specs.json` (24KB), `cve_api_specs.json` (79KB), `ioc_api_specs.json` (51KB)

**Prior coverage**: Zero references in any analysis file.

**Findings**:

1. **Only alert and asset specs are used**: The codebase only consumes the Alert API (via generated client from `alert_api_specs.json`) and Asset Configuration API (via hand-written client loosely matching `asm_assets_api_specs.json`).

2. **CVE and IOC specs are unused**: `cve_api_specs.json` and `ioc_api_specs.json` describe additional Cyberint APIs that poller-express does NOT poll. CVE models (`model_cve.go`, `model_extended_cve.go`, `model_cve_search_parameters.go`) exist in the generated client but are only referenced as nested fields within AlertData subtypes (e.g., `AVSAlertData` contains CVE data). The IOC spec models (`model_ioc.go`, `model_ioc_type.go`) similarly exist but only as nested fields in Alert.

   **Implication for port**: No need to implement CVE or IOC API endpoints. These models are only relevant as embedded data within alerts, which are passed through as opaque JSON.

### BS-003: scripts/setup.sh -- Development Credentials

**Prior coverage**: Never mentioned.

**Findings**: Contains hardcoded development credentials (`VECTOR_USERNAME=argos`, `VECTOR_PASSWORD=argos`, `VECTOR_ENDPOINT=http://127.0.0.1:4416`). Includes prominent WARNING comments about not using in production. Sourced by `make run` and `make vector` targets.

**Implication for port**: Development workflow detail only. No behavioral contracts.

### BS-004: docs/PROFILING.md -- Profiling Guide

**Prior coverage**: Never referenced despite `scripts/pprof-harness.sh` being partially mentioned.

**Findings**: Comprehensive profiling guide documenting:
- All pprof endpoints and their purposes
- The pprof-harness.sh script's environment variables and collection strategy
- Analysis workflows (web UI, CLI, profile comparison)
- Known performance hotspots to investigate: JSON marshaling in `enrichPayload`, sort overhead, TLS handshake costs, rate limiter map growth, response body buffering
- Implementation detail: blank import of `net/http/pprof` registers on `DefaultServeMux`

**Implication for port**: The profiling guide identifies `enrichPayload` JSON marshaling as a known performance hotspot. This validates the choice to use manual JSON construction rather than struct marshaling. The rate limiter map growth warning aligns with the anti-pattern documented in Pass 5.

### BS-005: Makefile Build Targets

**Prior coverage**: Listed in Pass 0 inventory as "13 targets" (corrected from original "11 targets") but content never analyzed.

**Findings from source review**: 13 targets (help, all, fmt, build, deps, get, clean, test, vector, run, lint, vuln, generate).

Key behavioral details:
- `make build`: Uses `-ldflags="-s -w"` for stripped binary, supports cross-compilation via `GOOS`/`GOARCH`
- `make lint`: Uses `go run -modfile=tools/go.mod` pattern (not globally installed binary)
- `make run`: Sources `scripts/setup.sh` then runs the collector
- `make vector`: Sources `scripts/setup.sh` then runs Vector with local config
- `make generate`: `go generate ./...` -- suggests generated code intent, though no `//go:generate` directives found in hand-written code

### BS-006: vector.yaml -- Local Vector Configuration

**Prior coverage**: Mentioned once in Pass 0 as "Local Vector config: HTTP source on 4416, console sink" but never read.

**Implication**: Development infrastructure file. Confirms Vector listens on port 4416 with HTTP source and outputs to console. No behavioral contracts needed.

### BS-007: `collector` Binary at Root

**Finding**: A compiled Mach-O arm64 binary exists at the repository root. This is a build artifact that should be in `.gitignore` but was committed. Not relevant to analysis.

---

## Audit Contracts

### BC-AUDIT-001: Generated client uses only 1 of 10 available API methods

**Preconditions:** OpenAPI-generated client initialized with Cyberint configuration
**Postconditions:** Only `GetAlertsApiV1AlertsPost` is called by the alert collector. All other 9 API methods (risk reports, single alert retrieval, metadata, attachments, indicators, status updates) are unused dead code.
**Evidence:** `alert_collector.go:213` -- single call site. No other `PublicAPI.*` calls anywhere in `internal/`.
**Confidence:** HIGH
**Implication:** The Rust port needs only implement the alerts list endpoint, not the full generated client API surface.

### BC-AUDIT-002: NullableTime.UnmarshalJSON has hand-written CyberintTime delegation

**Preconditions:** JSON containing nullable time fields in Cyberint responses
**Postconditions:** Null values handled by generated null check; non-null values parsed via `CyberintTime.UnmarshalJSON` which handles 4 format variants
**Evidence:** `pkg/cyberint/utils.go:327-345` -- hand-modified section within generated file
**Confidence:** HIGH
**Implication:** The Rust port must apply the same multi-format time parsing to all nullable time fields (ClosureDate, AcknowledgedDate, PublishDate), not just non-nullable CyberintTime fields.

### BC-AUDIT-003: Generated client uses strict JSON decoding (DisallowUnknownFields)

**Preconditions:** Cyberint API response received
**Postconditions:** Any unexpected JSON field in the response causes a parse error
**Evidence:** `pkg/cyberint/utils.go:366-370` -- `newStrictDecoder` with `DisallowUnknownFields()`
**Confidence:** HIGH
**Implication:** Forward-compatibility risk. If Cyberint adds new fields to their API, the collector would fail to parse responses. The Rust port should consider lenient deserialization (`#[serde(deny_unknown_fields)]` should NOT be used).

### BC-AUDIT-004: Server URL path construction includes /alert prefix

**Preconditions:** Cyberint base URL configured
**Postconditions:** Alert API calls go to `{baseURL}/alert/api/v1/alerts`. The `/alert` prefix comes from the generated Configuration.Servers default, overwritten by runner.go to `baseURL + "/alert"`.
**Evidence:** `configuration.go` Servers default `{URL: "/alert"}`, `runner.go:68-71` sets `cfg.Servers[0].URL`, `api_public.go:1077` appends `/api/v1/alerts`
**Confidence:** HIGH
**Implication:** The Rust port must construct the alert URL as `{baseURL}/alert/api/v1/alerts`, not `{baseURL}/api/v1/alerts`.

### BC-AUDIT-005: client_test.go provides 15 hand-written integration tests for the generated client

**Preconditions:** Generated OpenAPI client with custom modifications
**Postconditions:** Tests cover: client construction (3), configuration (2), alert list (3), single alert (2), error handling (3), network errors (2), headers (2), metadata/indicator endpoints (2), attachment download (1), analytics (1), error types (1)
**Evidence:** `pkg/cyberint/client_test.go` -- no "DO NOT EDIT" header, uses `httptest`
**Confidence:** HIGH
**Implication:** These tests validate the generated client works correctly with the hand-written CyberintTime modifications. They are integration-level tests that were completely missed by Pass 3 (behavioral contracts).

### BC-AUDIT-006: Four API specs exist but only two are consumed

**Preconditions:** API spec files in `docs/specs/`
**Postconditions:**
- `alert_api_specs.json` (166KB): Used -- source for OpenAPI-generated client
- `asm_assets_api_specs.json` (24KB): Partially used -- hand-written asset client loosely follows this spec
- `cve_api_specs.json` (79KB): Unused -- CVE API not polled
- `ioc_api_specs.json` (51KB): Unused -- IOC API not polled
**Evidence:** No CVE or IOC API calls in any Go file outside generated models
**Confidence:** HIGH
**Implication:** CVE and IOC API specs are available for future expansion but not needed for the initial port.

---

## Updated Gap Inventory

Previously unidentified gaps from this audit:

| Gap | Severity | Description |
|-----|----------|-------------|
| Strict JSON decoding | MEDIUM | Generated client rejects unknown fields, creating forward-compatibility risk |
| 9 unused API methods | LOW | Dead code in generated client; port should only implement 1 |
| NullableTime hand-modification | HIGH | Hand-written CyberintTime delegation in generated file; easy to miss during port |
| URL path construction | HIGH | `/alert` prefix embedded in Configuration.Servers; must be replicated in port |
| client_test.go uncovered | LOW | 15 integration tests were missed by behavioral contracts pass |
| CVE/IOC specs unused | LOW | Reference material only; no implementation needed |
| Compiled binary in repo | LOW | Build artifact committed to VCS; hygiene issue only |

---

## Verdict

**PASS** -- All substantive source code directories have been analyzed. The blind spots identified are:

1. **Generated client infrastructure** (BS-001): Substantive finding. Revealed the strict JSON decoding risk (BC-AUDIT-003), the NullableTime hand-modification (BC-AUDIT-002), the URL path construction detail (BC-AUDIT-004), and the 1-of-10 API method usage (BC-AUDIT-001). These produce 6 audit contracts.

2. **API spec files** (BS-002): Reference material. Confirms only alert and asset APIs are used. No new behavioral findings.

3. **Scripts and docs** (BS-003, BS-004, BS-005, BS-006): Development infrastructure. No behavioral contracts needed.

4. **Build artifact** (BS-007): Hygiene issue only.

No additional substantive gaps remain. The 6 audit contracts (BC-AUDIT-001 through BC-AUDIT-006) have been extracted and should be incorporated into the final synthesis.
