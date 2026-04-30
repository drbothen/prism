# AC-004: Unknown slug in with_failure deferred to build() as Err(UnknownOrg)
#
# Runs the unknown-slug deferral tests: calling with_failure for an unregistered slug
# must NOT panic at the call site. The error is deferred to build(), which returns
# Err(HarnessError::UnknownOrg). The builder call is infallible.
# Traces to: BC-3.6.001 EC-001, S-3.3.05 story EC-001

Output AC-004-unknown-slug-deferred.gif
Output AC-004-unknown-slug-deferred.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 700
Set FontSize 14
Set Padding 20
Set WaitTimeout 120s
Set Shell "bash"

Hide
  Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.3.05"
  Enter
  Sleep 500ms
Show

Type "# AC-004: Unknown slug deferred to build() — no panic at with_failure call site"
Enter
Sleep 500ms

Type "cargo test -p prism-dtu-harness --features dtu --test builder_ergonomics_test deferred_to_build -- --nocapture 2>&1 | tail -25"
Enter

Wait+Screen /test result/
Sleep 2s
