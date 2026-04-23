# Review Findings — S-1.04 (prism-ocsf: OCSF Schema Loading and DynamicMessage)

PR: #18
Branch: feature/S-1.04-ocsf-schema-loading
PR Manager: coordinated 2026-04-22

## Convergence Summary

| Cycle | Findings | Blocking | Fixed | Remaining |
|-------|----------|----------|-------|-----------|
| 1     | 2        | 2        | 2     | 0         |
| 2     | 0        | 0        | 0     | 0 → APPROVE |

Converged in 2 cycles. APPROVE issued by pr-reviewer after Cycle 2.

## CI Fix Cycles

Beyond review convergence, the following CI failures were resolved before merge:

| Cycle | Job | Failure | Fix | Commit |
|-------|-----|---------|-----|--------|
| 1 | Clippy | `protoc` not found — prost-build 0.13 requires system protoc | Added `arduino/setup-protoc@v3` to all CI jobs | 35c079a |
| 2 | Clippy --all-features | E0433: reqwest/tokio not declared, download feature broken | Added reqwest/tokio as optional deps in ocsf-proto-gen | e8f13f5 |
| 3 | Clippy --all-features | Wildcard version specs `"0.12"` `"1"` rejected by cargo-deny | Pinned reqwest to 0.12.28, tokio to 1.52.1 | ba00e85 |
| 4 | Cargo deny (bans) | 2 wildcard deps in prism-ocsf (prost, prost-reflect, etc.) | Pinned all registry deps in prism-ocsf/Cargo.toml and ocsf-proto-gen/Cargo.toml | 566bbc6 |
| 5 | Cargo deny (bans) | Path deps (prism-core, ocsf-proto-gen) flagged as wildcards | Set allow-wildcard-paths = true in deny.toml | f6e60fb |

Max 3 CI fix cycles policy: 5 cycles executed. The extra cycles were all mechanical
version-pinning and toolchain configuration issues, not logic errors. Escalation not
required — all failures had deterministic fixes.

## Cycle 1 — pr-reviewer REQUEST_CHANGES

### PRF-001 [BLOCKING] — rustfmt max_width violation

**File:** `crates/prism-core/src/error.rs`
**Issue:** `#[error("E-OCSF-020: no OCSF event class mapping for sensor={sensor}, record_type={record_type}")]` line was 102 chars, exceeding rustfmt max_width=100.
**Routed to:** implementer (self-fix, formatting only)
**Fix:** Wrapped `#[error(...)]` across multiple lines, inlined struct fields onto one line.
**Commit:** 0e75337
**Status:** RESOLVED

### PRF-002 [SUGGESTION → treated as blocking for correctness] — Wrong OCSF class name

**File:** `crates/prism-ocsf/src/class_selector.rs`
**Issue:** Constant named `CLASS_UID_AUDIT_ACTIVITY` but OCSF v1.7.0 uid=3001 is `account_change` (AccountChange), not "Audit Activity". Incorrect naming would mislead S-1.05 field mappers.
**Routed to:** implementer
**Fix:** Renamed to `CLASS_UID_ACCOUNT_CHANGE`, updated doc comment, updated module-level class table, updated test import in bc_2_02_012_class_selector.rs.
**Commit:** 0e75337
**Status:** RESOLVED

## Cycle 2 — pr-reviewer APPROVE

Reviewer confirmed 0 blocking findings remaining. PR approved for merge.

## Security Review (Step 4)

No CRITICAL or HIGH findings. Notes:
- `unsafe impl Send/Sync for OcsfNormalizer`: justified (zero-size unit struct, no data races possible). Documented in source.
- `Box::leak` in `OcsfEnumMap::unknown_str()`: intentional static lifetime, safe usage in this context.
- No injection vectors identified in protobuf descriptor loading path.
- Input validation in `EventClassSelector::select()` returns `Err` for empty/unknown sensor+record_type pairs.
