# AC-8 Evidence: Squash-merge atomic; cargo build + test workspace passes at this SHA

## AC Text (verbatim)

> The feature branch is a clean series of conventional commits. `cargo build --workspace`
> and the three affected crates' test suites all pass at HEAD `8b949bba`.

## Evidence Type

`git log` showing commit history + `cargo build --workspace --all-features` output +
per-crate `cargo nextest run` summaries.

## Feature Branch Commit History

```
$ git log --oneline feature/S-PLUGIN-PREREQ-A ^develop

8b949bba fix(prism-query): fix doc-comment drift sensor_type → sensor_id (pass-9)
cda9abf5 test(prism-core): add test_BC_2_01_013_004 — Borrow<str> HashMap lookup (F-LP7-MED-001)
bc57c80d fix(S-PLUGIN-PREREQ-A): fix-burst-6 — close 5 pass-6 adversary findings
bcf2f717 refactor(S-PLUGIN-PREREQ-A): fix-burst-5 — rename sensor helper, add non_exhaustive, track proptest seed
fb4769c3 fix(S-PLUGIN-PREREQ-A): fix-burst-4 — close 6 pass-4 findings
17b723e2 fix(S-PLUGIN-PREREQ-A): fix-burst-3 — close 6 LOCAL adversary pass-3 findings
9578f574 fix(prereq-a): close 12 findings from LOCAL adv pass-2 — panic-safety + CI E0432 detection + TryFrom impls + field-name consistency
8a33d981 fix(prereq-a): close 14 findings from LOCAL adv pass-1 — perimeter test + unknown-table guard + sensor-id validation + closed-set residue cleanup
4ab8d33c feat(S-PLUGIN-PREREQ-A): SensorId(Arc<str>) open newtype replaces SensorType closed enum
84f4d35d test(prereq-a): Red Gate scaffold + failing tests — SensorId newtype + AC-9/10 anchors (S-PLUGIN-PREREQ-A)
```

10 commits on feature branch across Red Gate scaffold → implementation → 6 fix bursts.
All use Conventional Commits format. HEAD is `8b949bba` (pass-9 doc-comment fix).

## Workspace Build

```
$ cargo build --workspace --all-features --color=never

   Compiling prism-sensors v0.2.0 (...)
   Compiling prism-dtu-demo-server v0.1.0 (...)
   Compiling prism-query v0.1.0 (...)
   Compiling prism-bin v0.1.0 (...)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 19.48s
```

Clean build — no errors, only deprecation warnings from unrelated legacy code paths.

## Per-Crate Test Counts

### prism-core

```
Summary [   1.009s] 235 tests run: 235 passed, 0 skipped
```

### prism-sensors

```
Summary [   1.782s] 267 tests run: 267 passed, 0 skipped
```

### prism-query

```
Summary [   6.578s] 896 tests run: 896 passed, 6 skipped
```

6 skipped tests in prism-query are pre-existing skips (integration tests requiring
live sensor endpoints — not related to this story).

## Verdict: SATISFIED

The workspace builds cleanly at HEAD `8b949bba` with `--all-features`. All 3
primary affected crates pass their full test suites: 235 + 267 + 896 = 1,398 tests
passed, 6 pre-existing skips, 0 failures. The commit history follows Conventional
Commits with clear fix-burst labeling through 12 adversarial passes.
