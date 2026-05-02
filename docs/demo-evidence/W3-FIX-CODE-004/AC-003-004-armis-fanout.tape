# AC-003 (CR-012/SEC-P2-001) + AC-004 (CR-013): Armis instance-identity guard + fan_out assert
#
# AC-003: Armis validate_org_id uses instance_org_id != DTU_DEFAULT_INSTANCE_ORG_ID guard.
#         Real-org clones reject missing X-Org-Id with 401 (consistent with Claroty/CrowdStrike).
#         BC-3.5.002 precondition 3
#
# AC-004: fan_out() has debug_assert_eq!(target.org_id, target.spec.org_id) before adapter.fetch().
#         Fires in debug/CI builds; no-op in release.
#         BC-3.2.001 precondition 4
#
# Story: W3-FIX-CODE-004, branch: feature/W3-FIX-CODE-004
# Commits: 360fd53b -> 37088d66 -> cfe585c3

Output AC-003-004-armis-fanout.gif
Output AC-003-004-armis-fanout.webm

Require cargo

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set PlaybackSpeed 1.0
Set WaitTimeout 120s

# --- Setup (hidden) ---
Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/W3-FIX-CODE-004"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

# --- AC-003: show Armis dual-mode guard in source ---
Type "# AC-003 (CR-012/SEC-P2-001): Armis uses instance-identity model"
Enter
Sleep 500ms
Type "grep -n 'DTU_DEFAULT_INSTANCE_ORG_ID\|is_real_org' crates/prism-dtu-armis/src/routes/devices.rs | head -6"
Enter
Sleep 2s

# --- AC-004: show debug_assert_eq in fanout.rs ---
Type "# AC-004 (CR-013): fan_out debug_assert_eq before adapter.fetch()"
Enter
Sleep 500ms
Type "grep -A6 'debug_assert_eq!' crates/prism-sensors/src/fanout.rs | head -8"
Enter
Sleep 2s
