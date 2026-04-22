# Pass-74 VP-TBD Application Manifest

**Date:** 2026-04-20
**Agent:** product-owner
**Source:** vp-tbd-decision-matrix.md v1.1, Pass-74 Extension (SS-14/15/16)
**Files touched:** 32 BC files (BC-2.14.011 absent from repo — not processed)

---

## Summary

| Category | Count |
|----------|-------|
| ADD-VP | 9 |
| MARK-NONE | 22 |
| DEFER | 1 |
| **Total** | **32** |

All files bumped from version 1.2 → 1.3. Changelog row added at top of each file's ## Changelog section.

---

## ADD-VP (9 BCs)

| BC ID | File | VP Assigned | Method | Priority |
|-------|------|-------------|--------|----------|
| BC-2.14.002 | BC-2.14.002-case-state-transitions.md | VP-051 | Kani | P0 |
| BC-2.14.003 | BC-2.14.003-update-case-tool.md | VP-052 | Proptest | P0 |
| BC-2.14.006 | BC-2.14.006-disposition-assignment.md | VP-053 | Kani | P0 |
| BC-2.14.008 | BC-2.14.008-mttd-mttr-computation.md | VP-054 | Proptest | P1 |
| BC-2.15.002 | BC-2.15.002-domain-kv-operations.md | VP-055 | Proptest | P1 |
| BC-2.15.004 | BC-2.15.004-audit-buffer-overflow.md | VP-056 | Proptest | P1 |
| BC-2.15.005 | BC-2.15.005-crash-recovery-dirty-bits.md | VP-057 | Kani | P0 |
| BC-2.15.007 | BC-2.15.007-watchdog-query-termination.md | VP-058 | Proptest | P0 |
| BC-2.16.009 | BC-2.16.009-spec-file-validation.md | VP-059 | Proptest | P1 |

---

## MARK-NONE (22 BCs)

| BC ID | File | Justification Summary |
|-------|------|-----------------------|
| BC-2.14.001 | BC-2.14.001-create-case-tool.md | case.write gate covered by VP-002; audit-on-create by VP-033; integration test only |
| BC-2.14.004 | BC-2.14.004-list-cases-tool.md | AND-filter is trivial iterator; truncation metadata is integration behavior |
| BC-2.14.005 | BC-2.14.005-get-case-tool.md | Metric computation covered by VP-054; orphaned-alert is integration behavior |
| BC-2.14.007 | BC-2.14.007-timeline-annotations.md | Immutability is code review; enum rejection is trivial unit test |
| BC-2.14.009 | BC-2.14.009-case-persistence.md | WriteBatch atomicity is RocksDB WAL guarantee; index consistency is integration test |
| BC-2.14.010 | BC-2.14.010-case-metrics-tool.md | Null propagation covered by VP-054; cross-client aggregation is integration test |
| BC-2.14.012 | BC-2.14.012-acknowledge-alert.md | Idempotency is conditional write omission; audit-before-write covered by VP-033 + DI-016 |
| BC-2.15.001 | BC-2.15.001-rocksdb-initialization.md | CF count is startup integration test; exclusive lock is OS-level RocksDB contract |
| BC-2.15.003 | BC-2.15.003-buffered-audit-log-persistence.md | Persist-before-forward covered by VP-033; crash-recovery is restart integration test |
| BC-2.15.006 | BC-2.15.006-resource-watchdog-initialization.md | Override merge is trivial Option::unwrap_or; cannot-disable is hardcoded constant; VP-014/015 cover enforcement |
| BC-2.15.008 | BC-2.15.008-query-denylisting.md | Counter pattern structurally identical to VP-057; denylist-survives-restart is integration test |
| BC-2.15.009 | BC-2.15.009-context-decorator-injection.md | Virtual fields require DataFusion context (integration); decorator-present is struct init (unit test) |
| BC-2.15.010 | BC-2.15.010-decorator-three-phase-model.md | Phase priority is HashMap::extend() merge; stale-on-failure requires live background task |
| BC-2.15.011 | BC-2.15.011-internal-table-registration.md | audit.read gate covered by VP-002; client_id scoping is DataFusion integration test |
| BC-2.16.001 | BC-2.16.001-sensor-spec-file-loading.md | Partial-failure isolation is behavioral loop; OCSF registration is side-effectful; VP-023 covers panic safety |
| BC-2.16.002 | BC-2.16.002-multi-step-fetch-pipeline.md | Fan-out requires HTTP mock; forward-reference scoping covered by VP-059 |
| BC-2.16.003 | BC-2.16.003-column-to-ocsf-mapping.md | Coercion-failure semantically identical to VP-017; cross-sensor requires full query engine integration |
| BC-2.16.004 | BC-2.16.004-rust-escape-hatch.md | catch_unwind is construction guarantee; TOML-only initial sensors is code review invariant |
| BC-2.16.005 | BC-2.16.005-reload-config-tool.md | Fail-closed proven by VP-032; MCP notification is behavioral integration test |
| BC-2.16.006 | BC-2.16.006-arc-swap-config-access.md | Snapshot stability is Arc reference counting (type system); wait-free is library property; VP-032 covers swap-side |
| BC-2.16.007 | BC-2.16.007-sensor-spec-hot-reload.md | In-flight safety covered by VP-032 + BC-2.16.006; cache invalidation is integration test |
| BC-2.16.008 | BC-2.16.008-add-sensor-spec-tool.md | temp+rename atomicity is OS contract; sensor_spec.write gate covered by VP-002 |
| BC-2.16.010 | BC-2.16.010-list-sensor-specs-tool.md | Always-visible is boolean field (unit test); structuredContent is serialization integration test |

---

## DEFER (1 BC)

| BC ID | File | Phase-3 Requirement |
|-------|------|---------------------|
| BC-2.14.013 | BC-2.14.013-auto-case-creation.md | VP-060 proposed pending Phase-3: requires confirming `CaseDedupRegistry::check_and_create()` is exposed as pure state-transition function; alert-before-case ordering must be confirmable as pure sequencing or integration-only. Tracked in Phase-3 story S-4.06. |

---

## Version Changes

All 32 processed files: v1.2 → v1.3

## Verification

No `(placeholder)` rows remain in any of the 32 processed BC files. Each file has exactly one row in ## Verification Properties — either a VP-NNN reference, `(none)` with justification, or `(Phase 3 DEFER)` with tracking note.
