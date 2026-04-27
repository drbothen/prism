---
document_type: wave-cycle-decision
decision_id: decision-w2-mutate-005-carveout
td_resolved: TD-W2-MUTATE-005
decision_log_entry: D-035
status: decided
option_chosen: B
date: 2026-04-26
decider: architect
wave: phase-3-dtu-wave-2
traces_to: .factory/tech-debt-register.md (TD-W2-MUTATE-005)
---

# Decision: TD-W2-MUTATE-005 — prism-sensors Mutation Testing Carve-Out (Wave 2 Close)

## Decision

**Option B — Scope mutation testing to specific Wave 2 deliverables in prism-sensors.**

prism-sensors is a large crate (~2,400 source lines across 12 source files, ~4,100
lines across 10 integration test files, ~102 unit tests + ~133 integration tests = ~235
total). A full-crate `cargo mutants -p prism-sensors` run against a test suite of
this size would take multiple hours on a developer machine and is disproportionate
to the goal. However, three Wave 2 story surfaces carry real mutation risk and must
not be deferred:

1. **S-2.07 pagination + timestamp parsing** (`pagination.rs`, `timestamp.rs`) —
   pure algorithmic logic, mutation-friendly, directly testable by the 12
   `test_pagination` and 11 `test_timestamp` integration tests. These are exactly
   the modules where mutation testing adds the most signal because they contain
   branching, arithmetic, and fallback-chain logic that is easily corrupted by
   operator mutations (boundary comparisons, fallthrough order, saturating-add
   semantics).

2. **W2-FIX-I AQL validator** (`auth/armis.rs::validate_aql`, `build_aql`) —
   security-critical. 18 integration tests in `test_wgs_w2_001_aql_validator.rs`
   cover the allowlist. This function is the primary injection mitigation for
   CWE-943 (ADR-005). A mutation escaping these tests would be a HIGH security
   regression. Must not be carried forward.

3. **W2-FIX-I SecretString plumbing** (`auth/armis.rs`, `auth/claroty.rs`,
   `auth/crowdstrike.rs`) — 3 tests in `test_wgs_w2_002_secretstring.rs`. Coverage
   is thin; the test surface is narrow but any mutation suggesting the `SecretString`
   wrapper can be bypassed (e.g., a plain `String` substitution survives) would be
   a CWE-312 regression. Include in scope.

S-2.06 itself (the subject of the TD) is explicitly scoped OUT of this run with
carve-out granted. The evidence-report documents that the 11 RED sentinel tests
covered all algorithmic implementations (`retry_with_backoff`, `fan_out`,
semaphore acquisition, `execute_target`, `RetryConfig` defaults). The 40 GBD tests
are struct-shape, enum-discriminant, and constant assertions — categories where
mutation testing adds little signal because a mutated constant or struct field
propagates deterministically to a failing assertion. The S-2.06 pattern is
materially distinct from TD-W2-MUTATE-001..004, where GBD tests covered
implemented behavioral logic that was never driven to failure.

S-2.08 event-table logic (`event_buffer.rs`, `table_dispatch.rs`, `poller.rs`)
is also scoped OUT of this run. Its dedicated unit test suite (evict_backend,
event_buffer_tests, table_dispatch_tests, poller_tests) already drove those paths
to RED. The known gap (TD-S208-002: concurrent-write validation) is tracked
separately.

## Scope (Option B)

The following source files and integration test targets are in scope for this run:

### Source files under mutation

| File | Story origin | Rationale |
|------|-------------|-----------|
| `crates/prism-sensors/src/pagination.rs` | S-2.07 | Pure algorithmic: OffsetCursor arithmetic, is_exhausted boundary, advance DI-001 invariant, paginate_claroty stream logic |
| `crates/prism-sensors/src/timestamp.rs` | S-2.07 | Multi-format fallback chain: RFC 3339, unix epoch, custom format — fallthrough order is mutation-sensitive |
| `crates/prism-sensors/src/auth/armis.rs` | W2-FIX-I | AQL validator (validate_aql, build_aql) — security-critical; 18 dedicated tests |
| `crates/prism-sensors/src/auth/claroty.rs` | W2-FIX-I | SecretString bearer_token wrapping |
| `crates/prism-sensors/src/auth/crowdstrike.rs` | W2-FIX-I | SecretString client_secret wrapping |

### Test targets that exercise this scope

- `test_pagination` — 12 tests; exercises OffsetCursor and paginate_claroty stream
- `test_timestamp` — 11 tests; exercises parse_timestamp fallback chain
- `test_wgs_w2_001_aql_validator` — 18 tests; exercises validate_aql allowlist
- `test_wgs_w2_002_secretstring` — 3 tests; exercises SecretString wrapping

### Files explicitly excluded from this run (carve-out granted)

| File | Reason |
|------|--------|
| `src/adapter.rs` | Enum/trait definition surface; algorithmic core (is_transient) tested by S-2.06 RED tests already passing |
| `src/fanout.rs` | S-2.06 RED tests cover fan_out and execute_target; dispatch_by_table_type is a documented stub pending S-3.02 |
| `src/retry.rs` | S-2.06 RED tests cover retry_with_backoff; RetryConfig constants are data-definition surface |
| `src/event_buffer.rs` | S-2.08 scope; TD-S208-002 tracks concurrent-write gap |
| `src/table_dispatch.rs` | S-2.08 scope |
| `src/poller.rs` | S-2.08 scope |
| `src/registry.rs` | 10-line data store; trivially tested by S-2.06 BC-2.01.013 tests |
| `src/http.rs` | OnceLock initialization; 7-test semaphore suite (S-2.06) covers semantic behavior |
| `src/auth/cyberint.rs` | S-2.07; no SecretString wrapping changes in W2-FIX-I; existing tests cover api_key type |
| `src/auth/mod.rs` | Trait definitions only; no behavioral logic |

## Estimated Runtime

**15–40 minutes** for this scoped run.

Basis for estimate:
- 5 source files in scope; `cargo mutants` generates approximately 8–20 mutants per
  100 source lines for algorithmic Rust code. The 5 files total approximately 900
  lines, yielding an estimated 70–180 mutants.
- Each mutant requires a full `cargo test -p prism-sensors` compile + test cycle.
  On a typical developer machine (Apple Silicon or equivalent), this is ~12–25 seconds
  per mutant compile when the base is already built.
- Estimated 180 mutants x 20 seconds = ~60 minutes worst-case; typical is lower
  because the scoped test filter (`--test-timeout` + file filter) skips unrelated
  crate-level compilation overhead.
- The estimate is 15–40 minutes with high confidence; 60 minutes is an outlier bound.

Compare: full-crate run would be ~2,400 source lines, estimated 200–500 mutants,
at 20–30 seconds each = 1–4 hours. Not appropriate for Wave 2 gate housekeeping.

## Implementer Instructions

Run the following command from the workspace root. The `--file` flag restricts
mutation to the five in-scope source files. The test binary selection uses the
four Wave 2 integration test targets rather than the full `--lib + all-integration`
set, to keep per-mutant compile time low.

```bash
cargo mutants \
  -p prism-sensors \
  --file crates/prism-sensors/src/pagination.rs \
  --file crates/prism-sensors/src/timestamp.rs \
  --file crates/prism-sensors/src/auth/armis.rs \
  --file crates/prism-sensors/src/auth/claroty.rs \
  --file crates/prism-sensors/src/auth/crowdstrike.rs \
  --test-timeout 120 \
  2>&1 | tee .factory/cycles/phase-3-dtu-wave-2/mutation-results-w2-mutate-005.txt
```

### Acceptance threshold

The same >=80% kill rate applied to TD-W2-MUTATE-001..004 applies here. If the
run surfaces survivors:

- **AQL validator survivors (auth/armis.rs `validate_aql` / `build_aql`)**: treat
  as BLOCKING — write additional tests before Wave 3 dispatch. This is a security
  surface (ADR-005, CWE-943).
- **SecretString survivors (claroty.rs, crowdstrike.rs)**: treat as HIGH — review
  whether the surviving mutant represents a real coverage gap or a trivially
  equivalent mutation (TEQ). Document disposition in
  `.factory/cycles/phase-3-dtu-wave-2/mutation-results-w2-mutate-005.txt`.
- **Pagination / timestamp survivors**: treat as MEDIUM — if kill rate >= 80%
  overall, file residual survivors as TD items with Wave 3 target.

### Verifying --file flag availability

The `--file` flag was introduced in `cargo-mutants` 24.x. Verify with:

```bash
cargo mutants --version
cargo mutants --help | grep -A2 "\-\-file"
```

If `--file` is not available on the installed version, use the diff-based filter as
the alternative:

```bash
git diff f13b5c76..HEAD -- \
  crates/prism-sensors/src/pagination.rs \
  crates/prism-sensors/src/timestamp.rs \
  crates/prism-sensors/src/auth/armis.rs \
  crates/prism-sensors/src/auth/claroty.rs \
  crates/prism-sensors/src/auth/crowdstrike.rs \
  > /tmp/w2-sensors-scope.patch

cargo mutants \
  -p prism-sensors \
  --in-diff /tmp/w2-sensors-scope.patch \
  --test-timeout 120 \
  2>&1 | tee .factory/cycles/phase-3-dtu-wave-2/mutation-results-w2-mutate-005.txt
```

Note: `--in-diff` mutates only lines changed in the diff. This is a tighter scope
than `--file` (which mutates the entire file). For the auth files, `--in-diff` is
more conservative and preferable if the version supports it — it targets exactly the
W2-FIX-I changes rather than all of `armis.rs`. Use whichever is available; document
which flag was used in the results file header.

## Follow-On Technical Debt

### TD-W2-SENSORS-FULL-001 (new — filed by this decision)

| Field | Value |
|-------|-------|
| ID | TD-W2-SENSORS-FULL-001 |
| Title | Full prism-sensors mutation testing sweep (non-Wave-2 surface) |
| Problem | Files carved out of this run (adapter.rs, fanout.rs, retry.rs, event_buffer.rs, table_dispatch.rs, poller.rs, registry.rs, http.rs, auth/cyberint.rs) have not been subject to mutation testing. S-2.06 RED coverage is adequate for the algorithmic core, but a full-crate sweep has never been run. |
| Action | Run `cargo mutants -p prism-sensors` (full crate, all tests) during the Wave 3 hardening window, after S-3.02 DataFusion integration ships. Running before S-3.02 would surface survivors in the dispatch_by_table_type stub that are expected until then. |
| Priority | P3 |
| Target | Wave 3 hardening (after S-3.02 merges) |
| Owner | test-writer / general-purpose implementer |

### S-2.06 carve-out policy note

This decision establishes the following precedent for future wave gates:

> A story with RED ratio below the Layer-2 ≥0.5 threshold is **not automatically
> enrolled in the retroactive mutation-test set** if the following conditions all hold:
> (a) the RED tests drove all algorithmic implementations to failure before the Green
> Gate; (b) the GBD tests are confined to data-definition surface (struct shape, enum
> discriminants, constants, trivially-correct constructors) rather than behavioral
> stubs; and (c) the evidence-report explicitly discloses the GBD surface and
> identifies the RED-covered algorithmic subset.
>
> Stories that shipped stub-as-impl (TD-W2-MUTATE-001..004 pattern) do NOT qualify
> for this carve-out and must be enrolled in the retroactive mutation-test set.

This policy note should be propagated to VSDD process documentation (TD-VSDD-003
context) when the vsdd-factory plugin receives the stub-as-impl anti-pattern
prevention layers.

## Decision Log Entry

This decision corresponds to **D-035** in STATE.md Decisions Log. State-manager
should record:

> D-035 | TD-W2-MUTATE-005 resolved: Option B chosen. Mutation testing for prism-sensors
> scoped to 5 Wave 2 files (pagination.rs, timestamp.rs, auth/armis.rs, auth/claroty.rs,
> auth/crowdstrike.rs) with >=80% kill rate threshold. S-2.06 data-definition surface
> granted explicit carve-out. Full-crate sweep deferred to Wave 3 hardening as
> TD-W2-SENSORS-FULL-001. Carve-out policy precedent established for future gates.
> Estimated runtime 15-40 minutes. | Architect decision; TD closed | 3 | 2026-04-26
