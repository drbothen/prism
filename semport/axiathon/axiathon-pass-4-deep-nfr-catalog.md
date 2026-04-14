# Pass 4 Deep: NFR Catalog -- Round 1

**Project:** Axiathon
**Pass:** 4 (NFR Catalog)
**Round:** 1
**Date:** 2026-04-13

---

## Purpose

Deepen the broad sweep's NFR catalog by incorporating Tier 1 discoveries (plugin SDK, dual parser, spike infrastructure) and Tier 2 findings (dual workspace, CI pipeline, tooling). Categorize NFRs by enforcement status (enforced vs aspirational vs missing).

---

## 1. Performance NFRs

### 1.1 Documented Targets (from README -- aspirational)

| NFR | Target | Evidence | Status |
|-----|--------|----------|--------|
| Ingestion throughput | 100K+ EPS single-node | README | ASPIRATIONAL -- no benchmarks validate this |
| Query p50 latency | <100ms on 1TB | README | ASPIRATIONAL -- QueryResult tracks query_time_ms but no validation |
| Query p99 latency | <500ms on 1TB | README | ASPIRATIONAL |
| Detection latency | <1s per event | README | ASPIRATIONAL |
| Event-to-alert | <5 seconds | README | ASPIRATIONAL |

### 1.2 Implemented Limits (enforced in code)

| NFR | Value | Source | CWE | Enforcement |
|-----|-------|--------|-----|-------------|
| Max query length | 64KB | parser.rs:36 | CWE-400, OWASP A05:2021 | Parse-time rejection |
| Max nesting depth | 128 | parser.rs:39 | CWE-674 | Parse-time rejection |
| Max pipe stages | 64 | parser.rs:42 | CWE-400 | Post-parse validation |
| Max regex pattern | 1024 bytes | parser.rs:569 | CWE-1333 | Parse-time rejection |
| Default query timeout | 30s | QueryConfig | -- | Configuration default |
| Max query timeout | 300s | QueryConfig | -- | Configuration default |
| Max result rows | 10,000 | QueryConfig | -- | Configuration default |
| Max concurrent queries | 50 | QueryConfig | -- | Configuration default |
| Max memory per query | 512MB | QueryConfig | -- | Configuration default |
| Pipeline buffer capacity | 10,000 RawEvents | state.rs | -- | mpsc channel bound |
| Alert broadcast capacity | 1,024 messages | state.rs | -- | broadcast channel bound |

### 1.3 Storage Performance Configuration

| NFR | Value | Source | Notes |
|-----|-------|--------|-------|
| Storage buffer size | 1,000 events | WriterConfig default | Flush on buffer full |
| Storage flush interval | 5 seconds | WriterConfig default | Time-based flush |
| Parquet compression | Zstd | writer.rs | Via WriterProperties |
| Compaction threshold | 5 files/partition | CompactionConfig default | Merge when exceeded |
| Compaction check interval | 30 seconds | CompactionConfig default | Background task |
| GC check interval | 120 seconds | GcConfig default | Background task |
| GC max snapshot age | 300 seconds | GcConfig default | Before expiry candidate |
| GC min snapshots kept | 1 | GcConfig default | Floor |
| Parquet columns per class | <200 | schema.rs comments | Iceberg comfort <300 |

### 1.4 Benchmark Infrastructure (NEW -- not in broad sweep)

The spike includes 2 criterion benchmark suites:
- `detection_stateless.rs` -- rule parsing + single-event evaluation benchmarks
- `detection_stateful.rs` -- correlation + sequence state management benchmarks

These use criterion with async_tokio feature for benchmarking the detection engine. No query or storage benchmarks exist.

---

## 2. Security NFRs

### 2.1 Enforced (production crates)

| NFR | Implementation | Evidence | CWE/OWASP |
|-----|---------------|----------|-----------|
| No unsafe code | `#![forbid(unsafe_code)]` in 8 crates + `workspace.lints.rust.unsafe_code = "forbid"` | lib.rs of every crate, Cargo.toml | -- |
| Input validation at boundaries | `new()` validates, `new_unchecked()` skips | types.rs, query_types.rs | CWE-20 |
| Query DoS prevention | Length (64KB), depth (128), stage (64) limits | parser.rs | CWE-400 |
| Regex DoS prevention | Pattern length 1024B + Rust regex crate (finite automaton) | parser.rs:569-585 | CWE-1333 |
| Integer overflow prevention | i128 intermediate for i64::MIN parsing | parser.rs:283 | CWE-190 |
| CIDR validation at parse time | IP + prefix validated before execution | parser.rs:511 | CWE-20 |
| Error sanitization | SECURITY comments on Display impl | error.rs | CWE-209 |
| License compliance | cargo-deny with allowlist (Apache-2.0, MIT, BSD, ISC, etc.) | deny.toml | -- |
| Advisory monitoring | cargo-deny checks advisories, 1 exemption (RUSTSEC-2024-0436) | deny.toml | -- |
| Dependency graph enforcement | cargo-depgraph-check with strict mode | depgraph-rules.toml, CI | -- |
| Commit security | No AI attribution in commits | .claude/rules/git-commits.md | -- |
| No shell injection | Bash rules forbid eval, require "$@" expansion | .claude/rules/bash.md | CWE-78 |
| CI supply chain | Pinned action SHAs + step-security/harden-runner egress audit | ci.yml | -- |
| Minimal permissions | `permissions: contents: read` in CI | ci.yml | -- |

### 2.2 Identified but NOT Enforced (spike only)

| NFR | Issue | CWE/OWASP | Spike Evidence |
|-----|-------|-----------|----------------|
| Vault key management | Hardcoded passphrase + static salt | CWE-798, CWE-760, OWASP A02:2021 | state.rs:429, vault.rs:121 |
| CORS policy | Completely permissive (any origin) | CWE-942 | main.rs:66 |
| Admin authorization | Admin endpoints unprotected | OWASP A01:2021 | main.rs:172 |
| Detection regex limits | No size limit on user regex in detection DSL | CWE-1333 | engine.rs:56 |
| Error information leakage | 5 identified call sites where errors leak to API | CWE-209 | spike error.rs comments |
| No forbid(unsafe_code) | Spike crates lack the attribute | -- | All spike lib.rs files |

### 2.3 Missing (expected but not found)

| NFR | Expected | Status |
|-----|----------|--------|
| Rate limiting | Expected for API endpoints | Not implemented (referenced in docs/.archive/rate-limiting-architecture.md) |
| Authentication | Expected for multi-tenant API | Not implemented (tenant extracted from header, no auth) |
| RBAC | Expected per SOUL.md (roles, permissions fields exist on TenantContext) | Fields present but not checked |
| Audit logging | Expected for compliance | Not implemented (docs/.archive/compliance-audit-architecture.md exists) |
| TLS/HTTPS | Expected for production | Not configured (bare HTTP in spike) |
| Input size limits on API | Expected beyond query limits | No request body size limits on ingest endpoint |
| Secrets scanning | Expected for CI | No .gitleaks.toml or similar |

---

## 3. Observability NFRs

### 3.1 Implemented

| NFR | Implementation | Coverage |
|-----|---------------|----------|
| Structured logging | tracing crate | 17 files, 63 call sites |
| JSON log output | tracing-subscriber with json feature | main.rs init |
| Environment-based filtering | tracing-subscriber env-filter | main.rs init |
| HTTP request tracing | tower-http TraceLayer | main.rs middleware |
| Query timing | query_time_ms in QueryResult | planner.rs |

### 3.2 Missing (SOUL.md #9 targets not met)

| NFR | Target | Status |
|-----|--------|--------|
| `#[instrument]` on functions | Every significant function | 0 instances found in entire codebase |
| trace_id in spans | Every span includes trace_id | Not propagated -- TenantContext.trace_id exists but unused in tracing |
| tenant_id in spans | Every span includes tenant_id | Not propagated |
| Metrics collection | Prometheus/OpenTelemetry | Not implemented |
| Health endpoint metrics | Detailed subsystem health | Basic health check exists (/health) |
| Alert latency metrics | p50/p99 detection-to-alert | Not measured |
| Ingestion throughput metrics | Events/second counters | Not measured |

---

## 4. Reliability NFRs

### 4.1 Implemented

| NFR | Implementation | Source |
|-----|---------------|--------|
| Regex pre-compilation | Cached compiled patterns in RuleEngine | engine.rs |
| Concurrent correlation state | DashMap for lock-free per-key access | correlation.rs |
| Buffered storage writes | Buffer + time-based flush prevents per-event I/O | writer.rs |
| Background compaction | Reduces file count per partition | compaction.rs |
| Background GC | Cleans unreferenced files after compaction | gc.rs |
| Query engine refresh | Provider reload on flush/compaction via Notify | state.rs |
| Graceful connector start | Errors logged, other connectors still start | pipeline.rs |
| Alert broadcast drop tolerance | Dropped broadcasts logged at debug level | pipeline.rs |

### 4.2 Planned but NOT Implemented

| NFR | Plan | Source |
|-----|------|--------|
| Hot-reload for config | arc-swap dependency present, not used in production code | Cargo.toml |
| Hot-reload for detection rules | arc-swap + VersionedPlugin trait in spike | hot_reload.rs |
| Circuit breaker for connectors | Referenced in architecture docs | docs/.archive/ |
| Graceful shutdown | Compaction/GC have shutdown senders (mpsc) but no graceful drain | state.rs |
| Retry logic for connector failures | Not implemented | pipeline.rs -- errors logged and skipped |
| Event replay | Not implemented | Referenced in architecture docs |

### 4.3 Concurrency Model (spike)

The spike uses a complex concurrency stack:
- `tokio::sync::RwLock` for per-tenant engine maps
- `tokio::sync::Mutex` for storage buffer access (within writer)
- `tokio::sync::mpsc` for pipeline + shutdown channels
- `tokio::sync::broadcast` for alert distribution
- `tokio::sync::Notify` for catalog change signaling
- `std::sync::RwLock` for storage writer internal state
- `DashMap` for correlation/sequence sliding windows
- `Arc<dyn Catalog>` for shared Iceberg catalog

**Risk:** Mixing `std::sync::RwLock` (blocking) with `tokio::sync::RwLock` (async) in the storage writer is a known anti-pattern that can block the async runtime if the std RwLock is held across an await point.

---

## 5. Scalability NFRs

### 5.1 Implemented Patterns

| NFR | Implementation | Notes |
|-----|---------------|-------|
| Table-per-class routing | Each OCSF class in separate Iceberg table | Prevents wide-table scanning |
| Partition by tenant + hour | `identity(tenant_id) + hour(event_time)` | Partition pruning for both tenant and time range |
| Two-tier columnar storage | Hot Parquet columns + event_data JSON | Avoids 83-class schema explosion |
| Field promotion | Schema evolution to add hot columns over time | events_to_record_batch_with_promotions |
| Per-tenant engine isolation | Separate RuleEngine per tenant | Prevents noisy neighbor in detection |
| Per-tenant connector instances | Connectors parameterized by tenant config | Port-per-tenant syslog, per-tenant Claroty |

### 5.2 Missing

| NFR | Expected | Status |
|-----|----------|--------|
| Horizontal scaling | Multiple nodes sharing Iceberg catalog | Referenced in docs/.archive/horizontal-scaling-architecture.md, not implemented |
| Connection pooling | Database/catalog connection management | SQLite-backed, no pooling needed yet |
| Worker pools | Parallel event processing | Single pipeline task |
| Queue-based ingestion | Kafka/NATS for durable event buffering | Direct mpsc only |
| Sharding | Multi-node partition assignment | Not implemented |

---

## 6. Testability NFRs

### 6.1 Production Testing Infrastructure

| NFR | Implementation | Evidence |
|-----|---------------|----------|
| Property-based testing | proptest crate | 4 property test files (property_fieldref, property_types, property_comments, property_parser_safety) |
| Snapshot testing | insta crate | snapshot_types.rs |
| Integration tests | Separate test files | 13+ test files in crates/axiathon-query/tests/ |
| Benchmark suite | criterion (spike only) | 2 benchmark files |
| CI test coverage | cargo test --all-targets + --doc --all-features | ci.yml |
| Parallel testing | cargo-nextest support | justfile recipe |
| Test naming convention | Descriptive: `tenant_id_new_rejects_empty()` | SOUL.md #8 |

### 6.2 Spike Integration Tests (NEW)

3 integration test files in spike/axiathon-storage/tests/:
- `tenant_isolation.rs` -- End-to-end write + query with TenantFilterRule proving cross-tenant isolation
- `field_promotion.rs` -- Full promotion pipeline: schema evolution -> dual-write -> COALESCE query -> compaction backfill
- `schema_evolution.rs` -- Schema evolution testing

These are the highest-value tests in the codebase -- they validate the complete storage + query pipeline.

---

## Delta Summary
- New items added: Benchmark infrastructure (2 criterion suites), 7 missing security NFRs identified (rate limiting, auth, RBAC, audit, TLS, input limits, secrets scanning), observability gap quantified (0 #[instrument], 0 trace_id propagation), 6 reliability gaps (hot-reload, circuit breaker, graceful shutdown, retry, replay), concurrency anti-pattern risk (std::sync::RwLock + tokio), scalability gaps (horizontal scaling, worker pools, queue-based ingestion), testability NFRs (property testing, snapshot testing, parallel testing), 3 spike integration tests documented, NFR enforcement status classification (enforced/spike-only/missing)
- Existing items refined: All NFRs classified by enforcement status (enforced in production vs identified in spike vs missing), security NFRs expanded with full CWE/OWASP citations from source, performance targets distinguished from implemented limits
- Remaining gaps: docs/.archive/ architecture docs likely contain detailed NFR specifications not reflected in code (rate-limiting, horizontal-scaling, compliance-audit docs exist)

## Novelty Assessment
Novelty: SUBSTANTIVE
The classification of NFRs into enforced/spike-only/missing is a model-changing insight. The broad sweep listed NFR values without distinguishing between "target stated in README" (aspirational) and "validated in parser.rs with CWE citation" (enforced). The observability gap (zero #[instrument] despite SOUL.md mandate), the concurrency anti-pattern risk, and the 7 missing security NFRs are all findings that would change how you'd spec Prism's NFR requirements.

## Convergence Declaration
Another round needed -- docs/.archive/ contains architecture documents for rate limiting, horizontal scaling, compliance auditing, and other NFR-relevant topics that may provide specification-level detail.

## State Checkpoint
```yaml
pass: 4
round: 1
status: complete
files_scanned: 30+
timestamp: 2026-04-13T00:00:00Z
novelty: SUBSTANTIVE
next_pass: 4-r2
```
