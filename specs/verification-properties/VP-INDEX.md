---
document_type: verification-property-index
level: L4
version: "1.29"
status: draft
producer: product-owner
timestamp: 2026-05-05T00:00:00
phase: 2-patch
inputs: [architecture/verification-architecture.md]
traces_to: architecture/ARCH-INDEX.md
---

# Verification Property Index: Prism

> **Context Engineering:** This index lists all verification properties with their
> status and method. Load individual VP files only when working on that specific property.

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
| VP-095 | [BC-3.3.001] Every ST type in DTU_DEFAULT_MODE triggers startup error paired with mode=shared | prism-spec-engine | unit_test | P0 | draft | S-3.3.01 |
| VP-096 | [BC-3.3.001] No MSSP Coordination type triggers startup error paired with mode=client | prism-spec-engine | unit_test | P0 | draft | S-3.3.01 |
| VP-097 | [BC-3.3.001] Startup error message contains DTU type string and config file path | prism-spec-engine | unit_test | P0 | draft | S-3.3.01 |
| VP-098 | [BC-3.3.001] Multi-error: N violations produce N errors in one pass before abort | prism-spec-engine | unit_test | P0 | draft | S-3.3.01 |
| VP-099 | [BC-3.3.002] Non-scheme credential-pattern field value always causes exit code 1 | prism-spec-engine | proptest | P0 | draft | S-3.3.01 |
| VP-100 | [BC-3.3.002] E-CFG-020 error message never contains the literal field value | prism-spec-engine | proptest | P0 | draft | S-3.3.01 |
| VP-101 | [BC-3.3.002] All four allowed scheme prefixes accepted for credential-pattern fields | prism-spec-engine | proptest | P0 | draft | S-3.3.01 |
| VP-102 | [BC-3.3.003] All integer schema_version values != 1 produce exit code 1 | prism-spec-engine | proptest | P0 | draft | S-3.3.01 |
| VP-103 | [BC-3.3.003] Absent schema_version produces E-CFG-030, not E-CFG-031 | prism-spec-engine | proptest | P0 | draft | S-3.3.01 |
| VP-104 | [BC-3.3.003] schema_version=1 never produces schema-version error regardless of other fields | prism-spec-engine | proptest | P0 | draft | S-3.3.01 |
| VP-105 | [BC-3.3.004] Exit code 0 implies OrgRegistry entry count equals file count | prism-spec-engine | proptest | P0 | draft | S-3.3.02 |
| VP-106 | [BC-3.3.004] Any validation error implies exit code 1 and empty OrgRegistry | prism-spec-engine | proptest | P0 | draft | S-3.3.02 |
| VP-107 | [BC-3.3.004] Validation error output always includes the offending filename | prism-spec-engine | integration_test | P0 | draft | S-3.3.02 |
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
| VP-129 | [BC-3.6.001] All FailureMode variants produce the documented HTTP status code or behavior | prism-dtu-harness | integration_test | P0 | draft | S-3.6.01 |
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

## Summary

| Method | Count | P0 | P1 |
|--------|-------|----|----|
| Kani | 30 | 23 | 7 |
| Proptest | 86 | 65 | 21 |
| Unit test | 4 | 4 | 0 |
| Fuzz | 6 | 5 | 1 |
| Integration test | 19 | 17 | 2 |
| **Total** | **145** | **114** | **31** |

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
