---
document_type: story
story_id: "DEFECT-DEMOSETUP-PRISMBIN-ACL-001"
title: "demo-setup.sh hardcodes prism binary path with no override flag"
wave: tbd
epic_id: demo-infra
priority: P2
status: draft
version: "0.1"
severity: MED
level: ops
producer: story-writer
timestamp: "2026-08-03"
modified: "2026-08-03"
inputs:
  - .factory/planning/findings-remediation-2026-07-20/triage-capture.md
  - scripts/demo-setup.sh
origin_finding: "DEFECT-DEMOSETUP-PRISMBIN-ACL-001 (D-1882 narrative only)"
origin_cascade: "D-1889 live-demo triage 2026-07-20; designated for PR #229 (closed lane)"
cycle: "v1.0.0-greenfield"
phase: 3
track: "Platform Engineering"
behavioral_contracts: []
# BC status: no governing BC authors this behavior for scripts/demo-setup.sh.
# Per S-7.01 gate: behavioral_contracts is empty — status MUST remain draft until
# a product-owner authors and anchors a BC, or the product-owner confirms no BC
# is needed for a scripting-convenience fix at this severity.
# This is a MED scripting defect; PO may waive BC authorship.
verification_properties: []
depends_on: []
blocks: []
points: 0
risk: MED
holdout_scenarios: []
assumption_validations: []
risk_mitigations: []
# tdd_mode: NOT SET — tdd_mode and Red Gate enumeration (RG-001..RG-NNN) are
# deferred to specification time per SAC-1.
---

# DEFECT-DEMOSETUP-PRISMBIN-ACL-001: demo-setup.sh hardcodes prism binary path with no override flag

## Problem

`scripts/demo-setup.sh` sets the prism binary path unconditionally:

```
PRISM_BIN="${REPO_ROOT}/target/release/prism"
```

There is no `--prism-bin` flag, no `PRISM_BIN` environment-variable override, and no
other mechanism to supply a binary from outside the default release build location. Any
caller that wants to run the demo setup against a binary built elsewhere — a CI artifact
directory, a cross-compiled binary at a custom path, a specific test binary, a release
candidate — cannot do so without editing the script.

This was not a problem when the demo setup was first authored, but it is a problem for
reproducible demo validation in CI and for local runs against non-default build targets
(e.g., when `CARGO_TARGET_DIR` is set to a non-default path, or when verifying a
pre-built release artifact).

## Origin

**D-1882 narrative registration (2026-07-20):** The defect was recorded narratively in
the D-1882 triage session. It was designated to fold into PR #229 alongside
`DEFECT-DEMOSETUP-CWD-001` and `DEFECT-DEMOSETUP-NEXTSTEPS-001`.

**PR #229 lane closure (2026-07-20):** PR #229 merged as commit `a41599fe0` carrying
`DEFECT-DEMOSETUP-CWD-001` (cwd-independence) and `DEFECT-DEMOSETUP-NEXTSTEPS-001`
(config-dir-aware guidance) only. The binary-path ACL fix was never included in the
PR. The lane closed without it.

**Confirmed still present on develop (2026-08-03):** `scripts/demo-setup.sh`
still sets `PRISM_BIN` to `${REPO_ROOT}/target/release/prism` with no override
mechanism anywhere in the file.

**Why this matters as a provenance case:** This defect is the concrete proof-case that
untracked narrative-only defects get silently lost. D-1882 named it; PR #229 was the
designated carrier; the lane closed; no tracking artifact existed; the fix never happened.
The registration effort that produced this stub (and its siblings) was prioritized
specifically because of this class of silent loss.

## Authority

No governing BC exists for `scripts/demo-setup.sh` behavior. No ADR governs
demo-script binary path resolution.

**Nearest governing artifacts (informational):**

| Artifact | Verbatim status | Relevance |
|----------|-----------------|-----------|
| S-DEMO-003 (demo-setup scripts and runbook) | — (check STORY-INDEX) | Original demo-setup story; may reference intended override behavior |

A product-owner decision is needed on whether a new BC is required for this scripting
convenience fix at MED severity. If the PO waives BC authorship (acceptable for a
single-script convenience improvement), `behavioral_contracts` may remain empty and the
implementer works from the acceptance criteria in the story body.

## Routing

Route: **implementer**

No architect adjudication needed. The fix is straightforward: add a `--prism-bin` flag
(or honor a `PRISM_BIN` environment-variable override if already set) before falling back
to the hardcoded default. The implementer defines the exact override mechanism and records
it in the acceptance criteria when the story is specified.

Product-owner must decide whether a BC is required before `status: ready`.

## Scope — NOT YET SPECIFIED

Acceptance criteria, Red Gate test enumeration (RG-001..RG-NNN), BC-5.38.001 density
check, task decomposition, and story-point estimate are deferred to specification time.
This stub registers the defect as a trackable artifact and records its provenance (D-1882
narrative → PR #229 dropped lane → confirmed present on develop).

`tdd_mode` and Red Gate enumeration will be set when acceptance criteria are authored.

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 0.1 | 2026-08-03 | story-writer | Initial registration stub; records D-1882 narrative-only origin, dropped-lane provenance from PR #229 (a41599fe0), and confirmed presence on develop; no ACs or implementation guidance |
