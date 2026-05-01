Output docs/demo-evidence/S-3.1.06-ImplPhase/AC-004-deprecated-warning.gif
Output docs/demo-evidence/S-3.1.06-ImplPhase/AC-004-deprecated-warning.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 24

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.1.06-ImplPhase && clear"
Enter
Sleep 1s
Show

# ── AC-004: init_registry #[deprecated] fires compile warning ────────────────
Type "# AC-004 (AC-005 in story): init_registry is #[deprecated]"
Enter
Sleep 0.4s

Type "grep -A3 '#\[deprecated' crates/prism-sensors/src/lib.rs | head -5"
Enter
Sleep 1s

Type "# Compile test that calls init_registry — deprecation warning fires"
Enter
Sleep 0.4s

Type "cargo test -p prism-sensors --test org_id_binding test_AC_004 2>&1 | grep -E 'deprecated|ok'"
Enter
Sleep 10s

Type "# AC-004 PASS: deprecation warning fires; legacy path still compiles"
Enter
Sleep 2s
