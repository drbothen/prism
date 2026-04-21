---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-04-21T00:00:00Z
phase: 2-patch
origin: greenfield
subsystem: "SS-20"
capability: "CAP-035"
lifecycle_status: active
introduced: cycle-1-pass-80
modified: 2026-04-21
deprecated: ~
deprecated_by: ~
replacement: ~
retired: ~
removed: ~
removal_reason: ~
inputs:
  - ".factory/specs/architecture/observability.md"
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "[md5]"
traces_to: ["CAP-035"]
extracted_from: ".factory/specs/architecture/observability.md"
---

# BC-2.20.002: Log Forwarder Min-Level Filter — Per-Destination min_level Applied Before Enqueue

## Description

Each external log forwarding destination declares a `min_level` configuration field
(`trace | debug | info | warn | error`). Before a diagnostic log entry is appended to a
destination's in-memory queue, the entry's severity is compared against the destination's
`min_level`. Entries below the threshold are silently discarded — they are NOT queued,
NOT counted in delivery metrics, and NOT deferred for later delivery. This allows
high-volume `info` traffic to be dropped for cost-sensitive destinations (e.g., Datadog)
while being delivered in full to cheaper internal destinations (e.g., Splunk).

## Preconditions

- At least one `[[server.log_forward]]` destination is configured with a `min_level` field
- The Prism process is emitting diagnostic log entries at various levels

## Postconditions

- For each diagnostic log entry `E` at level `L` and each configured destination `D` with
  `min_level = M`:
  - If `level_rank(L) >= level_rank(M)`: `E` is appended to `D`'s in-memory queue
  - If `level_rank(L) < level_rank(M)`: `E` is silently discarded for `D` — not queued,
    not counted, not deferred
- Level rank order (ascending): `trace < debug < info < warn < error`
- The filter is applied independently per destination; the same entry may be enqueued for
  one destination and discarded for another
- A destination that omits `min_level` defaults to `info`
- The local stderr/rolling-file sink is NOT subject to `min_level` filtering from the
  `[[server.log_forward]]` configuration; it is controlled by the global `log_level` setting

## Invariants

- `min_level` filtering is a pure function of `(entry.level, destination.min_level)` —
  no state accumulates for filtered entries
- Discarded entries are not recoverable (no buffer, no replay)
- Filtering occurs BEFORE the entry is written to the destination queue

## Error Conditions

| Error | Condition | Behavior |
|-------|-----------|----------|
| Invalid `min_level` value in config | `min_level = "verbose"` or unknown string | Config validation error at load time; forwarder rejected; other forwarders unaffected |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-20-005 | Two destinations: `datadog` with `min_level = "warn"`, `splunk` with `min_level = "info"` | An `info`-level entry is enqueued for `splunk` only; `warn+` entries are enqueued for both |
| EC-20-006 | Destination configured with `min_level = "trace"` | All entries enqueued (trace is the lowest level — nothing is filtered) |
| EC-20-007 | Destination configured with `min_level = "error"` | Only `error`-level entries enqueued; `trace/debug/info/warn` all silently discarded |
| EC-20-008 | Destination omits `min_level` | Defaults to `info`; behavior identical to `min_level = "info"` |
| EC-20-009 | Log entry at `error` level; all destinations have `min_level = "error"` | Entry enqueued for all destinations |

## Canonical Test Vectors

| ID | Input | Expected Output | Notes |
|----|-------|----------------|-------|
| TV-20-002-pass | Destination `min_level = "warn"`; entry at `warn` | Entry enqueued | Boundary: at threshold |
| TV-20-002-drop | Destination `min_level = "warn"`; entry at `info` | Entry silently discarded; queue size unchanged | Boundary: below threshold |
| TV-20-002-multi | Destinations: A `min_level = "error"`, B `min_level = "info"`; `info` entry emitted | Entry in B queue only; A queue unchanged | EC-20-005 |
| TV-20-002-default | Destination with no `min_level`; `debug` entry emitted | Entry discarded (default `min_level = "info"`); no queue write | EC-20-008 |
| TV-20-002-trace | Destination `min_level = "trace"`; entry at `trace` | Entry enqueued | EC-20-006 |

## Verification Properties

| VP ID | Description | Verification Method |
|-------|-------------|---------------------|
| VP-TBD-20-002 | For every `(entry.level, destination.min_level)` pair, enqueue/discard decision is deterministic and matches level-rank ordering; no entry below threshold appears in any destination queue | Unit test (proptest over all 5×5 level combinations) |

## Related BCs

- BC-2.20.001 — Recursive Prevention (filtering occurs before queue write; filters reduce recursion risk)
- BC-2.20.003 — Queue Cap (filtering reduces queue pressure; complementary overflow protection)
- BC-2.20.004 — Credential Resolution (filtering independent of credential resolution; filter applies first)

## Architecture Anchors

- `specs/architecture/observability.md` §External Log Forwarding — `min_level` field in TOML examples
- `specs/architecture/observability.md` §Forwarding Guarantees — "Per-destination min_level"
- `specs/architecture/observability.md` §What Each Level Provides — level definitions

## Story Anchor

S-5.09 — prism-mcp: External Log Forwarding Subsystem

## VP Anchors

TBD — unit test in `tests/log_forwarding_tests.rs`

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-035 (Diagnostic Log Forwarding) |
| ADR | observability.md §Forwarding Guarantees |
| Story | S-5.09 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.1 | pass-80-follow-on | 2026-04-21 | product-owner | Re-anchored CAP-025 → CAP-035 (business-analyst created CAP-035 post-hoc per pass-80 F80-002 follow-on); removed Capability Anchor Note; added capabilities.md to inputs |
| 1.0 | pass-80-remediation | 2026-04-21 | product-owner | Initial contract — F80-002 gap closure |
