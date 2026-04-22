#!/usr/bin/env bash
# reorder-bc-changelogs.sh
# Pass-73 deterministic remediation: reorder all BC changelog sections to descending version order.
# Usage: bash reorder-bc-changelogs.sh [--dry-run]
# Output: lists modified files to stdout.

set -euo pipefail

BC_DIR="/Users/jmagady/Dev/prism/.factory/specs/behavioral-contracts"
DRY_RUN=false
if [[ "${1:-}" == "--dry-run" ]]; then
  DRY_RUN=true
fi

MODIFIED_FILES=()

# Compare two version strings as tuples (major.minor).
# Returns 0 if $1 > $2, 1 otherwise.
version_gt() {
  local a="$1" b="$2"
  # Split on dot
  local a_major a_minor b_major b_minor
  a_major="${a%%.*}"
  a_minor="${a#*.}"
  b_major="${b%%.*}"
  b_minor="${b#*.}"
  # Numeric compare
  if [[ "$a_major" -gt "$b_major" ]]; then return 0; fi
  if [[ "$a_major" -lt "$b_major" ]]; then return 1; fi
  if [[ "$a_minor" -gt "$b_minor" ]]; then return 0; fi
  return 1
}

process_file() {
  local file="$1"
  local content
  content=$(cat "$file")

  # Find the line number of "## Changelog"
  local changelog_line
  changelog_line=$(grep -n "^## Changelog" "$file" | head -1 | cut -d: -f1)
  if [[ -z "$changelog_line" ]]; then
    return 0  # No changelog section
  fi

  # Find header row: the | Version | Burst | ... line
  local header_line
  header_line=$(awk -v start="$changelog_line" 'NR > start && /^\| *Version *\|/ { print NR; exit }' "$file")
  if [[ -z "$header_line" ]]; then
    return 0  # No header row found
  fi

  # Separator line is header_line + 1
  local sep_line=$(( header_line + 1 ))

  # Data rows start at sep_line + 1 — collect until blank line or next ## section
  local data_start=$(( sep_line + 1 ))

  # Extract data rows into array
  mapfile -t data_rows < <(awk -v start="$data_start" '
    NR >= start {
      if (/^\| *[0-9]/) { print; next }
      if (/^$/ || /^## / || /^---/) { exit }
    }
  ' "$file")

  if [[ ${#data_rows[@]} -le 1 ]]; then
    return 0  # 0 or 1 rows — nothing to sort
  fi

  # Extract versions from first column of each row
  local versions=()
  for row in "${data_rows[@]}"; do
    local ver
    ver=$(echo "$row" | sed 's/^| *//' | cut -d'|' -f1 | tr -d ' ')
    versions+=("$ver")
  done

  # Check if already in descending order
  local already_sorted=true
  for (( i=0; i<${#versions[@]}-1; i++ )); do
    local v_curr="${versions[$i]}"
    local v_next="${versions[$((i+1))]}"
    # Descending means curr >= next
    if version_gt "$v_next" "$v_curr"; then
      already_sorted=false
      break
    fi
  done

  if $already_sorted; then
    return 0  # Already correct order
  fi

  # Sort data rows by version descending using sort with version-aware key
  # Write rows to temp file with version prefix for sorting
  local tmpfile
  tmpfile=$(mktemp)
  for row in "${data_rows[@]}"; do
    local ver
    ver=$(echo "$row" | sed 's/^| *//' | cut -d'|' -f1 | tr -d ' ')
    echo "${ver}|||${row}"
  done > "$tmpfile"

  # Sort descending by version (treat as two-part numeric: major.minor)
  mapfile -t sorted_rows < <(sort -t. -k1,1rn -k2,2rn "$tmpfile" | sed 's/^[^|]*|||//')
  rm -f "$tmpfile"

  # Reconstruct the file
  # Strategy: replace data rows in-place using Python for reliable multiline replacement
  local header_text sep_text
  header_text=$(sed -n "${header_line}p" "$file")
  sep_text=$(sed -n "${sep_line}p" "$file")

  # Build new changelog block
  local new_block
  new_block="${header_text}"$'\n'"${sep_text}"
  for row in "${sorted_rows[@]}"; do
    new_block+=$'\n'"${row}"
  done

  # Build original block (for replacement)
  local orig_block
  orig_block="${header_text}"$'\n'"${sep_text}"
  for row in "${data_rows[@]}"; do
    orig_block+=$'\n'"${row}"
  done

  if [[ "$new_block" == "$orig_block" ]]; then
    return 0
  fi

  if $DRY_RUN; then
    echo "WOULD_MODIFY: $file"
    echo "  Before versions: ${versions[*]}"
    echo "  After versions: $(for r in "${sorted_rows[@]}"; do echo "$r" | sed 's/^| *//' | cut -d'|' -f1 | tr -d ' '; done | tr '\n' ' ')"
    return 0
  fi

  # Use Python to do the replacement safely (handles special chars in sed)
  python3 - "$file" "$orig_block" "$new_block" <<'PYEOF'
import sys

filepath = sys.argv[1]
old_block = sys.argv[2]
new_block = sys.argv[3]

with open(filepath, 'r') as f:
    content = f.read()

if old_block not in content:
    print(f"WARNING: could not find original block in {filepath}", file=sys.stderr)
    sys.exit(1)

new_content = content.replace(old_block, new_block, 1)

with open(filepath, 'w') as f:
    f.write(new_content)
PYEOF

  echo "MODIFIED: $file"
  MODIFIED_FILES+=("$file")
}

# Process all BC files
for bc_file in "$BC_DIR"/BC-*.md; do
  process_file "$bc_file"
done

echo "---"
echo "TOTAL_MODIFIED: ${#MODIFIED_FILES[@]}"
