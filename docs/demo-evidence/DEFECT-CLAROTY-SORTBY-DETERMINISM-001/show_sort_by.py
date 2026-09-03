#!/usr/bin/env python3
"""
Demo helper: show_sort_by.py
Extracts and displays the sort_by array for a Claroty xDome table
from claroty.sensor.toml.  Used in VHS demo recordings for
DEFECT-CLAROTY-SORTBY-DETERMINISM-001.

Live-data-free: reads only the static TOML spec file.
No network calls, no tenant data, no credentials required.

Usage:
  python3 show_sort_by.py                  # all 7 tables
  python3 show_sort_by.py vulnerabilities  # single table (short name)
  python3 show_sort_by.py claroty_vulnerabilities  # full table name
"""

import json
import sys
import tomllib
from pathlib import Path

TOML_PATH = Path("crates/prism-sensors/specs/claroty.sensor.toml")

STEP_TO_TABLE = {
    "fetch_vulnerabilities":              "claroty_vulnerabilities",
    "fetch_audit_logs":                   "claroty_audit_logs",
    "fetch_server_interfaces":            "claroty_server_interfaces",
    "fetch_organization_zones":           "claroty_organization_zones",
    "fetch_organization_zone_policies":   "claroty_organization_zone_policies",
    "fetch_organization_firewall_groups": "claroty_organization_firewall_groups",
    "fetch_organization_firewall_policies": "claroty_organization_firewall_policies",
}

# Short-name aliases for CLI convenience
SHORT_ALIASES = {
    "vulnerabilities":  "fetch_vulnerabilities",
    "audit_logs":       "fetch_audit_logs",
    "audit":            "fetch_audit_logs",
    "server_interfaces":"fetch_server_interfaces",
    "server":           "fetch_server_interfaces",
    "zones":            "fetch_organization_zones",
    "organization_zones": "fetch_organization_zones",
    "zone_policies":    "fetch_organization_zone_policies",
    "firewall_groups":  "fetch_organization_firewall_groups",
    "firewall_policies":"fetch_organization_firewall_policies",
}


def resolve_target(arg: str | None) -> str | None:
    """Return the canonical step name for a user-supplied target, or None for 'all'."""
    if arg is None:
        return None
    # Direct match on step name
    if arg in STEP_TO_TABLE:
        return arg
    # Short alias
    if arg in SHORT_ALIASES:
        return SHORT_ALIASES[arg]
    # Full table name  (claroty_vulnerabilities → fetch_vulnerabilities)
    for step, table in STEP_TO_TABLE.items():
        if arg == table:
            return step
    return arg  # Let caller handle unmatched


def main() -> None:
    target_step = resolve_target(sys.argv[1] if len(sys.argv) > 1 else None)

    with TOML_PATH.open("rb") as fh:
        spec = tomllib.load(fh)

    found = False
    for table in spec.get("tables", []):
        for step in table.get("steps", []):
            step_name = step.get("name", "")
            if step_name not in STEP_TO_TABLE:
                continue
            if target_step and step_name != target_step:
                continue

            found = True
            table_name = STEP_TO_TABLE[step_name]
            body_raw = step.get("body_template", "")

            # Replace TOML variable substitutions so JSON is parseable
            body_clean = body_raw.replace(
                "${query.filter._claroty_audit_filter_by}", '{"test":true}'
            )

            try:
                bt = json.loads(body_clean)
                sort_by = bt.get("sort_by", "MISSING - not present in body_template")
                has_filter_by = "filter_by" in bt
            except json.JSONDecodeError as exc:
                sort_by = f"JSON parse error: {exc}"
                has_filter_by = False

            label = f"Table: {table_name}"
            sep = "=" * len(label)
            print(sep)
            print(label)
            print(f"Step:  {step_name}")
            print(f"sort_by: {json.dumps(sort_by, indent=2)}")
            if table_name == "claroty_audit_logs":
                print(f"filter_by coexists: {has_filter_by}  "
                      "(audit_logs: filter_by MUST NOT be displaced)")
                print("Note: timestamp-only canonical — compound form with 'id' RETIRED")
                print("      (live-validated 2026-09-02: id in sort_by returns 0 rows)")
            print()

    if not found and target_step:
        print(f"ERROR: table '{sys.argv[1]}' not found in {TOML_PATH}")
        sys.exit(1)


if __name__ == "__main__":
    main()
