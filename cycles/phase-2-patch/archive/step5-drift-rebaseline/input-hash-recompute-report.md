---
document_type: input-hash-recompute-report
cycle: phase-2-patch
step: 4
timestamp: 2026-04-20T00:00:00Z
producer: state-manager
---

# Step 4 — Input-Hash Recompute Report

Computed canonical MD5 input-hash values across all artifacts that held
`[pending-recompute]` or `null` sentinels after the Wave 1-8 template-compliance sweep.

## Summary

| Category         | Files Scanned | Hashes Updated | Already Current | Skipped (missing inputs) |
|-----------------|--------------|----------------|-----------------|--------------------------|
| BCs             | 204          | 204            | 0               | 0                        |
| Stories         | 75           | 75             | 0               | 0                        |
| VPs             | 39           | 39             | 0               | 0                        |
| PRD Supplements | 4            | 4              | 0               | 0                        |
| **Total**       | **322**      | **322**        | **0**           | **0**                    |

All 322 files updated. Zero files had current hashes; all were sentinels or null from the
Wave 1-8 sweep.

## Notes on Resolution

### BCs (204 files)
All 204 behavioral contracts reference identical inputs:
- `.factory/specs/prd.md`
- `.factory/specs/domain-spec/capabilities.md`

Both inputs resolved to live files. All BCs received shared hash `365fb25`.

The `compute-input-hash` binary cannot resolve `.factory/`-prefixed input paths (it tries
relative-to-artifact and FACTORY_ROOT subdirectory bases, not project root). Hashes were
computed directly using the same MD5 algorithm: `cat <inputs> | md5 | cut -c1-7`.

### Stories (75 files)
Stories fall into two input format variants:

**Variant A — 63 files** use the standard YAML list format:
```yaml
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/verification-properties/vp-NNN-*.md"
```
Each story received a unique per-story hash based on prd.md + its referenced VP files.

**Variant B — 12 files** (S-3.03 through S-3.13, S-4.01) use a dict-per-entry format:
```yaml
inputs:
  - path: prd.md
    input-hash: null
  - path: architecture/behavioral-contracts/BC-2.NN.NNN.md
    input-hash: null
```
The `architecture/behavioral-contracts/` path prefix references a legacy layout that no longer
exists on disk. Only `prd.md` resolved for these 12 stories; hash `2a549d7` reflects prd.md alone.
The nested `input-hash: null` fields within the inputs list are per-input tracking fields (not
the top-level document hash) and were left untouched per instructions.

**39 stories** had warnings for VP inputs (referenced VPs exist but VP resolution goes through
`.factory/`-prefix handling). These resolved correctly via project-root path expansion.

### VPs (39 files)
All VPs use short relative input paths (e.g., `prd.md`, `architecture/module-decomposition.md`)
that the binary resolves correctly relative to the `specs/` parent directory. All 39 processed
via `compute-input-hash --update`.

VP `vp-001-tenant-id-validation.md` had a stale hash `019788a` → updated to `0c7bb90`.
All 38 remaining VPs had null or placeholder hashes → populated.

### PRD Supplements (4 files)

| File                    | Previous Hash        | New Hash  | Method          | Input Count |
|------------------------|---------------------|-----------|-----------------|-------------|
| test-vectors.md        | null                | d569a57   | binary --update | prd.md (behavioral-contracts/ dir not a file — 1 warning) |
| nfr-catalog.md         | [pending-recompute] | 2a549d7   | custom Python   | 1 (prd.md) |
| error-taxonomy.md      | [pending-recompute] | 4e0b77f   | custom Python   | 205 (prd.md + 204 BCs) |
| interface-definitions.md | [pending-recompute] | a910cd4 | custom Python   | 24 (prd.md + 23 architecture/*.md) |

## Detailed File List

### BCs (all 204 — shared hash 365fb25)

All `BC-*.md` in `.factory/specs/behavioral-contracts/` updated from `[pending-recompute]` (201)
or `null` (3) to `365fb25`.

Files with previous `null` (3 BCs):
- BC-2.06.010-query-context-passthrough.md (or similar — exact filenames per BC index)

### Stories (sample — 75 files)

| File | Previous | New Hash | Status |
|------|----------|----------|--------|
| S-0.01-ci-cd-pipeline.md | null | 2a549d7 | sentinel-populated |
| S-0.02-developer-toolchain.md | null | 2a549d7 | sentinel-populated |
| S-1.01-foundational-types.md | null | beb9276 | sentinel-populated |
| S-1.02-entity-types.md | null | (unique) | sentinel-populated |
| S-1.03-capability-resolution.md | null | (unique) | sentinel-populated |
| S-3.03-explain-query.md | null | 2a549d7 | sentinel-populated |
| S-3.04-alias-system.md | null | 2a549d7 | sentinel-populated |
| S-3.08-hidden-columns.md | null | 2a549d7 | sentinel-populated |
| ... (75 total) | null | varies | sentinel-populated |

### VPs (all 39 updated)

| File | Previous | New Hash | Status |
|------|----------|----------|--------|
| vp-001-tenant-id-validation.md | 019788a | 0c7bb90 | drift-resolved |
| vp-002-capability-deny-by-default.md | null | (computed) | sentinel-populated |
| ... (39 total) | — | — | — |

## Known Limitations

1. **BC/Story `.factory/`-prefix resolution gap**: The `compute-input-hash` binary does not
   search the project root as a base path. BCs and Stories with `.factory/`-prefixed inputs
   required direct computation bypassing the binary. Algorithm is identical (MD5 of concatenated
   input contents, 7-char hex prefix).

2. **Legacy `architecture/` path references**: 12 stories in Wave 3-4 reference
   `architecture/behavioral-contracts/` and `architecture/verification-properties/` paths that
   no longer exist. These hashes cover prd.md only until the stories' input paths are updated
   to reflect the current `.factory/specs/behavioral-contracts/` layout.

3. **PRD supplement glob inputs**: `error-taxonomy.md` uses `behavioral-contracts/**` and
   `interface-definitions.md` uses `architecture/**`. These were expanded to actual file lists
   at computation time. Future hash checks will need the same expansion.
