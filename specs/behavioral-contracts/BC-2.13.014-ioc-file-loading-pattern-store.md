---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-16T14:00:00
phase: 2-patch
origin: greenfield
subsystem: "SS-13"
capability: "CAP-020"
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "e5de7f9"
traces_to: ["CAP-020"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.13.014: IOC File Loading and Pattern Store — At-Startup Load with Hot Reload and Bounded Memory

## Description

IOC (Indicator of Compromise) files in `{config_dir}/ioc/` are loaded at startup and on
hot reload. Each file (extension `.ioc`) is a plain-text indicator list: one indicator
per line, `#`-prefixed lines are comments, empty lines are ignored. Each indicator line
is compiled as a `regex::Regex` pattern (finite automaton, CWE-1333 safe — no
backtracking). All patterns from a single file are aggregated into a `regex::RegexSet`
for O(n_patterns) multi-pattern matching with a single scan.

The file's basename (without the `.ioc` extension) becomes the list name used in
`ioc_match(field_expr, "list_name")` DataFusion UDF calls. The in-memory pattern
store is bounded: max 100,000 patterns per file, max 10 MB per file, max 50 files.
Malformed IOC files are rejected at load without crashing the server; the prior in-memory
version of that file (if any) is retained. Atomic arc-swap ensures concurrent queries
see either the old or new `RegexSet`, never a partial mix.

## Preconditions

- The `ioc/` subdirectory exists within the config directory (it may be empty)
- Each `*.ioc` file is readable by the Prism process
- For hot reload: the S-1.12 filesystem watcher has detected a change event for one
  or more files in `ioc/`

## Postconditions

### Successful load (startup or hot reload)

- Each `*.ioc` file in `{config_dir}/ioc/` is read and parsed:
  - Comment lines (`#` prefix) and empty lines are discarded
  - Each remaining line is compiled as a `regex::Regex`
  - All compiled patterns for the file are aggregated into a `regex::RegexSet`
  - The `RegexSet` is stored in the in-memory pattern store keyed by list name
    (filename without `.ioc` extension)
- The pattern store is updated atomically via arc-swap: the new `Arc<PatternStore>`
  replaces the old one; in-flight queries holding a reference to the old `Arc` complete
  against the old `RegexSet` without interruption
- An INFO log entry is emitted for each successfully loaded file:
  `"IOC file loaded: {list_name} ({pattern_count} patterns, {elapsed_ms}ms)"`
- The `check_sensor_health` tool (and `get_diagnostics(subsystem: "infusions")`)
  reflects the updated IOC file status

### Hot reload — file changed

- Changed files are recompiled independently (Tier 3: per-file independent)
- Unchanged files are NOT reloaded (their existing `RegexSet` is retained)
- If the changed file fails validation, the existing `RegexSet` for that file is
  retained in the pattern store; other files are unaffected
- A WARN log entry is emitted if the file fails: `"IOC file rejected: {filename} — {reason}. Previous version retained."`

### `ioc_match` UDF execution

- `ioc_match(field_expr: Utf8, list_name: Utf8) -> Boolean`
- The UDF acquires a clone of the `Arc<PatternStore>` at the start of execution
  (cheap — reference count increment only; no lock held during matching)
- For each row: `RegexSet::is_match(field_value)` is called; returns `true` if any
  pattern in the named list matches the field value, `false` otherwise
- The UDF is PURE: no I/O, no system clock access, no mutable state, deterministic for
  the same (field_value, list_name, pattern_store_snapshot) tuple
- If `list_name` does not exist in the pattern store: returns `false` for all rows +
  WARN log `"ioc_match: list '{list_name}' not found; returning false for all rows"`
  (error code `E-UDF-001` per BC-2.13.010)
- NULL input field values return NULL (DataFusion null propagation)

## Invariants

- **INV-IOC-001 (Atomic Hot Reload):** Concurrent `ioc_match` calls during a hot reload
  see either the old `RegexSet` or the new `RegexSet`, never a partial mix of patterns
  from both versions. This is enforced by the arc-swap: the pattern store swap is a
  single atomic pointer write; no locking is held during UDF execution
- **INV-IOC-002 (Pure UDF):** `ioc_match` is a pure function during any single query
  execution. Once the `Arc<PatternStore>` snapshot is acquired at query start, the UDF
  result for a given `(field_value, list_name)` pair is deterministic and does not
  change mid-query, even if a hot reload occurs during the query
- **INV-IOC-003 (Memory Cap Enforced):** Loading an IOC file that would exceed
  100,000 patterns, 10 MB, or bring the total file count beyond 50 files is REJECTED.
  The existing pattern store state is retained; the oversized file is not partially loaded
- **INV-IOC-004 (No Crash on Malformed File):** A malformed IOC file (invalid regex
  pattern, encoding error, I/O error) MUST NOT terminate the server. The error is logged,
  the file is rejected, and the server continues with the prior version of that file's
  patterns (or no patterns if the file is new)

## Error Cases

| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-IOC-001` | IOC file contains one or more invalid regex patterns | File rejected; WARN log with pattern count and first 3 failing patterns; prior `RegexSet` retained; `E-IOC-001` emitted |
| `E-IOC-002` | IOC file size exceeds 10 MB | File rejected before compilation; WARN log: `"IOC file '{filename}.ioc' exceeds size limit ({size} > 10MB)"`; prior state retained |
| `E-IOC-003` | IOC file exceeds 100,000 pattern count | File rejected after counting lines; WARN log: `"IOC file '{filename}.ioc' exceeds pattern count limit ({count} > 100000)"`; prior state retained |
| `E-IOC-004` | Loading the file would bring total file count beyond 50 | File rejected: `"Maximum IOC file count (50) exceeded. Remove an existing IOC file before adding '{filename}.ioc'."` |
| `E-UDF-001` | `ioc_match` references a list name not in the pattern store | Returns `false` for all rows; WARN logged; not a fatal query error |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-13-038 | IOC file contains 100,000 valid patterns (at cap boundary) | File loads successfully; `RegexSet` built with 100,000 patterns |
| EC-13-039 | IOC file contains 100,001 patterns | File rejected with `E-IOC-003`; prior state retained |
| EC-13-040 | Hot reload occurs while `ioc_match` query is in-flight against 500K rows | Query completes against the pre-reload `RegexSet` snapshot; hot reload completes concurrently; results are deterministic for the query's snapshot |
| EC-13-041 | IOC file is deleted from disk between startup and hot reload | List name is removed from the pattern store on reload; subsequent `ioc_match` calls for that list return `false` + `E-UDF-001` WARN |
| EC-13-042 | Two IOC files have the same basename but different casing (e.g., `Known_Bad_IPs.ioc` and `known_bad_ips.ioc`) on a case-insensitive filesystem | List names are lowercased at load time to prevent ambiguity; only one entry per logical list name is kept (last writer wins if both exist) |
| EC-13-043 | IOC file line contains a backtracking regex (e.g., `(a+)+b`) | Rejected with `E-IOC-001` if the `regex` crate rejects it (the crate uses finite automata and rejects patterns that cannot be compiled to DFA within the default size limit — CWE-1333 safe) |
| EC-13-044 | IOC file is empty (zero patterns after stripping comments) | File accepted; `RegexSet` with zero patterns; `ioc_match` returns `false` for all rows; INFO log: `"IOC file loaded: {list_name} (0 patterns)"` |
| EC-13-045 | 50 IOC files already loaded; 51st file added to `ioc/` | 51st file rejected with `E-IOC-004`; all 50 existing files continue to function |

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| IOC file with 100 valid regex patterns at startup | File loaded; `ioc_match` returns true for matching field values | happy-path |
| `ioc_match("evil.domain.com", "known_bad_domains")` with domain in list | true | happy-path |
| IOC file with 100,001 patterns | `E-IOC-003`; prior state retained | error |
| Hot reload while 10 concurrent `ioc_match` queries in-flight | All queries complete deterministically; no panic | edge-case |
| Empty IOC file (all comments) | File accepted; 0 patterns; `ioc_match` returns false for all | edge-case |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| VP-023 | Sensor spec parser: never panics on arbitrary TOML (analogous fuzz for IOC file content) | fuzz |

Integration test: `tests/ioc_tests.rs` — "Load 100K-pattern IOC file → verify ioc_match returns true for known match, false for non-match."

Integration test: `tests/ioc_tests.rs` — "Hot reload IOC file while 10 concurrent ioc_match queries are in-flight → verify all queries return deterministic results; no panic."

Fuzz test: `fuzz/fuzz_ioc_load.rs` — "Arbitrary IOC file content → verify server does not panic; invalid patterns produce E-IOC-001, valid patterns load successfully."

## Related BCs

- BC-2.13.010 — Security UDF Registration (`ioc_match` UDF contract; this BC specifies the
  backing store that `ioc_match` queries)
- BC-2.13.002 — Single-Event Detection (detection rules use `ioc_match` in filter expressions)
- BC-2.16.007 — Sensor Spec Hot Reload (same arc-swap pattern used for IOC hot reload)
- BC-2.16.009 — Spec File Validation (same Tier 3 per-file independent validation pattern)

## Architecture Anchors

- `specs/architecture/query-engine.md` §"IOC File Specification" — format (plain text,
  one regex per line), location (`{config_dir}/ioc/{list_name}.ioc`), loading,
  size limits (100K patterns, 10 MB, 50 files), hot reload, missing file behavior,
  arc-swap atomicity
- `specs/architecture/config-schema.md` §"Config Validation" — Tier 3: per-file
  independent validation (IOC files rejected individually; valid files still load)
- `specs/architecture/infusions.md` §"Use in Detection Rules" — `ioc_match` as the
  recommended locally-cached alternative to API-backed infusion UDFs in detection rules
- S-4.03 Task: `detection/ioc.rs` — `PatternStore`, arc-swap, per-file load/validate/compile,
  `ioc_match_udf()` registration

## Story Anchor

S-4.03 — Detection Engine (scope expansion: IOC file loading, per STATE.md Burst 2 scope list)

## VP Anchors

Integration test: `tests/ioc_tests.rs` — "Load 100K-pattern IOC file → verify ioc_match returns true for known match, false for non-match."

Integration test: `tests/ioc_tests.rs` — "Hot reload IOC file while 10 concurrent ioc_match queries are in-flight → verify all queries return deterministic results; no panic."

Fuzz test: `fuzz/fuzz_ioc_load.rs` — "Arbitrary IOC file content → verify server does not panic; invalid patterns produce E-IOC-001, valid patterns load successfully."

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-020 |
| L2 Invariants | DI-019, DI-024 |
| ADR | AD-018 (file watcher drives hot reload trigger) |
| Story | S-4.03 |
| Priority | P0 |
| Interface | query-engine.md §IOC File Specification |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | phase-2-patch | 2026-04-16 | product-owner | Initial contract (BC-2.13.014 added in Phase 2 patch) |
