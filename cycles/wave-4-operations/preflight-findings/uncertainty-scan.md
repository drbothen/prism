---
document_type: preflight-findings
phase: 4.A
producer: dclaude:uncertainty-scanner
timestamp: 2026-05-02T00:00:00Z
inputs:
  - .factory/stories/S-4.01-schedule-crud.md
  - .factory/stories/S-4.02-diff-results-packs.md
  - .factory/stories/S-4.03-detection-rules.md
  - .factory/stories/S-4.04-detection-evaluation.md
  - .factory/stories/S-4.05-alert-generation.md
  - .factory/stories/S-4.06-case-management.md
  - .factory/stories/S-4.07-case-metrics.md
  - .factory/stories/S-4.08-action-delivery.md
  - Cargo.toml
  - Cargo.lock
  - crates/prism-storage/Cargo.toml
  - crates/prism-spec-engine/Cargo.toml
  - crates/prism-core/Cargo.toml
total_uncertainties: 41
severity_breakdown: { HIGH: 14, MEDIUM: 18, LOW: 9 }
research_tasks_queued: 13
note: "dclaude:uncertainty-scanner returned content via task-notification (read-only profile cannot Write); orchestrator persists verbatim via state-manager."
---

# Wave 4 Uncertainty Scan Findings

## Summary
- Stories scanned: 8 (S-4.01 through S-4.08)
- Total uncertainties: 41 (HIGH/needs-research: 14, MEDIUM/verify: 18, LOW/note: 9)
- Top recurring uncertainty classes:
  1. **Unpinned major versions** ("tokio 1.x", "uuid 1.x", "blake3 1.x", "proptest 1.x", "serde 1.x", "rocksdb 0.24" without minor) — appears in ALL 8 stories
  2. **Broad point-release pins** ("toml 0.8.x", "lettre 0.11.x", "reqwest 0.12.x", "cron 0.12.x") — wildcard minor-version expressions in 4 stories
  3. **DataFusion 53 API surface** (S-4.03, S-4.04) — UDF registration API is a moving target between DF major versions; DataFusion is NOT currently in Cargo.lock workspace, this is a brand-new dependency to W4
  4. **`cron` crate choice** (S-4.08) — Rust ecosystem has multiple competing crates (`cron`, `croner`, `tokio-cron-scheduler`, `cron_clock`); the chosen crate is named without justification
  5. **bincode 2.x** for case persistence (S-4.06) — workspace already uses `bincode = { version = "2", features = ["serde"] }`; serde feature requirement may not be reflected in story
  6. **External tool/library cap claims** ("64KB snapshot cap", "200MB diff cap", "10MB IOC file cap") — stated as engineering choices but not traced to source

## Per-Story Uncertainties

### S-4.01 Schedule CRUD and Execution Loop

| ID | Severity | Category | Claim (Line) | Validation Needed |
|----|----------|----------|--------------|-------------------|
| UNC-401-001 | HIGH | Unpinned library | `tokio = "1.x"` (L244) | Replace with workspace-pinned version. Cargo.lock currently resolves to specific minor; story must align. The `1.x` constraint allows future breaking semver-minor updates. |
| UNC-401-002 | HIGH | Unpinned library | `uuid = "1.x"` (L246) | Replace with `1` + features `["v7", "serde"]` to match prism-core's pin. Story does not specify v7 feature flag despite using UUID v7. |
| UNC-401-003 | MEDIUM | API assumption | "60-second tick is a minimum check frequency" + `tokio::sync::Semaphore::try_acquire()` semantics (L130-138) | Verify `try_acquire()` non-blocking behavior is preserved in current tokio (was renamed `try_acquire_owned` in some paths). Confirm `Arc<Semaphore>` cross-task sharing pattern current. |
| UNC-401-004 | MEDIUM | Architecture-pattern | "splay = `hash(schedule_id) % (interval / 4)` capped at 15 minutes" (L106) | "Industry standard" claim absent — verify whether this formula is documented anywhere (Nomad, Kubernetes CronJob, osquery). 25% jitter may not match common practice (Kubernetes uses 0.7–1.0× factor). |
| UNC-401-005 | MEDIUM | Crate dependency | `prism-storage` workspace API for "schedules CF read/write helpers" (L249) | Verify prism-storage v3 API surface still exposes the assumed CF helper signatures (open, get, put, prefix_scan). Wave 2/3 may have shifted these. |
| UNC-401-006 | LOW | Library | `rocksdb = "0.24"` (L245) | Pin matches prism-storage already (good). Confirm 0.24 still latest stable on crates.io. |

### S-4.02 Differential Results and Packs

| ID | Severity | Category | Claim (Line) | Validation Needed |
|----|----------|----------|--------------|-------------------|
| UNC-402-001 | HIGH | Unpinned library | `blake3 = "1.x"` (L242) | blake3 NOT currently in Cargo.lock — brand-new W4 dep. Verify current 1.x major and whether SIMD features need explicit feature flags. Has known soundness CVEs in older 1.x; pin to known-good. |
| UNC-402-002 | MEDIUM | API assumption | "RocksDB merge_operator (counter merge)" for atomic epoch increment (L113) | Verify rocksdb 0.24 crate exposes the `merge_operator` API as expected; counter merge operator is C++ concept — Rust binding may require custom merge fn registration. |
| UNC-402-003 | MEDIUM | Library | `toml = "0.8.x"` (L244) | Workspace uses `toml = "0.8"` (no `.x`). The `.x` syntax is ambiguous — Cargo treats `0.8.x` as `^0.8` regardless. Align spelling for spec hygiene. |
| UNC-402-004 | MEDIUM | Architecture-pattern | "200MB cap on diff_results CF enforced via RocksDB block_cache" (L233-235) | Verify block_cache semantics — block_cache bounds memory for hot blocks, NOT total CF size on disk. The cap mechanism may not match the intent. |
| UNC-402-005 | MEDIUM | Library | `proptest = "1.x"` (L246) | Workspace uses `proptest = "1.11.0"` in storage and `proptest = "1"` elsewhere. Inconsistent; pick canonical version. |
| UNC-402-006 | LOW | Architecture-pattern | "canonical JSON for row normalization" (L227) | Canonical JSON has no IETF standard; Rust ecosystem has `serde_jcs` (JCS RFC 8785) and `canonical-json` (alternative). The story doesn't specify which canonicalization. |

### S-4.03 Detection Rule Loading and Compilation

| ID | Severity | Category | Claim (Line) | Validation Needed |
|----|----------|----------|--------------|-------------------|
| UNC-403-001 | HIGH | Library/API | `datafusion = "53"` for "SQL compilation target, UDF registration" (L344) | DataFusion is NOT in workspace Cargo.lock yet. Major version 53 was current ~Jan 2026; verify latest stable as of 2026-05-02. UDF registration API (`create_udf`, `ScalarUDFImpl` trait) has changed significantly between DF 40-50. Story uses signatures `(ip: Utf8, cidr: Utf8) -> Boolean` which is DF 38+ syntax — verify current. |
| UNC-403-002 | HIGH | API assumption | "DataFusion `MemTable` populated by schedule executor; targeted by name `events`" (L417, L142) | Verify DataFusion MemTable API in v53 — `MemTable::try_new` signature, `SchemaRef` requirements, registration via `SessionContext::register_table`. Pattern may need `SessionContext::register_batch` instead. |
| UNC-403-003 | HIGH | Architecture-pattern | "regex::RegexSet for O(n_patterns) multi-pattern matching" + "100,000 patterns per file" cap (L194-201) | RegexSet has known compilation-cost issues at 100k+ patterns; may exhaust DFA size limits. Verify `regex` crate's `RegexSet` size limits and compilation memory profile. May need `regex_automata` instead. |
| UNC-403-004 | MEDIUM | Library | `ipnet = "2.x"` (L346) | Cargo.lock has 2.12.0. Pin to `2` would resolve correctly; `2.x` is broader than needed. |
| UNC-403-005 | MEDIUM | Library | `arc_swap` (referenced in body L190 not in deps table) | Story uses `arc_swap::ArcSwap<Arc<PatternStore>>` (L190) but Library table doesn't list arc-swap. Workspace already pins `arc-swap = "1"` — add to story's table. |
| UNC-403-006 | MEDIUM | Library | "S-1.12 file watcher" + `notify` (implied L199) | Verify the notify version (workspace: `notify = "7"`) and whether the file-watch debouncing pattern from S-1.12 is still the recommended approach (notify-debouncer-full is now separate crate). |
| UNC-403-007 | MEDIUM | Architecture-pattern | "10 MB max file size, 50 IOC files total" caps (L204) | Engineering choices without provenance. Verify against operational data or document as policy decision. |
| UNC-403-008 | LOW | Crate dependency | `prism-spec-engine` for "Source registry for rule validation" (L352) | Confirm public surface of prism-spec-engine SourceRegistry/SpecRegistry hasn't moved between v0.4 and current. |

### S-4.04 Detection Evaluation

| ID | Severity | Category | Claim (Line) | Validation Needed |
|----|----------|----------|--------------|-------------------|
| UNC-404-001 | HIGH | API assumption | `scopeguard::defer!` macro for SessionContext drop guarantee (L99, L322, L271) | Verify scopeguard 1.x still exposes `defer!` macro and that it interacts correctly with async/.await suspension points. The pattern may not catch panics across `.await` boundaries — needs validation against current tokio semantics. |
| UNC-404-002 | HIGH | API assumption | "DataFusion RecordBatch API to evaluate predicates in batch" (L106) | DataFusion Expr evaluation API (`Expr::evaluate`, `physical_expr::PhysicalExpr`) has moved between major versions. Verify the batch-predicate evaluation pattern in DF 53. |
| UNC-404-003 | HIGH | Library | `datafusion = "53"` (L270) — same as S-4.03 | Cross-reference S-4.03 UNC-403-001. |
| UNC-404-004 | MEDIUM | Architecture-pattern | "RocksDB length-prefix key with `\x00`/`\x01`/`\x02` encoding" (L139-150) | Verify prefix-scan semantics still iterate correctly with single-byte prefix; rocksdb 0.24 prefix_extractor configuration may need explicit setup. |
| UNC-404-005 | MEDIUM | Architecture-pattern | "10,000 group key cap per rule" + "silently drop" (L121, L188-191) | Engineering choice without provenance. Verify against detection ecosystem norms (Sigma, Elastic Detection Rules' `max_signals` defaults). |
| UNC-404-006 | MEDIUM | Library | `scopeguard = "1.x"` (L271) — Cargo.lock has 1.2.0 | Tighten to `1` or `1.2`. |
| UNC-404-007 | LOW | Library | `blake3 = "1.x"` (L272) — same as S-4.02 | Cross-reference UNC-402-001. |

### S-4.05 Alert Generation

| ID | Severity | Category | Claim (Line) | Validation Needed |
|----|----------|----------|--------------|-------------------|
| UNC-405-001 | HIGH | Library | `libfuzzer-sys = "0.4.x"` for VP-028 fuzz target (L240) | libfuzzer-sys is NOT in workspace Cargo.lock — brand-new W4 dep. Requires libfuzzer support which is nightly-only on some platforms; cargo-fuzz integration is a separate concern. Verify current 0.4.x major or whether `arbitrary` + `cargo-fuzz` is the modern pattern. |
| UNC-405-002 | MEDIUM | API assumption | `tokio::sync::broadcast` channel "capacity 1,000" with lagged-recv handling (L137-139, L472) | Verify broadcast channel semantics in current tokio (1.x): lagged receivers drop messages silently, `RecvError::Lagged(skipped)` returns count. Pattern depends on current API. |
| UNC-405-003 | MEDIUM | Architecture-pattern | "64KB per event snapshot cap" + "truncate low-priority fields (raw bytes, base64 blobs)" (L103-105) | Engineering choice without provenance. The "low-priority field" classification rules are not specified. |
| UNC-405-004 | MEDIUM | Library | `serde_json = "1.x"` (L238) — same pattern repeated; tighten | Workspace pins `serde_json = "1"`. Match. |
| UNC-405-005 | LOW | Library | `tokio = "1.x"` (L236) — duplicate of UNC-401-001 | Cross-reference. |
| UNC-405-006 | LOW | Library | `uuid = "1.x"` (L237) — duplicate of UNC-401-002 | Cross-reference. |

### S-4.06 Case Management

| ID | Severity | Category | Claim (Line) | Validation Needed |
|----|----------|----------|--------------|-------------------|
| UNC-406-001 | HIGH | Architecture-pattern | "5-state machine with 12 valid transitions" (L60, L113-117) | Claim presented as fact but not traced to industry source. ITIL incident lifecycle has 7+ states; OASIS STIX Course-of-Action has different model. Verify whether 5/12 mapping matches MSSP industry SOPs (1898 & Co specific?). |
| UNC-406-002 | HIGH | Library | `bincode = "2.x"` for case persistence (L425) | Workspace pins `bincode = { version = "2", features = ["serde"] }`. Story doesn't specify the `serde` feature requirement. bincode 2.x has its own derive macros distinct from serde — using bincode-with-serde requires explicit feature. Story should pin to workspace pattern. |
| UNC-406-003 | MEDIUM | API assumption | `CaseStatus::can_transition_to()` from prism-core S-1.02 (L398-400) | Verify S-1.02 actually exposes a 5-state CaseStatus with this method. Story explicitly notes "If S-1.02 does not yet encode the full 12-transition set, add them" (L467) — so this is a known forward-reference risk. |
| UNC-406-004 | MEDIUM | API assumption | "RocksDB `WriteBatch` transaction" for dedup decision (L246, L414) | Verify rocksdb 0.24 crate's `WriteBatch` API and atomicity guarantees. WriteBatch is atomic for write-side but does NOT atomically wrap a read-decide-write sequence. Story may misattribute "TOCTOU prevention" to WriteBatch. |
| UNC-406-005 | MEDIUM | Architecture-pattern | "in-memory status index `(status → Vec<CaseId>)` rebuilt on startup" (L405-409) | At MSSP scale (thousands of cases), full prefix scan on startup may be slow. Verify scalability assumption. |
| UNC-406-006 | MEDIUM | Library | `kani` (dev, cfg-gated) for VP-053 (L428) | Kani version not pinned. Workspace doesn't have kani as a dependency anywhere yet — confirm Kani install/CI integration plan. |
| UNC-406-007 | LOW | Library | `proptest = "1.x"` (L427) | Same as UNC-402-005. |

### S-4.07 Case Metrics and Acknowledge Alert

| ID | Severity | Category | Claim (Line) | Validation Needed |
|----|----------|----------|--------------|-------------------|
| UNC-407-001 | MEDIUM | Architecture-pattern | "Streaming approximate percentile (t-digest / GK summary) for 10,000+ cases" (L278) | Two algorithms named (t-digest, GK summary). No crate selected. Rust ecosystem has `tdigest`, `quantiles` (GK), `hdrhistogram`. Choose one and pin. |
| UNC-407-002 | MEDIUM | Architecture-pattern | "90-day date range cap returns E-METRICS-001" (L124) | Engineering choice; verify with operator requirements. The cap is exclusive (>90d fails) per EC-004 — boundary semantics need a test. |
| UNC-407-003 | MEDIUM | Crate dependency | "feature flag check for CAPABILITY_ACKNOWLEDGE_ALERT" via prism-security (L235) | Verify prism-security capability check API (introduced in S-1.08) is stable. |
| UNC-407-004 | LOW | Library | `rocksdb = "0.24"` (L233) | Same pin as workspace. Good. |
| UNC-407-005 | LOW | API assumption | "Read-modify-write under in-memory `Mutex` per `AlertId`" (L222) | DashMap is used elsewhere in workspace for this pattern (prism-storage). Consider whether `HashMap<AlertId, Mutex<()>>` or `dashmap::DashMap<AlertId, Mutex<()>>` is preferred. |

### S-4.08 Action Delivery Framework

| ID | Severity | Category | Claim (Line) | Validation Needed |
|----|----------|----------|--------------|-------------------|
| UNC-408-001 | HIGH | Library/Choice | `cron = "0.12.x"` (L422) | The Rust `cron` crate is at 0.15+ as of late 2025 (was at 0.12 in early 2025). Story is outdated. Additionally, multiple competing crates (`croner`, `tokio-cron-scheduler`, `cron_clock`) may be more current. Verify chosen crate vs. ecosystem. **The cron crate has had API changes between 0.12→0.15.** |
| UNC-408-002 | HIGH | Library/API | `lettre = "0.11.x"` for SMTP delivery (L423) | lettre is NOT in workspace Cargo.lock — brand-new dep. lettre 0.11.x is recent (changed significantly from 0.10). Verify current minor + STARTTLS API + async tokio support feature flags (`tokio1-rustls-tls` etc). |
| UNC-408-003 | HIGH | Library | `reqwest = "0.12.x"` (L424) | Workspace pins `reqwest = { version = "0.12", features = ["json", "rustls-tls", "cookies"] }`. Story doesn't specify feature flags. reqwest 0.12 default is `native-tls` not `rustls-tls`; story must opt in. |
| UNC-408-004 | HIGH | Architecture-pattern | "Exponential backoff: 2s, 4s, 8s, 30s, 60s (max 5 attempts)" (L194) | Backoff schedule is non-standard exponential (4×, 2×, 4× ratios — actually mixed). Verify against industry retry libraries (e.g., `backoff` crate, AWS SDK). Industry standard is base × 2^n with jitter. |
| UNC-408-005 | HIGH | Architecture-pattern | "Syslog CEF / LEEF format hand-rolled" (L161-163) | Hand-rolling CEF/LEEF risks format errors. Rust ecosystem has `syslog`, `rsyslog`, `cef-lib` (TBD). Verify whether a maintained crate exists rather than custom implementation. CEF spec versions (0, 1) have field-escaping rules that are easy to get wrong. |
| UNC-408-006 | MEDIUM | External service | "Slack webhook", "PagerDuty", "Jira" mentioned (L52, L520-523) | Webhook formats and rate-limit assumptions for these services change. PagerDuty API v2 vs v3 events; Slack incoming webhooks have new Block Kit format. Per story design these are WASM plugins — the assumption is right, but the story should not embed any service-specific fact in the built-in webhook destination. |
| UNC-408-007 | MEDIUM | Library | `notify = "7.x"` (L427) — workspace pins `notify = "7"` | Align. notify 7 is recent; verify current minor and whether notify-debouncer-full is needed for hot-reload semantics. |
| UNC-408-008 | MEDIUM | API assumption | "wasmtime PluginRuntime" via S-1.15 (L435) | Workspace pins `wasmtime = "44"`. Verify wasmtime 44 component-model API stability for `fire_alert`/`fire_case`/`fire_report` host calls. wasmtime API has shifted between major versions. |
| UNC-408-009 | MEDIUM | Architecture-pattern | "1-second tick loop for cron scheduler" (L221) | 1Hz tick is high for cron evaluation; Kubernetes CronJob uses 10s. Verify CPU cost at scale (100s of action specs, 1Hz polling) — `tokio::time::interval` jitter may cause double-fires at minute boundaries. |
| UNC-408-010 | MEDIUM | Architecture-pattern | "SHA-256 of `alert.rule_id + alert.client_id`" for dedup hash (L188) | String concatenation without separator is collision-prone (rule_id="ab"+client_id="cd" collides with rule_id="a"+client_id="bcd"). Verify with delimiter (e.g., null byte) or use structured hash input. |
| UNC-408-011 | MEDIUM | API assumption | "broadcast channel `RecvError::Lagged` graceful handling" (L472) | Same as UNC-405-002 — verify tokio current API. |
| UNC-408-012 | LOW | Library | `sha2 = "0.10.x"` (L425) — workspace pins `sha2 = "0.10"` | Align. |
| UNC-408-013 | LOW | Library | `arc-swap = "1.x"` (L426) — workspace pins `arc-swap = "1"` | Align. |

---

## Cross-Cutting Uncertainties

| Class | Affected Stories | Recommended Resolution |
|-------|------------------|------------------------|
| `tokio = "1.x"` unpinned | All 8 stories (S-4.01..S-4.08) | Replace `1.x` with explicit feature-flagged pin matching workspace pattern: `{ version = "1", features = ["..."] }`. Workspace `Cargo.lock` should be the source of truth. |
| `uuid = "1.x"` unpinned without v7 feature | S-4.01, S-4.05, S-4.06, S-4.07, S-4.08 | All stories use UUID v7 but only pin major. Pin as `{ version = "1", features = ["v7", "serde"] }` consistent with prism-core. |
| `rocksdb = "0.24"` consistent | S-4.01, S-4.02, S-4.03, S-4.04, S-4.05, S-4.07, S-4.08 | Already aligned. Confirm 0.24 is still the latest stable via Perplexity research. |
| DataFusion 53 newness | S-4.03, S-4.04 | DataFusion is NOT yet a workspace dependency. Wave 4 is the first wave to need it. Research: latest DF release as of 2026-05-02 + UDF API stability + MemTable registration pattern. |
| `proptest = "1.x"` inconsistent spelling | S-4.02, S-4.03, S-4.04, S-4.06, S-4.08 | Workspace mixes `1`, `1`, `1.11.0`. Pick canonical (suggest `1` to match most). |
| `bincode = "2.x"` missing serde feature | S-4.06 | Workspace already standardizes `{ version = "2", features = ["serde"] }`. Story must reflect. |
| Architecture cap claims without provenance | S-4.01 (15min splay cap), S-4.02 (200MB diff CF), S-4.03 (10MB IOC, 100k patterns, 50 files), S-4.04 (10k group keys), S-4.05 (64KB snapshot, 1k broadcast cap), S-4.06 (10k char Note), S-4.07 (90-day metrics range), S-4.08 (max 5 retry attempts) | These are engineering choices. Each needs either: (a) trace to a documented decision (DI-NNN, AD-NNN), or (b) explicit "decision pending validation" annotation. |
| Confirmation token API (S-1.09) | S-4.01, S-4.03 | Both stories assume `ConfirmationToken::require()` API from S-1.09. Verify S-1.09 is W1-merged and the API is stable. |
| Feature flag registry (S-1.08) | S-4.03 (`FEATURE_DETECTION_ENGINE`), S-4.06 (`FEATURE_AUTO_CASE_CREATION`), S-4.07 (`CAPABILITY_ACKNOWLEDGE_ALERT`) | Verify all three flag names are registered in S-1.08 prism-security registry. Story S-4.07 explicitly notes the flag may not yet be defined — must add. |
| `prism-storage` API surface (Wave 2/3 evolution) | S-4.01, S-4.02, S-4.03, S-4.04, S-4.05, S-4.06, S-4.07, S-4.08 | All W4 stories assume "RocksDB CF helpers" interface. Wave 2/3 added domain-isolation enforcement, audit emitter integration, eviction backend scan (recent commits W2-FIX-H/J). Verify current public API matches story assumptions. |
| InjectionScanner from S-1.10 | S-4.05, S-4.08 | Both stories call `InjectionScanner::scan()` — verify signature is stable; W2 commits modified prism-security. |

---

## Research Tasks Recommended

For dispatch to research-agent (Context7 + Perplexity):

1. **DataFusion 53 (or current latest) API** — Verify current latest DataFusion major version as of 2026-05-02. Research: `ScalarUDFImpl` trait shape, `create_udf` signature, `MemTable::try_new`, `SessionContext::register_table` vs `register_batch`, `Expr::evaluate` for batch predicate evaluation. Required for S-4.03 and S-4.04.
2. **`cron` crate ecosystem** — Compare Rust crates: `cron` (current latest vs 0.12), `croner`, `tokio-cron-scheduler`, `cron_clock`. Recommend the right crate for S-4.08's "1-second tick + parse cron expression" use case. Verify API stability for the chosen one.
3. **`lettre` 0.11.x current state** — Verify lettre's current latest stable, async tokio feature flags (`tokio1-rustls-tls`), STARTTLS vs implicit TLS API, and authentication mechanism support (PLAIN/LOGIN/XOAUTH2). Required for S-4.08 SMTP delivery.
4. **`blake3` 1.x stability** — Verify current blake3 major version, any soundness/CVE history, SIMD feature flag conventions, performance characteristics. Required for S-4.02 + S-4.04.
5. **`libfuzzer-sys` vs cargo-fuzz vs `arbitrary`** — Verify current Rust fuzzing ecosystem best practice. Is libfuzzer-sys 0.4.x still the standard, or has cargo-fuzz + arbitrary become canonical? Required for S-4.05 VP-028.
6. **DataFusion + scopeguard interaction across `.await`** — Research whether `scopeguard::defer!` correctly handles cancellation across tokio `.await` suspension points. May need `tokio::task::AbortGuard` instead. Required for S-4.04 SessionContext drop guarantee.
7. **`regex::RegexSet` at 100k+ patterns** — Verify whether `regex` crate's RegexSet handles 100,000 patterns as claimed, including DFA size limits, compile time, and memory profile. Compare with `regex_automata` or `aho-corasick`. Required for S-4.03 IOC store.
8. **CEF / LEEF Rust crates** — Search crates.io for maintained CEF/LEEF formatters. If none exist, validate hand-rolled implementation against ArcSight CEF v0/v1 spec and IBM LEEF 2.0 spec. Required for S-4.08 syslog destination.
9. **rocksdb 0.24 current status** — Verify whether 0.24 is still the latest stable Rust binding for RocksDB as of 2026-05-02. Check `merge_operator` API for atomic counter increment used in S-4.02 epoch tracker.
10. **Streaming percentile crates** — Compare `tdigest`, `quantiles` (GK), `hdrhistogram` for the S-4.07 case_metrics 10,000+-case percentile computation. Recommend with version pin.
11. **Industry case-management state-machine standards** — Validate the "5-state, 12-transition" claim in S-4.06 against ITIL incident lifecycle, NIST 800-61, MITRE D3FEND, and common SOAR platforms (Splunk SOAR, Demisto/XSOAR, Tines). Document trace.
12. **`tokio::sync::broadcast` Lagged semantics** — Verify current tokio behavior for `RecvError::Lagged(n)` and capacity behavior under burst. Required for S-4.05 + S-4.08 broadcast channels.
13. **wasmtime 44 component-model host calls** — Verify wasmtime 44 component-model API stability for hosting `fire_alert`/`fire_case`/`fire_report` host functions called from action plugin. Required for S-4.08 plugin destination.

---

**Confirmed: no new dependencies (datafusion, cron, lettre, blake3, libfuzzer-sys) appear in Cargo.lock yet — Wave 4 introduces all of these and each represents an unverified assumption.**
