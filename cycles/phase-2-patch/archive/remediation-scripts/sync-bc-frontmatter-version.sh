#!/usr/bin/env bash
# sync-bc-frontmatter-version.sh
# Scan ALL BC files for frontmatter version vs. top changelog row mismatch.
# For each mismatch where top-row version != frontmatter version,
# update frontmatter version to match top changelog row.
# Reports which files were changed.

set -uo pipefail

BC_DIR="/Users/jmagady/Dev/prism/.factory/specs/behavioral-contracts"
CHANGED=0
CHECKED=0

for f in "$BC_DIR"/BC-*.md; do
  CHECKED=$((CHECKED + 1))
  # Extract frontmatter version (between the two --- lines, first block only)
  fm_version=$(awk '/^---/{c++; next} c==1 && /^version:/{print; exit} c==2{exit}' "$f" | sed 's/version:[[:space:]]*//' | tr -d '"')

  if [[ -z "$fm_version" ]]; then
    continue
  fi

  # Find ## Changelog heading line number
  changelog_line=$(grep -n "^## Changelog" "$f" | head -1 | cut -d: -f1)
  if [[ -z "$changelog_line" ]]; then
    continue
  fi

  # Find the first data row after the changelog header
  # Data rows start with | version-number | (version number starts with a digit)
  top_row_version=$(awk -v start="$changelog_line" '
    NR <= start { next }
    /^\|[[:space:]]*[0-9]/ {
      sub(/^\|[[:space:]]*/, "")
      n = split($0, cols, "|")
      ver = cols[1]
      gsub(/^[[:space:]]+|[[:space:]]+$/, "", ver)
      print ver
      exit
    }
  ' "$f")

  if [[ -z "$top_row_version" ]]; then
    continue
  fi

  if [[ "$fm_version" != "$top_row_version" ]]; then
    echo "MISMATCH: $(basename $f)  frontmatter=$fm_version  top-row=$top_row_version"
    # Rewrite file updating version field only in first frontmatter block
    python3 - "$f" "$top_row_version" <<'PYEOF'
import sys, re

filepath = sys.argv[1]
new_ver = sys.argv[2]

with open(filepath, 'r') as fh:
    content = fh.read()

# Replace version in first frontmatter block only
# frontmatter is between first two --- lines
parts = content.split('---', 2)
if len(parts) < 3:
    sys.exit(0)

preamble, fm, rest = parts[0], parts[1], parts[2]
fm_new = re.sub(r'^(version:\s*)["\']?[^"\'\n]*["\']?', f'version: "{new_ver}"', fm, count=1, flags=re.MULTILINE)

with open(filepath, 'w') as fh:
    fh.write(preamble + '---' + fm_new + '---' + rest)

PYEOF
    CHANGED=$((CHANGED + 1))
  fi
done

echo ""
echo "Checked: $CHECKED BC files"
echo "Changed: $CHANGED BC files"
