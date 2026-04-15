---
document_type: adversarial-review
pass: 24
status: complete-fixed
novelty: HIGH
findings: 12
critical: 5
high: 4
low: 3
---

# Pass 24 — output schema drift, confirmation tokens, dry-run params, risk classification

## CRITICAL (5)
- ADV-24-001: create_schedule/create_rule missing dry_run parameter despite BC mandate
- ADV-24-002: splay_percent max 50 (interface) vs 25 (BC) contradiction
- ADV-24-003: list_schedules output schema fundamentally mismatches BC-2.12.002 (scalar vs per-client maps)
- ADV-24-004: acknowledge_alert has contradictory risk classification (immediate vs reversible-write) and is a STUB
- ADV-24-005: Confirmation token field name inconsistency (token vs token_id across tools)

## HIGH (4)
- ADV-24-006: BC-2.04.007 risk classification table missing 9+ write tools
- ADV-24-007: create_pack input schema contradicts BC-2.12.009 (inline queries vs references)
- ADV-24-008: pack.write capability path undocumented in TOML config
- ADV-24-009: list_cases output missing severity/assignee/disposition despite having those as filters

## LOW (3)
- ADV-24-010: Pagination parameter inconsistency across list tools (4 paginated, 4 not)
- ADV-24-011: disposition output as flat string but input as structured object
- ADV-24-012: BC-2.04.008 references nonexistent update_alert_status tool
