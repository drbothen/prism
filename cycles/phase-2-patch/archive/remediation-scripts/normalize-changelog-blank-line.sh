#!/usr/bin/env bash
# normalize-changelog-blank-line.sh
# For each BC file: if ## Changelog heading is immediately followed (next non-empty line)
# by a table row (starts with |), insert a blank line between heading and table.
# Reports which files were modified.

set -uo pipefail

BC_DIR="/Users/jmagady/Dev/prism/.factory/specs/behavioral-contracts"
CHANGED=0
CHECKED=0

for f in "$BC_DIR"/BC-*.md; do
  CHECKED=$((CHECKED + 1))

  # Use python3 for reliable multi-line processing
  result=$(python3 - "$f" <<'PYEOF'
import sys, re

filepath = sys.argv[1]

with open(filepath, 'r') as fh:
    content = fh.read()

# Pattern: ## Changelog\n followed directly (possibly with no blank line) by | table row
# We want: ## Changelog\n\n| table row
# Match: ## Changelog heading, then optional whitespace-only lines == 0, then table row
pattern = r'(## Changelog\n)((?!\n)\|)'
if re.search(pattern, content):
    new_content = re.sub(pattern, r'\1\n\2', content)
    with open(filepath, 'w') as fh:
        fh.write(new_content)
    print("CHANGED")
else:
    print("OK")
PYEOF
)

  if [[ "$result" == "CHANGED" ]]; then
    echo "FIXED: $(basename $f)"
    CHANGED=$((CHANGED + 1))
  fi
done

echo ""
echo "Checked: $CHECKED BC files"
echo "Changed: $CHANGED BC files"
