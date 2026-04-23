Output docs/demo-evidence/S-1.13/AC-003-batch-limit-irreversible-warning.gif
Output docs/demo-evidence/S-1.13/AC-003-batch-limit-irreversible-warning.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 24
Set Shell zsh

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-1.13-sensor-write-specs"
Enter
Sleep 2s
Show

Type "# AC-3: batch_limit=0 + risk_tier=irreversible -> warning emitted, spec loads Ok"
Enter
Sleep 500ms
Type "cargo test -p prism-spec-engine --test write_endpoint_tests batch_limit_zero 2>&1 | tail -8"
Enter
Sleep 8s
Sleep 2s
