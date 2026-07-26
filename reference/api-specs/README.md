# Vendor OpenAPI Specifications — Reference Copies

**Purpose:** Canonical vendor OpenAPI specs for DTU fidelity remediation. Human-supplied 2026-07-20 (D-1888 session wrap). These files are the authoritative ground-truth for DTU route shapes, request/response schemas, and endpoint coverage during the findings-triage phase that follows the S-REL-001 merge.

## File Inventory

| File | Vendor | Scope | Size |
|------|--------|-------|------|
| `cyberint_alerts_openapi_06.20.2026.json` | Cyberint | Alerts API | ~90 KB |
| `cyberint_assets_openapi_06.20.2026.json` | Cyberint | Assets API | ~28 KB |
| `xdome_openapi_06.20.2026.json` | Claroty xDome | Full xDome API | ~4.2 MB |
| `armis_endpoint_research_07.20.2026.md` | Armis | Endpoint audit + auth fidelity findings | ~12 KB |

## Armis

No downloadable OpenAPI available from Armis. The canonical in-repo grounding reference is
`armis_endpoint_research_07.20.2026.md` — a web-corroborated endpoint audit from 2026-07-20
covering auth flow (token-exchange, raw-token header), AQL collections, field-fidelity gaps,
and Confirmed/Partial/Unconfirmed confidence tiers. ADR-053 §D1 "No-OpenAPI governance" makes
the confidence tiers in that document the binding grounding contract for Armis spec authoring.

Vendored from external repository 2026-07-25 to close adversary finding F-WASE-P64-HIGH-007.
Online Armis developer docs: https://dev.armis.com/reference/post_oauth_token_post

## Usage

DTU fidelity reviewers and adversarial passes for Cyberint and xDome stories should cross-reference these files when validating:
- Column names and types in sensor TOML specs (SAP-2 standing probe)
- DTU clone route shapes (`crates/prism-dtu-*/src/routes/*.rs`)
- Request body templates and response field mappings

CrowdStrike uses a different authentication model (OAuth2) and has no file here; its API docs are in the separate CrowdStrike developer portal.
