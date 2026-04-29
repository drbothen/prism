# AC-002 — Containment + detection isolation (HTTP routes, AC-007 tests)
# Story S-3.2.03: prism-dtu-crowdstrike multi-tenant state segregation
# Traces to: BC-3.2.001 invariant 1
# Verification: cargo test ... test_AC_007 -- --nocapture
# Shows 3 HTTP route tests: contain, lift_containment, patch_detections all return
# their expected status codes (202/202/200) with org-scoped keys.

Output docs/demo-evidence/S-3.2.03/AC-002-containment-detection-isolation-http-routes.gif
Output docs/demo-evidence/S-3.2.03/AC-002-containment-detection-isolation-http-routes.webm

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

Type "cargo test -p prism-dtu-crowdstrike --features dtu --test multi_tenant test_AC_007 -- --nocapture"
Enter
Sleep 10s
Sleep 2s
