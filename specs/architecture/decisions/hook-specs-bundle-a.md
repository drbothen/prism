---
document_type: hook-specifications
bundle: bundle-A-cleanup-2026-05-08
status: SPEC-ONLY
implementation_bundle: bundle-A.2
date: "2026-05-08"
producer: architect
references_adrs: [ADR-020, ADR-021]
references_policies: [POL-12, POL-13, POL-14, POL-15, POL-16]
---

# Hook Specifications: Bundle A (Status-Taxonomy Reform)

**SPEC ONLY — no implementation in this file.**

These hook specifications define the behavior, inputs, logic, and outputs for the five hooks
referenced by POL-12 through POL-16. Actual bash/Rust implementations are delivered in
Bundle A.2. This document serves as the design contract so Bundle A.2 implementers have
precise behavioral requirements.

---

## Hook 1: `hooks/check-stub-residue.sh`

**Enforces:** POL-12 (`production_stub_residue_blocks_merge`)
**References:** ADR-020 §Decision 2

### Trigger

- **Lefthook stage:** `pre-push` (runs on the pushing developer's machine before a PR is opened)
- **Factory stage:** adversary pre-pass check (invoked in the adversary dispatch before
  evaluating a story marked `status: merged`)
- **Optionally:** wave-gate hook as a workspace-wide sweep

### Input

- Changed file set (from `git diff --name-only origin/develop...HEAD` for pre-push mode)
- Or: story ID and its delivery file list (from story frontmatter `file_structure_requirements:`)
  for adversary mode
- Story frontmatter `status:` field for the story under review

### Logic (pseudocode)

```
FUNCTION check_stub_residue(mode):
  IF mode == "pre-push":
    files = git_changed_files(base="origin/develop")
    story_status = infer_status_from_changed_story_files(files)
  ELIF mode == "adversary":
    files = story.file_structure_requirements
    story_status = story.frontmatter.status

  IF story_status NOT IN {"merged", "partial-merge"}:
    EXIT 0  # only check stories claiming a merged state

  stub_hits = []
  FOR file IN files:
    IF file NOT matches "crates/*/src/**/*.rs":
      CONTINUE
    IF file matches "**/tests/**" OR "**/proofs/**":
      CONTINUE
    raw_hits = rg(
      patterns=[
        r'todo!\(',
        r'unimplemented!\(',
        r'panic!\(\s*"[^"]*(?:TODO|todo|not yet|stub|unimplemented|FIXME)'
      ],
      file=file
    )
    FOR hit IN raw_hits:
      IF NOT in_test_context(hit):  # not inside #[cfg(test)] or mod tests {}
        stub_hits.append(hit)

  IF story_status == "merged" AND len(stub_hits) > 0:
    EMIT findings (file:line, matched text)
    EXIT 2  # FAIL

  IF story_status == "partial-merge":
    unannotated = [h for h in stub_hits if NOT has_td_annotation(h)]
    IF len(unannotated) > 0:
      EMIT "partial-merge stub sites missing TD annotation" + unannotated
      EXIT 2  # FAIL

  EXIT 0  # clean
```

`in_test_context(hit)`: walks up from the hit line to find `#[cfg(test)]` or `mod tests {`
as the enclosing scope. Does NOT use regex alone — requires simple bracket-balancing or
relies on a pre-pass that marks test-scope line ranges.

`has_td_annotation(hit)`: checks the line immediately before the stub for a comment matching
`// TODO(TD-[A-Z0-9-]+):` or an allowlist entry in `.factory/stub-residue-allowlist.yaml`.

### Output / exit code

```
EXIT 0  — no violations found; hook passes
EXIT 2  — violations found; one finding per line to stderr:
          STUB-RESIDUE <file>:<line> [status:<status>] "<matched text>"
```

### False-positive escape valve

`.factory/stub-residue-allowlist.yaml` — a YAML list of explicitly approved stub sites:

```yaml
allowlist:
  - file: "crates/prism-dtu-common/src/clone.rs"
    line: 79
    pattern: "unimplemented!()"
    td: "TD-FUTURE-001"
    approved_by: "human"
    reason: "trait-default overridden in all concrete clones; never called in practice"
```

Allowlist entries suppress the finding for that exact file:line. The allowlist file is
committed to the repo and reviewed at wave gates.

---

## Hook 2: `hooks/check-story-index-consistency.sh`

**Enforces:** POL-13 (`story_frontmatter_index_consistency`)
**References:** ADR-020 §Decision 4 and §Decision 5

### Trigger

- **Lefthook stage:** `pre-push`
- **Factory stage:** pre-burst check (runs before any burst that touches `.factory/stories/`
  or STORY-INDEX.md)
- **State-manager:** automatically runs this check before finalizing any burst commit

### Input

- All `.factory/stories/*.md` files (excluding STORY-INDEX.md itself)
- `.factory/stories/STORY-INDEX.md`
- No file-set filtering — always operates on the complete stories directory

### Logic (pseudocode)

```
FUNCTION check_story_index_consistency():
  VALID_STATUSES = {draft, ready, in-progress, partial-merge, merged, retired}

  story_statuses = {}
  FOR file IN glob(".factory/stories/*.md"):
    IF file == "STORY-INDEX.md":
      CONTINUE
    fm = parse_frontmatter(file)
    story_id = fm.story_id  # or derive from filename slug
    status = fm.status
    IF status NOT IN VALID_STATUSES:
      EMIT "INVALID-STATUS <story_id> <file>: '${status}' not in valid enum"
      violations++
    story_statuses[story_id] = status

  index_statuses = parse_story_index_status_column("STORY-INDEX.md")
  # Returns: {story_id → normalized_status} where normalized_status is
  # inferred from the [MERGED ...], [PARTIAL-MERGE ...], [READY ...] annotations
  # or the bare status column cell.

  FOR story_id, fm_status IN story_statuses:
    IF story_id NOT IN index_statuses:
      EMIT "UNREGISTERED <story_id>: exists in stories/ but has no STORY-INDEX row"
      violations++
      CONTINUE
    idx_status = index_statuses[story_id]
    IF fm_status != idx_status:
      EMIT "STATUS-MISMATCH <story_id>: frontmatter='${fm_status}' index='${idx_status}'"
      violations++

  FOR story_id IN index_statuses:
    IF story_id NOT IN story_statuses:
      EMIT "ORPHAN-INDEX-ROW <story_id>: STORY-INDEX has row but no story file found"
      violations++

  IF violations > 0:
    EXIT 2
  EXIT 0
```

### Output / exit code

```
EXIT 0  — all story files match STORY-INDEX
EXIT 2  — mismatches found; one line per violation:
          STATUS-MISMATCH   <story_id> frontmatter=<fm_status> index=<idx_status>
          INVALID-STATUS    <story_id> <file> value=<invalid_value>
          UNREGISTERED      <story_id> <file>
          ORPHAN-INDEX-ROW  <story_id>
```

### False-positive escape valve

None. This hook has zero legitimate false positives: every story file must have a STORY-INDEX
row, and every status must agree. If a story is in transition, state-manager must update both
atomically before the burst lands.

---

## Hook 3: `hooks/check-bc-promotion.sh`

**Enforces:** POL-14 (`bc_vp_promotion_on_anchor_merge`)
**References:** ADR-021 §Decision 3

### Trigger

- **Factory stage:** post-merge audit (runs after state-manager closes a story at `status: merged`)
- **Scheduled:** monthly audit cadence (ADR-021 §Decision 4)
- **NOT lefthook:** BC promotion is a factory-internal concern; it runs after the story merge,
  not before the push

### Input

- All `.factory/stories/*.md` with `status: merged` (fully-merged, not partial-merge)
- `.factory/specs/behavioral-contracts/BC-INDEX.md`
- All BC files under `.factory/specs/behavioral-contracts/`

### Logic (pseudocode)

```
FUNCTION check_bc_promotion():
  merged_stories = [s for s in all_stories() if s.frontmatter.status == "merged"]

  expected_active = set()
  FOR story IN merged_stories:
    FOR bc_id IN story.frontmatter.behavioral_contracts:
      expected_active.add(bc_id)

  violations = []
  FOR bc_id IN expected_active:
    bc_file = resolve_bc_file(bc_id)
    bc_status = parse_frontmatter(bc_file).status
    IF bc_status == "draft":
      # Check if another merged story also owns this BC — dual-ownership OK
      owners = [s for s in merged_stories if bc_id in s.frontmatter.behavioral_contracts]
      # Should have at least 1 merged owner (which we already know from expected_active)
      violations.append({bc_id, bc_status, "missed-promotion"})

  # Reverse check: active BCs must have at least one merged anchor story
  active_bcs = [bc for bc in all_bcs() if bc.frontmatter.status == "active"]
  FOR bc IN active_bcs:
    owners = [s for s in merged_stories if bc.id in s.frontmatter.behavioral_contracts]
    IF len(owners) == 0:
      # Active BC with no merged anchor — either a partial-merge owner (OK if another
      # story truly merged it) or an orphan (violation)
      partial_owners = [s for s in all_stories()
                        if bc.id in s.frontmatter.behavioral_contracts
                        and s.frontmatter.status == "partial-merge"]
      IF len(partial_owners) == 0:
        violations.append({bc.id, "active", "orphan-no-merged-anchor"})

  EMIT all violations
  IF len(violations) > 0:
    EXIT 2
  EXIT 0
```

### Output / exit code

```
EXIT 0  — all BCs correctly promoted
EXIT 2  — violations found:
          MISSED-PROMOTION  <bc_id>: merged story <story_id> exists; BC still draft
          ORPHAN-ACTIVE-BC  <bc_id>: status=active but no merged anchor story found
```

### False-positive escape valve

BC-file frontmatter `promotion_deferred: true` with a `deferral_reason:` field suppresses
the missed-promotion finding for that BC. Deferral requires human approval (recorded in the
BC file's `## Traceability` section). The monthly audit flags deferrals older than 90 days.

---

## Hook 4: `skills/audit-runtime-wiring/`

**Enforces:** POL-15 (`runtime_wiring_required_for_accepted_adrs`)
**References:** ADR-021 §Decision 4; audit cross-pattern finding #4

### Trigger

- **Scheduled:** monthly (not lefthook; requires full cargo metadata + grep pass)
- **Wave gate:** the wave-gate hook invokes this skill before declaring a wave complete
- **On-demand:** `/vsdd-factory:audit-runtime-wiring` invocation

Note: this is a SKILL, not a bash hook. It requires cargo metadata and multi-file analysis
that exceeds what a fast git hook should do. It is listed under `lint_hook:` in POL-15 as
`skills/audit-runtime-wiring/` to distinguish it from bash hook scripts.

### Input

- `.factory/specs/architecture/ARCH-INDEX.md` ADR Registry (for accepted ADRs)
- Workspace `Cargo.toml` + `cargo metadata --format-version 1 --no-deps` (for binary targets)
- Per-ADR deliverable table (see Logic below for how deliverables are declared)

### ADR deliverable declaration format

Each ADR with crate-level deliverables should carry a `runtime_deliverables:` frontmatter
field (introduced by this hook spec; existing ADRs backfilled in Bundle A.2):

```yaml
runtime_deliverables:
  - type: "Server::new"
    crate: "prism-mcp"
    description: "MCP server initialization"
  - type: "ActionDeliveryEngine::new"
    crate: "prism-operations"
    description: "Action delivery engine start"
```

### Logic (pseudocode)

```
FUNCTION audit_runtime_wiring():
  accepted_adrs = [adr for adr in all_adrs() if adr.status == "ACCEPTED"]
  adrs_with_deliverables = [adr for adr in accepted_adrs
                             if adr.frontmatter.runtime_deliverables]

  binary_srcs = cargo_metadata_bin_targets()  # list of src/main.rs paths

  for adr in adrs_with_deliverables:
    for deliverable in adr.runtime_deliverables:
      found = False
      for bin_src in binary_srcs:
        if rg(pattern=deliverable.type, file=bin_src, recursive=True):
          found = True
          BREAK
      if NOT found:
        wiring_deferred = adr.frontmatter.get("wiring_deferred_to")
        if wiring_deferred:
          EMIT "DEFERRED <adr_id> <deliverable.type> deferred to <wiring_deferred_to>"
        else:
          EMIT "UNWIRED <adr_id> <deliverable.type> not reachable from any binary"
          violations++

  IF violations > 0:
    EXIT 2
  EXIT 0
```

### Output / exit code

```
EXIT 0  — all accepted ADR deliverables are wired into at least one binary
EXIT 2  — unwired deliverables found:
          UNWIRED   ADR-<id> <type>: not found in any bin/ target src_path
          DEFERRED  ADR-<id> <type>: deferred to story <story_id>
```

### False-positive escape valve

`wiring_deferred_to: "<story-id>"` in ADR frontmatter suppresses the violation and emits a
DEFERRED notice instead. The story referenced must exist and be non-retired. Deferred-to stories
that reach `status: merged` without wiring the deliverable re-trigger the violation.

---

## Hook 5: `hooks/check-inverted-polarity.sh`

**Enforces:** POL-16 (`no_inverted_polarity_tests_outside_red_gate`)
**References:** ADR-020 §Decision 2; F-AUD-D3-01

### Trigger

- **Lefthook stage:** `pre-push`
- **Factory stage:** adversary pre-pass (when reviewing any story with `status: merged` or
  `status: partial-merge`)

### Input

- All Rust test files in `crates/*/src/**/*.rs` and `crates/*/tests/**/*.rs`
- Story frontmatter (to determine story status for each crate's tests)

### Logic (pseudocode)

```
FUNCTION check_inverted_polarity(mode):
  stub_panic_pattern = r'#\[should_panic\s*\([^)]*expected\s*=\s*"[^"]*(?:not yet implemented|not implemented|TODO|stub|unimplemented)'

  hits = rg(pattern=stub_panic_pattern, globs=["crates/*/src/**/*.rs", "crates/*/tests/**/*.rs"])

  violations = []
  FOR hit IN hits:
    story = infer_owning_story(hit.file)  # by crate name → story crate: field
    IF story IS None:
      # Could not identify owning story; record as advisory
      EMIT "ADVISORY-NO-STORY <file>:<line> could not identify owning story"
      CONTINUE
    status = story.frontmatter.status
    IF status IN {"merged", "partial-merge"}:
      # Check if there is a suppress annotation on the previous line
      prev_line = get_line(hit.file, hit.line - 1)
      IF "// POL-16-OK:" IN prev_line:
        EMIT "SUPPRESSED <file>:<line> reason: ${prev_line}"
        CONTINUE
      violations.append({hit.file, hit.line, status, story.id})
    # status in {draft, ready, in-progress} → permitted transiently; do not flag

  FOR v IN violations:
    EMIT "INVERTED-POLARITY <v.file>:<v.line> story=<v.story_id> status=<v.status>"

  IF len(violations) > 0:
    EXIT 2
  EXIT 0
```

`infer_owning_story(file)`: maps a file path to a story by checking which story's `crate:`
field matches the crate extracted from the file path (e.g., `crates/prism-spec-engine/...`
maps to stories with `crate: prism-spec-engine`). If multiple stories match, use the one
with the highest status (merged > partial-merge > in-progress > ready > draft).

### Output / exit code

```
EXIT 0  — no inverted-polarity tests in merged/partial-merge stories
EXIT 2  — violations found:
          INVERTED-POLARITY <file>:<line> story=<story_id> status=<status>
          SUPPRESSED        <file>:<line> reason=<suppress_comment>
          ADVISORY-NO-STORY <file>:<line>
```

### False-positive escape valve

`// POL-16-OK: <justification>` on the line immediately before the `#[should_panic]`
attribute suppresses the finding for that specific test. The justification must explain why
the panic is intentional and not stub residue (e.g., "type-system invariant — panics on
impossible state, not on missing implementation"). This annotation is reviewed at adversary
passes and wave gates.

---

## Implementation Notes (for Bundle A.2)

- All bash hooks consume the stub-detection YAML config (`templates/stub-detection.yaml`
  from the vsdd-factory plugin) for language-agnostic pattern matching if available;
  fall back to hardcoded Rust patterns if not.
- Hooks 1, 2, and 5 are suitable for `lefthook.yml` registration in the project repo.
  They must be fast (<10s on the changed-file set) to remain viable as pre-push gates.
- Hook 3 (`check-bc-promotion.sh`) is NOT suitable for pre-push (reads all BC files);
  it runs as a post-merge audit step via factory orchestration.
- Hook 4 (`audit-runtime-wiring`) is a multi-minute skill, not a bash hook. It is invoked
  by the orchestrator at wave gates and on schedule.
- All hooks must emit machine-readable output (one finding per line with a consistent prefix)
  so the adversary agent can parse findings without text analysis.
- All hooks must respect `.factory/stub-residue-allowlist.yaml` for suppressions that have
  been human-approved (no silent suppression without an allowlist entry).
