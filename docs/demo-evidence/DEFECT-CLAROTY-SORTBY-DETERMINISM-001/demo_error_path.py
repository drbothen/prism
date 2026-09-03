#!/usr/bin/env python3
"""
Demo helper: demo_error_path.py
Simulates the error condition DEFECT-CLAROTY-SORTBY-DETERMINISM-001 fixed:
a body_template WITHOUT sort_by, showing what the RG detection catches.

This is a SYNTHETIC demonstration — it does not use live data or
the real sensor TOML. It shows the defect class that was present
before this fix, and demonstrates how the RG tests detect it.

Usage:
  python3 demo_error_path.py vulnerabilities
  python3 demo_error_path.py audit_logs
  python3 demo_error_path.py server_interfaces
  python3 demo_error_path.py zones
  python3 demo_error_path.py zone_policies
  python3 demo_error_path.py firewall_groups
  python3 demo_error_path.py firewall_policies
"""

import json
import sys

# Pre-fix body_templates (synthetic — without sort_by) for each table
# These represent the DEFECTIVE state before DEFECT-CLAROTY-SORTBY-DETERMINISM-001
PRE_FIX_TEMPLATES = {
    "vulnerabilities": {
        "fields": [
            "name", "vulnerability_type", "cve_ids", "cvss_v3_score",
            "adjusted_vulnerability_score", "published_date", "epss_score"
        ]
        # sort_by absent: xDome defaults to published_date desc (non-unique)
    },
    "audit_logs": {
        "filter_by": {"test": True}
        # sort_by absent: xDome uses undefined generic SortClause
    },
    "server_interfaces": {
        "fields": [
            "server_name", "interface_name", "interface_status", "site_id"
        ]
        # sort_by absent: xDome defaults to server_name asc only (non-unique alone)
    },
    "zones": {
        "fields": [
            "zone_name", "zone_description", "priority", "enabled"
        ]
        # sort_by absent: xDome defaults to priority asc (non-unique)
    },
    "zone_policies": {
        "fields": [
            "policy_name", "policy_source", "matching_devices"
        ]
        # sort_by absent: xDome defaults to matching_devices asc (non-unique)
    },
    "firewall_groups": {
        "fields": [
            "firewall_group_name", "firewall_group_description", "priority", "enabled"
        ]
        # sort_by absent: xDome defaults to priority asc (non-unique)
    },
    "firewall_policies": {
        "fields": [
            "policy_name", "policy_source", "matching_devices"
        ]
        # sort_by absent: xDome defaults to matching_devices asc (non-unique)
    },
}


def main() -> None:
    table = sys.argv[1] if len(sys.argv) > 1 else "vulnerabilities"
    if table not in PRE_FIX_TEMPLATES:
        print(f"Unknown table: {table}")
        print(f"Valid: {', '.join(PRE_FIX_TEMPLATES)}")
        sys.exit(1)

    body = PRE_FIX_TEMPLATES[table]

    print(f"Pre-fix body_template for {table} (synthetic — no sort_by):")
    print(f"  {json.dumps(body)}")
    print()
    print("Running RG assertion (mirrors RG-001..RG-007 logic):")
    print(f"  assert 'sort_by' in body_template")
    print()

    # Mirror the RG assertion that catches this defect
    # Print to stdout so VHS terminal capture matches correctly
    if "sort_by" not in body:
        print("DEFECT DETECTED: 'sort_by' absent from body_template")
        print(f"  Table:  {table}")
        print(f"  Result: offset-pagination non-deterministic")
        print(f"  xDome default sort for {table}: non-unique field")
        print(f"  Effect: records may duplicate or skip across page boundaries")
        print(f"  Fix:    add sort_by per BC contract")
        print(f"          (DEFECT-CLAROTY-SORTBY-DETERMINISM-001)")
        sys.exit(1)
    else:
        print(f"OK: sort_by present in {table} body_template")


if __name__ == "__main__":
    main()
