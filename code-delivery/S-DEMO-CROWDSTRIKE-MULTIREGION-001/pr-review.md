# PR #170 — Fresh-Eyes Review

**Story:** S-DEMO-CROWDSTRIKE-MULTIREGION-001 (CrowdStrike multi-region base_url via `${env.CROWDSTRIKE_BASE_URL}`)
**Head:** f283b8a7 · **Base:** develop@b38c1abc
**Verdict:** REQUEST-CHANGES (1 IMPORTANT demo-evidence accuracy finding; code change itself is sound)

---

## What I verified

- **Diff coherence (checklist 1):** PASS. Three logical changes only — the TOML `base_url` swap + region runbook comment, one new Red Gate test file, and the demo-evidence directory. No unrelated changes. No `prism-dtu-crowdstrike` commits (ADR-031 §D8-c honored). No `prism-spec-engine/src/` production changes (env resolver pre-exists from S-SPEC-ENV-VAR-001).
- **Dependency status (checklist 8):** PASS. S-SPEC-ENV-VAR-001 is merged to develop (`4feec93a`). Hard gate D-914 satisfied — the `${env.CROWDSTRIKE_BASE_URL}` token has a live resolver.
- **Code correctness:** PASS. Confirmed against the PR head:
  - `parse_and_validate_spec_toml` returns `Result<SensorSpec, Vec<ValidationError>>`; `ValidationError` has the `errors: Vec<String>` field the test iterates — test compiles against the real surface.
  - Env-var resolution runs AFTER TOML deserialization, BEFORE URL-format/field-empty validation, returning early with E-SPEC-024 on unset var (fail-closed). AC-003's behavior is real, and its `!contains("must start with http")` ordering guard is a meaningful (if belt-and-suspenders) regression assertion.
  - The DTU loopback `http://127.0.0.1:9999` value loads fine — there is no http/https enforcement on this path, so AC-004's unit assertion is valid.
- **Test coverage (checklist 3):** PASS. 3 non-ignored Red Gate tests cover AC-002/003/004 (spec-load path) and embed AC-005 (D-747 LOCKED `auth_type`/`auth_plugin` assertions). Tests are meaningful, not tautological.
- **SID-1 compliance:** PASS. The `#[ignore]`'d full-pipeline DTU test cites a specific blocking dependency (DTU-EXT-001), a specific future story (S-6.07), AND a specific named non-ignored substitute (`test_BC_2_16_013_crowdstrike_base_url_env_points_to_local_dtu_demo_works`) that genuinely exercises the production load path. This is textbook SID-1.
- **SAP-1 / SAP-2:** PASS by inspection. No `event_type =` emissions added (test + TOML only). No `[[tables]]`/column changes, so no DTU↔TOML parity risk.
- **Commit quality / diff size (checklist 5,6):** PASS. Conventional format with story ID; 857 additions are almost entirely the test file + binary demo assets, which is reasonable for this story type.

---

## Findings

### IMPORTANT-001 — AC-001 demo evidence overstates what the recording shows (demo / description accuracy)

| Field | Value |
|-------|-------|
| Severity | IMPORTANT |
| Category | demo-evidence / description |
| File | `docs/demo-evidence/S-DEMO-CROWDSTRIKE-MULTIREGION-001/evidence-report.md` (line ~758, AC-001 section + coverage table "hardcoded URL absent"); `docs/demo-evidence/.../AC-001-crowdstrike-toml-base-url-env-var.tape` (grep guard) |

**Finding.** The AC-001 evidence-report claims the recording's second command, `grep -c api.crowdstrike.com crates/prism-sensors/specs/crowdstrike.sensor.toml`, "returns `0` (absent), confirming the hardcoded us-1 URL was removed." The PR coverage table likewise asserts "hardcoded URL absent."

This is verifiably false against the PR-head TOML. The change adds a 4-region runbook comment whose first line is `#   us-1 (default):  https://api.crowdstrike.com`. I reproduced the exact command against the PR-head file:

```
$ git show <pr-head>:crates/prism-sensors/specs/crowdstrike.sensor.toml | grep -c api.crowdstrike.com
1
```

The literal `https://api.crowdstrike.com` still appears in the file — in the comment. So the recorded GIF shows `1`, not `0`, and the `|| echo absent` fallback in the tape never fires (grep exits 0 on a match). The report's "absent / removed" narrative contradicts its own recording.

**Why it matters.** Demo evidence is the human reviewer's proof that the AC holds. An evidence report that asserts the opposite of what its recording displays undermines the entire purpose of the artifact — a human cross-checking the GIF against the report will see `1` where the report promised `0` and lose trust in the rest of the evidence pack. The underlying TOML change is correct (the `base_url` *field* no longer hardcodes us-1), but the AC-001 demonstration is mis-described.

**Suggestion.** Make the grep guard discriminate the field from the comment, and correct the report to match. For example, target the assignment specifically:

```bash
# Confirm the base_url FIELD no longer hardcodes us-1 (comment lines are expected to mention it)
grep -E '^base_url\s*=' crates/prism-sensors/specs/crowdstrike.sensor.toml
# → base_url = "${env.CROWDSTRIKE_BASE_URL}"
grep -E '^base_url\s*=.*api\.crowdstrike\.com' crates/prism-sensors/specs/crowdstrike.sensor.toml || echo "field-not-hardcoded"
# → field-not-hardcoded
```

Then re-record AC-001 and update the evidence-report AC-001 section + coverage table to describe the true output (the comment intentionally retains all four region URLs as a runbook; the *field* is env-var driven).

**Routing:** `vsdd-factory:demo-recorder` (owner of `evidence-report.md` and the `.tape` scripts). The re-record + report correction is in-scope for the demo step; no spec change required.

---

## Verdict

**REQUEST-CHANGES.** No code-correctness, security, or contract defects — the TOML change, the env-resolver fail-closed behavior, the Red Gate tests, and the SID-1 `#[ignore]` justification are all production-grade. The single blocker-to-merge is the AC-001 demo-evidence inaccuracy: the evidence-report asserts a grep result (`0`/absent) that the recording cannot show (`1`), because the deliberately-added region runbook comment retains `https://api.crowdstrike.com`. Re-record AC-001 with a field-discriminating guard and correct the report, then this is a clean APPROVE.
