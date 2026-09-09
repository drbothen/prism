#!/usr/bin/env bash
# Helper for demo recording AC-008: PSScriptAnalyzer clean check
# Usage: bash docs/demo-evidence/S-REL-SPECS-TARBALL-001/run-psscriptanalyzer.sh
set -euo pipefail
result=$(pwsh -Command "Invoke-ScriptAnalyzer -Path scripts/install.ps1 -Severity Error | Format-Table -AutoSize | Out-String")
if [[ -z "${result}" ]]; then
  printf 'PSScriptAnalyzer: PASS (zero Error-severity findings)\n'
else
  printf 'PSScriptAnalyzer: FAIL\n%s\n' "${result}"
  exit 1
fi
