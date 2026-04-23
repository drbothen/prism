# AC-8: Feature gate -- cargo build without --features dtu fails with required-features error;
#        with --features dtu it succeeds.
#
# Shows the Cargo "required-features" enforcement in action at build time.

Output AC-8-feature-gate.gif
Output AC-8-feature-gate.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 700
Set FontSize 14
Set Padding 20
Set Shell "bash"

Hide
  Type "cd /Users/jmagady/Dev/prism/.worktrees/S-6.20-dtu-demo-server"
  Enter
  Sleep 500ms
Show

Type "# AC-8: Feature gate -- without --features dtu, binary is skipped"
Enter
Sleep 500ms

Type "cargo build -p prism-dtu-demo-server 2>&1 | grep -E 'required-features|Finished|error'"
Enter
Wait+Screen /required-features|Finished/
Sleep 1s

Type ""
Enter
Sleep 300ms

Type "# With --features dtu the binary builds successfully"
Enter
Sleep 500ms

Type "cargo build -p prism-dtu-demo-server --features dtu 2>&1 | grep -E 'Finished|Compiling prism-dtu-demo-server'"
Enter
Wait+Screen /Finished/
Sleep 2s
