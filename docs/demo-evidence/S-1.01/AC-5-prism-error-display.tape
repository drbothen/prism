# AC-5: PrismError Display begins with structured error code prefix
# Acceptance criteria: Every PrismError variant Display starts with its error code token

Output docs/demo-evidence/S-1.01/AC-5-prism-error-display.gif
Output docs/demo-evidence/S-1.01/AC-5-prism-error-display.webm

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Catppuccin Mocha"
Set Width 1200
Set Height 900
Set FontSize 14
Set Padding 20
Set Shell "zsh"

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-1.01-foundational-types"
Enter
Sleep 2s
Show

Type "/opt/homebrew/bin/cargo test --test ac_5_prism_error_display -p prism-core"
Enter
Sleep 15s
