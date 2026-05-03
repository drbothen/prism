---
document_type: adr
adr_id: "ADR-018"
title: "Differential Result Pack Format"
status: PROPOSED
version: "0.5"
date: 2026-05-03
wave: 4
phase: 4.A
authors: [architect]
producer: architect
timestamp: 2026-05-02T00:00:00Z
inputs:
  - .factory/cycles/wave-4-operations/cycle-manifest.md
  - .factory/cycles/wave-4-operations/preflight-findings/research-findings.md
  - .factory/STATE.md
  - .factory/stories/S-4.02-diff-results-packs.md
  - .factory/stories/S-4.01-schedule-crud.md
  - .factory/specs/architecture/decisions/ADR-006-multi-tenant-dtu-topology.md
  - .factory/specs/architecture/decisions/ADR-008-dtu-state-segregation.md
  - .factory/specs/architecture/decisions/ADR-010-customer-config-schema.md
  - .factory/specs/architecture/decisions/ADR-013-schedule-execution-semantics.md
anchor_stories: [S-4.02, S-4.01]
aligns_with: [ADR-006, ADR-008, ADR-010, ADR-013]
references_research: [R-4, R-9]
verification_properties: [VP-019, VP-141, VP-142]
supersedes: []
superseded_by: null
amendments: []
related_decisions:
  - D-207
  - D-208
  - D-210
  - D-213
subsystems_affected: [SS-12, SS-13]
traces_to: specs/architecture/ARCH-INDEX.md
---

# ADR-018: Differential Result Pack Format

## Status

PROPOSED 2026-05-03, v0.4. Pending review and acceptance prior to S-4.02 story remediation and BC authoring.

---

## 1. Context

### 1.1 Why Differential Results

The schedule executor (ADR-013) fires queries against sensor APIs on behalf of org-scoped schedules. Each execution produces a result set — rows of sensor data (device inventory, process list, event records). A naïve implementation stores or emits the full result set on every fire. For slow-changing data (e.g., a Claroty device inventory polled every five minutes), the full result set is overwhelmingly redundant: 99% of rows are identical to the prior execution.

Differential result computation addresses this by producing only the *added* and *removed* rows between consecutive executions. This reduces downstream noise, enables alert-rule evaluation against state-change events (S-4.03), and allows operators to reason about what changed rather than what exists.

### 1.2 Why Packs

Individual schedules (S-4.01) are flexible but require per-schedule configuration by operators. For repeatable, validated query sets — e.g., a CrowdStrike baseline that always polls processes, network connections, and installed software together — per-schedule configuration is operationally burdensome and error-prone.

Packs are pre-composed, versioned collections of schedules expressed in a single `.pack.toml` file. A pack is a unit of deployment: one registration produces N `ScheduleEntry` records in the `schedules` CF. Packs are capability-gated (S-1.08) so orgs without the required capability flag cannot accidentally activate packs they are not licensed for.

### 1.3 Relationship to ADR-013

ADR-013 defines the schedule store (`schedules` CF), the splay strategy, the tick loop, and the schedule-change watch channel. ADR-018 defines:

- How packs expand into `ScheduleEntry` records at load time (§2.3 / §2.4)
- How pack-derived schedules are differentiated from individually-defined schedules
- The `diff_results` CF design that stores per-schedule row hash sets and diff output
- The blake3 + canonical-JSON row hashing pipeline
- The RocksDB merge-operator epoch counter for fire-count tracking

This ADR does NOT redefine the tick loop, splay, or semaphore semantics — those are owned by ADR-013.

### 1.4 Scope of This ADR

- Row hashing canonicalization (hash function, canonical JSON crate, output usage)
- Epoch counter atomic semantics (RocksDB merge_operator pattern)
- Pack TOML schema and field semantics
- Pack expansion timing: load-time, not runtime
- Capability-gated pack execution (S-1.08 integration)
- `diff_results` CF design (key format, storage, TTL)
- Pack-vs-individual-schedule name collision policy

---

## 2. Decision

### 2.1 Row Hashing Canonicalization

**Hash function:** `blake3 = "1.8"` (per R-4: no CVEs, no RustSec advisories, SIMD-accelerated on x86/AArch64, 32-byte output, workspace standard). Use `blake3::hash` (the standard non-keyed mode). Do NOT use `blake3::Hasher::new_keyed` or `new_derive_key` for this purpose — those modes are for MAC/KDF use cases, not content-addressable identity.

**Input format:** Canonical JSON of the row, NOT raw serialized bytes. Raw bytes vary with serialization order, struct field ordering, padding choices, and encoding version bumps. Canonical JSON (JCS, RFC 8785) is content-addressable: two rows with identical field values produce identical canonical JSON regardless of the Rust struct layout that generated them.

**Canonical JSON crate:** Pin `serde_jcs = "0.1"` (JCS RFC 8785 implementation). This is the authoritative choice per R-4 research. The alternative `canonical-json` crate is rejected — it is less production-tested and does not fully implement RFC 8785 key-ordering requirements.

Story-writer must pin `serde_jcs = "0.1"` in the S-4.02 Library table (drift item DRIFT-402-006).

**Hash output usage:** The 32-byte blake3 hash output is used directly (as `[u8; 32]`) as the set membership key in a `HashSet<[u8; 32]>`. Set difference (added = current - previous; removed = previous - current) determines which rows changed between consecutive executions. Hash values are NOT hex-encoded in storage — raw bytes are stored for space efficiency. Hex encoding is only used in diagnostic/audit log output.

**Collision policy:** blake3 collision probability for realistic result set sizes (up to 1M rows per execution) is astronomically small (~2^-224 for a 1M-row set). No explicit collision handling is specified; if two distinct rows hash identically, the added/removed computation will treat them as equal (a missed change). This is an accepted engineering tradeoff documented here.

### 2.2 Epoch Counter Atomic Semantics

The `diff_results` CF tracks how many times each schedule has fired, for observability and pack-level fire-rate monitoring. Concurrent workers incrementing the same counter must not lose increments.

**Pattern:** RocksDB merge_operator (per R-9: `rocksdb = "0.24.0"`). Register via `Options::set_merge_operator_associative`:

```rust
fn epoch_merge(
    _key: &[u8],
    existing: Option<&[u8]>,
    operands: &MergeOperands,
) -> Option<Vec<u8>> {
    let mut total: u64 = existing
        .and_then(|b| b.try_into().ok())
        .map(u64::from_le_bytes)
        .unwrap_or(0);
    for op in operands.iter() {
        if let Ok(arr) = op.try_into() {
            total = total.saturating_add(u64::from_le_bytes(arr));
        }
    }
    Some(total.to_le_bytes().to_vec())
}
```

**Apply:** `db.merge(epoch_key, &(1u64).to_le_bytes())` on each schedule fire. Read via `db.get(epoch_key)`.

**Why merge, not put:** `put` overwrites; concurrent workers using `put` would produce lost-update races. The merge operator is associative and commutative: concurrent `merge` calls are composed correctly by RocksDB on the read path or at compaction, regardless of the order in which they arrive.

**Overflow policy:** `saturating_add` is used rather than wrapping or checked arithmetic. At u64::MAX (~1.8×10^19), the counter saturates and stops incrementing rather than wrapping to zero. This is the correct policy: a saturated counter is an observable anomaly (signals unbounded fire rate); a wrapped counter produces a false "small count" that would mislead monitoring. In practice, u64 saturation is unreachable in normal MSSP operation.

**Caveat (from R-9):** Merge operators are only invoked on `merge()` calls. A `put()` to the same key overwrites the value without invoking the merge operator. Story-writer must ensure no code path calls `put()` on epoch keys; only `merge()` and `get()` are permitted for epoch counters. This must be enforced by a module-visibility constraint in `diff/epoch.rs`.

**Epoch key format:** `{org_id}:epoch:{schedule_id}` stored in the `diff_results` CF alongside the hash sets.

### 2.3 Pack TOML Schema

Packs are declared in `.pack.toml` files, analogous to `.sensor.toml` sensor spec files (ADR-010). The schema:

```toml
[pack]
name = "windows-baseline-pack"
description = "Baseline Windows host telemetry: processes, connections, installed software"
version = "1.0"
required_capability = "FEATURE_WINDOWS_BASELINE"

[[schedules]]
name = "windows-process-list"
description = "Enumerate running processes every 5 minutes"
sql_query = "SELECT * FROM events WHERE source = 'windows.processes'"
interval_seconds = 300

[[schedules]]
name = "windows-network-connections"
description = "Enumerate active network connections every 5 minutes"
sql_query = "SELECT * FROM events WHERE source = 'windows.network'"
interval_seconds = 300
```

**Field semantics:**

- `pack.name`: the file-supplied identifier matching `^[a-z0-9-]+$` (same rules as OrgSlug per ADR-006 §2.1). `PackId` is derived as the `(org_id, pack.name)` tuple — pack names are unique **within an org**, not across orgs. Two different MSSP customers can both register a vendor-supplied pack with the same `pack.name` because their `org_id` differentiates them. See §2.7 for collision semantics.
- `pack.version`: semver-compatible string; stored in each derived `ScheduleEntry.pack_version` for traceability.
- `pack.required_capability`: capability flag name (S-1.08). If omitted, the pack is unconditionally enabled for all orgs.
- `schedules[*].name`: schedule name within the pack; must be unique within the pack. Qualified name for the derived `ScheduleEntry` is `{pack_name}:{schedule_name}`.
- `schedules[*].sql_query`: PrismQL query string. Validated at pack-load time against the PrismQL parser (S-3.01).
- `schedules[*].interval_seconds`: integer. The derived `ScheduleEntry.cron_expr` is generated as a **5-field** cron: `*/{interval_minutes} * * * *` where `interval_minutes = interval_seconds / 60` (integer division). **Minimum `interval_seconds` is 60** — packs specifying `interval_seconds < 60` are rejected at registration with `E-PACK-INTERVAL-TOO-SHORT` (minimum tick rate is 60s per ADR-013 §2.1). The 6-field `*/{interval_seconds} * * * * *` form is NOT used — action-delivery (ADR-016) accepts only 5-field cron expressions via `croner = "3"` (ADR-013 §2.8), and 6-field crons would be rejected by the schedule executor's cron parser.

**Pack-derived `ScheduleEntry` additions:**

Each schedule expanded from a pack carries two additional fields beyond a standard `ScheduleEntry`:

```rust
pub struct ScheduleEntry {
    // ... standard fields per ADR-013 §2.6 ...
    pub pack_origin: Option<PackId>,   // None for individually-defined schedules
    pub pack_version: Option<String>,  // None for individually-defined schedules
}
```

### 2.4 Pack Expansion Timing: Load-Time

Pack expansion happens at **load time** (pack registration), NOT at runtime (not on each tick).

At pack registration (`register_pack` tool invocation, S-4.02 Task 7):

1. Parse `.pack.toml` against the schema (§2.3).
2. Evaluate capability gate: call `prism-security::feature_flags::is_enabled(required_capability, org_id)`. If disabled, mark all derived schedules `enabled: false` (see §2.5).
3. For each `[[schedules]]` entry: construct a `ScheduleEntry` with `pack_origin: Some(pack_id)`, compute splay per ADR-013 §2.2, and persist to the `schedules` CF.
4. Emit a `PackRegistered` audit event with: `pack_id`, `org_id`, `schedule_count`, `enabled`.

**Rationale for load-time expansion:** Runtime expansion (expanding the `[[schedules]]` array on every tick) would add O(N schedules per pack) TOML parsing cost to every tick cycle. Load-time expansion is the correct pattern: packs change rarely (admin action); tick cycles are frequent (every 60 seconds per ADR-013 §2.1). This directly follows ADR-010's config-driven principle: structured artifacts are parsed once at load, not on every use.

**Idempotence:** Registering the same pack twice (same `pack_id`, same `org_id`) must produce an identical `ScheduleEntry` set. The second registration is a no-op if the pack version has not changed; if the version differs, it is treated as an update (existing derived schedules are replaced). VP-142 formalizes this idempotence property.

### 2.5 Capability-Gated Execution

Pack execution is conditional on the S-1.08 feature flag system, evaluated at load time.

**Load-time gate:** At pack registration, query `prism-security::feature_flags::is_enabled(required_capability, org_id)`. If the flag is disabled for the calling org:

- Pack is registered in the `packs` metadata CF (for admin visibility).
- All derived `ScheduleEntry` records are persisted with `enabled: false`.
- Schedule executor (ADR-013 §2.5 fire-eligibility check) skips schedules where `enabled == false`.

**Flag toggle propagation:** When an operator enables or disables a capability flag for an org:

1. A capability-flag-changed event is emitted.
2. The pack manager receives the event and re-evaluates all packs for the org that reference the changed capability.
3. For newly-enabled packs: update derived `ScheduleEntry` records to `enabled: true`; emit `ScheduleChangeNotification::Updated(org_id, schedule_id)` for each via the ADR-013 §2.7 watch channel.
4. For newly-disabled packs: update to `enabled: false`; emit `ScheduleChangeNotification::Updated(org_id, schedule_id)`.

**Rationale for load-time gating (not per-fire):** Capability flag checks on every tick fire would add a cross-module RPC-equivalent call (or a shared-state read) on every schedule evaluation. Capability flag changes are low-frequency admin events; per-fire checking adds cost for a condition that changes rarely. Load-time gating with event-driven re-evaluation gives the same correctness guarantee with negligible steady-state overhead.

### §2.5.1 Capability-Flag-Changed Event Source

Capability-flag-changed events are emitted by `prism-security` (S-1.08 owner). Emission is via `tokio::sync::watch::channel<FeatureFlagSnapshot>`, exposed at:

```rust
prism_security::feature_flags::flag_change_watcher() -> tokio::sync::watch::Receiver<FeatureFlagSnapshot>
```

The pack manager (S-4.02) subscribes to this channel at startup. On receipt of a `FeatureFlagSnapshot`, the pack manager re-evaluates all packs for the affected org whose `required_capability` matches any changed flag. Pack manager is the only consumer of this channel for the purpose of toggling `ScheduleEntry.enabled`; other subsystems that need flag state call `is_enabled()` directly.

### 2.6 `diff_results` Column Family Design

CF name: `diff_results`. Key prefix: `{org_id}:diff:` per ADR-008 universal re-keying rule (referencing ADR-008 §2.2). Full key structure:

| Key | Value Type | Description |
|-----|-----------|-------------|
| `{org_id}:diff:{schedule_id}:prev` | `Vec<[u8; 32]>` (sorted) | Row hash set from the N-1 execution |
| `{org_id}:diff:{schedule_id}:cur`  | `Vec<[u8; 32]>` (sorted) | Row hash set from the N execution |
| `{org_id}:diff:{schedule_id}:added` | `Vec<RawRow>` (bincode) | Rows present in cur but not in prev |
| `{org_id}:diff:{schedule_id}:removed` | `Vec<RawRow>` (bincode) | Rows present in prev but not in cur |
| `{org_id}:epoch:{schedule_id}` | `[u8; 8]` (u64 le) | Fire count via merge_operator (§2.2) |

**Value encoding:** bincode 2.x with serde feature (workspace standard per ADR-008). Hash vectors are stored as sorted `Vec<[u8; 32]>` to enable binary search membership queries without a `HashSet` deserialization round-trip.

**Version-prefixed value format for `:cur` and `:prev`:** To support future hash-size changes, the `:cur` and `:prev` values include a 1-byte version prefix: `[version_byte: 0x01][bincode-encoded Vec<[u8; 32]>]`. A future change to a different hash size bumps the version byte to `0x02` (etc.), allowing the reader to dispatch on version without a format migration. All other keys (`:added`, `:removed`, epoch) do not use version prefixing.

**Compression:** Configured via the rocksdb 0.24 API call sequence:
```rust
Options::set_compression_type(DBCompressionType::Zstd)
Options::set_compression_options(/*window_bits=*/-14, /*level=*/3, /*strategy=*/0, /*max_dict_bytes=*/0)
```
Hash sets and row vectors compress well under zstd due to structural repetition.

**Memory cap:** The 200MB figure in story drafts applies to the RocksDB block cache for this CF, not total disk usage (disk is unbounded by the block cache setting). Block cache is configured via `BlockBasedOptions::set_block_cache`. Disk usage is managed via:

- **Rolling retention:** Only the most recent two execution snapshots (`:prev` and `:cur`) are retained per schedule. On each fire, the old `:prev` is deleted, `:cur` is moved to `:prev`, and new results populate `:cur`. The `:added` and `:removed` keys are overwritten on each fire.
- **Age-based eviction:** Schedules that have not fired in 30 days have their `diff_results` entries evicted via a background compaction filter. The threshold is configurable via `PRISM_DIFF_EVICTION_DAYS`.

**Diff computation algorithm:**

```
prev_set = HashSet::from(load(prev_key))
cur_set  = HashSet::from(load(cur_key))
added    = cur_set.difference(&prev_set).collect()
removed  = prev_set.difference(&cur_set).collect()
```

This is the pure-core operation for VP-019 (diff computation deterministic).

### 2.7 Pack-vs-Individual-Schedule Name Collision Policy

If a pack-derived schedule qualified name `{pack_name}:{schedule_name}` collides with an individually-defined schedule name, or if two packs produce the same qualified schedule name for the same org:

- Pack-derived schedules WIN over individually-defined schedules with the same name.
- Attempting to register an individually-defined schedule with a name that matches an existing pack-derived schedule returns `Err(ScheduleNamePackCollision { pack_id, schedule_name })`.
- Error code: `E-SCHEDULE-NAME-PACK-COLLISION`.
- Two packs with the same `(org_id, pack.name)` tuple (same `PackId`) use the following collision semantics:
  - Same `(org_id, pack.name)` + same `version` → no-op (success; second registration is idempotent).
  - Same `(org_id, pack.name)` + DIFFERENT `version` (higher) → update (replace existing derived schedules).
  - Same `(org_id, pack.name)` + DIFFERENT `version` (lower, regression) → `Err(PackVersionRegression)`.
  - Different org (`org_id_A` vs `org_id_B`) with the same `pack.name` → allowed; pack names are unique WITHIN an org only. This is the normal case for vendor-supplied packs deployed to multiple tenants.

**Rationale:** Packs are system-managed, vetted artifacts (shipped by 1898 & Co. or tenant admins with elevated privilege). Individual schedules are operator-authored. When names collide, the more-carefully-vetted artifact takes precedence. The explicit error code ensures the collision is visible to the registering operator rather than silently overwriting.

---

## Rationale

The six decisions in Section 2 are jointly necessary. Each addresses a distinct correctness, verifiability, or operational hazard that would otherwise remain unresolved across S-4.01 and S-4.02.

**Canonical JSON hashing (§2.1) is required for VP-019's determinism proof.** Raw-byte hashing is rejected because serialization order for Rust structs is not a stable contract — a field reorder or bincode version bump would produce different hashes for logically-identical rows, generating spurious added/removed entries. JCS (RFC 8785) is a published external standard; its key-ordering semantics are stable and version-independent. This gives VP-019 a formal foundation that raw-byte hashing cannot provide.

**Blake3 (§2.1) is the workspace-standard hash and is already confirmed safe.** R-4 documents no CVEs, no RustSec advisories, and SIMD acceleration on all target platforms. Adopting a different hash function for diff results would fragment the workspace's hash-function story without benefit. ADR-013 §2.2 already established blake3 as the workspace standard for splay computation; ADR-018 extends that standard to row identity.

**Merge_operator epoch counters (§2.2) are required for concurrent correctness.** The schedule executor can spawn up to 8 concurrent workers per ADR-013 §2.3. Concurrent `get → increment → put` sequences produce lost-update races under this concurrency model. The RocksDB associative merge operator is the canonical solution for atomic counter increment in RocksDB (R-9); it eliminates the race without requiring a lock or a transaction.

**Load-time pack expansion (§2.4) follows the ADR-010 config-driven principle and eliminates a TOCTOU hazard.** Config artifacts (sensor specs, pack files) are parsed once at registration and their derived records stored in RocksDB. The tick loop reads from RocksDB, not from disk files. This prevents a race between a file modification and a tick evaluation, and avoids O(N pack files × M schedules) parsing on every tick cycle.

**Capability-gated load-time evaluation (§2.5) preserves per-fire performance.** Capability flag checks are low-frequency admin events; per-fire checks would add a cross-module call on every schedule evaluation (every 60 seconds per ADR-013 §2.1). The event-driven re-evaluation pattern gives the same correctness guarantee (disabled packs produce disabled schedules) without steady-state overhead.

**Rolling two-snapshot retention (§2.6) is a deliberate scope boundary, not a limitation.** Extended diff history for forensic purposes belongs in a purpose-built event store or SIEM, not in a RocksDB CF optimized for fast set-difference computation. The `diff_results` CF is a computation cache, not an audit log. Retaining only prev + cur keeps the CF size bounded and the diff computation O(N) in the current result set size, not O(N × history_depth).

---

## 3. Consequences

### 3.1 Positive

- **Content-addressable deduplication.** Blake3 + canonical JSON means row identity is determined by content, not by serialization artifact. Two rows from different execution contexts that represent the same device/event are correctly identified as equal. This gives VP-019 (diff determinism) a solid foundation.
- **Deterministic packs.** Load-time expansion means pack deployment is auditable: one `register_pack` call produces a documented set of `ScheduleEntry` records. There is no runtime ambiguity about which schedules a pack activates.
- **Capability-gated without per-fire overhead.** The load-time capability check + event-driven re-evaluation pattern provides correct capability enforcement with zero steady-state cost (no flag lookup on each tick fire).
- **Merge_operator eliminates lost-update races.** The RocksDB associative merge for epoch counters is correct under concurrent access: multiple workers incrementing the same key never produce a lost update, unlike `get → increment → put` patterns.
- **zstd compression on hash sets.** Row hash vectors and result sets compress significantly under zstd (hash repetition across executions, structural row similarity). Expected compression ratio 3x-10x on typical sensor result sets.

### 3.2 Negative

- **Canonical JSON CPU cost.** `serde_jcs` serializes each row to canonical JSON before hashing. For large result sets (10,000+ rows per fire), this adds measurable CPU time compared to hashing raw bytes. Mitigation: blake3 is SIMD-accelerated (R-4) and the per-row cost is dominated by `serde_jcs` serialization, not the hash. If profiling reveals this is a bottleneck, caching row hashes across executions (hash-on-insert, not hash-on-compare) can be adopted in a future amendment.
- **Merge_operator debugging complexity.** RocksDB merge stacks are resolved lazily on read or at compaction. If a write bug causes a malformed operand (e.g., a 7-byte instead of 8-byte operand), the `try_into().ok()` in the merge function silently ignores it (sum is under-counted). Operators cannot easily inspect the pre-resolution merge stack using standard RocksDB tooling. Mitigation: all `merge()` calls on epoch keys are gated behind a dedicated `epoch::increment(cf, key)` function that enforces the 8-byte invariant at the call site. Any non-8-byte merge operand is a programming error, not a data error.
- **Load-time expansion couples pack schema to startup.** If a `.pack.toml` parse fails at registration time (invalid SQL, missing required fields), the pack registration is rejected. This is correct behavior but means pack deployments require a validated file. Operators cannot register a partial/draft pack and fix it later — the pack must be valid at registration.
- **Rolling retention deletes history.** Only the two most recent execution snapshots are retained per schedule. Operators who need longer diff history (e.g., "what changed in the last 24 hours across 12 fires") cannot retrieve it from `diff_results`. If extended history is needed, it must be forwarded to an external store (e.g., via alert/notification rules in S-4.03). This is an intentional scope boundary.

---

## 4. Alternatives Considered

### 4.1 Raw-Bytes Row Hashing (Rejected)

Hashing the raw serialized bytes of each row (bincode or protobuf output) was considered as a simpler alternative to canonical JSON. Rejected because:

- Serialization order for Rust structs is determined by field declaration order, which can change across refactors without semantic intent.
- Two RocksDB writes of logically-identical rows using different bincode versions or different struct field orders would hash differently, producing spurious "added" and "removed" entries.
- The correctness guarantee of VP-019 (diff determinism) cannot be proven for raw-byte hashing without pinning the serialization format as a permanent contract. Canonical JSON (JCS, RFC 8785) is a stable, external standard.

### 4.2 Runtime Pack Expansion (Rejected)

Expanding pack `[[schedules]]` entries on each tick (parsing `.pack.toml` on every tick evaluation) was considered to simplify the registration path. Rejected because:

- Adds O(N schedules × M pack files) TOML parsing to every tick cycle (every 60 seconds per ADR-013 §2.1).
- Makes the set of active schedules a function of the pack file on disk at tick time, introducing a TOCTOU hazard: modifying a `.pack.toml` file mid-cycle could alter which schedules are evaluated in a single tick without an explicit registration event.
- Prevents the load-time capability gate (§2.5) from working cleanly: runtime expansion would require per-tick capability checks.

### 4.3 PUT Overwrite for Epoch Counters (Rejected)

Using `get → increment → put` for epoch counters was considered for its simplicity. Rejected because:

- Two concurrent workers reading the same epoch value, both incrementing, and both writing back will produce a lost update: the second writer's value overwrites the first's increment.
- The schedule executor can spawn multiple concurrent workers per org (up to 8 per ADR-013 §2.3). Concurrent epoch increments are the expected case, not an edge case.
- The RocksDB merge_operator pattern (R-9) is the canonical solution for this exact use case. It adds complexity only in the CF initialization (one call to `set_merge_operator_associative` at CF open time) with no per-increment complexity.

### 4.4 Individual-Schedule-Wins Name Collision Policy (Rejected)

An alternative collision policy would allow individually-defined schedules to override pack-derived schedules. Rejected because:

- Packs are versioned, tested artifacts. Allowing an operator-authored schedule to silently shadow a pack schedule creates an invisible override that may be discovered only when the pack fires differently than expected.
- The explicit `E-SCHEDULE-NAME-PACK-COLLISION` error ensures the operator is aware of the collision and must resolve it (rename their individual schedule or deregister the conflicting pack).

---

## Phase 4.A Pass 14 Remediation Notes

Applied during Wave 4 Phase 4.A adversarial Pass 14 fix-burst (2026-05-03). Version bumped 0.4 → 0.5.

- **F-P14-M-001-CASCADE-A fix (ScheduleChangeNotification tuple form):** 3 enum-variant references updated to tuple form `(OrgId, ScheduleId)` per ADR-013 v0.7 §2.7 enum signature change. Sites: §2.5 flag toggle propagation steps 3–4 (lines ~210, ~211), §7 References ADR-013 §2.7 cross-reference (line ~547). All `Updated(schedule_id)` occurrences updated to `Updated(org_id, schedule_id)`; bare `::Updated` reference in §7 updated to `::Updated(org_id, schedule_id)`.

---

## Phase 4.A Pass 4 Remediation Notes

v0.4 body Status section synced from stale v0.3 (P4-XADR-A-H-001).

---

## Phase 4.A Pass 3 Remediation Notes

Applied during Wave 4 Phase 4.A adversarial Pass 3 fix-burst (2026-05-02). Version bumped 0.3 → 0.4.

- **P3-ADR-018-A-H-001 fix (CF key prefix order):** All `diff_results` CF keys corrected to place `{org_id}:` first, matching the sibling-ADR convention (ADR-013/015/016/017). Epoch key: `epoch:{org_id}:{schedule_id}` → `{org_id}:epoch:{schedule_id}`. Diff keys: `diff:{org_id}:{schedule_id}:*` → `{org_id}:diff:{schedule_id}:*`. §2.6 table, §2.2 epoch key format, and all Source/Origin + References cross-references updated. This makes `reset_for(org_id)` work correctly via single prefix scan on `{org_id}:` per ADR-008 §2.4.

## Phase 4.A Pass 2 Remediation Notes

Applied during Wave 4 Phase 4.A adversarial Pass 2 fix-burst (2026-05-02). Version bumped 0.2 → 0.3.

- **P2-ADR-018-A-M-001 fix (cron 5-vs-6 field discrepancy):** §2.3 `schedules[*].interval_seconds` updated. Pack-derived cron changed from 6-field `*/{interval_seconds} * * * * *` to 5-field `*/{interval_minutes} * * * *` where `interval_minutes = interval_seconds / 60`. `interval_seconds < 60` is now rejected at pack registration with `E-PACK-INTERVAL-TOO-SHORT`. Rationale: action-delivery (ADR-016) and the schedule executor (ADR-013 §2.8) use `croner = "3"` which accepts 5-field cron expressions; 6-field crons would be rejected at runtime.
- **P2-ADR-018-A-M-002 fix (pack name uniqueness scope ambiguity):** §2.3 `pack.name` description updated: `PackId` is now explicitly defined as the `(org_id, pack.name)` tuple — pack names are unique WITHIN an org, not across orgs. §2.7 collision semantics updated to use `(org_id, pack.name)` as the dedup key. The case of different orgs with the same `pack.name` is now explicitly documented as allowed (normal multi-tenant vendor pack deployment).

## Phase 4.A Pass 1 Remediation Notes

Applied during Wave 4 Phase 4.A adversarial Pass 1 fix-burst (2026-05-02). Version bumped 0.1 → 0.2.

- **P1-ADR-018-A-H-001 fix:** `subsystems_affected` corrected from `[SS-04]` to `[SS-12, SS-13]`. SS-12 (Scheduler) produces diff results; SS-13 (Detection) consumes them.
- **P1-ADR-018-A-H-002 fix:** §2.5.1 added: capability-flag-changed events emitted by `prism-security` (S-1.08 owner) via `tokio::sync::watch::channel<FeatureFlagSnapshot>` at `prism_security::feature_flags::flag_change_watcher()`. Pack manager (S-4.02) subscribes at startup.
- **P1-ADR-018-A-M-003 fix:** §2.6 `:cur` and `:prev` values now include a 1-byte version prefix `[0x01][bincode-encoded data]`. Future hash-size changes bump the version byte.
- **P1-ADR-018-A-M-004 fix:** §2.7 pack idempotence/collision semantics fully specified: same pack_id + same version → no-op; same pack_id + higher version → update; same pack_id + lower version → `Err(PackVersionRegression)`; different pack_id + same name → `Err(PackNameCollision)`.
- **P1-ADR-018-A-M-005 fix:** §2.6 zstd compression specification replaced with exact rocksdb 0.24 API call sequence: `Options::set_compression_type(DBCompressionType::Zstd)` + `Options::set_compression_options(-14, 3, 0, 0)`.

---

## Source / Origin

- **Architectural decisions (STATE.md §Wave 4 Decision Log):**
  - D-207: 6-ADR topology; ADR-018 scoped to differential result pack format (logged 2026-05-02).
  - D-208: OrgId/ClientId dual hierarchy; all Wave 4 domain types gain `org_id: OrgId`; `diff_results` CF keys gain `{org_id}:diff:` prefix per ADR-008.
  - D-210: Differential result computation using blake3 row hashing and set-difference (logged 2026-05-02).
  - D-213: Pack TOML schema and load-time expansion to `ScheduleEntry` records (logged 2026-05-02).
- **Research findings (research-findings.md):**
  - R-4 §blake3: `blake3 = "1.8.5"`, no CVEs, workspace standard. Confirmed for row identity hashing (2026-05-02).
  - R-9 §rocksdb: `rocksdb = "0.24.0"` latest stable. `set_merge_operator_associative` API and `u64::to_le_bytes` epoch counter pattern documented (2026-05-02).
- **Story drafts:**
  - S-4.02-diff-results-packs.md: Primary anchor story; Task 7 (pack registration) specifies load-time expansion; Library table identifies `serde_jcs` pin (drift DRIFT-402-006).
  - S-4.01-schedule-crud.md: Peer story; `ScheduleEntry` struct extended with `pack_origin`/`pack_version` fields; schedule-change watch channel (ADR-013 §2.7) reused for capability-flag toggle propagation.
- **Prior ADRs:**
  - ADR-006 §2.1: OrgId canonical routing key; packs and diff results are org-scoped; `pack.name` uniqueness rules follow OrgSlug convention.
  - ADR-008 §2.2: Universal `{org_id}:` CF key prefix rule; `{org_id}:diff:{schedule_id}:*` key format derives from this rule.
  - ADR-010: `PRISM_*` env-var convention; `.pack.toml` config-file-as-spec pattern; `PRISM_DIFF_EVICTION_DAYS` follows this convention.
  - ADR-013 §2.2: Blake3 workspace standard for splay; ADR-018 extends to row hashing with the same `blake3 = "1.8"` pin. ADR-013 §2.6: `schedules` CF and `ScheduleEntry` struct; `pack_origin`/`pack_version` fields added here. ADR-013 §2.7: Schedule-change watch channel; capability-flag toggle propagation emits via this channel.
- **Verification properties:**
  - VP-019 (vp-019-diff-computation-deterministic.md): pre-existing P0 proptest; scope formalized in §5.1.
  - VP-141 (epoch counter merge atomicity): proposed in this ADR; §5.2. VP file and VP-INDEX update required before Phase 4.B BC authoring.
  - VP-142 (pack expansion idempotence): proposed in this ADR; §5.3. VP file and VP-INDEX update required before Phase 4.B BC authoring.

---

## 5. Verification Plan

### 5.1 VP-019 — Diff Computation Determinism (Pre-Existing)

**Property:** For any two result sets A and B, `diff(A, B)` produces the same added/removed sets regardless of the order in which rows within A and B were inserted, and regardless of process restart between executions.

**Method:** Proptest. Generate arbitrary pairs of row sets; compute diff via both orderings of row insertion; assert `added` and `removed` sets are identical. Also: serialize row set to RocksDB, restart (simulated via CF re-open), reload, re-diff, assert same result.

**Module:** `prism-operations`
**Priority:** P0
**Anchor story:** S-4.02

**Harness skeleton:**
```rust
#[cfg(test)]
mod diff_determinism {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn diff_is_order_independent(
            rows_a in vec(any_row(), 0..500usize),
            rows_b in vec(any_row(), 0..500usize),
        ) {
            let hashes_a1: HashSet<[u8; 32]> = rows_a.iter()
                .map(canonical_hash).collect();
            let hashes_a2: HashSet<[u8; 32]> = rows_a.iter().rev()
                .map(canonical_hash).collect();
            let hashes_b: HashSet<[u8; 32]> = rows_b.iter()
                .map(canonical_hash).collect();

            let added1: HashSet<_> = hashes_b.difference(&hashes_a1).collect();
            let added2: HashSet<_> = hashes_b.difference(&hashes_a2).collect();
            prop_assert_eq!(added1, added2);
        }
    }
}
```

**Status:** draft (VP-019 pre-exists; this ADR formalizes scope and adds the harness skeleton).

### 5.2 VP-141 — Epoch Counter Merge Atomicity (Proposed by This ADR)

**Property:** Concurrent `merge()` calls on the same epoch key never lose increments. For N concurrent increments from an initial value of 0, the final epoch value equals N.

**Method:** Proptest (concurrent simulation against MockStorageEngine with merge semantics). Also: integration test with actual RocksDB CF.

**Module:** `prism-operations`
**Priority:** P1
**Anchor story:** S-4.02

**Harness skeleton:**
```rust
#[cfg(test)]
mod epoch_merge_atomicity {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn concurrent_increments_not_lost(n_increments in 1usize..=1000) {
            // Apply n_increments merge operands to a single key
            // using the epoch_merge function directly (no RocksDB needed)
            let operands: Vec<Vec<u8>> = (0..n_increments)
                .map(|_| 1u64.to_le_bytes().to_vec())
                .collect();
            let result = apply_epoch_merge_sequence(None, &operands);
            let total = u64::from_le_bytes(result.try_into().unwrap());
            prop_assert_eq!(total, n_increments as u64);
        }
    }
}
```

**Status:** proposed; assigned VP-141 in this ADR. VP file and VP-INDEX update to be produced before Phase 4.B BC authoring begins.

### 5.3 VP-142 — Pack Expansion Idempotence (Proposed by This ADR)

**Property:** Registering the same pack (same `pack_id`, same `version`) twice for the same `org_id` produces an identical set of `ScheduleEntry` records. No duplicate entries are created; the second registration is a no-op.

**Method:** Proptest. Generate arbitrary pack TOML; register for an org; register again; assert `ScheduleEntry` count and content are identical to after the first registration.

**Module:** `prism-operations`
**Priority:** P1
**Anchor story:** S-4.02

**Harness skeleton:**
```rust
#[cfg(test)]
mod pack_idempotence {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn double_register_is_noop(pack in arbitrary_pack()) {
            let org = OrgId::new_test();
            let mut store = MockScheduleStore::new();

            register_pack(&mut store, &pack, org).unwrap();
            let entries_after_first = store.list_schedules(org);

            register_pack(&mut store, &pack, org).unwrap();
            let entries_after_second = store.list_schedules(org);

            prop_assert_eq!(entries_after_first.len(), entries_after_second.len());
            for (e1, e2) in entries_after_first.iter().zip(&entries_after_second) {
                prop_assert_eq!(e1.schedule_id, e2.schedule_id);
                prop_assert_eq!(e1.cron_expr, e2.cron_expr);
                prop_assert_eq!(e1.pack_origin, e2.pack_origin);
            }
        }
    }
}
```

**Status:** proposed; assigned VP-142 in this ADR. VP file and VP-INDEX update to be produced before Phase 4.B BC authoring begins.

---

## 6. Migration Path

Not applicable. The `diff_results` CF and the pack registration system are greenfield for Wave 4. No prior differential result infrastructure exists in production.

Deployment note: the `diff_results` CF must be created via `create_cf` during process startup if it does not exist. The `diff_results` CF requires `set_merge_operator_associative` registered at CF open time — opening the CF without the merge operator registered will cause RocksDB to error when it encounters merge operands. Story-writer must ensure the CF initialization sequence (BC-2.12.010 or equivalent) includes merge-operator registration before the CF is opened for read/write.

---

## 7. References

### Research Findings

- **R-4** (`research-findings.md §R-4`): `blake3 = "1.8.5"`, no CVEs, no RustSec advisories, SIMD-accelerated, workspace standard. Confirmed use for row hashing.
- **R-9** (`research-findings.md §R-9`): `rocksdb = "0.24.0"` latest stable. `set_merge_operator_associative` API confirmed for associative counter increment. Canonical `u64::to_le_bytes` encoding pattern documented.

### Architecture Decisions

- **ADR-006 §2.1**: OrgId is canonical routing key; packs and diff results are org-scoped.
- **ADR-008 §2.2**: Universal `{org_id}:` CF key prefix rule; `diff_results` CF key format `{org_id}:diff:{schedule_id}:*` derives directly from this rule.
- **ADR-010**: Config-driven sensor spec pattern; `.pack.toml` follows the same config-file-as-spec convention. `PRISM_DIFF_EVICTION_DAYS` env-var follows the `PRISM_*` convention.
- **ADR-013 §2.2**: Blake3 splay hash — workspace standard for blake3 established here. ADR-018 adopts the same pin (`blake3 = "1.8"`). ADR-013 §2.6: `schedules` CF key format; derived `ScheduleEntry` fields defined there, extended here with `pack_origin` and `pack_version`. ADR-013 §2.7: Schedule-change watch channel; pack capability-flag toggles emit `ScheduleChangeNotification::Updated(org_id, schedule_id)` via this channel.

### Verification Properties

- **VP-019** (vp-019-diff-computation-deterministic.md): pre-existing; scope formalized in §5.1.
- **VP-141** (epoch counter merge atomicity): proposed in this ADR; §5.2.
- **VP-142** (pack expansion idempotence): proposed in this ADR; §5.3.
