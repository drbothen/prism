Output docs/demo-evidence/S-3.2.06/AC-002-concurrent-orgid.gif
Output docs/demo-evidence/S-3.2.06/AC-002-concurrent-orgid.webm

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Require vhs
Require cargo

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.2.06 && clear"
Enter
Sleep 2s
Show

Type "# S-3.2.06 AC-003: Concurrent OrgId tagging — org_A and org_B incidents distinguished"
Enter
Sleep 500ms
Type "cargo test -p prism-dtu-pagerduty --features dtu --test org_tagging test_BC_3_2_004_ac003_concurrent_incidents_distinguished -- --nocapture 2>&1"
Enter
Sleep 15s
