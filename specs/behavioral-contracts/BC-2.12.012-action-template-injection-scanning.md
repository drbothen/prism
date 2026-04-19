---
document_type: behavioral-contract
level: L3
version: "1.0"
status: removed
lifecycle_status: retired
introduced: cycle-1
deprecated_by: "v3-patch-burst-4b"
replacement: "BC-2.18.006"
deprecated: "2026-04-16"
producer: product-owner
timestamp: 2026-04-16T22:00:00
phase: 3
origin: greenfield
subsystem: "SS-12"
capability: "CAP-021"
---

> **RETIRED (2026-04-16):** Superseded by BC-2.18.006 (Action Delivery Engine subsystem, INV-ACTION-006).
> BC-2.12.012 was a high-level cross-subsystem summary written before subsystem 18 was established.
> BC-2.18.006 is the normative specification. In all conflicts, BC-2.18.006 wins.
> This file is retained for historical traceability only.

# BC-2.12.012: Action Template Injection Scanning

## Preconditions
- An action template is being rendered with variable interpolation
- Variables may contain untrusted data from sensor events or alert fields

## Postconditions
- All template variables are scanned by InjectionScanner (BC-2.09.003) before interpolation
- Variables containing detected injection patterns are flagged (not stripped, per BC-2.09.004 "flag don't strip")
- Safety flags are included in the action delivery payload metadata
- Template rendering uses the same JSON-escape and percent-encode safety rules as sensor spec variable interpolation (S-1.11)

## Invariants
- DI-006: Untrusted data is never rendered into action templates without injection scanning
- Action templates support the same 4-level variable resolution as alert templates (S-4.05)
