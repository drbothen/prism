---
document_type: verification-property-index
level: L4
version: "1.87"
status: draft
producer: state-manager
timestamp: 2026-06-12T00:00:00Z
phase: 2-patch
inputs: [architecture/verification-architecture.md]
traces_to: architecture/ARCH-INDEX.md
total_vps: 159
active_vps: 146
retired_vps: 13  # VP-095..VP-107 retired 2026-06-10 per ADR-037 (BC-3.3.001..004 retired); rows retained per POL-1
---

# Verification Property Index: Prism

> **Context Engineering:** This index lists all verification properties with their
> status and method. Load individual VP files only when working on that specific property.

> **`source_invariant:` schema convention (F-LP5-LOW-001, fix-burst-5):** The `source_invariant:`
> frontmatter field in VP files accepts only workspace-canonical DI-NNN identifiers from
> `domain-spec/invariants.md`. BC-local invariants (INV-* identifiers scoped to a single BC,
> valid only within that BC's body) MUST NOT populate `source_invariant:`; they are traced via
> `source_bc:` and cited in the VP's §Source Contract body prose. A VP with no DI-NNN invariant
> traces to `source_invariant: null`. Example: VP-153 `source_invariant: DI-012` (workspace
> invariant); VP-156 `source_invariant: null` with INV-INVALIDATION-EXT-001 cited in body only.

## Properties

| ID | Property | Module | Method | Priority | Status | Anchor Story |
|----|----------|--------|--------|----------|--------|--------------|
| VP-001 | OrgSlug rejects invalid characters | prism-core | kani | P0 | draft | S-1.01 |
| VP-002 | Capability resolution: deny-by-default | prism-core | kani | P0 | draft | S-1.03 |
| VP-003 | Capability resolution: most-specific-path wins | prism-core | kani | P0 | draft | S-1.03 |
| VP-004 | Capability resolution: deny overrides allow at same specificity | prism-core | kani | P0 | draft | S-1.03 |
| VP-005 | Case state machine: exactly 12 valid transitions | prism-core | kani | P0 | draft | S-1.02 |
| VP-006 | Case state machine: no self-transitions | prism-core | kani | P0 | draft | S-1.02 |
| VP-007 | Confirmation token expiry: expired at boundary (inclusive) | prism-security | kani | P0 | draft | S-1.09 |
| VP-008 | Confirmation token: single-use enforcement | prism-security | kani | P0 | draft | S-1.09 |
| VP-009 | Confirmation token: content hash mismatch rejects | prism-security | kani | P0 | draft | S-1.09 |
| VP-010 | Token cap: store rejects at 100 active tokens | prism-security | kani | P0 | draft | S-1.09 |
| VP-011 | Credential name sanitization: rejects path traversal | prism-core | kani | P0 | draft | S-1.02 |
| VP-012 | Alias depth: rejects composition beyond depth 3 | prism-query | kani | P0 | draft | S-3.04 |
| VP-013 | Alias cycles: detects and rejects cyclic references | prism-query | proptest | P0 | draft | S-3.04 |
| VP-014 | Query security limits: rejects oversized queries | prism-query | kani | P0 | verified (f5212641, 2026-05-05) | S-3.01 |
| VP-015 | Query security limits: rejects excessive nesting depth | prism-query | kani | P0 | verified (f5212641, 2026-05-05) | S-3.01 |
| VP-016 | OCSF normalization: output is valid protobuf | prism-ocsf | proptest | P0 | draft | S-1.04 |
| VP-017 | OCSF normalization: unmapped fields preserved | prism-ocsf | proptest | P0 | draft | S-1.05 |
| VP-018 | Detection rule validation: rejects invalid rules | prism-operations | proptest | P0 | draft | S-4.03 |
| VP-019 | Diff computation: deterministic | prism-operations | proptest | P0 | draft | S-4.02 |
| VP-020 | Feature flag: compile AND runtime must both permit | prism-security | kani | P0 | draft | S-1.08 |
| VP-021 | PrismQL parser: never panics on arbitrary input | prism-query | fuzz | P0 | draft | S-3.01 |
| VP-022 | OCSF normalizer: never panics on arbitrary input | prism-ocsf | fuzz | P0 | draft | S-1.04 |
| VP-023 | Sensor spec parser: never panics on arbitrary TOML | prism-spec-engine | fuzz | P0 | draft | S-1.11 |
| VP-024 | Injection scanner: detects known injection patterns | prism-security | proptest | P0 | draft | S-1.10 |
| VP-025 | Cache key derivation: deterministic | prism-query | kani | P1 | draft | S-3.05 |
| VP-026 | Splay computation: deterministic per (query, client) | prism-operations | kani | P1 | draft | S-4.01 |
| VP-027 | Alert dedup key: correct per match mode | prism-operations | proptest | P0 | draft | S-4.04 |
| VP-028 | Template interpolation: never panics | prism-operations | fuzz | P0 | draft | S-4.05 |
| VP-029 | Cursor cap: rejects at 200 active | prism-core | kani | P1 | draft | S-1.02 |
| VP-030 | Schedule/rule count caps: rejects beyond limits | prism-operations | kani | P1 | draft | S-4.01 |
| VP-031 | Required column enforcement: rejects unconstrained | prism-query | proptest | P0 | draft | S-3.02 |
| VP-032 | Hot reload atomicity: failed validation retains old config | prism-spec-engine | proptest | P1 | draft | S-1.12 |
| VP-033 | Audit buffer: RocksDB write completes before delivery attempt | prism-dtu-crowdstrike | integration_test | P0 | draft | S-6.07 |
| VP-034 | Encryption round-trip: encrypt then decrypt returns plaintext | prism-credentials | proptest | P0 | draft | S-1.06 |
| VP-035 | Key derivation: same inputs produce same key | prism-credentials | proptest | P1 | draft | S-1.06 |
| VP-036 | SessionContext dropped before error propagation and on panic | prism-dtu-crowdstrike | integration_test | P0 | draft | S-6.07 |
| VP-037 | Alias expansion: never panics on arbitrary alias graphs | prism-query | fuzz | P1 | draft | S-3.04 |
| VP-038 | Injection scanner: never panics on arbitrary input strings | prism-security | fuzz | P0 | draft | S-1.10 |
| VP-039 | Audit forward watermark: monotonically non-decreasing per destination across ACK, failure, and restart sequences | prism-audit | kani | P0 | draft | S-5.10 |
| VP-040 | Plugin Linker excludes all WASI namespace imports | prism-spec-engine | kani | P1 | draft | S-1.15 |
| VP-041 | Plugin memory limit boundary: at-limit succeeds, over-limit traps | prism-spec-engine | proptest | P1 | draft | S-1.15 |
| VP-042 | Plugin hot reload: failed compile retains old InstancePre | prism-spec-engine | proptest | P1 | draft | S-1.15 |
| VP-043 | WIT validation rejects component missing required exports | prism-spec-engine | proptest | P1 | draft | S-1.15 |
| VP-044 | Action retry state machine: bounded by 5 attempts, dead-letter terminal | prism-operations | kani | P0 | draft | S-4.08 |
| VP-045 | Action delivery semaphore: try_acquire used (non-blocking), never acquire | prism-operations | proptest | P0 | draft | S-4.08 |
| VP-046 | Action inline credential rejected at load time; value not in error message | prism-operations | proptest | P0 | draft | S-4.08 |
| VP-047 | UUID v7 validation: non-v7 always rejected, v7 always accepted, order preserved | prism-operations | proptest | P0 | draft | S-4.08 |
| VP-048 | Infusion spec: N fields produces exactly N UDF descriptors; duplicates error | prism-spec-engine | kani | P1 | draft | S-1.14 |
| VP-049 | Infusion per-query dedup: source calls = unique value count | prism-spec-engine | proptest | P1 | draft | S-1.14 |
| VP-050 | MCP sensor resource response redacts credentials and full API URLs | prism-mcp | proptest | P0 | draft | S-5.03 |
| VP-051 | Case state machine: exhaustive 5×5 transition table — 12 accept, 13 reject | prism-core | kani | P0 | draft | S-1.02 |
| VP-052 | update_case: disposition applied before status transition in single-call update | prism-operations | proptest | P0 | draft | S-4.06 |
| VP-053 | Resolved case always has non-null disposition; transition rejects without disposition | prism-operations | kani | P0 | draft | S-4.06 |
| VP-054 | TTR uses first resolution timestamp across reopen cycles; null aggregate when no resolved cases | prism-operations | proptest | P1 | draft | S-4.06 |
| VP-055 | StorageEngine put_batch atomicity and domain isolation (MockStorageEngine) | prism-storage | proptest | P1 | draft | S-1.02 |
| VP-056 | Audit buffer overflow purge: oldest entries deleted, newest preserved, purge-event produced | prism-audit | proptest | P1 | draft | S-5.10 |
| VP-057 | Crash recovery: denylist triggered at consecutive_crashes >= 3; exact threshold | prism-storage | kani | P0 | draft | S-1.02 |
| VP-058 | Watchdog memory grace period: single check does not terminate; two consecutive checks do | prism-storage | proptest | P0 | draft | S-2.02 |
| VP-059 | Spec validator: all errors collected (no fail-fast); warning-only specs return Ok | prism-spec-engine | proptest | P1 | draft | S-1.11 |
| VP-060 | Dedup decision: Link(c.id) iff existing case within window; Create otherwise | prism-operations | proptest | P0 | draft | S-4.06 |
| VP-061 | Log forwarder min-level filter: per-destination enqueue/discard matches level-rank ordering for all 5×5 level pairs | prism-mcp | proptest | P1 | draft | S-5.09 |
| VP-062 | Log forwarder queue cap: queue.len() never exceeds 10 × batch_size; drop_count +1 per overflow enqueue | prism-mcp | proptest | P1 | draft | S-5.09 |
| VP-063 | [BC-3.1.001] OrgRegistry round-trip: resolve(slug) then slug_for(id) returns original slug | prism-core | proptest | P0 | draft | S-3.1.01 |
| VP-064 | [BC-3.1.001] No-side-effect: resolve or slug_for never changes registry size | prism-core | proptest | P0 | draft | S-3.1.01 |
| VP-065 | [BC-3.1.001] O(1) bound: lookup completes in bounded steps regardless of registry size | prism-core | kani | P1 | draft | S-3.1.01 |
| VP-066 | [BC-3.1.002] Every AuditEntry has non-null org_id and non-null org_slug | prism-audit | proptest | P0 | draft | S-3.1.07 |
| VP-067 | [BC-3.1.002] org_id is stable across rename: same UUID with different slugs both returned by org_id query | prism-audit | proptest | P0 | draft | S-3.1.07 |
| VP-068 | [BC-3.1.002] Denormalized slug matches OrgRegistry slug at time of emission | prism-audit | integration_test | P0 | draft | S-3.1.07 |
| VP-069 | [BC-3.1.003] Bijection invariant: forward-map size == reverse-map size after every operation | prism-core | proptest | P0 | draft | S-3.1.03 |
| VP-070 | [BC-3.1.003] No duplicate slug: two successful registrations with same slug is impossible | prism-core | kani | P0 | draft | S-3.1.03 |
| VP-071 | [BC-3.1.003] No duplicate uuid: two successful registrations with same uuid is impossible | prism-core | kani | P0 | draft | S-3.1.03 |
| VP-072 | [BC-3.1.003] Rename atomicity: no intermediate state observed by concurrent reader | prism-core | proptest | P0 | draft | S-3.1.03 |
| VP-073 | [BC-3.1.004] Registry size unchanged after any Err return from register | prism-core | proptest | P0 | draft | S-3.1.03 |
| VP-074 | [BC-3.1.004] Err(SlugConflict) message contains both existing UUID and attempted UUID | prism-core | proptest | P0 | draft | S-3.1.03 |
| VP-075 | [BC-3.1.004] Err(IdConflict) message contains both existing slug and attempted slug | prism-core | proptest | P0 | draft | S-3.1.03 |
| VP-076 | [BC-3.1.004] After N successful registrations and one rejected, resolve correct for all N pairs | prism-core | proptest | P0 | draft | S-3.1.03 |
| VP-077 | [BC-3.2.001] Cross-org lookup returns empty/None: write under org_id_A, lookup under org_id_B | prism-sensors | proptest | P0 | draft | S-3.2.01 |
| VP-078 | [BC-3.2.001] Write under org_id_A does not modify any entry keyed under org_id_B | prism-sensors | proptest | P0 | draft | S-3.2.01 |
| VP-079 | [BC-3.2.001] OrgId-flipping mutation: replacing org_id in lookup key returns wrong result | prism-sensors | proptest | P0 | draft | S-3.2.01 |
| VP-080 | [BC-3.2.001] reset_for(org_id_A) removes exactly org_id_A entries and no others | prism-sensors | proptest | P0 | draft | S-3.2.01 |
| VP-081 | [BC-3.2.002] Cross-org cred lookup returns NotFound: cred stored under org_id_A not returned for org_id_B | prism-credentials | proptest | P0 | draft | S-3.1.04 |
| VP-082 | [BC-3.2.002] Namespace key never contains slug string after OrgId migration | prism-credentials | proptest | P0 | draft | S-3.1.04 |
| VP-083 | [BC-3.2.002] Rename does not invalidate credential: same org_id returns same cred before and after rename | prism-credentials | integration_test | P0 | draft | S-3.1.04 |
| VP-084 | [BC-3.2.003] Cross-org token validation always false: token under org_id_A invalid in org_id_B context | prism-credentials | proptest | P0 | draft | S-3.2.08 |
| VP-085 | [BC-3.2.003] Refresh preserves org binding: new token stored under same org_id as expired token | prism-credentials | proptest | P0 | draft | S-3.2.03 |
| VP-086 | [BC-3.2.003] reset_for(org_id_A) removes only org_id_A tokens; org_id_B tokens survive | prism-credentials | proptest | P0 | draft | S-3.2.03 |
| VP-087 | [BC-3.2.004] OrgId appears in payload body: shared-mode payload JSON contains "org_id" key | prism-sensors | proptest | P0 | draft | S-3.2.05 |
| VP-088 | [BC-3.2.004] OrgId absent from HTTP routing fields: URL and headers contain no org_id or org_slug | prism-sensors | proptest | P0 | draft | S-3.2.05 |
| VP-089 | [BC-3.2.004] Concurrent sends produce independent payloads with distinct org_id values | prism-sensors | proptest | P0 | draft | S-3.2.05 |
| VP-090 | [BC-3.2.004] Mode metadata absent from query results: result rows contain no mode field | prism-sensors | integration_test | P0 | draft | S-3.2.05 |
| VP-091 | [BC-3.2.005] DtuMode has no setter: no public method accepts DtuMode after startup | prism-sensors | proptest | P0 | draft | S-3.2.05 |
| VP-092 | [BC-3.2.005] Startup rejects unknown mode values: serde of non-shared/non-client string returns Err | prism-sensors | proptest | P0 | draft | S-3.2.05 |
| VP-093 | [BC-3.2.005] Security Telemetry type with mode=shared causes startup error | prism-sensors | proptest | P0 | draft | S-3.2.05 |
| VP-094 | [BC-3.2.005] reload_config does not apply mode changes | prism-sensors | integration_test | P0 | draft | S-3.3.06 |
| VP-095 | ~~[BC-3.3.001] Every ST type in DTU_DEFAULT_MODE triggers startup error paired with mode=shared~~ | prism-spec-engine | unit_test | P0 | retired (ADR-037) | S-3.3.01 |
| VP-096 | ~~[BC-3.3.001] No MSSP Coordination type triggers startup error paired with mode=client~~ | prism-spec-engine | unit_test | P0 | retired (ADR-037) | S-3.3.01 |
| VP-097 | ~~[BC-3.3.001] Startup error message contains DTU type string and config file path~~ | prism-spec-engine | unit_test | P0 | retired (ADR-037) | S-3.3.01 |
| VP-098 | ~~[BC-3.3.001] Multi-error: N violations produce N errors in one pass before abort~~ | prism-spec-engine | unit_test | P0 | retired (ADR-037) | S-3.3.01 |
| VP-099 | ~~[BC-3.3.002] Non-scheme credential-pattern field value always causes exit code 1~~ | prism-spec-engine | proptest | P0 | retired (ADR-037) | S-3.3.01 |
| VP-100 | ~~[BC-3.3.002] E-CFG-020 error message never contains the literal field value~~ | prism-spec-engine | proptest | P0 | retired (ADR-037) | S-3.3.01 |
| VP-101 | ~~[BC-3.3.002] All four allowed scheme prefixes accepted for credential-pattern fields~~ | prism-spec-engine | proptest | P0 | retired (ADR-037) | S-3.3.01 |
| VP-102 | ~~[BC-3.3.003] All integer schema_version values != 1 produce exit code 1~~ | prism-spec-engine | proptest | P0 | retired (ADR-037) | S-3.3.01 |
| VP-103 | ~~[BC-3.3.003] Absent schema_version produces E-CFG-030, not E-CFG-031~~ | prism-spec-engine | proptest | P0 | retired (ADR-037) | S-3.3.01 |
| VP-104 | ~~[BC-3.3.003] schema_version=1 never produces schema-version error regardless of other fields~~ | prism-spec-engine | proptest | P0 | retired (ADR-037) | S-3.3.01 |
| VP-105 | ~~[BC-3.3.004] Exit code 0 implies OrgRegistry entry count equals file count~~ | prism-spec-engine | proptest | P0 | retired (ADR-037) | S-3.3.02 |
| VP-106 | ~~[BC-3.3.004] Any validation error implies exit code 1 and empty OrgRegistry~~ | prism-spec-engine | proptest | P0 | retired (ADR-037) | S-3.3.02 |
| VP-107 | ~~[BC-3.3.004] Validation error output always includes the offending filename~~ | prism-spec-engine | integration_test | P0 | retired (ADR-037) | S-3.3.02 |
| VP-108 | [BC-3.4.001] Generator idempotent: generate(inputs) == generate(inputs) | prism-dtu-common | kani | P0 | draft | S-3.7.01 |
| VP-109 | [BC-3.4.001] Different seeds produce different records with overwhelming probability | prism-dtu-common | proptest | P0 | draft | S-3.7.01 |
| VP-110 | [BC-3.4.001] Different orgs produce different records for same seed with overwhelming probability | prism-dtu-common | proptest | P0 | draft | S-3.7.01 |
| VP-111 | [BC-3.4.001] No thread_rng or SystemTime::now in generator call stack | prism-dtu-common | proptest | P0 | draft | S-3.7.01 |
| VP-112 | [BC-3.4.002] All non-SchemaDrift archetype records pass schema validation | prism-dtu-common | integration_test | P0 | draft | S-3.7.00 |
| VP-113 | [BC-3.4.002] SchemaDrift archetype: provenance.schema_valid false and at least one record fails | prism-dtu-common | proptest | P0 | draft | S-3.7.00 |
| VP-114 | [BC-3.4.002] Schema validation absent from release build (cfg(test) gate) | prism-dtu-common | proptest | P0 | draft | S-3.7.00 |
| VP-115 | [BC-3.4.003] Each archetype at scale=1.0 produces documented baseline record count | prism-dtu-common | integration_test | P0 | draft | S-3.7.01 |
| VP-116 | [BC-3.4.003] floor(baseline*scale) formula holds for all archetypes and scales in [0.01,100.0] | prism-dtu-common | proptest | P0 | draft | S-3.7.01 |
| VP-117 | [BC-3.4.003] DormantTenant always produces 0 records for all scale values | prism-dtu-common | proptest | P0 | draft | S-3.7.01 |
| VP-118 | [BC-3.4.003] SchemaDrift always produces exactly 1 non-conformant record | prism-dtu-common | proptest | P0 | draft | S-3.7.01 |
| VP-119 | [BC-3.4.004] Generated record ID sets disjoint for all org pairs with distinct slugs | prism-dtu-common | proptest | P0 | draft | S-3.7.02 |
| VP-120 | [BC-3.4.004] Every record primary ID contains org slug as a substring | prism-dtu-common | proptest | P0 | draft | S-3.7.02 |
| VP-121 | [BC-3.4.004] OrgRegistry lookup failure returns Err(UnregisteredOrg) and does not panic | prism-dtu-common | proptest | P0 | draft | S-3.7.02 |
| VP-122 | [BC-3.5.001] endpoints map entry count equals orgs-count times dtu-types-per-org after build() | prism-dtu-harness | proptest | P0 | draft | S-3.3.03 |
| VP-123 | [BC-3.5.001] All socket addresses in endpoints are pairwise distinct (no port collision) | prism-dtu-harness | proptest | P0 | draft | S-3.3.03 |
| VP-124 | [BC-3.5.001] After drop(harness), TcpStream::connect to every clone addr returns ConnectionRefused | prism-dtu-harness | integration_test | P0 | draft | S-3.3.03 |
| VP-125 | [BC-3.5.002] All SocketAddrs in customer_endpoints are pairwise distinct after build() | prism-dtu-harness | proptest | P0 | draft | S-3.3.04 |
| VP-126 | [BC-3.5.002] Wrong-org credentials to live clone returns HTTP 401, never HTTP 200 | prism-dtu-harness | integration_test | P0 | draft | S-3.3.04 |
| VP-127 | [BC-3.5.002] devices(OrgA) ∩ devices(OrgB) = ∅ for all org pairs in 3-org canonical scenario | prism-dtu-harness | integration_test | P0 | draft | S-3.3.04 |
| VP-128 | [BC-3.6.001] inject_failure on (OrgA,X) does not mutate FailureLayerShared of (OrgB,Y) | prism-dtu-harness | proptest | P0 | draft | S-3.6.01 |
| VP-129 | [BC-3.6.001] All supported FailureMode variants for each clone produce the documented HTTP status code or behavior (per Invariant 5 supported-mode table) | prism-dtu-harness | integration_test | P0 | draft | S-3.6.01 |
| VP-130 | [BC-3.6.001] clear_failure followed by request always returns HTTP 200 | prism-dtu-harness | integration_test | P0 | draft | S-3.6.01 |
| VP-131 | [BC-3.6.002] Clone panic detected within 1s of task exit | prism-dtu-harness | integration_test | P0 | draft | S-3.6.02 |
| VP-132 | [BC-3.6.002] drop(harness) after any number of clone crashes completes without hanging | prism-dtu-harness | integration_test | P0 | draft | S-3.6.02 |
| VP-133 | [BC-3.6.002] Targeted crashed clone returns CloneCrashed, never ConnectionRefused | prism-dtu-harness | integration_test | P0 | draft | S-3.6.02 |
| VP-134 | [BC-3.7.001] check-crate-layout.sh exits 0 for all 22 workspace crates after fixture migration | prism-bin | integration_test | P1 | draft | S-3.5.01 |
| VP-135 | [BC-3.7.001] check-crate-layout.sh exits non-zero for synthetic non-conformant crate | prism-bin | proptest | P1 | draft | S-3.5.01 |
| VP-136 | [BC-3.7.001] check-crate-layout.sh is read-only: no files created, modified, or deleted | prism-bin | integration_test | P1 | draft | S-3.5.01 |
| VP-137 | Schedule executor liveness: per-subsystem semaphore non-starvation | prism-operations | proptest | P1 | draft | S-4.01, S-4.08 |
| VP-138 | Cross-org case access denied (INV-CASE-003): Wave 4 case-management isolation invariant | prism-operations | proptest | P0 | draft | S-4.06 |
| VP-139 | IOC matching layered correctness (aho-corasick + RegexSet split equivalence) | prism-operations | proptest | P1 | draft | S-4.03 |
| VP-140 | Dedup window scheduling-time resolution + invalidation correctness | prism-operations | proptest | P1 | draft | S-4.03, S-4.04 |
| VP-141 | Epoch counter merge_operator atomicity (concurrent increments never lost) | prism-operations | proptest | P1 | draft | S-4.02 |
| VP-142 | Pack expansion idempotence (double-register produces identical ScheduleEntry set) | prism-operations | proptest | P1 | draft | S-4.02 |
| VP-143 | Action delivery non-starvation (per-subsystem semaphore non-starvation for action delivery side) | prism-operations | proptest | P1 | draft | S-4.08 |
| VP-144 | CEF v0 + LEEF 2.0 encoder correctness (13 proptest invariants: INV-CEF-001..005, INV-LEEF-001..005, INV-RT-001..003) | prism-siem-formats | proptest | P1 | draft | S-4.08 |
| VP-145 | Case reopen_count monotonic increment (INV-CASE-006) | prism-operations | proptest | P1 | draft | S-4.06, S-4.07 |
| VP-146 | No production hardcoded sensor references (FORBIDDEN-SYMBOLS-001 compile-fail perimeter) — VP-PLUGIN-001 alias | prism-spec-engine | integration_test | P0 | draft | PLUGIN-MIGRATION-001-A |
| VP-147 | PipelineExecutor::execute returns non-empty records against wiremock DTU clone (integration test target) — VP-PLUGIN-002 alias | prism-spec-engine | integration_test | P0 | draft | S-PLUGIN-PREREQ-B |
| VP-148 | DTU parity: TOML+plugin path output matches deleted Rust adapter path per sensor — VP-PLUGIN-003 alias | prism-spec-engine | integration_test | P0 | draft | PLUGIN-MIGRATION-001-D |
| VP-149 | Boot warning fires on unsigned plugin load (v1.0 scope) — VP-PLUGIN-004 alias | prism-spec-engine | integration_test | P0 | draft | PLUGIN-PREREQ-D |
| VP-150 | OAuth2 refresh-on-401 via declarative TOML retry policy (PipelineExecutor) — VP-PLUGIN-005 alias | prism-spec-engine | integration_test | P0 | draft | S-PLUGIN-PREREQ-B |
| VP-151 | OCSF column mapping fixture catalog (6 representative cases, SpecDrivenMapper) — VP-PLUGIN-006 alias | prism-spec-engine | integration_test | P1 | draft | PLUGIN-MIGRATION-001-C |
| VP-152 | Plugin manifest allowlist explicit Vec<String> after PREREQ-D (allowed_urls enforcement under default-deny semantics) — VP-PLUGIN-007 alias | prism-spec-engine | integration_test | P0 | draft | PLUGIN-PREREQ-D |
| VP-153 | SensorAuth runtime cross-composition prevention (DI-012 runtime replacement): all invalid (auth_type, credential_type) pairs rejected at spec-load time; error messages redact credential values | prism-spec-engine | proptest | P0 | active — v0.23 | S-PLUGIN-PREREQ-E |
| VP-154 | CustomAdapter behavioral equivalence: PluginRuntime WASM dispatch produces non-empty records matching plugin fixture output; TOML fallthrough when no plugin registered | prism-spec-engine | integration_test | P1 | draft | PLUGIN-MIGRATION-001-A |
| VP-155 | CustomAdapter absent from prism-spec-engine public API: compile-fail perimeter asserts CustomAdapter and CustomAdapterRegistry are unimportable post-PREREQ-E | prism-spec-engine | integration_test | P0 | draft | PLUGIN-MIGRATION-001-A |
| VP-156 | WriteToolInvalidationMap registration uniqueness: duplicate tool_name returns Err(DuplicateWriteToolRegistration); first registration persists unchanged | prism-query | proptest | P1 | active — v0.24 | S-PLUGIN-PREREQ-E |
| VP-157 | [BC-3.6.001] POST /dtu/configure with unsupported mode returns HTTP 400 with unsupported_failure_mode error; no state change | prism-dtu-harness | unit_test | P1 | draft | S-3.6.01 |
| VP-158 | [BC-2.06.019] E-DEMO-006 fires when two scenario-enabled clones share same seed but have different org_ids; no clone constructed — VP-019-I alias | prism-dtu-demo-server | unit_test | P1 | draft | S-DEMO-DTU-LIVE-SCENARIO-001-B |
| VP-159 | [BC-2.16.014] DeclarativeHttpAuthProvider lazy acquisition and refresh-on-expiry: zero network at construction; cold get_token → one HTTP POST + cache; warm get_token within TTL → zero HTTP POSTs; stale get_token → one HTTP POST re-acquisition; acquire_token → one HTTP POST cache bypass; TTL arithmetic for both ExpiryMode variants (absolute_utc_string, relative_seconds); CachedAuthToken never stores credential values (AD-017) | prism-spec-engine | integration_test | P1 | draft | [PLANNED — Wave-A CrowdStrike retirement / Armis token-exchange engine story] |

## VP-PLUGIN-001..007 Named Series (PREREQ-F Registration, ADR-023 §Architectural Constraints)

The VP-PLUGIN-NNN series is the canonical named series for plugin-only architecture verification
properties. Each is registered with `module: prism-spec-engine` per ADR-023 L511-512 and
F-PASS3-HIGH-001. The numeric VP-146..VP-152 entries above are the sequential index aliases.

| Named ID | Aliases | Property | Module | Method | Priority | Status | Anchor Story |
|----------|---------|----------|--------|--------|----------|--------|--------------|
| VP-PLUGIN-001 | VP-146 | No production hardcoded sensor references (FORBIDDEN-SYMBOLS-001 compile-fail perimeter) — zero non-test references to any of 9 forbidden symbols post-PREREQ-A | prism-spec-engine | integration_test | P0 | draft | PLUGIN-MIGRATION-001-A |
| VP-PLUGIN-002 | VP-147 | PipelineExecutor::execute returns non-empty records against at least one wiremock DTU clone (replaces Ok(Vec::new()) stub) | prism-spec-engine | integration_test | P0 | draft | S-PLUGIN-PREREQ-B |
| VP-PLUGIN-003 | VP-148 | DTU-parity: plugin-produced OCSF record matches DTU-clone reference per TS-PLUGIN-PARITY-001 canonicalization | prism-spec-engine | integration_test | P0 | draft | PLUGIN-MIGRATION-001-D |
| VP-PLUGIN-004 | VP-149 | Boot warning fires on unsigned plugin load (v1.0 scope): WARN-level log + audit log entry event_type=plugin_load_unsigned on every startup with plugins present | prism-spec-engine | integration_test | P0 | draft | PLUGIN-PREREQ-D |
| VP-PLUGIN-005 | VP-150 | OAuth2 refresh-on-401 via declarative TOML retry policy (PipelineExecutor) | prism-spec-engine | integration_test | P0 | draft | S-PLUGIN-PREREQ-B |
| VP-PLUGIN-006 | VP-151 | OCSF column mapping fixture catalog (6 representative cases, SpecDrivenMapper): all 13 mapping patterns covered, byte-equal post-canonicalization per TS-PLUGIN-PARITY-001 | prism-spec-engine | integration_test | P1 | draft | PLUGIN-MIGRATION-001-C |
| VP-PLUGIN-007 | VP-152 | Plugin manifest allowlist explicit Vec<String> after PREREQ-D: manifest without allowed_urls field rejected at load time per AC-5 manifest gate (default-deny consumer is AC-7); allowed_urls=[] blocks all HTTP; non-empty list enforces host-only allowlist | prism-spec-engine | integration_test | P0 | draft | PLUGIN-PREREQ-D |

## Summary

| Method | Count | P0 | P1 |
|--------|-------|----|----|
| Kani | 30 | 23 | 7 |
| Proptest | 88 | 66 | 22 |
| Unit test | 6 | 4 | 2 |
| Fuzz | 6 | 5 | 1 |
| Integration test | 29 | 24 | 5 |
| **Total** | **159** | **122** | **37** |

> Note: VP-PLUGIN-001..007 are named aliases for VP-146..VP-152; they do not add to the sequential count. VP-019-I is a BC-2.06.019 named alias for VP-158; it does not add to the sequential count. Total sequential VPs is 159 (VP-153, VP-154, VP-155 added for S-PLUGIN-PREREQ-E in burst 2026-05-15; VP-156 added in prereq-e-fix-burst-1 2026-05-15; VP-157 added D-1099 2026-06-11; VP-158 added 2026-06-12 BC-2.06.019 E-DEMO-006 guard; VP-159 added D-1947 2026-07-22 BC-2.16.014 DeclarativeHttpAuthProvider lazy acquisition and refresh-on-expiry).

> **Count basis (POL-1 append-only):** The table above counts REGISTERED rows — retired VPs are never deleted. Of the 159 rows, 13 are retired per ADR-037 (2026-06-10): VP-095..VP-098 (unit_test, P0), VP-099..VP-106 (proptest, P0), VP-107 (integration_test, P0). **Active basis: 146 VPs; active P0 = 109** (Kani 23, Proptest 58, Unit test 0, Fuzz 5, Integration test 23); active P1 = 37 (VP-157 unit_test P1 added D-1099; VP-158 unit_test P1 added 2026-06-12; VP-159 integration_test P1 added D-1947 2026-07-22). Retired VPs are excluded from the release verification gate. Per-VP disposition: see §ADR-037 Retirement below.

### ADR-037 Retirement (2026-06-10) — BC-3.3.001..004 VPs

BC-3.3.001 through BC-3.3.004 were retired at BC-INDEX v6.11 per ADR-037 (prism-customer-config crate retirement; ADR-010 superseded — the `customers/{org_slug}.toml` schema these VPs verified no longer exists). All 13 VPs anchored to those BCs are retired in place: rows above retain Module/Method/Priority verbatim (row-count basis), Status → `retired (ADR-037)`. No individual VP files exist for VP-095..VP-107 (registered index-only during wave-3-registration v1.12), so no VP-file withdrawal documents are required.

| VP | Source BC (retired) | Method | Disposition |
|----|--------------------|--------|-------------|
| VP-095 | BC-3.3.001 | unit_test | Retired — property surface gone: per-customer `mode` declaration retired with the `[[dtu]]` schema, so the ST+shared misconfiguration is no longer expressible. Isolation is structural: prism-core `DtuMode` + per-instance binding (BC-2.06.017) + BC-3.2.005 (VP-091..VP-094 remain active, incl. VP-093 ST+shared serde/spec-layer rejection). No successor VP. |
| VP-096 | BC-3.3.001 | unit_test | Retired — same surface as VP-095 (MSSP-Coordination/client acceptance leg of the retired guard). No successor VP. |
| VP-097 | BC-3.3.001 | unit_test | Retired — error-message property of retired guard (E-CFG-017 path retired with the schema). No successor VP. |
| VP-098 | BC-3.3.001 | unit_test | Retired — multi-error reporting over `customers/*.toml`; collect-all-errors intent survives on the superseding surfaces (BC-2.21.001 boot step 3; BC-2.06.015/016 OverlayLoader). No successor VP. |
| VP-099 | BC-3.3.002 | proptest | Retired — credential-pattern scan of retired schema. No-credential-values intent carried by BC-2.06.003 / ADR-032 reference-based resolution (credential-redaction coverage in prism-credentials per ADR-037 §Consequences). No successor VP. |
| VP-100 | BC-3.3.002 | proptest | Retired — E-CFG-020 no-value-echo property; error code retired with the schema. Redaction intent carried by BC-2.06.003 / ADR-032 surfaces. No successor VP. |
| VP-101 | BC-3.3.002 | proptest | Retired — four-scheme `credential_ref` prefix acceptance; the scheme grammar itself was superseded by the ADR-032 env-var convention + ADR-034 Tier-3 keyring. No successor VP. |
| VP-102 | BC-3.3.003 | proptest | Retired — `schema_version` exclusive to the retired schema; ADR-037 §Consequences accepts no successor. No successor VP. |
| VP-103 | BC-3.3.003 | proptest | Retired — E-CFG-030/031 distinction retired with `schema_version`. No successor VP. |
| VP-104 | BC-3.3.003 | proptest | Retired — `schema_version = 1` acceptance leg of the retired check. No successor VP. |
| VP-105 | BC-3.3.004 | proptest | Retired — exit-0 ⇒ OrgRegistry-count==file-count over `customers/*.toml`; superseded surface is BC-2.21.001 (prism.toml `[[orgs]]` boot step 3 validation) + BC-2.06.012–016 (OverlayLoader). Equivalent coverage exists as boot tests in prism-bin per ADR-037 §Consequences; no successor VP registered for the retired file-count formulation. |
| VP-106 | BC-3.3.004 | proptest | Retired — any-error ⇒ exit-1 + empty OrgRegistry; refuse-to-start intent carried by BC-2.21.001 (exit 2 on failure, ADR-022 §B step 3). No successor VP. |
| VP-107 | BC-3.3.004 | integration_test | Retired — offending-filename property of retired directory loader; OverlayLoader validation (BC-2.06.013/015/016) names the offending overlay file on its superseding surface. No successor VP. |

### Phase 3-Patch Addition (2026-04-16, Burst 2.5)

**VP-039** proposed by BC-2.05.011 (Audit Forwarding At-Least-Once). Kani harness proves the per-destination forward watermark is monotonically non-decreasing across all event sequences: ACK, transient network failure, permanent destination failure, and process restart with RocksDB watermark recovery. Story anchor: S-5.10.

### Phase 3-Patch Reassignment (2026-04-16, Burst 6b)

**VP-033 and VP-036** reassigned to `prism-dtu-crowdstrike` (anchor story S-6.07):

- **VP-033** (Audit buffer RocksDB-write-before-delivery ordering): module `prism-audit` → `prism-dtu-crowdstrike`; anchor S-2.04 → S-6.07
- **VP-036** (SessionContext drop on error): module `prism-operations` → `prism-dtu-crowdstrike`; anchor S-4.04 → S-6.07

Both VPs remain `integration_test` method. VP-033 and VP-036 are integration tests that exercise the CrowdStrike behavioral clone. The test code lives in `crates/prism-dtu-crowdstrike/tests/`. The VPs verify cross-crate interaction behavior (prism-audit ordering / prism-operations SessionContext drop) but the execution vehicle is the DTU crate. Since the DTU crate (`prism-dtu-crowdstrike`, story S-6.07) provides the behavioral clone against which these tests run, S-6.07 is the authoritative anchor story.

### VP-029 Anchor Justification (2026-04-19, P3P41-A-OBS-001 — updated to Option B)

**VP-029** (Cursor cap: rejects at 200 active) is anchored to S-1.02 and module `prism-core`. The cap invariant has **joint ownership** across two subsystems:

- **Enforcement vehicle:** S-1.02 / `prism-core` — The 200-cursor cap is enforced at the `CursorRegistry::allocate()` boundary inside `crates/prism-core/src/cursor.rs`. The `CursorId` newtype and `CursorRegistry` struct are foundational prism-core entities; the invariant (reject when `active.len() >= 200`) is a type-level allocation boundary. S-1.02 delivers `CursorId`, `CursorRegistry`, and the VP-029 Kani proof at `crates/prism-core/src/proofs/cursor.rs`.

- **Policy owner:** SS-07 (Adapter Pagination & Response Cache, owned by `prism-query`) — SS-07 owns the semantic cap requirement: concurrent pagination must be bounded to 200 active cursors to enforce memory safety and prevent unbounded allocation across all pagination consumers. SS-07 calls `allocate()` and `release()` to drive pagination semantics; the cap value itself is SS-07's policy, enforced at the allocation site in prism-core.

S-1.02 frontmatter has been updated to `subsystems: [SS-03, SS-07, SS-11, SS-12, SS-14]`, making the cross-subsystem contribution explicit. SS-07 is named because S-1.02's `CursorRegistry` directly enforces SS-07's cap policy — not merely because SS-07 consumes the type.

**Conclusion (Option B):** VP-029 anchor to S-1.02/prism-core is correct as the enforcement vehicle. SS-07 is additionally named in S-1.02's subsystem list as the cap policy owner. Joint ownership is now explicit in both artifacts. Supersedes Option C justification-only resolution from v1.4. Closes P3P41-A-OBS-001.

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.87 | wave-a-spec-evolution-fix-burst-6 | 2026-07-22 | state-manager | VP-153 v0.22→v0.23 (F-WASE-P6-MED-001: §Feasibility Assessment "Harness dependencies" row — stale file path `spec_loader.rs` corrected to `spec_parser.rs`; sole stale reference confirmed by workspace sweep across .factory/specs/). Status cell updated to `active — v0.23`. VP-INDEX v1.86→v1.87. |
| 1.86 | wave-a-spec-evolution-fix-burst-5 | 2026-07-22 | state-manager | VP-159 v1.2→v1.3 (F-WASE-P5-MED-001: input-hash trail reconciliation — generic at-commit-time wording replaces explicit hash citations in v1.3 changelog row to prevent future frontmatter-vs-row contradictions; LESSON: changelog rows must use generic wording for current-burst hash changes; F-WASE-P5-LOW-001: §Proof Harness Skeleton constructor fixes — 10 MockCredentialResolver::default()/with_secret() sites rewritten to as-built ::new(...) constructor). VP-INDEX v1.85→v1.86. |
| 1.85 | D-1953 | 2026-07-22 | state-manager | VP-159 v1.1→v1.2 (F-WASE-P4-OBS-001: 5 new TTL-arithmetic skeleton fns for AC-6/AC-7 incl. malformed-RFC-3339 + default-1799 branches; OBS-003: §Source Contract authoring-source vs verified-set disambiguation); VP-153 v0.21→v0.22 (F-WASE-P4-OBS-002: §Re-verification Gate — engine story must re-run all 8 proptests with token_exchange arms activated, merge-blocking). VP-INDEX v1.84→v1.85. |
| 1.84 | D-1947/D-1948 Wave-A fix-burst 1 | 2026-07-22 | architect | F-WASE-P1-MED-001: v1.83 Burst cell corrected D-1946→D-1947; §Note and §Count-basis burst references updated D-1946→D-1947 (VP-159 and VP-INDEX registration is burst 2, D-1947; BC-2.16.014 authoring is burst 1, D-1946). |
| 1.83 | D-1947 | 2026-07-22 | architect | VP-159 added (integration_test, P1, prism-spec-engine, BC-2.16.014, source_invariant DI-012 — DeclarativeHttpAuthProvider lazy acquisition and refresh-on-expiry; folds DRIFT-D849-002). Summary: Integration test 28→29 (P1 4→5), Total 158→159 (P1 36→37). Active basis: 145→146 VPs, active P1 36→37. POL-9 same-burst: verification-architecture.md v1.44→v1.45, verification-coverage-matrix.md v1.45→v1.46. VP-INDEX v1.82→v1.83. |
| 1.82 | D-1926 | 2026-07-21 | state-manager | VP-153 v0.19→v0.20 (D-1926 adversary pass-30 MED root fix: §Proof Harness Skeleton reconciled from divergent typed-enum pseudocode to AS-BUILT two-file design — vp153_sensorauth_cross_composition.rs Rules A+B constructs + vp153_rule_c_shaped_probe.rs Rule C constructs, all verified in source; phantom-helper references removed from live text; POL-31 closure). VP-INDEX v1.81→v1.82. |
| 1.81 | D-1915 | 2026-07-21 | state-manager | VP-153 v0.18→v0.19 (D-1915 adversary pass-15 OBS-1 bookkeeping fix): §Changelog rows v0.16 (FB75) and v0.15 (FB71) were out of monotonic order; changelog converted to newest-first (descending) convention per validate-changelog-monotonicity hook. No property semantics changed. VP-INDEX v1.80→v1.81. |
| 1.80 | demo-readiness-2026-06-24 | 2026-06-24 | state-manager | POL-9 back-link propagation for D-1317 demo-readiness contract layer: VP-014 v1.6→v1.7 (BC-2.11.020 "Also cited by" added — SqlPipe FORBID-BOTH composed queries subject to same 64KB size limit); VP-021 v1.4→v1.5 (BC-2.11.020, BC-2.11.021, BC-2.11.022, BC-2.11.023 "Also cited by" added — new parse entry points + mode-bridge paths must not panic). No new VP IDs allocated. No property semantics changed. VP-INDEX v1.79→v1.80. |
| 1.79 | BC-2.06.019-vp-propagation | 2026-06-12 | architect | BC-2.06.019 v1.2 OBS-1 VP propagation (POL-9 same-burst): VP-158 added (unit_test, P1, prism-dtu-demo-server, BC-2.06.019 PRE-6, S-DEMO-DTU-LIVE-SCENARIO-001-B) — E-DEMO-006 org_id-equality guard; named alias VP-019-I. Summary: Unit test 5→6, Total 157→158, P1 35→36. Active basis: 144→145 VPs, active P1 = 36. POL-9 same-burst propagation: verification-architecture.md v1.43→v1.44, verification-coverage-matrix.md v1.44→v1.45. VP-INDEX v1.78→v1.79. |
| 1.78 | D-1099 | 2026-06-11 | state-manager | BC-3.6.001 v0.5 POL-9 propagation: VP-129 description updated with Invariant 5 per-clone supported-mode table scope qualifier. VP-157 added (unit_test, P1, prism-dtu-harness, BC-3.6.001, S-3.6.01) — POST /dtu/configure unsupported-mode 400 guard. **ID note:** PO authored BC-3.6.001 v0.5 citing VP-131 for this property; VP-131 was already registered to BC-3.6.002 (Clone panic detection). Allocated next available sequential ID VP-157. BC-3.6.001 references to VP-131 for this property require a PO correction burst (BC body edit outside state-manager scope). Summary: Unit test 4→5, Total 156→157, P1 34→35. Active basis: 144 VPs, active P1 = 35. POL-9 same-burst propagation: verification-architecture.md v1.42→v1.43, verification-coverage-matrix.md v1.43→v1.44. VP-INDEX v1.77→v1.78. |
| 1.77 | review-2026-06-10-architect-burst-2 | 2026-06-10 | architect | ADR-037 VP retirement (POL-9 same-burst propagation with verification-architecture.md v1.42 + verification-coverage-matrix.md v1.43): VP-095..VP-107 (13 VPs anchored to BC-3.3.001..004, retired at BC-INDEX v6.11) Status draft→retired (ADR-037), Property text struck through; Module/Method/Priority retained verbatim (rows never deleted per POL-1 — Summary table stays row-count basis: 156 total / 122 P0 / 34 P1, satisfying validate-vp-consistency row-count symmetry). Active-basis counts introduced: 143 active VPs, 109 active P0 (Kani 23 / Proptest 58 / Unit 0 / Fuzz 5 / Integration 23), 34 active P1. Frontmatter active_vps/retired_vps fields added. New §ADR-037 Retirement section records per-VP disposition (no successor VPs registered; intent carried by BC-2.06.017/BC-3.2.005 [VP-091..094 active], BC-2.06.003/ADR-032, BC-2.21.001 + BC-2.06.012–016; BC-3.3.003 schema_version checks have no successor per ADR-037 §Consequences). No VP files exist for VP-095..107, so no withdrawal documents required. VP-INDEX v1.76→v1.77. |
| 1.76 | FB-IMPL-10 | 2026-05-18 | product-owner | F-LP-IMPL-P13-MED-001 closure: VP-156 v0.23→v0.24 (line 171 cfg-gate sibling-sweep — `#[cfg(test)]` → `#[cfg(any(test, feature = "test-helpers"))]`). VP-INDEX v1.75→v1.76. |
| 1.75 | D-717-state-mgr | 2026-05-18 | state-manager | D-717 state-manager closure: VP-156 v0.22→v0.23 (§Changelog v0.20/v0.21 monotonic ordering repair — second-order POL-26 recurrence surfaced by architect FB-IMPL-9 ZERO-DRIFT discipline; closed in same burst). VP-INDEX v1.74→v1.75. |
| 1.74 | pass-12-spec-hygiene | 2026-05-18 | architect | D-717 pass-12 INDEX cascade: VP-156 v0.21→v0.22 (F-LP-IMPL-P12-OBS-001 closure: §Test-only reset hooks line 175 — `dynamic_write_tool_count()` corrected from "`#[cfg(test)]`-gated" to "unconditional `pub fn`"). VP-INDEX v1.73→v1.74. |
| 1.73 | pass-11-spec-hygiene | 2026-05-18 | product-owner | pass-11-spec-hygiene INDEX cascade: VP-156 v0.20→v0.21 (F-LP-IMPL-P11-MED-001 closure: §Feasibility Assessment row 184 symbol corrections — `reset_for_test()` → two-function pattern `reset_query_phase_global()` + `reset_dynamic_registry_global()`; `invalidation_map()` → `dynamic_write_tool_count()`). VP-INDEX v1.72→v1.73. |
| 1.72 | pass-10-spec-hygiene | 2026-05-18 | product-owner | pass-10-spec-hygiene INDEX cascade: VP-156 v0.19→v0.20 (F-LP-IMPL-P10-OBS-002 closure: §Proof Harness Skeleton stale symbol corrections — `reset_for_test` → two-function pattern; `invalidation_map()` → `dynamic_write_tool_count()`; POL-26 row-order repair 0.19/0.18 swapped). VP-INDEX v1.71→v1.72. |
| 1.71 | pass-10-spec-hygiene | 2026-05-18 | product-owner | pass-10-spec-hygiene INDEX cascade: VP-153 v0.17→v0.18 (F-LP-IMPL-P10-IMP-001 closure: §Proof Harness Skeleton stale symbol corrections — `AuthTypeInvalid` → `AuthTypeCrossComposition`; `validate_auth_coherence` → `SpecLoader::validate_cross_composition` at 2 sites; Feasibility + Harness authoring note updated to as-built state). VP-INDEX v1.70→v1.71. |
| 1.70 | FB-IMPL-6 | 2026-05-18 | state-manager | (D-711) FB-IMPL-6 INDEX cascade: VP-156 v0.18→v0.19 status:draft→active (sibling-sweep VP-156 P1 proptest PROACTIVELY LANDED — 5 proptests across 2 binaries; tool_name-only uniqueness keying confirmed per BC-2.16.012). VP-INDEX v1.69→v1.70. |
| 1.69 | FB-IMPL-6 | 2026-05-18 | state-manager | (D-711) FB-IMPL-6 INDEX cascade: VP-153 v0.16→v0.17 status:draft→active (F-P8-IMP-001 VP-153 P0 proptest LANDED — 8 proptests across 2 crates per cross-crate dep direction; Rule C uses ShapedProbe injection per D-706 amendment; 6 in prism-spec-engine Rules A+B; 2 in prism-bin Rule C ShapedProbe). VP-INDEX v1.68→v1.69. |
| 1.68 | FB73 | 2026-05-17 | state-manager | (D-695) FB73 INDEX cascade: VP-156 v0.17→v0.18 (PO F-LP85-HIGH-001 closure: ADR-026 D7 v1.22→v1.23 sweep at 4 live-narrative sites — cross-value-class side-effect bump detection; step 8g first-application). VP-INDEX v1.67→v1.68. |
| 1.67 | FB71 | 2026-05-17 | state-manager | (D-693) FB71 INDEX cascade: VP-153 v0.14→v0.15 (PO F-LP83-HIGH-001 closure: error-taxonomy v1.35→v1.37 sweep — 11-site sweep across 4 files; META-META-META-META recursion; POL-29 v1.23→v1.24 step 8e fixed-point iteration mandate codified in-burst). VP-INDEX v1.66→v1.67. |
| 1.66 | FB69 | 2026-05-17 | state-manager | (D-691) FB69 INDEX cascade: VP-156 v0.16→v0.17 (PO F-LP81-HIGH-002 closure: ADR-026 D7 v1.21→v1.22 sweep at live-narrative sites — META-META step 8b self-induced bump gap closed via POL-29 v1.22→v1.23 step 8d). VP-INDEX v1.65→v1.66. |
| 1.65 | FB64 | 2026-05-17 | state-manager | F-LP76-HIGH-002 closure + INDEX cascade: VP-INDEX §Changelog row order repair — v1.63 row (D-684 FB62 closure VP-156 v0.14→v0.15) was prepended ABOVE v1.64 row (POL-29 v1.18 step 8b TRANSITIVE CLOSURE CATCH VP-153 v0.13→v0.14) in same FB62 dispatch; descending convention violated. Swap restores monotonic descending. POL-26 9th cascade recurrence at state-manager bookkeeping scope. Plus VP-156 v0.15→v0.16 INDEX cascade for FB64 PO bump. |
| 1.64 | FB62 | 2026-05-17 | state-manager | POL-29 v1.18 step 8b TRANSITIVE CLOSURE CATCH: VP-153 v0.13→v0.14 (error-taxonomy v1.34→v1.35 propagation at proof-harness comment lines 167 + 210 — 2 live-narrative sites). Same FB62 burst. VP-INDEX v1.63→v1.64. |
| 1.63 | FB62 | 2026-05-17 | state-manager | D-684 FB62 closure: VP-156 v0.14→v0.15 (PO cascade: ADR-026 D7 v1.19→v1.21 sweep at 4 live-narrative sites; F-LP74-HIGH-001 closure). VP-INDEX v1.62→v1.63. |
| 1.62 | FB58 | 2026-05-17 | state-manager | F-LP70-HIGH-001 closure (state-manager scope): VP-INDEX line 183 Property cell bookkeeping marker `[v0.13 FB57 POL-26 §Changelog row swap]` removed; restored verbatim VP-153 property description per FB54 v1.57 convention ("Version-tracking lives in §Changelog rows per existing convention for all 154 catalog rows"). Within-FB57 regression of F-LP66-MED-001 closed. POL-26 + POL-4 + POL-23 restoration. Sibling-sweep across BC-INDEX + ARCH-INDEX + STORY-INDEX: 0 additional sites found. VP-INDEX v1.61→v1.62. |
| 1.61 | FB57 | 2026-05-17 | state-manager | D-679 FB57 closure: VP-153 v0.12→v0.13 (POL-26-COROLLARY bookkeeping repair: §Changelog rows v0.11 + v0.12 swapped to restore ascending convention; FB56 PO row inserted above pre-existing v0.10 row and SM catch row both misordered during LARGEST-burst; F-LP69-MED-001 closure). VP-INDEX v1.60→v1.61. |
| 1.60 | FB56+FB56b+SM-step8a | 2026-05-17 | state-manager | D-678 FB56+FB56b combined + SM step 8a catch: VP-153 v0.11→v0.12 (SM step 8a catch: error-taxonomy v1.33→v1.34 propagation at VP-153 code-comment lines 167, 210 — FB56b bumped error-taxonomy to v1.34 creating new stale cites; state-manager step 8a final verification caught and closed) + VP-156 v0.13→v0.14 (FB56b architect cascade: ADR-026 D7 v1.18→v1.19 sweep at 4 live-narrative sites). VP-INDEX v1.59→v1.60. |
| 1.59 | FB56 | 2026-05-17 | state-manager | D-678 FB56: VP-153 v0.10→v0.11 (F-LP68-HIGH-001 closure PO scope: error-taxonomy v1.32→v1.33 propagation at VP-153 lines 167, 210 — 2 live-narrative backtick-quoted sites; POL-29 v1.16 class (a) recurrence #20 closed). |
| 1.58 | FB55 | 2026-05-17 | state-manager | D-677 FB55: VP-156 v0.12→v0.13 (F-LP67-HIGH-001 closure: ADR-026 D7 pin v1.17→v1.18 at 4 live-narrative sites — lines 42, 86, 90, 124; POL-29 v1.16 class (b) recurrence #18 closed). |
| 1.57 | FB54 | 2026-05-17 | state-manager | F-LP66-MED-001 closure: VP-153 (line 183) + VP-156 (line 186) catalog rows had 8 cells in 7-column table; trailing `\| v0.10 \|` and `\| v0.12 \|` cells removed to restore canonical 7-cell schema. Version-tracking lives in §Changelog rows per existing convention for all other 154 catalog rows. Latent since FB52 (same-day 2026-05-17). POL-26 schema_integrity + POL-4 semantic_anchoring_integrity restoration. |
| 1.56 | FB52 | 2026-05-17 | state-manager | VP-153 v0.9→v0.10 (F-LP64-HIGH-001 closure: error-taxonomy v1.31→v1.32 — 2 live-narrative pins updated; multi-value-class sibling-sweep for FB51 error-taxonomy value class). POL-9 same-burst propagation; POL-11 bump. |
| 1.55 | FB51 | 2026-05-17 | state-manager | VP-156 v0.11→v0.12 (F-LP63-HIGH-003 closure: §Changelog v0.10/v0.11 row positions swapped to ascending order — 7th POL-26 recurrence; bookkeeping repair; row content immutable per POL-26 corollary) |
| 1.54 | FB50 | 2026-05-17 | state-manager | VP-156 v0.10→v0.11 (POL-29 sibling-sweep for OBS-LP62-002 D7 v1.16 → v1.17 across 4 live-narrative pins; 17-site total D7 sweep Interpretation #2 per D-672) |
| 1.53 | FB45 | 2026-05-16 | state-manager | VP-156 v0.9→v0.10 (POL-23 sibling-sweep: 4 ADR-026 D7 live-narrative pins v1.15→v1.16) |
| 1.52 | FB44 | 2026-05-16 | state-manager | VP-156 v0.8→v0.9 (POL-23 sibling-sweep: 4 ADR-026 D7 live-narrative pins v1.10→v1.15) |
| 1.51 | state(D-659) | 2026-05-16 | state-manager | FB40 D-659: VP-153 row tracking v0.8→v0.9 (F-LP50-MED-002 §Changelog row ordering corrected to monotonic ascending per POL-26 — 49-pass-surviving defect). POL-9 same-burst propagation; POL-11 bump. |
| 1.50 | state(D-658) | 2026-05-16 | state-manager | FB39 D-658: VP-153 row tracking v0.7→v0.8 (F-LP49-HIGH-001 5-site error-taxonomy v1.30→v1.31 cascade closure — 13th+ POL-23 recurrence; VP-153 inline-comment cites at lines 167+210 updated by architect). POL-9 same-burst propagation; POL-11 bump. |
| 1.49 | state(D-653) | 2026-05-16 | state-manager | FB34 D-653: VP-153 row bumped v0.6→v0.7 (F-LP44-MED-002 §Proof Harness Skeleton expanded — Rules A+B proptests scaffolded). POL-9 same-burst propagation; POL-11 bump. |
| 1.48 | state(D-646) | 2026-05-16 | state-manager | FB29 D-646: VP-153 row bumped v0.5→v0.6 propagating architect's Option A byte-verbatim sync of Rule A/B/C message-format quotations to canonical error-taxonomy.md v1.30 E-SPEC-012/013/014. POL-9 same-burst propagation. |
| 1.47 | fix-burst-24-combined-D-638 | 2026-05-16 | state-manager | F-LP31-HIGH-001 — Summary table arithmetic correction: Integration test P0 25→24 (independent row count = 24 sequential P0 integration_test VPs); Total P0 123→122 (cascade from integration row). v1.32 changelog row historical narrative "P0 120→123" was incorrect (actual increment was +2: VP-153 P0 + VP-155 P0; VP-154 was P1). Survived 30 prior passes incl. 7 CLEAN until pass-31 fresh-context arithmetic re-derivation surfaced violation. VP-INDEX v1.46→v1.47. |
| 1.46 | prereq-e-fix-burst-13 | 2026-05-16 | architect | F-LP14-HIGH-001 — VP-156 sibling-sweep reflection: VP-156 advanced v0.7→v0.8 (all 4 live-narrative ADR-026 D7 v1.9 pins → v1.10; BC-2.16.012 §VPs VP-156 row pin v1.9→v1.10). VP-INDEX row for VP-156 is description-stable — no row text change required. VP-INDEX v1.45→v1.46. 5th RECURRENCE of POL-23 within-FB asymmetry CLOSED; single-bump-per-source-artifact discipline applied (ADR-026 stays at v1.10). |
| 1.45 | prereq-e-fix-burst-9 | 2026-05-16 | architect | F-LP10-HIGH-001 — POL-21 phantom-anchor closure: VP-155 §Property Statement + §Source Contract `ADR-023 §VP-PLUGIN-001` → `ADR-023 §Verification Properties (VP-PLUGIN-001 bullet)` (VP-155 v0.4→v0.5). VP-INDEX row for VP-155 is description-stable — no row text change required (module/method/priority/status unchanged). VP-INDEX v1.44→v1.45. |
| 1.44 | prereq-e-fix-burst-8 | 2026-05-16 | architect | F-LP8-HIGH-001 final close: VP-156 ADR-026 D7 version pins advanced v1.8→v1.9 across 4 live-narrative sites (§Property Statement, §Source Contract BC row, §Source Contract ADR row, proof harness skeleton comment) (v0.6→v0.7). VP-INDEX row for VP-156 is description-stable — no row text change required. VP-INDEX v1.43→v1.44. |
| 1.43 | prereq-e-fix-burst-7 | 2026-05-16 | architect | F-LP7-HIGH-001 sibling sweep: VP-156 ADR-026 D7 version pins advanced v1.7→v1.8 across 4 live-narrative sites (§Property Statement, §Source Contract BC row, §Source Contract ADR row, proof harness skeleton comment) (v0.5→v0.6). VP-INDEX row for VP-156 is description-stable — no row text change required. VP-INDEX v1.42→v1.43. |
| 1.42 | prereq-e-fix-burst-6 | 2026-05-16 | architect | F-LP6-HIGH-001 + F-LP6-MED-001 + F-LP6-LOW-002 sibling sweep: VP-155 source_bc set to BC-2.16.011 (v0.3→v0.4); VP-156 ADR-026 D7 version pins updated from stale v1.2 to v1.7 across §Property Statement, §Source Contract, and proof harness skeleton (v0.4→v0.5). VP-INDEX rows for VP-155 and VP-156 are description-stable — no row text changes required (module/method/priority/status unchanged). VP-INDEX v1.41→v1.42. |
| 1.41 | prereq-e-fix-burst-5 | 2026-05-15 | architect | F-LP5-LOW-001 Path A: `source_invariant:` schema convention documented — field accepts DI-NNN workspace-canonical identifiers only; BC-local INV-* identifiers belong in `source_bc:` + body prose, not `source_invariant:`. Convention note added as blockquote above §Properties table. Confirms VP-153 (`source_invariant: DI-012`) and VP-156 (`source_invariant: null`) are both correct; no individual VP frontmatter changes needed. VP-INDEX v1.40→v1.41. |
| 1.40 | prereq-e-fix-burst-3 | 2026-05-15 | architect | F-LP3-HIGH-001 sibling-sweep: VP-156 description updated from "uniqueness + happens-before" to "uniqueness only" — aligns VP-INDEX row with VP-156 v0.2 body (happens-before claim removed in fix-burst-2). Propagated to verification-architecture.md (v1.34), BC-2.16.012 §VP Anchors and §Verification Properties (v1.4). Stale "ADR-026 D7 v1.2" pin in BC-2.16.012 §Verification Properties updated to v1.5. |
| 1.39 | prereq-e-fix-burst-1 | 2026-05-15 | architect | F-LP1-MED-003 resolution: VP-156 (WriteToolInvalidationMap registration uniqueness + happens-before, proptest P1, module: prism-query, anchor: S-PLUGIN-PREREQ-E) added to main Properties table. BC-2.16.012 §VP Anchors "(none in this story)" coverage gap closed. Summary table: Proptest 87→88, Total VPs 155→156, P1 33→34. Sequential count note updated to 156. total_vps frontmatter updated 155→156. |
| 1.38 | PREREQ-E-ADR-burst | 2026-05-15 | state-manager | Frontmatter version bumped 1.37→1.38 to record PREREQ-E ADR burst: VP-153 (proptest P0), VP-154 (integration_test P1), VP-155 (integration_test P0) added. Changelog row was missing from v1.38 — backfilled here at v1.39 to maintain contiguous history. Corresponding content changes were applied in that burst (main table rows 175-177, summary table 152→155 total, P0 120→123, P1 32→33, named-alias note updated). |
| 1.37 | fix-burst-36 | 2026-05-14 | state-manager | (D-539) F-LP38-MED-001 closure: v1.36 + v1.35 changelog rows rewritten to canonical 5-col schema (Burst column restored; D-NNN folded into Change cell). 2nd cascade recurrence of §Changelog schema-corruption META-class (1st: F-LP34-HIGH-001 story rows). Root cause: orchestrator dispatch prompt templates prescribed incorrect row format. OBS-LP38-001 POL-26 codification candidate (§Changelog schema-integrity validator) routed cycle-close. |
| 1.36 | fix-burst-35 | 2026-05-14 | state-manager | (D-538) F-LP37-MED-001 closure: VP-PLUGIN-007 row line 190 description rewritten from "rejected at load time per AC-7 default-deny" to "rejected at load time per AC-5 manifest gate (default-deny consumer is AC-7)" — 4th-cascade sibling-document propagation sweep (bursts 32→33→34→37); restores canonical AC-5 manifest gate anchor matching BC-2.17.007:138/161 (post fix-burst-34). OBS-LP37-001 strengthens POL-25 candidate to HIGH-priority cycle-close codification. |
| 1.35 | fix-burst-32 | 2026-05-14 | state-manager | (D-533) F-LP34-LOW-001 closure: VP-152 + VP-PLUGIN-007 descriptions rewritten from pre-AC-7 "not-None" Option-semantics to post-AC-7 "explicit Vec<String> under default-deny" semantic; reflects AC-7 + AC-17 type-system contract change (Option<Vec<String>> → Vec<String>). Cross-document propagation: story §References:1034 mirror updated same-burst per POL-9. |
| 1.34 | F-LP2-LOW-006-fix | 2026-05-13 | architect | F-LP2-LOW-006 closure: removed stale trailing annotation "— VP-150 number" from VP-PLUGIN-005 named-alias description row. Sibling rows VP-PLUGIN-001..004/006/007 carry no trailing annotation; VP-PLUGIN-005 now matches the established convention. No semantic change — description content is identical to v1.33 corrected text. |
| 1.33 | F-LP1-CRITICAL-001-fix | 2026-05-13 | architect | F-LP1-CRITICAL-001 closure: corrected 4 mis-anchored VP-PLUGIN-NNN named-alias rows per ADR-023 §E canonical definitions. VP-PLUGIN-001 (VP-146): was "SensorId open-newtype replaces SensorType" → now "No production hardcoded sensor references (FORBIDDEN-SYMBOLS-001 compile-fail perimeter)". VP-PLUGIN-004 (VP-149): was "TOML grammar accepts four new constructs" → now "Boot warning fires on unsigned plugin load (v1.0 scope)". VP-PLUGIN-006 (VP-151): was "Cross-sensor auth-composition rejection — DI-012 rules" → now "OCSF column mapping fixture catalog (6 representative cases, SpecDrivenMapper)". VP-PLUGIN-007 (VP-152): was "Zero hardcoded CustomAdapter Rust adapters" → now "Plugin manifest allowlist not-None after PREREQ-D". VP-PLUGIN-002/003/005 verified correct. Added total_vps: 152 to frontmatter for hook anchor. Sequential VP-146..VP-152 rows and verification-architecture.md/verification-coverage-matrix.md already carried correct descriptions — named-alias table was the sole drift location. |
| 1.32 | prereq-b-fix-burst-6 | 2026-05-11 | state-manager | D-410 F-LP6-HIGH-001+HIGH-002 closure: VP-PLUGIN-002 numbered row (VP-INDEX:168) anchor corrected PLUGIN-MIGRATION-001-D→S-PLUGIN-PREREQ-B and description corrected to "PipelineExecutor::execute returns non-empty records against wiremock DTU clone"; VP-PLUGIN-005 numbered row (line 171) anchor corrected to S-PLUGIN-PREREQ-B; VP-PLUGIN-005 named-alias row (line 187) description rewritten to OAuth2 refresh-on-401 + anchor corrected to S-PLUGIN-PREREQ-B; internal contradiction between lines 171 and 187 eliminated (commits 1474a682 + 99a6b07a). |
| 1.31 | prereq-b-pass-6-backfill | 2026-05-11 | state-manager | Version bump acknowledging VP-PLUGIN-002/005 content was updated by product-owner commits 1474a682+99a6b07a (D-409/D-410 burst); frontmatter version alignment. |
| 1.30 | prereq-f | 2026-05-11 | product-owner | PREREQ-F: Registered VP-PLUGIN-001..007 named series per ADR-023 L511-512 + F-PASS3-HIGH-001. Added VP-PLUGIN-NNN named table (aliases to VP-146..VP-152) with module: prism-spec-engine. Summary note clarifies named series does not increment sequential count. |
| 1.29 | pr-127-pass4-remediation | 2026-05-05 | architect | Property-text correction for VP-014 (v1.5→v1.6) and VP-015 (v1.6→v1.7): replaces non-existent `ParseError::QueryTooLarge` and `ParseError::NestingTooDeep` enum-variant references with accurate `Err(Vec<ParseError>)` API description (message contains `E-QUERY-003`). Identified by adversary pass-4 (F-MEDIUM-001). Status, verification_lock, proof_file_hash, proof_completed_date unchanged — only property statement text corrected. Cross-ref: vp-014 v1.6, vp-015 v1.7, verification-architecture.md v1.30, ARCH-INDEX v2.31. |
| 1.28 | pr-127-formal-verify | 2026-05-05 | architect | VP-014 and VP-015 promoted to `verified` following successful Kani proof runs at commit f5212641 (PR #127). VP-014: harness `proof_check_query_size_rejects_oversize`, 0/4371 failed (285 unreachable). VP-015: 4 harnesses including new `proof_sql_query_depth_limit`, 0/5664 failed (397 unreachable). Status column updated; lock=true; proof_completed_date=2026-05-05. Cross-ref: vp-014 v1.5, vp-015 v1.6, verification-architecture.md v1.29, ARCH-INDEX.md v2.30. |
| 1.27 | pr-127-review-remediation | 2026-05-05 | product-owner | PR-127 Task 2: VP-021 v1.3→v1.4 — clarified Property Statement to explicitly state that `PrismQlParser::parse` takes `&str` not `&[u8]`; the from_utf8 conversion step is now spelled out inline. Removed "malformed unicode" from inputs list (from_utf8 filters that before parse receives input). Harness skeleton was already correct; property statement is now unambiguous on type signature. |
| 1.26 | W4-Phase4A-Pass20-fix | 2026-05-03 | state-manager | F-P20-H-001: VP-045 description updated "Schedule semaphore" → "Action delivery semaphore: try_acquire used (non-blocking), never acquire" (Pass-6 BC-H1 BC-2.18.004 rename to "Action Delivery Semaphore" failed to propagate to VP catalog ecosystem). |
| 1.25 | W4-Phase4A-Pass5-fix | 2026-05-03 | state-manager | P5-S407-A-M-005: VP-145 anchor column expanded S-4.06 → S-4.06, S-4.07 (reopen_count monotonic increment invariant is exercised by both the case-management story S-4.06 and the case-query story S-4.07 per adjudication). |
| 1.23 | W4-Phase4A-Pass3-fix | 2026-05-02 | state-manager | P3-VPINDEX-A-M-004: VP-138 story anchor narrowed S-4.06, S-4.07 → S-4.06 only (cross-org case isolation is a S-4.06 CaseStore boundary invariant per ADR-017 §3.5/§8; S-4.07 consumes via read path, does not implement enforcement). |
| 1.19 | pass-22-remediation (backfill) | 2026-04-27 | product-owner | m-22-001: changelog row backfilled to record the v1.18→v1.19 transition. The actual content change (VP-001 description TenantId → OrgSlug, line 22) was applied by the pass-14/15 OrgSlug sweep but never received a dedicated v1.19 changelog row at that time. |
| 1.18 | pass-11-remediation | 2026-04-27 | product-owner | m-001: v1.15 retrospective annotation corrected — "26" → "28" additional mismatches found by Pass-9 audit (was already corrected in v1.17 body of v1.16 row, now also reflected in the v1.15 NOTE text for historical accuracy). |
| 1.17 | pass-10-remediation | 2026-04-27 | product-owner | M-001: Corrected changelog miscount — Total: 26 → 28 VP anchor corrections in v1.16. Enumeration: C-001=14 (VP-108..121) + C-002=3 (VP-066..068) + C-003=4 (VP-073..076) + C-004=3 (VP-081..083) + C-005=4 (VP-087..090) = 28. "All other 48 Wave 3 VPs verified correct" updated to "46 verified correct" (28+46=74 total Wave 3 VPs, consistent with VP-INDEX 1.12 wave-3-registration). |
| 1.16 | pass-9-remediation | 2026-04-27 | product-owner | C-001 (M-001): VP-108..VP-121 (14 VPs) re-anchored from E-3.4 test migration stories to E-3.7 data generator stories — BC-3.4.001/002/003/004 Story Anchor fields all cite S-3.7.xx. VP-108..111 S-3.4.01→S-3.7.01; VP-112..114 S-3.4.02→S-3.7.00; VP-115..118 S-3.4.03→S-3.7.01; VP-119..121 S-3.4.04→S-3.7.02. C-002 (M-002): VP-066/067/068 re-anchored from S-3.1.02 (TenantId rename story) to S-3.1.07 — BC-3.1.002 Story Anchor = S-3.1.07 (prism-audit: add org_id + org_slug). C-003 (new): VP-073/074/075/076 re-anchored from S-3.1.04 (credential namespace migration) to S-3.1.03 — BC-3.1.004 Story Anchor = S-3.1.03/S-3.3.02; S-3.1.04 is a different BC (BC-3.2.002). C-004 (new): VP-081/082/083 re-anchored from S-3.2.02 (prism-dtu-armis state segregation) to S-3.1.04 — BC-3.2.002 Story Anchor = S-3.1.04 (prism-credentials credential namespace key migration). C-005 (new): VP-087/088/089/090 re-anchored from S-3.2.04 (prism-dtu-cyberint state segregation, BC-3.2.001/003 territory) to S-3.2.05 — BC-3.2.004 Story Anchor = S-3.1.06/S-3.2.05/06/07/S-3.4.05; S-3.2.04 is not in that list. Total: 28 VP anchor corrections (corrected from "26" in original row — see v1.17). All other 46 Wave 3 VPs (VP-063..065, VP-069..072, VP-077..080, VP-084..086, VP-091..094, VP-095..107, VP-122..136) verified correct. |
| 1.15 | pass-6-remediation | 2026-04-27 | product-owner | C-001: Corrected story anchors for 9 Wave 3 VPs: VP-122/123/124 S-3.5.01→S-3.3.03 (BC-3.5.001 harness logical isolation); VP-125/126/127 S-3.5.02→S-3.3.04 (BC-3.5.002 harness network isolation; S-3.5.02 does not exist); VP-134/135/136 S-3.7.01→S-3.5.01 (BC-3.7.001 src/ convention sweep; S-3.7.01 is data generator). NOTE: Pass-6 claim "no other mismatches" was incorrect — Pass-9 audit found 28 additional mismatches in VP-066..121 (corrected from initial "26" miscount in v1.16 — see v1.17). |
| 1.14 | pass-4-remediation | 2026-04-27 | product-owner | M-001: Summary table Proptest P0/P1 corrected 73/4 → 64/13 (recount from per-row priority fields). Total row corrected 122/14 → 113/23. Verification: Kani 23P0+7P1=30; Proptest 64P0+13P1=77; Unit 4P0=4; Fuzz 5P0+1P1=6; Integ 17P0+2P1=19; sum 113P0+23P1=136. |
| 1.13 | pass-2-adversary | 2026-04-27 | product-owner | M-005: VP-084 anchor story S-3.2.03 → S-3.2.08 (cross-org token validation more specifically implemented in CrowdStrike session ID OrgId scoping story); VP-094 anchor story S-3.2.05 → S-3.3.06 (reload_config mode-change prevention is exactly S-3.3.06's purpose). |
| 1.12 | wave-3-registration | 2026-04-27 | product-owner | Wave 3 VP registration: VP-063..VP-136 (74 new VPs). Kani 26→30; Proptest 28→81; Integration 2→19; Total 62→136. Summary table updated. |
| 1.11 | pass-90-F90-004 | 2026-04-21 | architect | F90-004: VP-052 and VP-054 module canonicalized prism-core → prism-operations (matches S-4.06 story evidence). |
| 1.10 | pass-87-remediation | 2026-04-21 | architect | F87-002: VP-025 anchor_story S-3.04 → S-3.05 (cache_key lives in S-3.05; S-3.04 alias semantic is the bug). F87-004: VP-055/057/058 module prism-persistence → prism-storage (canonical module name sweep). |
| 1.9 | pass-86-remediation | 2026-04-21 | architect | Prior version (no changelog row recorded at time of edit). |
