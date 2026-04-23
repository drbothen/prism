# AC-1: All 6 clones start on configured ports; URL table printed with 6 entries.
#
# Drives the AC-1 integration test which calls DemoHarness::start_all() and
# print_url_table() using ephemeral ports, asserting all 6 pairs are bound.

Output AC-1-all-clones-start.gif
Output AC-1-all-clones-start.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 700
Set FontSize 14
Set Padding 20
Set WaitTimeout 60s
Set Shell "bash"

Hide
  Type "cd /Users/jmagady/Dev/prism/.worktrees/S-6.20-dtu-demo-server"
  Enter
  Sleep 500ms
Show

Type "# AC-1: All 6 clones start on configured ports (URL table shows 6 entries)"
Enter
Sleep 500ms

Type "cargo test --test ac_1_all_clones_start --features dtu -- --nocapture 2>&1 | tail -30"
Enter
Wait+Screen /test result/
Sleep 2s
