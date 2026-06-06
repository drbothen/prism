Output docs/demo-evidence/OCSF-CLASS-MIGRATION-001/AC-ALL-ocsf-class-migration-red-gate-suite.gif
Output docs/demo-evidence/OCSF-CLASS-MIGRATION-001/AC-ALL-ocsf-class-migration-red-gate-suite.webm

Require cargo
Require rg

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set WindowBar Colorful

# --- Setup: navigate to worktree and pre-warm build cache ---
Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/OCSF-CLASS-MIGRATION-001"
Enter
Sleep 1s
Type "cargo build -p prism-ocsf --tests --quiet 2>/dev/null"
Enter
Sleep 30s
Type "clear"
Enter
Sleep 500ms
Show

# --- AC-001: Grep audit — zero security_finding in production TOMLs ---
Type "# AC-001: no ocsf_class=security_finding in production TOMLs"
Enter
Sleep 400ms
Type "rg 'ocsf_class.*security_finding' crates/prism-sensors/specs/ || echo 'PASS: zero matches'"
Enter
Sleep 1s
Type "rg 'ocsf_class.*detection_finding' crates/prism-sensors/specs/"
Enter
Sleep 1s
Type "cargo nextest run -p prism-ocsf -E 'test(test_BC_2_02_012_no_production_toml_uses_security_finding)' 2>&1"
Enter
Wait+Line /passed/
Sleep 1s

# --- AC-002: detection_finding → 2004, no WARN ---
Type "# AC-002: detection_finding -> Ok(2004), no WARN"
Enter
Sleep 400ms
Type "cargo nextest run -p prism-ocsf -E 'test(test_BC_2_02_012_select_by_class_name_detection_finding_returns_2004_no_warn)' 2>&1"
Enter
Wait+Line /passed/
Sleep 1s

# --- AC-003: security_finding → 2004 with deprecation WARN ---
Type "# AC-003: security_finding -> Ok(2004) + ocsf.deprecated_class_alias WARN"
Enter
Sleep 400ms
Type "cargo nextest run -p prism-ocsf -E 'test(test_BC_2_02_012_select_by_class_name_security_finding_returns_2004_with_warn)' 2>&1"
Enter
Wait+Line /passed/
Sleep 1s

# --- AC-004: select() path never returns 2001 ---
Type "# AC-004: INV-NO-2001-SELECT-PATH enforced"
Enter
Sleep 400ms
Type "cargo nextest run -p prism-ocsf -E 'test(test_BC_2_02_012_select_path_no_token_returns_2001)' 2>&1"
Enter
Wait+Line /passed/
Sleep 1s

# --- AC-005: no stale 2001 assertions ---
Type "# AC-005: no stale class_uid==2001 assertions in workspace"
Enter
Sleep 400ms
Type "cargo nextest run -p prism-ocsf -E 'test(test_BC_2_02_012_no_stale_2001_assertions_in_workspace)' 2>&1"
Enter
Wait+Line /passed/
Sleep 2s
