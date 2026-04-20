---
document_type: remediation-manifest
pass: 60
track: A
producer: story-writer
date: 2026-04-20
findings_addressed: [HIGH-001, MED-001, MED-002, MED-003]
findings_deferred_to_state_manager: [LOW-001]
status: complete
---

# Remediation Pass-60 Track A

Story-writer remediation of pass-60 adversarial findings. State-manager handles LOW-001
(manifest gap), input-hash recompute, STATE.md update, and atomic commit.

---

## HIGH-001: 29 Broken BC Paths in 5 Stories

**Root cause:** Stem-only BC filenames (no slug suffix) in `inputs:` frontmatter. S-5.04
and S-5.08 also had missing `specs/` directory prefix (`.factory/behavioral-contracts/`
instead of `.factory/specs/behavioral-contracts/`).

### S-5.01 — 7 paths fixed

**Before → After** (prefix `.factory/specs/behavioral-contracts/`):

| Before | After |
|--------|-------|
| BC-2.04.014.md | BC-2.04.014-tools-list-changed-notification.md |
| BC-2.10.001.md | BC-2.10.001-server-handler-implementation.md |
| BC-2.10.002.md | BC-2.10.002-tool-registration-via-tool-router.md |
| BC-2.10.003.md | BC-2.10.003-conditional-tool-registration.md |
| BC-2.10.005.md | BC-2.10.005-notifications-tools-list-changed.md |
| BC-2.10.006.md | BC-2.10.006-stdio-transport.md |
| BC-2.10.010.md | BC-2.10.010-graceful-shutdown.md |

Version bumped: 1.6 → 1.7. Changelog row added.

### S-5.02 — 3 paths fixed

**Before → After** (prefix `.factory/specs/behavioral-contracts/`):

| Before | After |
|--------|-------|
| BC-2.10.004.md | BC-2.10.004-client-id-parameter-requirement.md |
| BC-2.10.007.md | BC-2.10.007-structured-error-responses.md |
| BC-2.10.011.md | BC-2.10.011-list-capabilities-meta-tool.md |

Version bumped: 1.2 → 1.4. Duplicate 1.1 rows (B-40 + Wave-5-patch) also renumbered
1.1/1.1 → 1.1/1.2; pass-59-fix row renumbered 1.2 → 1.3. Changelog row added.

### S-5.04 — 5 paths fixed (prefix correction + slug resolution)

**Before** (wrong prefix `.factory/behavioral-contracts/`, stem only):
**After** (correct prefix `.factory/specs/behavioral-contracts/`, full slug):

| Before | After |
|--------|-------|
| .factory/behavioral-contracts/BC-2.08.001.md | .factory/specs/behavioral-contracts/BC-2.08.001-on-demand-connectivity-check.md |
| .factory/behavioral-contracts/BC-2.08.002.md | .factory/specs/behavioral-contracts/BC-2.08.002-auth-validity-check.md |
| .factory/behavioral-contracts/BC-2.08.003.md | .factory/specs/behavioral-contracts/BC-2.08.003-rate-limit-state-detection.md |
| .factory/behavioral-contracts/BC-2.08.004.md | .factory/specs/behavioral-contracts/BC-2.08.004-last-successful-query-timestamp.md |
| .factory/behavioral-contracts/BC-2.08.007.md | .factory/specs/behavioral-contracts/BC-2.08.007-partial-health-status.md |

Version bumped: 1.4 → 1.5. Changelog row added.

### S-5.08 — 2 paths fixed (prefix correction + slug resolution)

**Before** (wrong prefix `.factory/behavioral-contracts/`, stem only):
**After** (correct prefix `.factory/specs/behavioral-contracts/`, full slug):

| Before | After |
|--------|-------|
| .factory/behavioral-contracts/BC-2.08.008.md | .factory/specs/behavioral-contracts/BC-2.08.008-get-diagnostics-tool.md |
| .factory/behavioral-contracts/BC-2.08.009.md | .factory/specs/behavioral-contracts/BC-2.08.009-diagnostic-resource-templates.md |

Version bumped: 1.3 → 1.4 (was already bumped by MED-001 to 1.3; HIGH-001 bumps to 1.4).
Changelog row updated to cover both MED-001 and HIGH-001.

### S-6.04 — 12 paths fixed

**Before → After** (prefix `.factory/specs/behavioral-contracts/`):

| Before | After |
|--------|-------|
| BC-2.03.001.md | BC-2.03.001-credential-store-trait.md |
| BC-2.03.002.md | BC-2.03.002-keyring-backend.md |
| BC-2.03.003.md | BC-2.03.003-encrypted-file-fallback.md |
| BC-2.03.004.md | BC-2.03.004-namespace-isolation.md |
| BC-2.03.005.md | BC-2.03.005-credential-crud-operations.md |
| BC-2.03.006.md | BC-2.03.006-credential-resolution-at-query-time.md |
| BC-2.03.007.md | BC-2.03.007-secret-redaction.md |
| BC-2.03.008.md | BC-2.03.008-credential-name-sanitization.md |
| BC-2.03.009.md | BC-2.03.009-resolve-secret-env-file.md |
| BC-2.03.010.md | BC-2.03.010-credential-access-audit-logging.md |
| BC-2.03.011.md | BC-2.03.011-keyring-startup-probe.md |
| BC-2.03.012.md | BC-2.03.012-backend-selection-fallback.md |

Version bumped: 1.2 → 1.4. Duplicate 1.1 rows (B-40 + B-pre-build-sweep-W7) renumbered
1.1/1.1 → 1.1/1.2; pass-59-fix renumbered 1.2 → 1.3. Changelog row added.

**Total HIGH-001: 5 files, 29 paths resolved.**

---

## MED-001: Duplicate Changelog Version Rows (53 Stories)

**Root cause:** Pre-build sweep (Wave patches) added `| 1.1 | pre-build-sweep |` rows to
stories that already had `| 1.1 | B-40 |` rows (46 files), and in 7 files there were
duplicate `| 1.2 |` rows (B-4x + pre-build-sweep).

**Strategy used:** RENUMBERING (as directed). Rows renumbered sequentially 1.0, 1.1,
1.2, ... in the order they appear (oldest first). New `| N | pass-60-fix |` row inserted
at top (after changelog header/separator) with the next highest version.

### Files Fixed and Renumber Maps

#### 1.1 Duplicate Group (46 files)

| File | Old Rows | New Final Version |
|------|----------|------------------|
| S-0.01-ci-cd-pipeline.md | 1.0,1.1(B-40),1.1(pre-build-sweep),1.2 → 1.0,1.1,1.2,1.3 | 1.4 |
| S-0.02-developer-toolchain.md | 1.0,1.1(B-40),1.1(pre-build-sweep) → 1.0,1.1,1.2 | 1.3 |
| S-1.03-capability-resolution.md | 1.0,1.1,1.1,1.2 → 1.0,1.1,1.2,1.3 | 1.4 |
| S-1.04-ocsf-schema-loading.md | 1.0,1.1,1.1,1.2 → 1.0,1.1,1.2,1.3 | 1.4 |
| S-1.05-ocsf-field-mapping.md | 1.0,1.1,1.1,1.2 → 1.0,1.1,1.2,1.3 | 1.4 |
| S-1.06-credential-store.md | 1.0,1.1,1.1 → 1.0,1.1,1.2 | 1.3 |
| S-1.08-feature-flags.md | 1.0,1.1,1.1 → 1.0,1.1,1.2 | 1.3 |
| S-1.09-confirmation-tokens.md | 1.0,1.1,1.1 → 1.0,1.1,1.2 | 1.3 |
| S-1.10-prompt-injection-defense.md | 1.0,1.1,1.1 → 1.0,1.1,1.2 | 1.3 |
| S-1.11-spec-loading.md | 1.0,1.1,1.1 → 1.0,1.1,1.2 | 1.3 |
| S-1.12-hot-reload.md | 1.0,1.1,1.1 → 1.0,1.1,1.2 | 1.3 |
| S-1.13-sensor-write-specs.md | 1.0,1.1,1.1 → 1.0,1.1,1.2 | 1.3 |
| S-1.14-infusion-specs.md | 1.0,1.1,1.1,1.2 → 1.0,1.1,1.2,1.3 | 1.4 |
| S-2.01-rocksdb-init.md | 1.0,1.1,1.1,1.2 → 1.0,1.1,1.2,1.3 | 1.4 |
| S-2.02-audit-buffer-watchdog.md | 1.0,1.1,1.1 → 1.0,1.1,1.2 | 1.3 |
| S-2.03-decorators-internal-tables.md | 1.0,1.1,1.1 → 1.0,1.1,1.2 | 1.3 |
| S-2.04-audit-construction.md | 1.0,1.1,1.1,1.2 → 1.0,1.1,1.2,1.3 | 1.4 |
| S-2.05-audit-events.md | 1.0,1.1,1.1 → 1.0,1.1,1.2 | 1.3 |
| S-2.06-datasource-trait.md | 1.0,1.1,1.1,1.2 → 1.0,1.1,1.2,1.3 | 1.4 |
| S-2.07-per-sensor-auth.md | 1.0,1.1,1.1 → 1.0,1.1,1.2 | 1.3 |
| S-2.08-event-tables.md | 1.0,1.1,1.1 → 1.0,1.1,1.2 | 1.3 |
| S-3.01-prismql-parser.md | 1.0,1.1,1.1 → 1.0,1.1,1.2 | 1.3 |
| S-3.02-query-materialization.md | 1.0,1.1,1.1,1.2,1.3 → 1.0,1.1,1.2,1.3,1.4 | 1.5 |
| S-3.03-explain-query.md | 1.0,1.1,1.1,1.2 → 1.0,1.1,1.2,1.3 | 1.4 |
| S-3.04-alias-system.md | 1.0,1.1,1.1,1.2 → 1.0,1.1,1.2,1.3 | 1.4 |
| S-3.06-prismql-write-parser.md | 1.0,1.1,1.1,1.2,1.3 → 1.0,1.1,1.2,1.3,1.4 | 1.5 |
| S-3.07-write-execution.md | 1.0,1.1,1.1,1.2,1.3 → 1.0,1.1,1.2,1.3,1.4 | 1.5 |
| S-3.08-hidden-columns.md | 1.0,1.1,1.1,1.2 → 1.0,1.1,1.2,1.3 | 1.4 |
| S-3.09-query-profiling.md | 1.0,1.1,1.1,1.2 → 1.0,1.1,1.2,1.3 | 1.4 |
| S-3.10-cost-estimation.md | 1.0,1.1,1.1,1.2 → 1.0,1.1,1.2,1.3 | 1.4 |
| S-3.11-in-query-caching.md | 1.0,1.1,1.1,1.2 → 1.0,1.1,1.2,1.3 | 1.4 |
| S-3.12-column-pruning.md | 1.0,1.1,1.1,1.2 → 1.0,1.1,1.2,1.3 | 1.4 |
| S-5.07-multi-repo-git-config.md | 1.0,1.1,1.1,1.2 → 1.0,1.1,1.2,1.3 | 1.4 |
| S-5.08-diagnostics-logs-cli.md | 1.0,1.1,1.1 → 1.0,1.1,1.2 | 1.3 (HIGH-001 bumps to 1.4) |
| S-5.09-external-log-forwarding.md | 1.0,1.1,1.1,1.2,1.3 → 1.0,1.1,1.2,1.3,1.4 | 1.5 |
| S-6.01-cli-startup.md | 1.0,1.1,1.1,1.2,1.3 → 1.0,1.1,1.2,1.3,1.4 | 1.5 (MED-003 bumps to 1.6) |
| S-6.03-installation.md | 1.0,1.1,1.1,1.2 → 1.0,1.1,1.2,1.3 | 1.4 (MED-003 bumps to 1.5) |
| S-6.08-dtu-claroty.md | 1.0,1.1,1.1,1.2,1.3,1.4 → 1.0,1.1,1.2,1.3,1.4,1.5 | 1.6 |
| S-6.09-dtu-cyberint.md | same pattern as S-6.08 | 1.6 |
| S-6.10-dtu-armis.md | same pattern as S-6.08 | 1.6 |
| S-6.14-dtu-threatintel.md | same pattern as S-6.08 | 1.6 |
| S-6.15-dtu-nvd.md | same pattern as S-6.08 | 1.6 |
| S-6.16-dtu-datadog.md | same pattern as S-6.08 | 1.6 |
| S-6.17-dtu-splunk-hec.md | same pattern as S-6.08 | 1.6 |
| S-6.18-dtu-elasticsearch.md | same pattern as S-6.08 | 1.6 |
| S-6.19-dtu-otlp.md | same pattern as S-6.08 | 1.6 |

#### 1.2 Duplicate Group (7 files)

| File | Old Rows | New Final Version |
|------|----------|------------------|
| S-1.15-wasm-runtime.md | 1.2(B-37),1.1(B-36),1.0,1.2(pre-build-sweep) → 1.0,1.1,1.2,1.3 | 1.4 |
| S-3.05-pagination-caching.md | 1.0,1.1,1.2(B-43),1.2(pre-build-sweep),1.3 → 1.0,1.1,1.2,1.3,1.4 | 1.5 |
| S-5.10-audit-trail-forwarding.md | 1.0,1.1(B-41),1.2(pre-build-sweep),1.3,1.4 → 1.0,1.1,1.2,1.3,1.4 | 1.5 |
| S-6.02-e2e-smoke-tests.md | 1.0,1.1,1.2(B-43),1.2(pre-build-sweep),1.3 → 1.0,1.1,1.2,1.3,1.4 | 1.5 (MED-003 bumps to 1.6) |
| S-6.11-dtu-slack.md | 1.0,1.1,1.2(B-44),1.2(pre-build-sweep),1.3,1.4,1.5 → 1.0,1.1,1.2,1.3,1.4,1.5,1.6 | 1.7 |
| S-6.12-dtu-pagerduty.md | same as S-6.11 | 1.7 |
| S-6.13-dtu-jira.md | same as S-6.11 | 1.7 |

**Total MED-001: 53 files renumbered.**

**Ambiguity note:** S-1.15 had rows in non-chronological order in the changelog (B-37
appeared before B-36). The renumbering preserves file order (not chronological order) per
the renumbering approach: whichever row appeared first in the file gets the lower version
number. This is consistent with how other files were treated and preserves the git diff
as minimal.

**Additional files found but NOT in MED-001 scope:** S-5.02 (had duplicate 1.1 rows with
burst labels "B-40" and "Wave-5-patch", not matching the `pre-build-sweep` grep) and
S-6.04 (burst label "B-pre-build-sweep-W7") were handled as a byproduct of HIGH-001
remediation in those files.

---

## MED-002: S-5.09 and S-5.10 Frontmatter Version Out of Sync

**Resolution:** Subsumed by MED-001. After renumbering:
- S-5.09: version bumped to 1.5 (changelog rows renumbered 1.0→1.4, pass-60-fix=1.5)
- S-5.10: version bumped to 1.5 (changelog rows renumbered 1.0→1.4, pass-60-fix=1.5)

Both frontmatter `version:` fields now match the highest (newest) changelog row.

---

## MED-003: `subsystems: []` Contradicts `anchor_subsystem:` (3 Stories)

**Fix:** Populated `subsystems:` to match `anchor_subsystem:` values.

| File | Before | After | anchor_subsystem (reference) |
|------|--------|-------|------------------------------|
| S-6.01-cli-startup.md | subsystems: [] | subsystems: [SS-06, SS-10] | ["SS-06", "SS-10"] |
| S-6.02-e2e-smoke-tests.md | subsystems: [] | subsystems: [SS-06, SS-08, SS-10] | ["SS-06", "SS-08", "SS-10"] |
| S-6.03-installation.md | subsystems: [] | subsystems: [SS-10] | ["SS-10"] |

Version bumps:
- S-6.01: 1.5 → 1.6 (combined with MED-001 pass-60-fix row update)
- S-6.02: 1.5 → 1.6 (combined with MED-001 pass-60-fix row update)
- S-6.03: 1.4 → 1.5 (combined with MED-001 pass-60-fix row update)

---

## Files Modified Summary

### HIGH-001 (5 files)
- `/Users/jmagady/Dev/prism/.factory/stories/S-5.01-mcp-bootstrap.md` → v1.7
- `/Users/jmagady/Dev/prism/.factory/stories/S-5.02-tool-routing.md` → v1.4
- `/Users/jmagady/Dev/prism/.factory/stories/S-5.04-sensor-health.md` → v1.5
- `/Users/jmagady/Dev/prism/.factory/stories/S-5.08-diagnostics-logs-cli.md` → v1.4
- `/Users/jmagady/Dev/prism/.factory/stories/S-6.04-credential-cli.md` → v1.4

### MED-001 (53 files, unique to MED-001 only — excludes HIGH-001 overlap)
All files listed in the renumber maps above.

### MED-003 (3 files, consolidated with MED-001 pass-60-fix rows)
- `/Users/jmagady/Dev/prism/.factory/stories/S-6.01-cli-startup.md` → v1.6
- `/Users/jmagady/Dev/prism/.factory/stories/S-6.02-e2e-smoke-tests.md` → v1.6
- `/Users/jmagady/Dev/prism/.factory/stories/S-6.03-installation.md` → v1.5

**Grand total: 58 story files modified.**

---

## Deferred to State-Manager

- **LOW-001:** Manifest gap (this document itself — state-manager registers it in INDEX.md)
- **Input-hash recompute:** `input-hash` fields left unchanged; state-manager runs
  `compute-input-hash` on all modified files
- **STATE.md update:** state-manager updates session checkpoint
- **Atomic git commit:** state-manager commits all changes
