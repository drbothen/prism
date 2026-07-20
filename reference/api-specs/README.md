# Vendor OpenAPI Specifications — Reference Copies

**Purpose:** Canonical vendor OpenAPI specs for DTU fidelity remediation. Human-supplied 2026-07-20 (D-1888 session wrap). These files are the authoritative ground-truth for DTU route shapes, request/response schemas, and endpoint coverage during the findings-triage phase that follows the S-REL-001 merge.

## File Inventory

| File | Vendor | Scope | Size |
|------|--------|-------|------|
| `cyberint_alerts_openapi_06.20.2026.json` | Cyberint | Alerts API | ~90 KB |
| `cyberint_assets_openapi_06.20.2026.json` | Cyberint | Assets API | ~28 KB |
| `xdome_openapi_06.20.2026.json` | Claroty xDome | Full xDome API | ~4.2 MB |

## Armis

No OpenAPI file available. Canonical API docs at: https://dev.armis.com/reference/post_oauth_token_post

## Usage

DTU fidelity reviewers and adversarial passes for Cyberint and xDome stories should cross-reference these files when validating:
- Column names and types in sensor TOML specs (SAP-2 standing probe)
- DTU clone route shapes (`crates/prism-dtu-*/src/routes/*.rs`)
- Request body templates and response field mappings

CrowdStrike uses a different authentication model (OAuth2) and has no file here; its API docs are in the separate CrowdStrike developer portal.
