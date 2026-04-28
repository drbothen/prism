Output docs/demo-evidence/S-3.0.01/AC-2-fmt-bad.gif
Output docs/demo-evidence/S-3.0.01/AC-2-fmt-bad.webm

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1000
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Require vhs
Require cargo

Hide
Type "cd /tmp/prism-demo-fmt-bad && clear"
Enter
Sleep 2s
Show

Type "# AC-2: misformatted .rs -- lefthook fmt (cargo fmt --all --check) exits non-zero"
Enter
Sleep 500ms
Type "cat src/lib.rs"
Enter
Sleep 1s
Type "cargo fmt --all --check && echo exit_code=0 || echo exit_code=1"
Enter
Sleep 8s
Sleep 2s
