---
document_type: uncertainty-map
story_id: S-3.04
version: "1.0"
producer: dclaude-uncertainty-scanner
timestamp: "2026-05-04T00:00:00Z"
status: draft
---

# S-3.04 Uncertainty Map — Alias System

## Summary verdict

**GREEN-leaning-YELLOW** — Few external library claims. Persistence is
file-based (aliases.toml) not RocksDB, removing a major version-pin risk.
The minor concerns are around Chumsky 0.12 integration timing, Kani version,
and an internal architecture inconsistency (line 412 vs lines 131/424/444).

## Findings

| Severity | Category | Finding | Recommended action |
|---|---|---|---|
| Important | architecture-pattern | Internal inconsistency: line 131 says "Persistence target: aliases.toml (NOT RocksDB)" and line 444 reinforces "Do NOT use RocksDB or any RocksDB column", but line 412 says "Aliases are persisted in the `aliases` RocksDB column family". This is a story-internal contradiction the implementer would hit. | Spec-level fix: remove or correct line 412. NOT a tech uncertainty per se but blocks implementation. (Cross-listed for orchestrator visibility.) |
| Important | api-assumption | Line 333, 442, 524, 538: integration with the S-3.01 Chumsky 0.12 parser is described but the alias expander runs before parsing (no Chumsky dependency in this story). | LOW risk — story does not actually pull Chumsky in. Confirm S-3.01 produces the parser entry-point this story expects. |
| Suggestion | unpinned-version | "Kani" referenced (line 82, proofs/vp012_depth_limit.rs) without version pin. | Add Kani version pin (current is 0.55+). RESEARCH-NEEDED if first introduction. |
| Suggestion | version-pin | `proptest` referenced for VP-013 (line 83). Workspace baseline is `proptest = "1.11"` (prism-storage) and `"1"` (prism-core). | Confirm S-3.04 inherits matching pin. |
| Tech Debt | feature-claim | "Atomic write pattern (temp file + rename)" claim. Reliable on POSIX; on Windows requires `MoveFileEx` semantics. | Confirm whether Prism targets Windows; if so, use a crate like `tempfile::persist` or `atomicwrites`. |

## Cross-references

- BCs BC-2.11.008/009/013/014/015 all confirmed active in BC-INDEX v4.32.
- VP-012/013/037 verification properties referenced.

## RESEARCH-NEEDED queries

1. "Latest stable Kani version as of 2026-05-04 for Rust 2021 edition. Has `kani::any` API or proof harness syntax changed?"

