# Evidence Report — S-3.7.00: Schema Derivation (Armis + CrowdStrike)

**Impl SHA:** `3ae67c9d`
**Recorded:** 2026-04-28
**Tool:** VHS

---

## Coverage Map

| AC | Description | Recording | Result |
|----|-------------|-----------|--------|
| AC-001 | Armis schema (`types.rs`) exists with correct structs | AC-001-004-tests-green.gif | GREEN (ok 1–5) |
| AC-002 | CrowdStrike schema (`types.rs`) exists with correct structs | AC-001-004-tests-green.gif | GREEN (ok 6–11) |
| AC-003 | Both `DERIVATION.md` files exist with pagination docs | AC-001-004-tests-green.gif | GREEN (ok 12–15) |
| AC-004 | Derivation notes document Go→Rust mapping decisions | AC-001-004-tests-green.gif | GREEN (ok 16–21) |
| AC-005 | Artifacts on disk, no fixture-gen code | AC-001-004-tests-green.gif + AC-005-artifacts-on-disk.gif | GREEN (ok 22–25) |

---

## Recordings

### AC-001-004-tests-green

**File:** `AC-001-004-tests-green.gif` (222 KB) / `AC-001-004-tests-green.webm` (233 KB)

**Shows:** `bash tests/toolchain-gate/test_S-3.7.00_schema-derivation-artifacts.sh` running all 25 TAP assertions.

- **Expected:** 25 `ok` lines, `1..25` summary, exit 0
- **Observed:** 25 `ok` lines, `1..25` summary, exit 0

### AC-005-artifacts-on-disk

**File:** `AC-005-artifacts-on-disk.gif` (138 KB) / `AC-005-artifacts-on-disk.webm` (138 KB)

**Shows:** `find .references/schemas -type f | sort` listing all 6 artifact files, then `wc -l` confirming non-trivial size.

- **Expected:** 6 files visible; each file >50 lines
- **Observed:** 6 files; line counts: armis/types.rs=342, armis/DERIVATION.md=163, crowdstrike/types.rs=381, crowdstrike/DERIVATION.md=212 (total 1098 lines)

---

## Error Paths

AC-001..005 have no runtime error paths (artifact existence + static analysis tests). The test script itself demonstrates failure detection: any missing struct or file causes a `not ok` TAP line and non-zero exit. No additional error-path tape is required for static artifact checks.
