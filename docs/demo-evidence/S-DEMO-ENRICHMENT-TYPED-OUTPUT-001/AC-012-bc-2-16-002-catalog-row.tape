# AC-012 — BC-2.16.002 Canonical Structured Event Catalog gains row for infusion.coercion_failed
#
# Demonstrates SAP-1 compliance:
#   Step 1: tracing emission exists in crates/prism-query/src/infusion_udf.rs
#   Step 2: BC-2.16.002 catalog row in .factory/specs/behavioral-contracts/
#   (Both in same commit per SAP-1 standing obligation)
#
# BC: BC-2.16.002 v1.96 SAP-1 Canonical Structured Event Catalog; BC-2.19.001 v2.2 E-INFUSE-014
# Branch: feature/S-DEMO-ENRICHMENT-TYPED-OUTPUT-001 HEAD 622bd2fa

Output docs/demo-evidence/S-DEMO-ENRICHMENT-TYPED-OUTPUT-001/AC-012-bc-2-16-002-catalog-row.gif
Output docs/demo-evidence/S-DEMO-ENRICHMENT-TYPED-OUTPUT-001/AC-012-bc-2-16-002-catalog-row.webm

Set Shell "zsh"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 16
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 40ms

Hide
Type "export PATH=/Users/jmagady/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin"
Enter
Sleep 500ms
Show

Type "# AC-012: SAP-1 — infusion.coercion_failed emission + BC-2.16.002 catalog row"
Enter
Sleep 300ms
Type "# Emission site in crates/:"
Enter
Sleep 200ms
Type "rg 'infusion.coercion_failed' crates/ --type rust -l"
Enter
Sleep 2s

Type "# BC-2.16.002 catalog row in .factory/:"
Enter
Sleep 200ms
Type "rg 'infusion.coercion_failed' /Users/jmagady/Dev/prism/.factory/specs/behavioral-contracts/ -l"
Enter
Sleep 2s
