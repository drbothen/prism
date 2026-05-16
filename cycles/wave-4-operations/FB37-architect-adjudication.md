---
document_type: architect-adjudication
burst_id: FB37
created: "2026-05-16"
producer: architect
status: final
findings_addressed:
  - F-LP47-HIGH-001
  - F-LP47-LOW-001
source_pass: pass-47
---

# FB37 Architect Adjudication — AtomicBool Set-Time Semantics + Story Frontmatter Scope Intent

## §1 F-LP47-HIGH-001 — AtomicBool Query-Phase Flag Set-Time Canonical Answer

### Evidence Synthesized

Five locations cite the flag set-time. The canonical authority (CLAUDE.md Source-of-Truth
Precedence Rule 2: ADR supersedes story on contract semantics) is ADR-026 §D7.

| Location | Phrasing | Temporal claim |
|----------|----------|----------------|
| ADR-026 §D7 line 287 | "called after step 8 (query-engine init) **started**" | step 8 START |
| ADR-026 §D7 line 296 | "called after step 8 **starts**" | step 8 START |
| Story Task 7b line 188 | "as its first act **after all plugin registrations complete**" | step 8 START (see analysis) |
| BC-2.16.012 EC-016-012-005 line 109 | "set when query engine init **completes** at step 8" | step 8 COMPLETION |
| BC-2.16.002 row 33 line 110 | "called after the query engine init **completes** (step 8+)" | step 8 COMPLETION |
| HS-PREREQ-E-003-05 line 183 | "set when query engine init **completes** at step 8" | step 8 COMPLETION |

ADR-022 §B boot sequence (authoritative step table):
- Step 7.5 [BLOCKING]: `PluginRuntime::load_all_plugins` — all plugin registrations occur here.
- Step 8 [BLOCKING]: `QueryEngine + WriteExecutor construction` — begins only after step 7.5 returns.
- `boot.rs` confirms: `plugin_load_step_with_audit(...).await?` is followed by `step8_init_query_engine().await?` in strict sequential order. Step 8 cannot begin until step 7.5 has fully returned.

### Analysis

The ADR-026 §D7 canonical phrasing is "after step 8 **starts**" (lines 287 and 296). This means the
`AtomicBool` flag is set as the query engine init's first act — immediately when step 8 begins
execution, before any query engine construction work proceeds.

Story Task 7b's phrase "as its first act after all plugin registrations complete" is semantically
equivalent to "at step 8 start" because ADR-022 §B enforces that step 7.5 (all plugin
registrations) completes before step 8 begins. The two phrasings describe the same moment.
Task 7b is not wrong; it is ambiguously worded in a way that suggests the flag is set at
step 7.5 completion rather than at step 8 initiation. The ADR-026 framing ("after step 8 starts")
is clearer and is the canonical form.

BC-2.16.012:109, BC-2.16.002:110, and HS-003-05:183 all say "when query engine init **completes**."
This is a real divergence from ADR-026's "after step 8 **starts**." Semantically, the difference
matters for reasoning about concurrent access: setting the flag at step 8 START (first act) is
the production-grade choice because it closes the write window before any query engine
construction could interleave with a late-arriving (erroneous) registration call. Setting it at
step 8 COMPLETION would leave a window — however brief — during construction where the flag is
not yet set. The ADR-026 "after step 8 starts" framing correctly captures the intended
fail-closed design: the flag fires immediately when boot enters the query-engine phase.

PREREQ-D compatibility check: PREREQ-D (merged at SHA `ec90fe8f`) wires `PluginRuntime` at step
7.5. Write-tool registration via `register_write_tool` happens during step 7.5 (before step 8
begins). Setting the flag at step 8 START does not affect PREREQ-D's registration calls — those
calls complete during 7.5 and the flag is not yet set. PREREQ-D is fully compatible with both
"step 8 starts" and "step 8 completes" semantics, but the former is the correct and canonical
choice per ADR-026.

### Decision: Option A (with clarification)

The flag is set at **step 8 START** — as the query engine init's first act, before QueryEngine
construction proceeds. This matches ADR-026 §D7 lines 287 and 296 (the authoritative spec).

Story Task 7b is correct in intent but ambiguously worded. BC-2.16.012, BC-2.16.002, and
HS-003-05 use "completes" where the canonical word is "starts" — these three locations require
text corrections to align with the ADR-026 authority.

### Canonical Phrasing (durable semantic anchor)

> The `AtomicBool` query-phase flag (`QUERY_PHASE_STARTED`) is set to `true` as the first act
> of step 8 (query-engine init), immediately when the step begins and before any QueryEngine
> construction proceeds — this closes the write window permanently at the step-8 boundary.

This single sentence replaces all variant phrasings across affected artifacts.

### Sibling-Sweep Mandate

Four sites require text edits. Story Task 7b requires a minor reword for clarity; the three
BC/HS sites require substantive correction of "completes" to "starts."

**Site 1 — Story Task 7b, line 188**

Current:
```
The flag is set to `true` by the query-engine init (boot step 8, ADR-022 §B) as its first act
after all plugin registrations complete — this closes the write window permanently.
```

Replace with:
```
The flag is set to `true` as the first act of step 8 (query-engine init, ADR-022 §B) — immediately
when step 8 begins, before any QueryEngine construction proceeds. All plugin registrations at step
7.5 are already complete when step 8 starts; setting the flag here closes the write window
permanently at the step-8 boundary.
```

**Site 2 — BC-2.16.012 EC-016-012-005, line 109**

Current:
```
An `AtomicBool` query-phase flag (set when query engine init completes at step 8) gates the write.
```

Replace with:
```
An `AtomicBool` query-phase flag (set when query engine init starts at step 8 — as the first act
of step 8, before QueryEngine construction proceeds) gates the write.
```

**Site 3 — BC-2.16.002 row 33, line 110 (trigger column)**

Current:
```
`register_write_tool` called after the query engine init completes (step 8+); the `AtomicBool`
query-phase flag is set, gating the write
```

Replace with:
```
`register_write_tool` called after query-engine init starts (step 8+, ADR-026 D7); the `AtomicBool`
query-phase flag is set at step 8 start (first act of step 8, before QueryEngine construction
proceeds), gating the write
```

**Site 4 — HS-PREREQ-E-003-05, line 183 (Preconditions)**

Current:
```
The `AtomicBool` query-phase flag (set when query engine init completes at step 8) is set to
`true` in the test fixture, simulating post-boot context
```

Replace with:
```
The `AtomicBool` query-phase flag (set at step 8 start — as the first act of step 8, before
QueryEngine construction proceeds, per ADR-026 D7) is set to `true` in the test fixture,
simulating post-step-8-start context
```

Note: HS-003-05 step 1 (line 188, "Set the query-phase `AtomicBool` flag to `true` (simulating
step 8 completion)") also uses "completion" — PO must update this to "simulating step 8 start
(mark_query_phase_started() called)".

---

## §2 F-LP47-LOW-001 — Story Frontmatter Scope Intent Decision

### Current frontmatter

```yaml
architectural_decisions: [ADR-026, ADR-027, ADR-023]
subsystems: [SS-01, SS-07, SS-16]
```

### ADR-022 — ADD to `architectural_decisions`

**Decision: YES — add ADR-022.**

Rationale: Story Task 7b explicitly cites "ADR-022 §B" as the boot-step ordering authority for
when the AtomicBool flag is set. Task 7b's entire temporal argument rests on ADR-022 §B's
step 7.5 / step 8 sequence. The frontmatter `architectural_decisions` field enumerates ADRs
that govern the story's implementation decisions; ADR-022 §B is load-bearing for Task 7b.
Omitting it is a completeness gap, not intentional strict-scope discipline. The story already
references ADR-022 §B in body text — the frontmatter omission is an oversight.

### SS-17 — ADD to `subsystems`

**Decision: YES — add SS-17.**

Rationale: ADR-026 v1.7 changelog explicitly added SS-17 to `subsystems_affected` because D7's
runtime deliverables (`register_write_tool` API, `RwLock<Vec<WriteToolInvalidationMap>>`,
`DuplicateWriteToolRegistration` variant, `WriteToolRegistrationAfterBoot` variant, `AtomicBool`
query-phase flag) all land in `crates/prism-query/src/invalidation.rs`. Per ARCH-INDEX:144,
SS-17 owns PluginRuntime. Task 7 wires PluginRuntime to call `register_write_tool` — a direct
SS-17 interaction. The `anchor_subsystem:` field (line 59) also omits SS-17. Both `subsystems:`
and `anchor_subsystem:` must be updated in the same edit.

The omission is not intentional strict-scope discipline — ADR-026's own `subsystems_affected`
already includes SS-17 precisely because of the D7 deliverables this story implements.

---

## §3 PO Dispatch Instructions

PO owns story files, BC files, and holdout scenario files. The following edits must be made as
a single atomic burst (one commit). Version bumps are required where indicated.

### File 1: `.factory/stories/S-PLUGIN-PREREQ-E-unseal-sensor-auth-deprecate-customadapter.md`

**Edit 1a — Frontmatter `architectural_decisions` (line 48–51):**
Add `ADR-022` entry after the `ADR-023` line:
```yaml
  - ADR-022  # Production runtime wiring — §B step 7.5/8 ordering authoritative for Task 7b AtomicBool flag set-time
```

**Edit 1b — Frontmatter `subsystems` (line 24):**
Change:
```yaml
subsystems: [SS-01, SS-07, SS-16]
```
To:
```yaml
subsystems: [SS-01, SS-07, SS-16, SS-17]
```

**Edit 1c — Frontmatter `anchor_subsystem` (line 59):**
Change:
```yaml
anchor_subsystem: [SS-01, SS-07, SS-16]
```
To:
```yaml
anchor_subsystem: [SS-01, SS-07, SS-16, SS-17]
```

**Edit 1d — Task 7b body (line 188):** Apply Site 1 replacement from §1 above.

**Edit 1e — Version bump `version` field:** Increment `"1.18"` to `"1.19"`.

**Edit 1f — `updated` field:** Update to `"2026-05-16"` (already current; verify).

**Edit 1g — Add changelog entry** at bottom of changelog table:
```
| 1.19 | FB37 | 2026-05-16 | architect | F-LP47-HIGH-001: Task 7b AtomicBool flag set-time phrasing clarified — "as its first act after all plugin registrations complete" replaced with canonical ADR-026 D7 framing "first act of step 8, before QueryEngine construction proceeds." F-LP47-LOW-001: ADR-022 added to architectural_decisions; SS-17 added to subsystems and anchor_subsystem (both fields). |
```

### File 2: `.factory/specs/behavioral-contracts/BC-2.16.012-plugin-registry-dispatch-migration.md`

**Edit 2a — EC-016-012-005 (line 109):** Apply Site 2 replacement from §1 above.

**Edit 2b — Version bump:** Increment BC version field (check current value) by one patch.

**Edit 2c — Changelog:** Add entry recording F-LP47-HIGH-001 closure — EC-016-012-005 "completes at step 8" corrected to "starts at step 8" per ADR-026 D7 canonical authority.

### File 3: `.factory/specs/behavioral-contracts/BC-2.16.002-multi-step-fetch-pipeline.md`

**Edit 3a — Row 33 trigger column (line 110):** Apply Site 3 replacement from §1 above.

**Edit 3b — Version bump:** Increment BC version field by one patch.

**Edit 3c — Changelog:** Add entry recording F-LP47-HIGH-001 closure — row 33 trigger "completes (step 8+)" corrected to "starts (step 8+)" per ADR-026 D7.

### File 4: `.factory/holdout-scenarios/S-PLUGIN-PREREQ-E-HS-003-plugin-registry-dispatch.md`

**Edit 4a — HS-PREREQ-E-003-05 Preconditions (line 183):** Apply Site 4 replacement from §1 above.

**Edit 4b — HS-PREREQ-E-003-05 Step 1 (line 188):** Change "simulating step 8 completion" to
"simulating post-step-8-start context (mark_query_phase_started() called)".

**Edit 4c — Version bump:** Increment HS version field by one patch if versioned; otherwise note amendment in document header.

### Commit discipline

All four files must be staged and committed as one commit (TD-VSDD-053 single-commit-per-burst).
Suggested commit subject:
```
fix(specs): F-LP47 — canonicalize AtomicBool flag set-time to ADR-026 D7 "step 8 starts" + add ADR-022/SS-17 to PREREQ-E frontmatter
```

No `Co-Authored-By` line per project git rules.

### Post-edit validation

After commit, PO or state-manager must confirm:
1. All four files use "step 8 start / starts" (grep `"completes at step 8"` across `.factory/` — expect zero hits in PREREQ-E scope).
2. Story frontmatter `architectural_decisions` includes ADR-022.
3. Story frontmatter `subsystems` and `anchor_subsystem` both include SS-17.
4. ADR-026 §D7 lines 287 and 296 remain unchanged (they are already canonical — do NOT edit ADR-026).
