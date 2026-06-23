---
document_type: research
producer: black-box-audit (claude, MCP-only information asymmetry)
timestamp: 2026-06-23
topic: Prism MCP onboarding / PrismQL enrichment discoverability audit (re-conducted)
status: complete
feeds: S-DEMO-PRISMQL-ONBOARDING-001-C remediation
---

# Prism MCP Onboarding Discoverability Audit — 2026-06-23

Role: a competent security analyst (LLM agent) connecting Prism's MCP server for the
first time. Strict information asymmetry: the only knowledge of Prism used to drive
discovery came from the live MCP surface (tool list + schemas, `prism_describe`,
MCP resources, MCP prompts, and the tool/query responses + error messages).

Branch `fix/enrichment-complete` @ 307c7741, worktree `.worktrees/enrich-integrated`,
DTU mode.

---

## 1. Method — how the MCP surface was accessed (validity basis)

- **Bring-up (opaque ops tooling, not discovery input):** `scripts/demo-setup.sh`
  (provisions demo config + 10 keyring credentials) then `scripts/demo-run.sh`
  (starts `prism-dtu-demo-server start-multi`, writes per-org overlays with ephemeral
  DTU ports, prints the `prism start` env block). I read `demo-run.sh` only to learn the
  launch command and env vars; I deliberately did **not** use the spec/BC references in
  its comments, nor the `SELECT * FROM crowdstrike_detections`/`cyberint_alerts` example
  echoed in setup output, as discovery hints. A naive MCP client never sees those.
- **Transport:** `prism start` is a **stdio MCP server** (blocks until SIGTERM; confirmed
  via `prism start --help` / `prism --help`). I drove it exactly as an MCP client does:
  a small line-delimited JSON-RPC client (`/tmp/mcp_client.py`) that spawns
  `prism start --config-dir ~/.config/prism-demo`, performs the `initialize` handshake +
  `notifications/initialized`, then issues `tools/list`, `resources/list`,
  `prompts/list`, `resources/read`, `prompts/get`, and `tools/call`.
- **Handshake result:** `serverInfo {name: prism, version: 0.1.0}`, capabilities
  `{prompts, resources(subscribe), tools}`, protocol `2024-11-05`.
- **Surface inventory observed:** 53 tools, 3 resources
  (`prism://config/clients`, `prism://sensors/health`, `prismql://reference`),
  5 prompts (`client_overview`, `cross_client_status`, `investigate_host`,
  `query_tutorial`, `triage_alerts`).
- No `crates/`, `.factory/`, `docs/`, `src/`, `.rs`/`.toml`/`.md` spec file was read for
  discovery, and no codebase grep/glob/git-history was used. (The one file read,
  `demo-run.sh`, was treated as opaque launch tooling per the audit charter.)

---

## 2. Goal 1 — "Show me recent alerts for the Cyberint client"

### Transcript
1. `query` tool description (read from `tools/list`) explicitly says:
   *"Call `prism_describe` with the client_id before writing queries… Read
   prismql://reference for full grammar."* Good — names the discovery path.
2. **Obvious-tool trap:** `list_alerts` looks like the right tool for "list recent
   alerts". Its description and a live call both return **`-32003` "not implemented —
   prism-operations not yet merged"**. The alerts path is actually PrismQL `query`, not
   `list_alerts`. (`get_help`, `get_diagnostics`, `list_packs`, `list_infusions` are
   likewise `-32003`.)
3. Read resource `prism://config/clients` →
   `[{client_id:"org-a",sensors:[crowdstrike,armis]}, {org-b:[claroty,cyberint]},
   {org-c:[armis,crowdstrike,claroty,cyberint]}]`. So "the Cyberint client" = **org-b**
   (cyberint also on org-c). Good discoverability — clean mapping from sensor → client_id.
4. `prism_describe(client_id="org-b")` → tables array. The cyberint table is reported as
   `{ name: "alerts", sensor_type: "cyberint", columns:[alert_id,title,severity,status,
   created_at, ioc_type, ioc_value_singleton, iocs_value, alert_data_ip, …] }`. Note:
   **two** tables are named `alerts` (one claroty, one cyberint) — the `name` field alone
   is ambiguous.
5. **The trap fires.** Using the `prism_describe` `name` verbatim:
   `SELECT * FROM alerts …` → **`E-QUERY-036` unknown source table 'alerts'**.
6. Guessed `FROM cyberint` (sensor name) → **`E-QUERY-037`**, and *that error lists the
   real table names*: `[claroty_alerts, claroty_audit_logs, claroty_devices,
   cyberint_alerts, cyberint_incidents]`. This is the self-correction lifeline.
7. `SELECT * FROM cyberint_alerts LIMIT 5 clients=[org-b]` → **5 rows, data_source
   cyberint**. Final query
   `SELECT alert_id,title,severity,status,created_at FROM cyberint_alerts ORDER BY
   created_at DESC LIMIT 5` → 5 recent alerts returned correctly.

### Verdict — SUCCEEDED-with-friction
The data is reachable, and the `E-QUERY-037` error is genuinely good (lists valid table
names). But the **primary discovery tool actively misleads**: `prism_describe` reports
`name:"alerts"`, yet `FROM` requires the underscore-qualified `cyberint_alerts`, which the
tool never emits as a single usable token. A naive agent that trusts `prism_describe`'s
`name` field fails on the first query and only recovers by guessing a second wrong form
(`FROM cyberint`) whose error happens to leak the real names.

---

## 3. Goal 2 — Enrichment (IOC threat scores + device-CVE CVSS)

### Transcript (every documented discovery path, then black-box probing)
1. **`prismql://reference`** (the resource the `query` tool tells you to read) — 6,471
   chars. BNF grammar covers only `SELECT/FROM/WHERE/GROUP BY/ORDER BY/LIMIT`, operators,
   datetime arithmetic, examples, error codes. **Zero occurrences** of enrich / infuse /
   threat / cvss / cve / score / JOIN. No enrichment syntax documented at all.
2. **Infusion tools** — the only tools whose descriptions say "data enrichment pipelines":
   `list_infusions`, `infusion_status`, `reload_infusion`. All three are documented as
   "not yet wired"; a live `list_infusions` call returns **`-32003` "Feature not yet
   available: infusion management (prism-operations not merged)"**. Dead end.
3. **`list_capabilities(org-b)`** → empty capability set. No enrichment capability
   advertised.
4. **Prompts.** `query_tutorial(client_id=org-b, goal="enrich alerts with threat
   intelligence and CVE CVSS scores")` echoes the goal in Step 5 but teaches only
   `prism_describe` + the (enrichment-free) grammar reference — **no enrichment syntax**.
   `triage_alerts(org-b)` teaches `SELECT * FROM crowdstrike.alerts …` and
   `claroty.alerts …` — **dot-qualified** table names.
5. **Dot syntax contradiction.** `SELECT * FROM cyberint.alerts` (the form the server's own
   prompt teaches) → **`E-QUERY-036` unknown source table 'cyberint.alerts'**. The prompts
   teach a table-reference syntax the query engine rejects.
6. **Black-box syntax probing** of the `query` tool (errors as the only teacher):
   - `… threat_score(ioc_value_singleton) …` → **parses OK, returns 3 rows**, but the
     output rows contain **only `alert_id`** — both the function column *and* the
     `ioc_value_singleton` column are silently dropped. No error, no warning.
   - `… | ENRICH threat_intel ON …` and `… ENRICH WITH threatintel …` → `E-QUERY-001`
     parse errors that incidentally reveal the grammar **does** support `JOIN`
     (INNER/LEFT/RIGHT/FULL/CROSS) — undocumented in `prismql://reference`.
   - `… LEFT JOIN threatintel ON …` and `… JOIN nvd …` → **`E-QUERY-037`**: `threatintel`
     / `nvd` are not registered tables; only the 5 sensor tables exist. No joinable
     enrichment source is exposed.
   - `ENRICH SELECT …` → parse error revealing another undocumented keyword, **`MATCHES`**.
   - `… behaviors_ioc_value MATCHES threat_intel …` → parse error (`MATCHES` is a
     string-match operator expecting a quoted literal, not an enrichment verb).
   - `cvss(device_cves_first) FROM armis_devices` → **`E-INT-001` Internal error; see audit
     log** (a function call on a real column triggers an internal crash; the "see audit
     log" remedy is inaccessible to an MCP client).
   - Plain `SELECT device_cves_first FROM armis_devices` and
     `SELECT ioc_value_singleton FROM cyberint_alerts` → rows returned but with
     **only the id column** — the advertised IOC/CVE columns silently vanish from output.
7. `explain_query`, `list_aliases` (empty), `list_packs` (`-32003`), `get_diagnostics`
   (`-32003`), `get_help` (`-32003`) — none reveal enrichment.

### Verdict — FAILED (from the surface)
Enrichment is genuinely the flagship per the worktree name, and the DTU back-ends
(threatintel @ :58792, nvd @ :58793) are running — but **nothing on the MCP surface
teaches, names, or exposes the enrichment syntax or functions.** Every documented path is
empty (`prismql://reference`), -32003 (`list_infusions`, `get_help`), contradictory
(prompts teach `sensor.alerts` which the engine rejects), or silent (advertised IOC/CVE
columns drop from results without error). A real naive agent would conclude enrichment is
not available and give up — or, worse, believe `threat_score(...)` "worked" because it
returned rows with no error, while silently getting no enriched data.

---

## 4. FINDINGS LIST

**AUDIT-001 — `prism_describe` table `name` is not what `FROM` accepts (BLOCKER)**
The discovery tool reports `name:"alerts"` (+ separate `sensor_type`), but
`FROM alerts` → `E-QUERY-036`. The accepted token is the underscore-qualified
`cyberint_alerts`, which `prism_describe` never emits as a usable identifier. An agent that
trusts the primary discovery tool fails its first query.
Surface element at fault: **`prism_describe` tool output (`tables[].name`)**; error
`E-QUERY-036`. Fix: make `prism_describe` emit the FROM-ready qualified name (e.g.
`from_name: "cyberint_alerts"`) or have its `example_query` use the qualified name.

**AUDIT-002 — Enrichment is completely undiscoverable from the MCP surface (BLOCKER)**
No tool description, no resource, no prompt, and no error message documents how to enrich.
`prismql://reference` has zero enrichment content; the `infusion_*` tools are `-32003`;
`list_capabilities` is empty. The flagship feature has no MCP-discoverable entry point.
Surface element at fault: **`prismql://reference` resource** (missing enrichment grammar)
+ **`query` tool description** (mentions no enrichment) + **`list_infusions`/`infusion_status`/`reload_infusion`** (all `-32003`).

**AUDIT-003 — Advertised columns silently drop from results (BLOCKER)**
`prism_describe` advertises `ioc_value_singleton`, `iocs_value`, `device_cves_first`, etc.,
but selecting them returns rows containing only the id column — no error, no `safety_flag`,
no `_meta` warning. Combined with AUDIT-002 this is the most dangerous failure: an agent
"succeeds" (rows + no error) while getting none of the data it asked for, including the
exact IOC/CVE columns enrichment depends on.
Surface element at fault: **`query` tool result projection** vs **`prism_describe` columns**
contract; no error code emitted (should be at least a `safety_flag`/warning or
`E-QUERY-0xx` for an unresolvable advertised column).

**AUDIT-004 — Server-provided prompts teach a non-working table syntax (MAJOR)**
`triage_alerts` instructs `SELECT * FROM crowdstrike.alerts` / `claroty.alerts`
(dot-qualified), but `FROM cyberint.alerts` → `E-QUERY-036`. The engine accepts only
underscore form. The server's own guidance contradicts the engine.
Surface element at fault: **prompt `triage_alerts`** (and the `<sensor>.<table>` mental
model it sets up). Fix: regenerate prompt bodies from the real table registry.

**AUDIT-005 — Function call on a real column returns `E-INT-001` with an inaccessible remedy (MAJOR)**
`cvss(device_cves_first)` parsed then failed with `E-INT-001 "Internal error; see audit
log"`. An MCP client cannot see the audit log, so the error is unactionable; and a
function-call projection that the parser accepts should never reach an internal-error path.
Surface element at fault: **`query` tool**, error `E-INT-001`. Either reject unknown
projection functions at parse time with a `did_you_mean`, or implement them — not crash.

**AUDIT-006 — Grammar reference omits `JOIN` and `MATCHES` (MAJOR)**
The parser supports `JOIN` (INNER/LEFT/RIGHT/FULL/CROSS) and a `MATCHES` keyword, both
discovered only by reading parse-error token lists. `prismql://reference` documents
neither. Agents cannot use real grammar features they can't see, and JOIN is the natural
mechanism an agent would reach for to enrich.
Surface element at fault: **`prismql://reference` resource** (incomplete BNF).

**AUDIT-007 — Obvious "alerts" tool is a `-32003` dead end with no redirect (MAJOR)**
`list_alerts` is the natural first choice for "list recent alerts" but returns `-32003
not implemented` and does **not** point the agent to the working path (`query` +
`cyberint_alerts`). Same for `get_help` (the documentation tool) and `get_diagnostics`.
Surface element at fault: **`list_alerts` tool**, error `-32003`. Fix: either implement, or
return a redirect message naming the `query` tool + `prism_describe`.

**AUDIT-008 — Two tables share `name:"alerts"` in `prism_describe` with no disambiguation guidance (MINOR)**
For org-b, both claroty and cyberint tables report `name:"alerts"`; the only differentiator
is the separate `sensor_type` field. Nothing tells the agent the disambiguated FROM token
is `<sensor_type>_<name>`. (Root cause shared with AUDIT-001.)
Surface element at fault: **`prism_describe` `tables[]`** shape.

**AUDIT-009 — `query_tutorial` accepts an enrichment `goal` but gives no enrichment help (MINOR)**
Passing `goal="enrich alerts with threat intel and CVE CVSS"` produces a tutorial that
echoes the goal but routes only to `prism_describe` + the enrichment-free grammar ref. The
prompt advertises goal-awareness it does not deliver for the flagship use case.
Surface element at fault: **prompt `query_tutorial`**.

---

## 5. Bottom line

- **Goal 1 (recent Cyberint alerts): SUCCEEDED-with-friction.** Reached real rows via
  `SELECT … FROM cyberint_alerts … clients=[org-b]`, but only after the
  `prism_describe`-name trap (AUDIT-001) and recovering off the `E-QUERY-037` error that
  leaks the real table names.
- **Goal 2 (enrichment — IOC threat scores + device-CVE CVSS): FAILED.** The MCP surface
  does not teach, name, or expose any working enrichment syntax or function. Worse,
  advertised IOC/CVE columns silently drop (AUDIT-003) and `threat_score(...)` returns
  rows with no error — so a naive agent is led to believe enrichment "worked" while
  getting nothing. Unaided, a real analyst agent would give up on enrichment.
