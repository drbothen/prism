# AC-001 — All 14 multi_tenant tests GREEN
# Story S-3.2.03: prism-dtu-crowdstrike multi-tenant state segregation
# Traces to: BC-3.2.001, BC-3.2.003
# Verification: cargo test -p prism-dtu-crowdstrike --features dtu --test multi_tenant

Output docs/demo-evidence/S-3.2.03/AC-001-all-14-multi-tenant-tests-green.gif
Output docs/demo-evidence/S-3.2.03/AC-001-all-14-multi-tenant-tests-green.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set FontSize 14
Set Width 1200
Set Height 700
Set Padding 20
Set Shell "zsh"

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.2.03 && clear"
Enter
Sleep 1s
Show

Type "cargo test -p prism-dtu-crowdstrike --features dtu --test multi_tenant"
Enter
Sleep 10s
Sleep 2s
