Output docs/demo-evidence/S-DEMO-003/AC-001-demo-setup-toml.gif
Output docs/demo-evidence/S-DEMO-003/AC-001-demo-setup-toml.webm

Require cargo
Require cargo-nextest

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set PlaybackSpeed 1

# AC-001: demo-setup.sh creates valid prism.toml (generate_demo_prism_toml Red Gate test)
# Red Gate test: test_BC_2_06_001_demo_setup_generates_valid_prism_toml
Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-003"
Enter
Sleep 200ms
Type "clear"
Enter
Sleep 200ms
Show

Type "# AC-001: demo-setup.sh generates a schema-valid prism.toml"
Enter
Sleep 500ms

Type "# Red Gate test: test_BC_2_06_001_demo_setup_generates_valid_prism_toml"
Enter
Sleep 500ms

Type "cargo nextest run -p prism-bin -E 'test(BC_2_06_001)'"
Enter
Sleep 8s
