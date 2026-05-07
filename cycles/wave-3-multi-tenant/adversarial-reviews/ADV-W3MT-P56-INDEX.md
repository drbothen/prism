---
document_type: adversarial-review-index
level: ops
version: "1.0"
status: in-review
producer: adversary
timestamp: 2026-05-07T00:00:00
phase: maintenance
pass: 56
inputs:
  - PR #132 diff (S-3.05 pagination + caching — 13 files)
  - BC-2.07.001–006 (authoritative spec context)
traces_to: pass-56.md
total_findings: 8
severity_distribution: { CRIT: 0, HIGH: 1, MED: 3, LOW: 4 }
---

# Adversarial Review — Pass 56 (PR #132 diff review — S-3.05 pagination + caching)

## Finding Catalog

| ID | Severity | Category | Title | Status | Depends On | Blocks |
|----|----------|----------|-------|--------|-----------|--------|
| ADV-W3MT-P56-HIGH-001 | HIGH | concurrency / spec-fidelity | put_with_ttl eviction-path race worst-case orphan size understated for multi-eviction paths | open | -- | -- |
| ADV-W3MT-P56-MED-001 | MEDIUM | coverage-gap / missing-edge-cases | No test for next_page with exhausted cursor token → E-QUERY-014 | open | -- | -- |
| ADV-W3MT-P56-MED-002 | MEDIUM | coverage-gap / verification-gaps | BC-2.07.003 absolute TTL invariant test is `#[ignore]` with `todo!()` — zero behavioral coverage | open | -- | -- |
| ADV-W3MT-P56-MED-003 | MEDIUM | missing-edge-cases / performance | moka TTL 300s ceiling causes alert entries (60s TTL) to retain memory 240s extra when not re-accessed | open | -- | -- |
| ADV-W3MT-P56-LOW-001 | LOW | code-quality | Focus area #5 confirmed non-bug — moka does not serve stale alert data (is_expired guards get()) | resolved | -- | -- |
| ADV-W3MT-P56-LOW-002 | LOW | code-quality | Focus area #1 confirmed clean — cursor TTL is absolute from created_at, not sliding | resolved | -- | -- |
| ADV-W3MT-P56-LOW-003 | LOW | code-quality | Focus area #4 confirmed clean — error codes E-QUERY-012/013/014 are monotonic, non-colliding | resolved | -- | -- |
| ADV-W3MT-P56-LOW-004 | LOW | code-quality | vp025_cache_key.rs dynamic_tests still carry stale RED-by-design annotations | open | -- | -- |

## Dependency Graph

```text
All findings are independent — no blocking dependencies.
ADV-W3MT-P56-LOW-001, LOW-002, LOW-003 are informational / confirmed-clean.
```

## Category Groups

| Category | Finding IDs | Can Triage in Parallel? |
|----------|------------|------------------------|
| concurrency / doc accuracy | HIGH-001 | Yes — doc-only fix |
| coverage-gap | MED-001, MED-002 | Yes |
| performance / memory | MED-003 | Yes — doc-only fix recommended |
| code-quality | LOW-004 | Yes |
| confirmed-clean (informational) | LOW-001, LOW-002, LOW-003 | N/A |

## Critical Path to Merge

1. **Resolve HIGH-001 (recommended before merge):** Update `put_with_ttl` doc comment to accurately state worst-case orphan bound is `N_evicted × MAX_ENTRY_BYTES` for over-count partition recovery paths, not a flat 5MB. Doc-only fix — no behavior change.
2. **Resolve MED-001 (recommended before merge):** Add test for `next_page` with exhausted cursor token → `PrismError::CursorTokenUnknown` (E-QUERY-014). Test skeleton provided in pass-56.md.
3. **MED-002 and MED-003** are coverage / documentation gaps — non-blocking for merge but should be captured as TD items.
4. **LOW-004** is a straightforward doc sweep identical to the one done for `pagination_tests.rs` — apply to `vp025_cache_key.rs`.
