#!/usr/bin/env bash
# test_S-3.7.00_schema-derivation-artifacts.sh
#
# Acceptance tests for S-3.7.00: Schema derivation Armis + CrowdStrike.
# Traces to: BC-3.4.002 (precondition 2), BC-3.4.003 (PaginationEdgeCases baseline)
# Verification properties: VP-112, VP-114
#
# RED GATE: All tests FAIL before implementation because schema files do not exist.
# Implementer must create the files listed in S-3.7.00 §File Structure Requirements.
set -euo pipefail

WORKTREE="$(cd "$(dirname "$0")/../.." && pwd)"
SCHEMAS="$WORKTREE/.references/schemas"
TAP_COUNT=0
FAIL=0

tap_ok()   { TAP_COUNT=$((TAP_COUNT+1)); echo "ok $TAP_COUNT - $1"; }
tap_fail() { TAP_COUNT=$((TAP_COUNT+1)); printf 'not ok %d - %s\n' "$TAP_COUNT" "$1"; FAIL=1; }

# ---------------------------------------------------------------------------
# AC-001 — Armis types.rs exists (BC-3.4.002 precondition 2)
# ---------------------------------------------------------------------------
ARMIS_TYPES="$SCHEMAS/armis/types.rs"

if [[ -f "$ARMIS_TYPES" ]]; then
  tap_ok "AC-001: .references/schemas/armis/types.rs exists"
else
  tap_fail "AC-001: .references/schemas/armis/types.rs missing (Red Gate: schema not yet derived)"
fi

# AC-001: ArmisAsset struct defined
if [[ -f "$ARMIS_TYPES" ]] && grep -q 'struct ArmisAsset' "$ARMIS_TYPES"; then
  tap_ok "AC-001: ArmisAsset struct defined in types.rs"
else
  tap_fail "AC-001: ArmisAsset struct not found in armis/types.rs (Red Gate)"
fi

# AC-001: ArmisAlert struct defined
if [[ -f "$ARMIS_TYPES" ]] && grep -q 'struct ArmisAlert' "$ARMIS_TYPES"; then
  tap_ok "AC-001: ArmisAlert struct defined in types.rs"
else
  tap_fail "AC-001: ArmisAlert struct not found in armis/types.rs (Red Gate)"
fi

# AC-001: AqlResponse generic type defined
if [[ -f "$ARMIS_TYPES" ]] && grep -q 'AqlResponse' "$ARMIS_TYPES"; then
  tap_ok "AC-001: AqlResponse type defined in types.rs"
else
  tap_fail "AC-001: AqlResponse type not found in armis/types.rs (Red Gate)"
fi

# AC-001: Pagination wrapper type defined
if [[ -f "$ARMIS_TYPES" ]] && grep -q 'ArmisPage' "$ARMIS_TYPES"; then
  tap_ok "AC-001: ArmisPage pagination wrapper defined in types.rs"
else
  tap_fail "AC-001: ArmisPage pagination wrapper not found in armis/types.rs (Red Gate)"
fi

# ---------------------------------------------------------------------------
# AC-002 — CrowdStrike types.rs exists (BC-3.4.002 precondition 2)
# ---------------------------------------------------------------------------
CS_TYPES="$SCHEMAS/crowdstrike/types.rs"

if [[ -f "$CS_TYPES" ]]; then
  tap_ok "AC-002: .references/schemas/crowdstrike/types.rs exists"
else
  tap_fail "AC-002: .references/schemas/crowdstrike/types.rs missing (Red Gate: schema not yet derived)"
fi

# AC-002: FalconDevice struct defined
if [[ -f "$CS_TYPES" ]] && grep -q 'struct FalconDevice' "$CS_TYPES"; then
  tap_ok "AC-002: FalconDevice struct defined in types.rs"
else
  tap_fail "AC-002: FalconDevice struct not found in crowdstrike/types.rs (Red Gate)"
fi

# AC-002: FalconDetection struct defined
if [[ -f "$CS_TYPES" ]] && grep -q 'struct FalconDetection' "$CS_TYPES"; then
  tap_ok "AC-002: FalconDetection struct defined in types.rs"
else
  tap_fail "AC-002: FalconDetection struct not found in crowdstrike/types.rs (Red Gate)"
fi

# AC-002: ContainmentResponse struct defined
if [[ -f "$CS_TYPES" ]] && grep -q 'struct ContainmentResponse' "$CS_TYPES"; then
  tap_ok "AC-002: ContainmentResponse struct defined in types.rs"
else
  tap_fail "AC-002: ContainmentResponse struct not found in crowdstrike/types.rs (Red Gate)"
fi

# AC-002: OAuth2TokenResponse struct defined
if [[ -f "$CS_TYPES" ]] && grep -q 'struct OAuth2TokenResponse' "$CS_TYPES"; then
  tap_ok "AC-002: OAuth2TokenResponse struct defined in types.rs"
else
  tap_fail "AC-002: OAuth2TokenResponse struct not found in crowdstrike/types.rs (Red Gate)"
fi

# AC-002: IdPage struct defined (2-step IDs->detail pattern, EC-002)
if [[ -f "$CS_TYPES" ]] && grep -q 'struct IdPage' "$CS_TYPES"; then
  tap_ok "AC-002: IdPage struct defined in types.rs"
else
  tap_fail "AC-002: IdPage struct not found in crowdstrike/types.rs (Red Gate — 2-step pattern not captured)"
fi

# ---------------------------------------------------------------------------
# AC-003 — default_page_size documented in both DERIVATION.md files (D-055)
# Traces to BC-3.4.003 PaginationEdgeCases baseline
# ---------------------------------------------------------------------------
ARMIS_DERIV="$SCHEMAS/armis/DERIVATION.md"

if [[ -f "$ARMIS_DERIV" ]]; then
  tap_ok "AC-003/AC-004: .references/schemas/armis/DERIVATION.md exists"
else
  tap_fail "AC-003/AC-004: .references/schemas/armis/DERIVATION.md missing (Red Gate)"
fi

if [[ -f "$ARMIS_DERIV" ]] && grep -qi 'default_page_size' "$ARMIS_DERIV"; then
  tap_ok "AC-003: Armis default_page_size documented in DERIVATION.md"
else
  tap_fail "AC-003: default_page_size not found in armis/DERIVATION.md (Red Gate — D-055 not satisfied)"
fi

CS_DERIV="$SCHEMAS/crowdstrike/DERIVATION.md"

if [[ -f "$CS_DERIV" ]]; then
  tap_ok "AC-003/AC-004: .references/schemas/crowdstrike/DERIVATION.md exists"
else
  tap_fail "AC-003/AC-004: .references/schemas/crowdstrike/DERIVATION.md missing (Red Gate)"
fi

if [[ -f "$CS_DERIV" ]] && grep -qi 'default_page_size' "$CS_DERIV"; then
  tap_ok "AC-003: CrowdStrike default_page_size documented in DERIVATION.md"
else
  tap_fail "AC-003: default_page_size not found in crowdstrike/DERIVATION.md (Red Gate — D-055 not satisfied)"
fi

# ---------------------------------------------------------------------------
# AC-004 — Derivation notes contain required sections (BC-3.4.002 invariant 3)
# Required: source Go struct mapping, nullable->Option<T> rationale,
# polymorphic field handling, omitted fields rationale.
# ---------------------------------------------------------------------------

for sensor in armis crowdstrike; do
  DERIV="$SCHEMAS/$sensor/DERIVATION.md"

  # Source Go struct section
  if [[ -f "$DERIV" ]] && grep -qi 'source\|go struct\|go type' "$DERIV"; then
    tap_ok "AC-004 [$sensor]: DERIVATION.md documents source Go struct mapping"
  else
    tap_fail "AC-004 [$sensor]: DERIVATION.md missing source Go struct section (Red Gate)"
  fi

  # Nullable -> Option<T> rationale
  if [[ -f "$DERIV" ]] && grep -qi 'option\|nullable\|null' "$DERIV"; then
    tap_ok "AC-004 [$sensor]: DERIVATION.md documents nullable -> Option<T> decisions"
  else
    tap_fail "AC-004 [$sensor]: DERIVATION.md missing nullable/Option<T> section (Red Gate)"
  fi

  # Polymorphic field handling
  if [[ -f "$DERIV" ]] && grep -qi 'polymorphic\|interface{}\|serde_json::Value\|ArmisId\|deserialize_any' "$DERIV"; then
    tap_ok "AC-004 [$sensor]: DERIVATION.md documents polymorphic field handling"
  else
    tap_fail "AC-004 [$sensor]: DERIVATION.md missing polymorphic field handling section (Red Gate)"
  fi
done

# ---------------------------------------------------------------------------
# AC-005 — No generator code (no generate() fn, no fixture-gen feature gate)
# Traces to BC-3.4.002 invariant 3
# ---------------------------------------------------------------------------

for f in "$SCHEMAS/armis/types.rs" "$SCHEMAS/crowdstrike/types.rs"; do
  fname="$(basename "$(dirname "$f")")/$(basename "$f")"
  if [[ ! -f "$f" ]]; then
    # File absent: AC-005 trivially holds (nothing to check — already failed AC-001/002)
    tap_ok "AC-005 [$fname]: file absent — no generator code possible"
    continue
  fi

  if grep -q 'fn generate' "$f" 2>/dev/null; then
    tap_fail "AC-005 [$fname]: generate() function found — generator code must NOT appear in schema reference files"
  else
    tap_ok "AC-005 [$fname]: no generate() function present"
  fi

  if grep -q 'fixture-gen' "$f" 2>/dev/null; then
    tap_fail "AC-005 [$fname]: fixture-gen feature gate found — must NOT appear in schema reference files"
  else
    tap_ok "AC-005 [$fname]: no fixture-gen feature gate present"
  fi
done

echo "1..$TAP_COUNT"
exit $FAIL
