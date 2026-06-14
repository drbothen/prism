Output docs/demo-evidence/S-DEMO-004/AC-001-010-e2e-multi-org-full-run.gif
Output docs/demo-evidence/S-DEMO-004/AC-001-010-e2e-multi-org-full-run.webm

Require cargo
Require cargo-nextest

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 13
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set PlaybackSpeed 1

# S-DEMO-004: Multi-Org × Multi-Sensor Isolation Smoke Test
# AC-001..AC-010: full e2e-multi-org suite (10/10 PASS)
Hide
Type "clear"
Enter
Sleep 200ms
Show

Type "# S-DEMO-004: Multi-Org x Multi-Sensor Isolation Smoke Test"
Enter
Sleep 400ms
Type "# 3 orgs x mixed sensors - 8 DTU clone instances - 10 isolation ACs"
Enter
Sleep 600ms
Type "cargo nextest run -p prism-bin --profile e2e-multi-org --run-ignored ignored-only"
Enter
Sleep 60s
