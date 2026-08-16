#!/usr/bin/env python3
"""Per-symbol #[non_exhaustive] violation verification.

Parses cargo --message-format=json output and verifies each expected
#[non_exhaustive] type produces a unique E0639 or E0004 error.

Extraction rules:
  E0639 struct literal: extracts the last path segment from `= path::TypeName {`
    in spans[0].text[0].text (the source-code line in the error).
  E0004 exhaustive match: extracts the last path segment from
    `note: \`path::TypeName\` defined here` in the rendered message text.

Guards:
  - EXPECTED_COUNT is derived as len(EXPECTED_SYMBOLS) — no manual sync needed.
  - Every distinct symbol in EXPECTED_SYMBOLS appears at least once in the
    collected error output (catches removed annotations).
  - No symbol in E0639/E0004 error output is absent from EXPECTED_SYMBOLS
    (unregistered-addition detection).

Usage: check-non-exhaustive-per-symbol.py <json-log-file>
       check-non-exhaustive-per-symbol.py --count   (prints EXPECTED_COUNT and exits 0)
Returns: 0 (all symbols present) or 1 (one or more missing).

Single source of truth: this file. ci.yml calls scripts/check-non-exhaustive.sh
which calls this script; the two do NOT maintain separate lists.
(F-CSD-P29-OBS-001 — net-zero regression hardening, DEFECT-CSDEVICES-EMPTY-PIPELINE-001)
"""
import json
import re
import sys

# ---------------------------------------------------------------------------
# Expected symbol list — one entry per violation function.
# E0639 names: as they appear in the struct literal expression in
#   struct_violations.rs (may be a local alias, e.g. TypesSensorTableDescriptor).
# E0004 names: last path segment from `note: \`path::TypeName\` defined here`
#   in the E0004 rendered message (always the canonical type name leaf).
# EXPECTED_COUNT is derived from len(EXPECTED_SYMBOLS) below — no manual update.
# ---------------------------------------------------------------------------
EXPECTED_SYMBOLS = [
    # ── E0639 struct literal violations (69 total) ──────────────────────────
    # Names match the identifier used in the struct literal expression in
    # tests/external/non-exhaustive-violation/src/struct_violations.rs.
    # Some are local aliases (e.g. TypesSensorTableDescriptor for types::SensorTableDescriptor).
    "CredentialRef",              # v01 spec_parser::CredentialRef
    "SensorSpec",                 # v02 spec_parser::SensorSpec
    "SensorTableDescriptor",      # v03 spec_parser::SensorTableDescriptor
    "FetchStep",                  # v04 spec_parser::FetchStep
    "ColumnSpec",                 # v05 spec_parser::ColumnSpec
    "TableSpec",                  # v06 spec_parser::TableSpec
    "RateLimitHints",             # v09 spec_parser::RateLimitHints
    "TypesSensorTableDescriptor", # v10 types::SensorTableDescriptor (alias)
    "TypesCredentialRef",         # v11 types::CredentialRef (alias)
    "InfusionCredentialRef",      # v12 infusion::CredentialRef (alias)
    "WriteStep",                  # v16 write_endpoint::WriteStep
    "WriteEndpointSpec",          # v17 write_endpoint::WriteEndpointSpec
    "InfusionSourceConfig",       # v20 infusion::InfusionSourceConfig
    "InfusionField",              # v21 infusion::InfusionField
    "PipeStageConfig",            # v22 infusion::PipeStageConfig
    "PluginConfig",               # v23 infusion::PluginConfig
    "InfusionSpec",               # v24 infusion::InfusionSpec
    "ColumnDef",                  # v26 prism_core::ColumnDef
    "LoadedPlugin",               # v33 plugin::LoadedPlugin
    "WriteToolInvalidationMap",   # v32 prism_query::invalidation::WriteToolInvalidationMap
    "SensorInstanceOverlay",      # v34 overlay::SensorInstanceOverlay
    "OverlayProvenance",          # v35 overlay::OverlayProvenance
    "ResolvedSensorSpec",         # v36 overlay::ResolvedSensorSpec
    "ResponseMeta",               # v37 prism_mcp::safety_envelope::ResponseMeta
    "ContentEntry",               # v38 prism_mcp::safety_envelope::ContentEntry
    "StructuredContent",          # v39 prism_mcp::safety_envelope::StructuredContent
    "ResponseEnvelope",           # v40 prism_mcp::safety_envelope::ResponseEnvelope
    "SafetyFlagSchema",           # v41 prism_mcp::safety_envelope::SafetyFlagSchema
    "MetaEnvelopeSchemaType",     # v42 prism_mcp::safety_envelope::MetaEnvelopeSchemaType
    "ResponseEnvelopeSchema",     # v43 prism_mcp::safety_envelope::ResponseEnvelopeSchema
    "BoundingMetadata",           # v47 prism_security::confirmation_token::BoundingMetadata
    "ToolRegistration",           # v45 prism_mcp::tool_registry::ToolRegistration
    "BearerStaticAuthProvider",   # v49 prism_bin::spec_driven_adapter::BearerStaticAuthProvider
    "SpecDrivenSensorAdapter",    # v50 prism_bin::spec_driven_adapter::SpecDrivenSensorAdapter
    "ScenarioEntityCatalog",      # v51 prism_dtu_common::scenario::ScenarioEntityCatalog
    "IncidentTimeline",           # v52 prism_dtu_common::scenario::IncidentTimeline
    "IncidentStage",              # v53 prism_dtu_common::scenario::IncidentStage
    "MultiInstanceConfig",        # v54 prism_dtu_demo_server::MultiInstanceConfig
    "InstanceEntry",              # v55 prism_dtu_demo_server::InstanceEntry
    "DemoBindError",              # v56 prism_dtu_demo_server::DemoBindError
    "MultiInstanceHarness",       # v57 prism_dtu_harness::MultiInstanceHarness
    "HarnessEntry",               # v58 prism_dtu_harness::HarnessEntry
    "BindError",                  # v59 prism_dtu_harness::error::BindError
    "MultiInstanceServers",       # v61 prism_dtu_demo_server::MultiInstanceServers
    "StructuredErrorFields",      # v62 prism_mcp::error_mapping::StructuredErrorFields
    "CapabilityEntry",            # v63 prism_mcp::CapabilityEntry
    "ResolutionStep",             # v64 prism_mcp::ResolutionStep
    "TableNotAvailableDetails",   # v66 prism_core::error::TableNotAvailableDetails
    "TableRegistry",              # v67 prism_query::table_registry::TableRegistry
    "Tier3CacheEntry",            # v68 prism_spec_engine::infusion::cache::Tier3CacheEntry
    "InfusionUdfDescriptor",      # v69 prism_spec_engine::infusion::udf::InfusionUdfDescriptor
    "EnrichStageDescriptor",      # v70 (struct_violations) prism_spec_engine::infusion::enrich_descriptor::EnrichStageDescriptor
    "ClientInventoryEntry",       # v71 prism_mcp::ClientInventoryEntry
    "SensorConfigEntry",          # v72 prism_mcp::SensorConfigEntry
    "SensorHealthResult",         # v73 prism_mcp::SensorHealthResult
    "RateLimitInfo",              # v74 prism_mcp::RateLimitInfo
    "ResourcePressure",           # v75 prism_mcp::ResourcePressure
    "SensorHealthStructuredContent",  # v76 prism_mcp::SensorHealthStructuredContent
    "HttpLookupCredentialConfig", # v77 prism_spec_engine::infusion::HttpLookupCredentialConfig
    "HttpLookupConfig",           # v78 prism_spec_engine::infusion::HttpLookupConfig
    "PrismDescribeResponse",      # v80 prism_mcp::PrismDescribeResponse
    "TableDescriptor",            # v81 prism_mcp::TableDescriptor
    "ColumnDescriptor",           # v82 prism_mcp::ColumnDescriptor
    "ColumnNotFoundDetails",      # v83 prism_core::error::ColumnNotFoundDetails
    "HealthSummary",              # v84 prism_mcp::resources::HealthSummary
    "SqlPipeQuery",               # v86 (struct_violations) prism_query::ast::SqlPipeQuery
    "UnknownSourceTableDetails",  # v87 prism_core::error::UnknownSourceTableDetails
    "EnrichUdfNotFoundDetails",   # v88 prism_core::error::EnrichUdfNotFoundDetails
    "ParseError",                 # v92 prism_query::error::ParseError (DEFECT-PQL-FNCALL-LHS-001 §OBS-005)
    # ── S-DEMO-CLAROTY-DAR-001 Task 4 / AC-005: prism-dtu-claroty device_alert_relations types ──
    "ClarotyDeviceAlertRelation", # v93 prism_dtu_claroty::types::ClarotyDeviceAlertRelation
    "GetDeviceAlertsBody",        # v94 prism_dtu_claroty::types::GetDeviceAlertsBody
    "GetDeviceAlertsResponse",    # v95 prism_dtu_claroty::types::GetDeviceAlertsResponse
    # ── S-CLAROTY-AUDITLOG-TIMEBOX-001 fix-burst 5 (LOW-4 gate registration): pre-existing ──
    # ── gap from S-1.11 — FetchContext + PipelineResult had #[non_exhaustive] from initial ──
    # ── commit but were never registered in this gate. Fixed in-scope. ─────────────────────
    "FetchContext",               # v96 prism_spec_engine::pipeline::FetchContext
    "PipelineResult",             # v97 prism_spec_engine::pipeline::PipelineResult
    # ── E0004 enum match violations (23 entries; 22 unique after prism_core::ColumnType dedup) ──
    # Names are the last path segment from `note: \`path::TypeName\` defined here`.
    "PaginationConfig",      # v07 spec_parser::PaginationConfig
    "AuthType",              # v08 spec_parser::AuthType
    "ColumnType",            # v13 prism_core::column::ColumnType
    "ColumnOptions",         # v14 prism_core::column::ColumnOptions
    "BatchMode",             # v15 write_endpoint::BatchMode
    "InfusionType",          # v18 infusion::InfusionType
    "BuiltInSourceType",     # v19 infusion::BuiltInSourceType
    "ColumnType",            # v25 types::ColumnType = pub use prism_core::column::ColumnType
    "PaginationType",        # v27 types::PaginationType
    "SpecStatus",            # v28 types::SpecStatus
    "ClientStatus",          # v29 types::ClientStatus
    "PluginError",           # v31 prism_core::error::PluginError
    "BoundingDmlOperation",  # v46 prism_security::confirmation_token::BoundingDmlOperation
    "DataSource",            # v44 prism_mcp::safety_envelope::DataSource
    "AdapterAuthStrategy",   # v48 prism_bin::spec_driven_adapter::AdapterAuthStrategy
    "MultiInstanceBindError",# v60 prism_dtu_demo_server::MultiInstanceBindError
    "CapabilityStatus",      # v65 prism_mcp::CapabilityStatus
    "InfusionError",         # v70 (enum_violations) prism_core::error::InfusionError
    "HttpLookupAuthType",    # v79 prism_spec_engine::infusion::HttpLookupAuthType
    "TemporalLiteralPosition",    # v86 (enum_violations) prism_core::error::TemporalLiteralPosition
    "prism_core::VirtualField",  # v90 prism_core::VirtualField (re-exported; compiler note is 2-part → 2-seg join = "prism_core::VirtualField"; disambiguates from v91)
    "ast::VirtualField",     # v91 prism_query::ast::VirtualField (2-seg: disambiguates from v90)
    "ExampleKind",           # v85 prism_mcp::resources::ExampleKind
]

# EXPECTED_COUNT is derived from the list — no manual update needed when symbols are added/removed.
EXPECTED_COUNT = len(EXPECTED_SYMBOLS)

# Unique set of expected symbols (for per-symbol check). ColumnType appears
# twice (v13 + v25 both test prism_core::ColumnType) and is deduplicated here.
# For disambiguated symbols like virtual_fields::VirtualField / ast::VirtualField,
# 2-segment suffixes ensure both remain distinct in EXPECTED_UNIQUE.
EXPECTED_UNIQUE = set(EXPECTED_SYMBOLS)


def extract_e0639_symbol(msg: dict) -> str | None:
    """Extract the last path segment of the struct name from an E0639 error.

    Looks at spans[0].text[0].text for the pattern `= path::TypeName {`.
    Returns the last segment (TypeName), or None if not found.
    """
    spans = msg.get("spans", [])
    if not spans:
        return None
    texts = spans[0].get("text", [])
    if not texts:
        return None
    src_line = texts[0].get("text", "")
    m = re.search(r"=\s+([\w:]+)\s*\{", src_line)
    if not m:
        return None
    full_path = m.group(1)
    return full_path.split("::")[-1]


def extract_e0004_symbol(msg: dict) -> list[str]:
    """Extract path segments of the enum name from an E0004 error.

    Looks in the rendered text for `note: \`path::TypeName\` defined here`.

    Returns a list of forms:
      - The last segment (TypeName) for backward-compatible single-segment expected entries.
      - The last-2-segment form ("module::TypeName") for disambiguating types that share
        a last-segment name across crates (e.g., prism_core::VirtualField (rustc emits the
        2-part path for the re-export) vs prism_query::ast::VirtualField (rustc emits the
        3-part path) both produce "VirtualField" as last segment; the 2-segment forms
        "prism_core::VirtualField" and "ast::VirtualField" are distinct and allow
        per-symbol verification to tell them apart).

    Empty list if no matching note is found.
    """
    rendered = msg.get("rendered", "")
    results = []
    for line in rendered.splitlines():
        m = re.search(r"note:\s+`([^`]+)`\s+defined here", line)
        if m:
            full_path = m.group(1)
            parts = full_path.split("::")
            results.append(parts[-1])              # 1-segment (backward compat)
            if len(parts) >= 2:
                results.append("::".join(parts[-2:]))  # 2-segment (disambiguation)
    return results


def main() -> int:
    if len(sys.argv) < 2:
        print("Usage: check-non-exhaustive-per-symbol.py <json-log-file>", file=sys.stderr)
        return 1

    log_path = sys.argv[1]
    found: set[str] = set()
    # Symbols from E0639/E0004 errors not present in EXPECTED_UNIQUE — unregistered additions.
    unregistered: list[str] = []

    try:
        with open(log_path) as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue
                try:
                    record = json.loads(line)
                except json.JSONDecodeError:
                    continue
                if record.get("reason") != "compiler-message":
                    continue
                msg = record.get("message") or {}
                code_block = msg.get("code") or {}
                code = code_block.get("code", "") if isinstance(code_block, dict) else ""
                if code == "E0639":
                    sym = extract_e0639_symbol(msg)
                    if sym:
                        found.add(sym)
                        if sym not in EXPECTED_UNIQUE:
                            unregistered.append(sym)
                elif code == "E0004":
                    # Returns list of 1-seg and 2-seg forms; update found with all.
                    syms = extract_e0004_symbol(msg)
                    found.update(syms)
                    # Unregistered if none of the extracted forms (1-seg or 2-seg) match
                    # any entry in EXPECTED_UNIQUE.
                    if syms and not any(s in EXPECTED_UNIQUE for s in syms):
                        # Use 2-seg form for reporting when available (more informative).
                        label = syms[1] if len(syms) > 1 else syms[0]
                        unregistered.append(label)
    except FileNotFoundError:
        print(f"Error: log file not found: {log_path}", file=sys.stderr)
        return 1

    failures: list[str] = []
    for sym in sorted(EXPECTED_UNIQUE):
        if sym in found:
            print(f"  PASS  {sym}")
        else:
            print(f"  FAIL  {sym}  (not found in E0639/E0004 error output)")
            failures.append(sym)

    print()
    if failures:
        # Canary diff: print all actually-extracted symbols so rustc diagnostic-path
        # drift produces an actionable message rather than a bare "not found" failure.
        # (F-CSD-P32-OBS-001 — drift produces actionable diff, not a bare fail)
        print("\nActually extracted E0639/E0004 symbols (canary diff):", file=sys.stderr)
        for sym in sorted(found):
            print(f"  extracted: {sym!r}", file=sys.stderr)
        print(
            f"\nExpected {len(EXPECTED_UNIQUE)} unique symbols; extracted {len(found)}.",
            file=sys.stderr,
        )
        for sym in failures:
            print(
                f"::error::Symbol {sym!r} did not appear in E0639/E0004 error output — "
                f"#[non_exhaustive] may have been removed from this type, or the rustc "
                f"diagnostic path changed (see canary diff above). "
                f"Regression detected (F-CSD-P29-OBS-001 per-symbol gate).",
                file=sys.stderr,
            )
        print(
            f"Per-symbol check FAILED: {len(failures)} of {len(EXPECTED_UNIQUE)} symbols "
            f"missing from error output.",
            file=sys.stderr,
        )

    if unregistered:
        # An E0639/E0004 error was emitted for a symbol absent from EXPECTED_SYMBOLS.
        # This means a new #[non_exhaustive] type was added to the violation crate
        # without being registered — the count gate should also have fired.
        print(
            f"\nUnregistered symbols found in E0639/E0004 errors but absent from "
            f"EXPECTED_SYMBOLS ({len(set(unregistered))} symbol(s)):",
            file=sys.stderr,
        )
        for sym in sorted(set(unregistered)):
            print(f"  unregistered: {sym!r}", file=sys.stderr)
        print(
            f"\n  To fix: append the symbol(s) to EXPECTED_SYMBOLS in\n"
            f"  scripts/check-non-exhaustive-per-symbol.py. Each #[non_exhaustive]\n"
            f"  type added to the violation crate must be registered before the gate passes.",
            file=sys.stderr,
        )

    if failures or unregistered:
        return 1

    print(
        f"Per-symbol check passed: all {len(EXPECTED_UNIQUE)} distinct symbols "
        f"appear in E0639/E0004 error output ({EXPECTED_COUNT} violations, "
        f"{len(EXPECTED_UNIQUE)} unique types validated)."
    )
    return 0


if __name__ == "__main__":
    if len(sys.argv) == 2 and sys.argv[1] == "--count":
        print(EXPECTED_COUNT)
        sys.exit(0)
    sys.exit(main())
