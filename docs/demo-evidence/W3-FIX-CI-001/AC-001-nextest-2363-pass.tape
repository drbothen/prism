Output AC-001-nextest-2363-pass.gif
Output AC-001-nextest-2363-pass.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Catppuccin Mocha"
Set Padding 20
Set Shell "zsh"

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/W3-FIX-CI-001"
Enter
Sleep 1s
Show

Type "cargo nextest run --workspace --all-features --no-fail-fast 2>&1 | tail -30"
Enter
Sleep 480s
Sleep 2s
