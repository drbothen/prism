---
document_type: research
produced_by: research-agent
status: capture
do_not_execute: true
timestamp: "2026-06-27"
program: day-2-vision SIDE-ANALYSIS (OUT-OF-BAND; SEPARATE from the live VSDD factory pipeline)
pillar: C9 — Config-Management (storage-mechanism narrow cut)
scope_fence: "git-as-primary-vs-write-behind STORAGE MECHANISM for config + detection-content versioning. Storage TAXONOMY (Postgres central / SQLite satellite) is SETTLED and NOT reopened. DB-authoritative + write-behind is DECIDED by the human; this pass validates that decision against prior art and answers the runtime-config-vs-detection-content domain split for the mechanism."
non_contradiction_reads:
  - .factory/research/config-management-depth-2026-06-27.md (C9 depth pass — Q2 domain-split hybrid; this pass is the storage-MECHANISM cut under that, not a re-run of the authority-domain split)
  - matured-vision-day2-requirements.md §11.2 (config store), §14.3 (storage taxonomy DECIDED), §3.1 (central pivot)
  - CLAUDE.md (AD-007 ArcSwap hot-path snapshot; AD-017 reference-based AI-opaque credentials; eat-our-own-dogfood TOML connectors)
decided_context_not_reopened:
  - "Prism day-2 = centralized appliance; config authored ONLY via UI/API (no hand-edited TOML outside UI)."
  - "Storage taxonomy SETTLED: Postgres (bundled central) + SQLite (embedded satellite-edge)."
  - "Human DECIDED: DB-authoritative for ALL config; UI-only authoring; LOCAL git-backed (or git-semantics-over-the-store) WRITE-BEHIND for history/rollback/audit; detection-content+recipes additionally exportable + OPT-IN remote git for detection-content specifically (residency-gated); NO GitHub dependency for anything else; air-gap-capable is HARD."
  - "Config read on the HOT PATH every query via ArcSwap snapshot (AD-007); multi-tenant; per-tenant hot-reload."
# CAPTURE artifact. Records cited prior art + a clear analytical position on the storage-mechanism
# question. Modifies NO live spec/BC/ADR/story/STATE/SESSION-HANDOFF/RESEARCH-INDEX. NOT git-added.
# Leans/verdict are discussion input only — not decisions.
---

# Git-as-Primary vs. DB-Primary + Write-Behind — Storage-Mechanism Research

> **READ FIRST.** Out-of-band day-2 side-analysis CAPTURE. `do_not_execute`. Modifies no live artifact;
> not added to RESEARCH-INDEX.md (per the hard boundary on this dispatch). The storage TAXONOMY
> (Postgres central / SQLite satellite) and the DB-authoritative-with-write-behind DECISION are SETTLED
> premises. This pass answers the narrow STORAGE-MECHANISM question: *can git be the primary store, is
> DB-primary+write-behind the mainstream/correct default, and does detection-content warrant a different
> mechanism than runtime config?* It is the mechanism cut UNDER the C9 depth pass's Q2 authority-domain
> split — it does not re-run that split.

**Confidence legend:** [web] = verified web/doc finding with citation · [web-bench] = published benchmark number · [model-knowledge] = model knowledge, not independently re-verified this pass · [INCONCLUSIVE] = could not verify from sources this pass.

**Landscape date:** findings current as of 2026-06-27. Git-for-data and detection-as-code landscapes move fast; version pins spot-checked below.

**Relationship to the C9 depth pass.** `config-management-depth-2026-06-27.md` already took a position on the *authority-domain* split (Q2): Git-authoritative-with-store-as-cache for the global/high-blast-radius domain, DB-authoritative-with-git-audit for the per-tenant/runtime-mutable domain. This pass is **consistent with that** but answers a different and lower-level question — the *physical storage mechanism* (real-git-as-datastore vs. relational DB vs. DB-native temporal vs. embedded-git write-behind). Where the depth pass said "store is a materialized projection," this pass characterizes *what that store is made of* and *what the write-behind is made of*. No contradiction; this is the layer beneath.

---

## 0. The question, stated precisely

Three sub-questions, answered with a clear position in §7:

- **(a)** Can a git-backed store serve as the **PRIMARY source of truth** (the thing read on the hot path, the thing mutated by the UI)?
- **(b)** Is **"DB-authoritative + git/version-log as a write-behind rollback/audit layer"** the mainstream-and-correct default?
- **(c)** Does the answer differ between **(i) RUNTIME CONFIG** (hot-path, UI-mutated, multi-tenant, ArcSwap snapshot) and **(ii) DETECTION CONTENT / RECIPES** (code-like, versioned, diffable, shareable, opt-in remote)?

---

## 1. Git-as-primary-datastore — the real category

This is a genuine product category ("git-for-data"), not vaporware. Four representative systems, characterized below, plus the "config-directly-in-a-git-repo" pattern.

### 1.1 Dolt — git-for-data over SQL tables (the strongest candidate)

Dolt is "a SQL database that you can fork, clone, branch, merge, push and pull just like a Git repository," MySQL wire-compatible, with version-control surfaced via SQL system tables/functions/procedures. [web — github.com/dolthub/dolt] Data is stored in a **prolly tree** (a probabilistic B-tree / Merkle-DAG hybrid) which is what makes diff/merge scalable and gives **structural sharing across revisions** — storage cost is proportional to the *rate of change*, not the number of revisions (100 rows + 10 updated ≈ 110 rows of storage, not 210). [web — HN 31847416; r/programming ly7no4]

**Workload fit.** Collaborative data curation, regulatory reconstruction ("data as-of a reporting date"), branch-per-environment dev/prod gating. DoltHub states most paying customers run Dolt **online as an OLTP database backing applications** and offline for dataset versioning. [web — dolthub.com/blog/2021-03-09 use-cases; heise.de 8990862]

**Performance (published benchmarks).** Dolt 1.0 (2023) introduced a from-scratch OLTP storage format and reported **~1.9× slower than MySQL on sysbench** (1.3× writes, 2.3× reads). [web-bench — dolthub 2023-05-05] The original noms-based format had been ~4.4× slower. [web-bench — HN 31847416] By **Dec 2025 Dolt reported it "officially matches MySQL's performance on Sysbench"** — read_write mean multiplier 1.12, down from 1.16. [web-bench — dolthub 2025-12-12] On **TPC-C transactional throughput** Dolt achieves **~40% of MySQL's throughput** (~40 tps vs ~100 tps at SF=1) — a real gap on high-throughput transaction benchmarks even after the read optimizations. [web-bench — research synthesis citing dolthub benchmarks] Each change costs **~4KB on disk minimum**, so a high-commit-rate workload needs more disk than the row count alone implies. [web — HN 31847416]

**Concurrency model — load-bearing nuance.** Dolt does **no locking**; it "tries to sort out everything via merge at commit time," in contrast to MySQL which locks heavily for client consistency. [web — console.substack.com/p/console-132] It supports `REPEATABLE_READ` isolation; two transactions touching the same rows resolve by last-committer-fails-and-rolls-back (normal SQL isolation, not a "merge"). [web — HN 31847416] **Merge semantics are row/cell-level**: two writers editing *different columns of the same row* merge cleanly; *same column, different values* is a conflict to resolve. Works for schema changes too. [web — r/programming ly7no4]

**Maturity.** Dolt 1.0 shipped 2023 with forward-compatible storage, ACID, 99.87% MySQL compatibility on ~6M correctness tests, stable git-style CLI, and binlog-replica capability (run Dolt as a versioned MySQL replica). [web — dolthub 2023-05-05] Real production OLTP users exist, but the base is far narrower and younger than Postgres/MySQL. [web — heise.de 8990862]

**Failure modes for the appliance's config use case.** (1) **Repo/metadata bloat from many tiny commits** — commit-per-config-change generates many objects; storage is rate-of-change proportional but the commit graph deepens and GC/repack cost accrues [model-knowledge, consistent with git-content-addressed-store experience; not in Dolt's published docs]. (2) **Read latency vs. an indexed DB** — competitive on point reads now, but reconstruction across the commit graph for as-of queries is heavier than a single indexed row read [model-knowledge]. (3) **No-locking-merge-at-commit** is great for branch workflows but is the *opposite* of what hot-path multi-tenant config wants (config wants a single canonical value per tenant, applied atomically, not merged). (4) **Critically: no cited prior art runs Dolt as the authoritative hot-path store for multi-tenant runtime config.** [web — absence confirmed across DoltHub case studies] Dolt-for-*rules* (detection content) is far more defensible than Dolt-for-runtime-config.

### 1.2 TerminusDB — git-for-data over semantic graph/document

TerminusDB is a "model-based, in-memory and distributed RDF+JSON knowledge graph database with git-for-data collaboration," with succinct auto-indexing data structures and a closed-world assumption. [web — terminusdb.com] v12 added a `sys:JSON` type for arbitrary JSON storage *with full version control* (up to 256 decimal places in unstructured JSON; structured numeric capped at 20 dp "for performance and to comply with XML Schema"). [web — terminusdb v12 release blog] **Workload fit:** semantic content infrastructures, document/knowledge graphs, versioned schema evolution — *not* general runtime config. **Performance:** only qualitative ("high in-memory performance") in available sources — **no published benchmarks comparable to Dolt's sysbench** [INCONCLUSIVE on numbers]. **Concurrency/isolation:** commit/branch model; isolation levels not documented in available sources [INCONCLUSIVE]. **Verdict for Prism:** attractive for *richly-structured detection content*, but its specialization and absence of config-store production prior art argue strongly against it as the authoritative runtime-config store.

### 1.3 Irmin — embedded git-like branchable store (OCaml, not Rust)

Irmin is the MirageOS branchable, content-addressed, git-like embedded store. **It is written in OCaml; there is no production-ready Rust port.** [web — perplexity_ask, citing lib.rs/crates.io ecosystem; corroborated by absence of any Irmin crate] This is decisive for Prism: a Rust appliance cannot embed Irmin in-process without an FFI/runtime bridge that does not exist as a maintained artifact. Irmin's *concepts* (branchable key-value, content-addressed, in-process) are exactly what a Rust write-behind would emulate, but the *implementation* is off the table for a Rust codebase. Treat Irmin as **conceptual prior art only** [model-knowledge for its internals; OCaml-language fact is web-verified].

### 1.4 Datomic — git-LIKE (immutable, time-travel), not git

Datomic is immutable-fact storage with bitemporal-ish time-travel and **sublinear-in-DB-size index construction**. [web — Datomic intro docs] **Concurrency:** a *single transactor* serializes all writes (ACID, CAP-consistent + ACID-consistent); peers scale reads horizontally and cache indexes (Memcached ~1ms/segment). [web — Datomic docs: transactor/peer model, monitoring] The single transactor *eliminates write conflicts* but *caps write throughput* — a hazard for very-frequent multi-tenant config churn. **Maturity:** high; years of production use, detailed ops/capacity guides. [web — Datomic docs] **Storage growth** is significant (immutable + append-only indexes); capacity planning is a documented concern. **Relevance to Prism:** Datomic is *not* in the taxonomy (Postgres/SQLite is settled), so it is **comparative prior art for the time-travel model**, not an adoption candidate. Its lesson — immutable + as-of + serialized-writes delivers most of git's history benefits *without a git repo* — directly informs the DB-native-temporal option (§3).

### 1.5 git-config-as-primary (config files live in a git repo as the live source)

This pattern is **common for authoring/governance, rare on the hot path.** IaC (Kubernetes manifests, Terraform in git), config-as-code, and detection-as-code all keep *definitions* in git — but **the runtime engine does not synchronously read git on every query.** Instead, config is periodically pulled from git and materialized into a DB or in-memory representation that serves runtime. [web — Panther/Elastic/Google SecOps detection-as-code; see §6] **The surveyed literature contains no example of a latency-sensitive system reading config directly from a git repo on the hot path of every request.** [web — absence confirmed] Repo bloat, clone/fetch times, merge conflicts, and unpredictable packfile-GC timing are exactly the failure modes that make git-on-the-hot-path a poor fit; when git is *off* the hot path these costs hit only CI/CD and developer workflows.

### 1.6 §1 takeaway

Git-for-data is real and (for Dolt) production-credible **as an OLTP/data-curation store**, and Dolt has now closed the sysbench gap to MySQL. But across all four systems plus the git-config pattern, **there is no cited prior art for git-as-primary serving hot-path, multi-tenant *runtime config*.** The git-primary case is materially stronger for **detection content/rules** (code-like, branch/merge-valuable) than for runtime config — which is exactly the domain split in §5.

---

## 2. The "DB-primary + git/version-log write-behind" pattern

### 2.1 Is it a named, recognized pattern?

**Not crisply named in the literature as a single pattern**, but it is a composition of three well-established patterns: **(a)** audit/history tables (DB-native version log), **(b)** change-data-capture / event-sourcing to an append log, and **(c)** config-as-code export. [web — research synthesis] Detection-as-code platforms ship a *close cousin*: Elastic's DaC reference explicitly recommends "version control of rule definitions and a log of deployments" — a git-backed audit trail behind a DB-authoritative engine. [web — dac-reference.readthedocs.io] So the pattern is real and precedented even if it lacks a single canonical name.

### 2.2 Two implementation strategies and their hazards

**(A) Synchronous dual-write (DB + git in one logical op).** The classic distributed-write hazard: **there is no transaction that spans Postgres and a git repo.** If you write DB-then-git and the git commit fails, history silently omits the change; if you write git-then-DB and the DB commit fails, git records a change that never went live. [web — research synthesis] Mitigations are compensations (retries, idempotency, operator alerts), not atomicity. **This is the dual-write consistency problem the human's question names directly, and it is unsolvable as true atomicity.**

**(B) Asynchronous CDC / write-behind (DB is primary, git is derived).** DB commit is authoritative; a separate process (trigger, logical-decoding/WAL tail, or poll) derives git commits afterward. Git can lag or retry on failure; worst case git history trails the DB but eventually catches up. This is the standard data-warehouse/audit CDC shape. **This is the correct shape for the write-behind** because it makes the consistency hazard *one-directional and recoverable* (git can be rebuilt from the DB's authoritative history) rather than *bidirectional and lossy*.

### 2.3 What guarantees the write-behind actually gives

- **Strongest achievable for git write-behind: at-least-once replication of DB changes into git**, with retries + drift detection (git history may have duplicates or slight reorder, never authoritative). This mirrors how Elastic/Google-SecOps DaC pipelines push validated rules on merge-to-main and rely on the pipeline + human approval to minimize missed deployments. [web — dac-reference; Google SecOps DaC]
- **Stronger guarantees come from the DB-native history table, not git**: because the history row is written *in the same transaction* as the config row, it can be **exactly-once** and is **authoritative** (see §3). [web — hyPiRion Postgres system-versioning; PG temporal_tables]

**Position:** the version-history *source of truth* should be the **in-transaction DB history (exactly-once, authoritative)**; the git write-behind is a **best-effort, non-authoritative, exportable projection** derived from it. This inverts the naive "git is the history" mental model into "the DB *is* the history; git is a *shareable rendering* of it." That inversion is what makes the dual-write hazard go away: git can always be regenerated from the authoritative DB history, so a failed git commit is a deferred re-render, not lost data.

---

## 3. DB-native versioning — the "git-semantics over the store" option (no actual git)

The human explicitly named "git-semantics over the store." This section characterizes it honestly: **DB-native temporal/audit can deliver history, point-in-time, rollback, and diff WITHOUT a git repo — but NOT branch/merge.**

### 3.1 The mechanisms (all production-real)

- **SQL:2011 system-versioned temporal tables.** `PERIOD FOR SYSTEM_TIME` + a mirrored **history table**; the engine auto-moves the prior row to history on every UPDATE/DELETE; `SELECT ... FOR SYSTEM_TIME AS OF <t>` reconstructs any past state. History table is non-updatable so history can't be forged. Supported natively in **SQL Server** (system-versioned temporal tables) [web — Microsoft Learn] and **MariaDB** [web — research synthesis]; **standardized SQL:2011** as an optional feature [web — wiki.postgresql.org FOSDEM 2015 temporal PDF].
- **PostgreSQL: no built-in system-versioning**, but two well-attested routes: the **`temporal_tables` extension** (trigger writes a history row per update; later versions add a flag to record only real changes; integer version column supported) [web — temporal_tables docs], and the **hand-rolled snapshot+history-table pattern** (hyPiRion) — two tables per versioned table, `tstzrange systime`, GiST exclusion constraint preventing overlapping intervals, insert/update/delete triggers, explicit rollback by copying a history row forward. [web — hypirion.com system-versioned-tables] Note the extension is "not supported on major cloud providers," which is *irrelevant* for Prism's bundled-Postgres appliance (we control the binary) — a point in favor of either route for an air-gapped appliance.
- **Bitemporal stores (XTDB / Datomic).** XTDB ("time-travel SQL database") tracks **system time AND valid time** [web — XTDB v2 launch]; ties to the prior `prismql-asof-version-resolution` research the dispatch referenced. Not in Prism's taxonomy, but the bitemporal model is the gold standard for "what did we believe at time T about state valid at time U" — relevant if detection-rule provenance ever needs valid-time semantics.

### 3.2 What DB-native temporal CAN do (matches git's *history* features)

- **History:** inherent — every change recorded with timestamp/version. [web — hyPiRion; PG temporal PDF]
- **Point-in-time / as-of:** `FOR SYSTEM_TIME AS OF` (or `systime @> :t` predicate). [web — SQL Server; hyPiRion]
- **Rollback:** copy/UPDATE the snapshot row from its historical counterpart; SQL Server documents `UPDATE ... SET col = (SELECT col FROM t AS OF SYSTEM TIME :x ...)`. [web — PG temporal PDF: "point in time recovery"]
- **Diff:** self-join the history table on key across two versions and compare columns — standard SQL. [web — research synthesis]
- **Audit / who-changed-what-when:** history rows + an actor/identity column = a forge-resistant change log (history table non-updatable). [web — SQL Server; PG temporal PDF: "history can only be forged by modifying the history table directly"]

### 3.3 What DB-native temporal CANNOT do (where real git wins)

- **Branch + merge.** This is the one capability temporal rows do not give. As one practitioner summarized: "Temporal versioning is less sophisticated than git-like versioning (no branching etc.) but is usually more aligned with common end-user requirements." [web — HN 37955617] Branches *can* be emulated with schemas/separate tables, but merge semantics get complex and unsafe for blob-shaped config. [web — research synthesis]

### 3.4 When is real-git's content-addressed diff/merge worth more than temporal rows?

Real git earns its keep when the artifacts are **(1) text-based and structured (code/YAML/JSON), (2) edited collaboratively by humans who need line-diffs/blame/merge tools, and (3) subject to branching workflows (feature/experimental/long-running forks).** [web — research synthesis] **Detection rules fit all three.** Runtime config fits *none* well — operators want a single canonical per-tenant value applied atomically, rarely merge config branches, and a large-JSON-blob merge is exactly where git's merge gets *unpredictable and unsafe*. [web — research synthesis]

### 3.5 §3 takeaway

**For runtime config, DB-native temporal (snapshot + history table, in-transaction, exactly-once, authoritative) delivers everything the human wants from "git-backed versioning" — diff, history, point-in-time, rollback, audit — without a git repo, without the dual-write hazard, and without git's hot-path costs.** The only thing it lacks (branch/merge) is the thing runtime config does not need. This is the cleanest realization of "git-semantics over the store."

---

## 4. Rust embedded-git reality (if real git is used as the write-behind/primary)

### 4.1 The two options and verified versions

- **`git2` (libgit2 Rust bindings) — current: 0.19.0** [web — crates.io/lib.rs, via perplexity_ask 2026-06-27]. Mature, feature-complete, widely used in production Rust. FFI to the C libgit2; binary must be present (manageable in an appliance build). libgit2 is "more complete because it's been around longer," but development "doesn't really seem to be progressing much," lacks shallow clones and protocol-v2, and has at least one known nasty error-handling bug and expensive-diff quirks. [web — r/rust gitoxide retrospective] **Recommended default for a write-behind today** given maturity. [web — Sebastian Thiel/GitMerge 2024: "if you are using rust you probably want to use git2 if it comes to ease of use of API"]
- **`gix` (gitoxide, pure Rust) — current: 0.85.0** [web — crates.io/lib.rs, via perplexity_ask 2026-06-27]. Pure-Rust (no FFI — attractive for air-gapped reproducible builds), high development velocity, designed for correctness + multi-threaded performance on large monorepos. **But explicitly NOT stable yet** [web — Sebastian Thiel/GitMerge 2024: "gitoxide is not stable yet"], and feature gaps remain (historically: clone/shallow-clone/push were incomplete; push specifically called out as a gap). [web — r/rust retrospectives] A 2023 third-party test found `gix clone` of a bare local repo ~60× slower than `git` because it re-packed all objects [web — twdev.blog 2023], though the maintainer reports `gix clone` of Linux in 1m20s vs git's 2m20s on his machine via superior parallel pack resolution [web — GitMerge 2024] — i.e. performance is workload-dependent and the project is mid-maturation.

### 4.2 Can they drive commit-on-every-config-change cleanly in-process?

**Yes, mechanically** — both can serialize config to a working tree, stage, and commit programmatically with metadata (tenant id, change id, actor) in the commit message; push asynchronously only when a remote is configured (air-gap-safe by default). [web — research synthesis] **But the hard parts are operational, not API:**

- **Repo bloat / loose objects:** commit-per-change produces many loose objects → must **repack** periodically; deep fragmented commit graphs slow history browsing. [web — research synthesis; consistent with git internals]
- **GC/pack cost:** repack/prune must be **scheduled into maintenance windows** in an air-gapped appliance so it does not contend with runtime; GC timing/impact is not always predictable. [web — research synthesis]
- **Concurrency:** embedded git expects **a single writer per repository**; concurrent commit attempts must be serialized (or run against per-tenant clones). [web — research synthesis] This is a real constraint for a multi-tenant write-behind: either one serialized writer goroutine/task, or per-tenant repos.
- **Mitigation:** **batch/group commits on a schedule rather than strictly per-change**, and treat the write-behind as a **background, best-effort process — never on the runtime critical path.** [web — research synthesis]

### 4.3 §4 takeaway

If real git is used for write-behind, **`git2` 0.19.0 is the production-grade choice today; `gix` 0.85.0 is the strategically-preferable pure-Rust future but is not yet stable and has feature gaps** — re-evaluate at morph time and do not pin from this report without a fresh crates.io check. Either way, the write-behind must be a **serialized, batched, background process with scheduled GC**, never synchronous on the hot path. **The single-writer + GC + bloat operational burden is itself an argument for preferring DB-native temporal for the *config* write-behind and reserving real-git for *detection content* (where the commit rate is human-paced, not machine-paced).**

---

## 5. The domain split — runtime config vs detection content

### 5.1 Runtime config: requirements point hard at DB-primary + DB-native-temporal

Hot-path read via atomic in-memory snapshot (ArcSwap, AD-007); multi-tenant partitioning; per-tenant hot-reload with atomic visibility; UI/API authoring demanding schema validation + transactional semantics. [decided context; CLAUDE.md AD-007] These map cleanly onto **ACID + row-level isolation + indexing of a relational DB** (Postgres central / SQLite satellite, already settled), and onto **DB-native temporal for versioning** (§3). Branching is *not* a requirement — a single canonical config per tenant is desired; staging-vs-prod is better modeled as separate tenants/environments than git branches. [web — research synthesis] **Git is not on the hot path for config in any surveyed system.** [web — absence confirmed]

### 5.2 Detection content / recipes: requirements point at git-primary-capable

Code-like; authored/reviewed collaboratively; benefits from precise diffs, blame, branching; unit-tested + validated before deploy; shareable across orgs. [web — Panther, Elastic DaC, Google SecOps DaC] Every surveyed detection-as-code platform treats **git as the authoritative source for rule *definitions* and the DB/engine as authoritative for the *runtime materialization*** — Panther rules-as-code uploaded via `panther_analysis_tool`/API [web — github.com/panther-labs/panther-analysis], Elastic VCS→Kibana via `kibana import-rules` CLI + CI/CD [web — elastic.co dac-beta], Google SecOps rules-dir→content-manager-CLI on merge-to-main [web — Google SecOps DaC], Splunk YAML-in-GitHub→REST-API saved-searches [web — r/cybersecurity DaC]. This is **exactly "git-primary-capable for detection content, DB-primary for runtime"** and it is the dominant precedented shape.

### 5.3 Is "DB-primary for runtime config, git-primary-capable for detection content" a sound, precedented split?

**Yes — strongly precedented.** It is what Panther, Elastic Security, and Google SecOps ship: rules authored/versioned as code (git-primary), materialized into a DB-backed engine for runtime (DB-authoritative at execution). [web — all three] It also matches the C9 depth pass's Q2 authority split (Git-authoritative for the global/high-blast-radius detection domain; DB-authoritative for per-tenant runtime), confirming this mechanism cut is consistent with the prior pass.

### 5.4 Reconciling the seam: a detection rule is BOTH git content AND a runtime config object

The seam is **the materialization point** where a rule moves from code (git) to runtime (DB). The precedented reconciliation:

1. **Git definitions are canonical for the rule-as-authored**; the DB row is the **compiled/active projection** read on the hot path (same ArcSwap discipline as config). [web — Panther/Elastic/SecOps all materialize into the engine's store]
2. **On deploy, record the git commit hash + branch in the DB row** alongside the materialized rule; an audit row links the runtime change to the git provenance. Rollback = either git-revert-and-redeploy *or* DB-temporal-revert to the prior materialized version. [web — research synthesis; consistent with Elastic "log of deployments"]
3. **A version-lock file** keeps git and engine in sync and decides directionality; Elastic's locked-versions file lets *either* VCS or the engine be authoritative for a given sync, or reconcile bidirectionally. [web — elastic.co dac-beta] Prism's residency/air-gap stance argues for **local-store-authoritative-at-runtime with git-as-the-reviewed-source** (don't let an external remote silently overwrite the running engine).
4. **The materialization is itself the validate-before-swap gate** (C9 depth pass Q3): a rule that fails validation at materialization keeps the last-good runtime projection — the git side can hold an invalid-but-committed draft without endangering runtime.

So the rule object is **content-in-git for authoring/review/sharing AND a DB-row-for-execution** — the two are bridged by a recorded commit-hash↔DB-version mapping plus an audit row, with the **DB projection authoritative for what actually runs.**

### 5.5 §5 takeaway

The split is sound and precedented: **runtime config → DB-primary (+ DB-native temporal versioning); detection content → git-primary-capable (materialized into the DB for execution).** They reconcile at the materialization seam via a commit-hash↔DB-version link, with the DB projection authoritative on the hot path.

---

## 6. The opt-in remote for detection content (residency-gated, air-gap-safe)

### 6.1 How detection-as-code platforms do opt-in remotes

- **Panther:** the public `panther-labs/panther-analysis` repo is added as a *remote* (`git remote add panther-upstream ...`; `git pull panther-upstream main`) and detections are uploaded to the deployment via API/CLI. For private orgs, the recommended pattern is a **private cloned/mirrored repo** (`git push --mirror` to seed, then a `sync-panther-analysis-from-upstream` GitHub Action on a weekly cron opening PRs for upstream changes). [web — github.com/panther-labs/panther-analysis; docs.panther.com private-cloned-repo] **Key property: the remote is opt-in and the local/private repo + the Panther instance remain authoritative; upstream is pulled as PRs you choose to merge.**
- **Elastic:** bidirectional VCS↔Kibana sync via `kibana import-rules`/`export-rules` + a **version-lock file**, with GitHub Actions for manual-dispatch pull, scheduled pull, and push-to-prod-on-merge. **Either Kibana or VCS can be authoritative, or reconcile via the lock file** — the platform explicitly supports "local-authoritative with optional remote." [web — elastic.co/security-labs/dac-beta; dac-reference PDF]
- **Google SecOps:** rules-dir validated/tested, merged to main, pushed via content-manager CLI. [web — Google SecOps DaC]
- **Splunk:** YAML-in-GitHub, CI/CD test instance, push to search-head via REST API. [web — r/cybersecurity DaC]

### 6.2 The model that satisfies Prism's constraints

**Local store authoritative at runtime + opt-in remote git for sharing, with air-gap as the no-remote default.** When no remote is configured, all git operations are local (Panther's private-clone pattern works fully offline; the appliance's local repo is the only git). When a remote *is* configured, the sync is **pull-as-PR (operator-gated merge)** in (so the remote can never silently overwrite the running engine) and **push-on-explicit-action** out — never an automatic bidirectional overwrite of runtime. This is precisely the Panther private-clone + Elastic-lock-file shape. [web — both]

### 6.3 Residency-gating the push

The hazard the human named: **don't push residency-bound content to an external remote.** No surveyed platform documents residency-aware push gating natively [INCONCLUSIVE — not found in sources], so this is **Prism-bespoke** but mechanically simple and uniform with the C9 depth pass's structural residency filter (reject-before-wire): **tag every detection-content artifact with a residency class; the push step filters out any artifact whose residency class forbids the target remote's jurisdiction**, exactly as the satellite config-bundle computation filters region-bound config (C9 depth Q5 / D-C5-3). A residency-bound rule is *never placed on the wire to an external remote*, not merely refused remotely. This reuses the existing residency-tag taxonomy rather than inventing a second one.

### 6.4 §6 takeaway

Opt-in remote git for detection content is **standard practice** (Panther private-clone + cron-PR sync; Elastic lock-file bidirectional) and is **fully compatible with air-gap** (no-remote = local-only git is the default and works). **Residency-gated push is Prism-bespoke but trivial** given a residency-class tag: filter-before-push, uniform with the data/config residency model.

---

## 7. VERDICT

### 7a. Can git be the PRIMARY store?

**For runtime config: no — not recommended, and unprecedented.** Git-for-data (Dolt) is now production-credible as an OLTP store (sysbench parity with MySQL by Dec 2025 [web-bench]), but **no surveyed system runs git-primary as the hot-path, multi-tenant *runtime config* store**, and git's no-lock-merge-at-commit, packfile-GC unpredictability, repo bloat from machine-paced commits, and single-writer constraint are all *misaligned* with hot-path config (which wants atomic single-canonical-value-per-tenant, not branch/merge). [web + model-knowledge]

**For detection content: yes — git-primary-capable is the dominant precedented shape.** Panther/Elastic/Google-SecOps/Splunk all treat git as authoritative for rule *definitions*, materialized into a DB engine for runtime. [web — all]

### 7b. Is DB-primary + git/version-log write-behind the right default?

**Yes — this is the mainstream-and-correct choice, and the human's decision is well-founded.** The crucial refinement from the evidence: **the authoritative version history should be DB-NATIVE (in-transaction history/temporal table — exactly-once, forge-resistant), and the git layer should be a BEST-EFFORT, NON-AUTHORITATIVE, EXPORTABLE PROJECTION derived from it (async CDC, not synchronous dual-write).** This dissolves the dual-write consistency hazard the human named: because git is regenerable from the authoritative DB history, a failed git commit is a deferred re-render, not lost data. **Synchronous dual-write is the wrong implementation** (unsolvable as true atomicity — no transaction spans Postgres and a git repo); **async write-behind/CDC with at-least-once + drift detection is the right one.**

### 7c. Does detection content warrant a different mechanism?

**Yes.** Runtime config and detection content have genuinely different lifecycles, and the precedent is unambiguous:

| Domain | Primary store | Versioning mechanism | Remote |
|---|---|---|---|
| **Runtime config** (hot-path, UI-mutated, multi-tenant) | **DB-authoritative** (Postgres/SQLite, ArcSwap snapshot) | **DB-NATIVE temporal/history table** (exactly-once, authoritative); optional best-effort git export | none (local only) |
| **Detection content / recipes** (code-like, reviewed, shareable) | **DB projection authoritative at runtime; git-primary for the authored definition** | **Real embedded git** (`git2` 0.19.0 today / `gix` 0.85.0 future), human-paced commits, content-addressed diff/merge worth its cost here | **opt-in, residency-gated** (Panther/Elastic pattern); air-gap default = local-only |

---

## Recommended write-behind/versioning implementation (per domain)

- **Runtime config → DB-native temporal, NOT real-git.** Use a snapshot+history-table pattern (hyPiRion model: `tstzrange systime`, GiST exclusion, insert/update/delete triggers) or the `temporal_tables` extension on the bundled Postgres (cloud-provider non-support is irrelevant — Prism owns the binary; SQLite satellites use the equivalent trigger-history pattern). Delivers diff/history/as-of/rollback/audit, exactly-once, in-transaction, no dual-write hazard, no git GC burden, no hot-path cost. An *optional* async git export of config snapshots can exist purely for human-readable sharing, explicitly non-authoritative.
- **Detection content → real embedded git as the authored-definition store, materialized into the DB for execution.** Background, serialized, batched commits with scheduled GC; `git2` 0.19.0 as the production-grade default today, `gix` 0.85.0 as the strategically-preferred pure-Rust target pending stability (re-verify versions at morph). Record commit-hash↔DB-version in the materialized row + audit row at the seam. Opt-in residency-gated remote (filter-before-push by residency class).
- **Hybrid is the answer, not a single mechanism.** "DB-native temporal for config, real-git for detection content" is the recommended split — *not* one git layer for everything (which would import git's machine-paced-commit GC/bloat/single-writer burden into the config plane where it buys nothing) and *not* DB-temporal for everything (which would deny detection content the branch/merge/diff/review/sharing that is its whole value).

---

## Consolidated Open Design Questions

| # | Open question | Where it lands | Notes |
|---|---|---|---|
| OQ-GW-1 | Confirm DB-native temporal (snapshot+history vs `temporal_tables` extension) choice for the config write-behind; pick one and define the SQLite-satellite equivalent | morph ADR + data-engineer | hyPiRion hand-rolled gives most control (GiST exclusion, no-overlap); extension is less code. Both web-attested. |
| OQ-GW-2 | Is an *optional* git export of config snapshots worth building at all, or is DB-temporal + a UI diff view sufficient? | morph PO+architect | Git export buys human-readable sharing only; config is UI-authored so the audience is thin. Lean: defer unless a sharing requirement appears. |
| OQ-GW-3 | `git2` vs `gix` for the detection-content write-behind — re-verify versions + `gix` stability/feature-completeness at morph | morph research + architect | git2 0.19.0 / gix 0.85.0 as of 2026-06-27; gix not yet stable, push historically a gap. Do NOT pin from this report. |
| OQ-GW-4 | Single serialized writer vs per-tenant repos for the detection-content git write-behind (multi-tenant single-writer constraint) | morph ADR | Embedded git expects one writer per repo. Per-tenant repos isolate but multiply GC/bloat surface. |
| OQ-GW-5 | GC/repack scheduling for the detection-content git repo in an air-gapped maintenance window | morph ops | Must not contend with runtime; timing unpredictable. |
| OQ-GW-6 | Materialization-seam contract: commit-hash↔DB-version mapping, rollback directionality (git-revert vs DB-temporal-revert), version-lock-file semantics | morph BC | Elastic lock-file is the prior art; Prism leans local-store-authoritative-at-runtime. |
| OQ-GW-7 | Residency-class tag taxonomy reuse for detection-content push gating (confirm uniform with config/data residency tags from C9 depth Q5 / D-C5-3) | morph BC | Filter-before-push; Prism-bespoke (no platform documents native residency push-gating). |
| OQ-GW-8 | Opt-in remote sync model: pull-as-PR-in / push-on-explicit-action-out vs bidirectional lock-file reconcile | morph ADR | Panther private-clone + Elastic lock-file are the two precedents; pick the air-gap-safe directionality. |

---

## Honest Costs & Caveats

- **The hybrid is two mechanisms, not one.** DB-native temporal for config + real-git for detection content means two versioning subsystems and a materialization seam between them. That is more surface than "git for everything" or "DB-temporal for everything" — but each single-mechanism alternative is *worse* (see "Hybrid is the answer" above). The complexity is essential, not accidental.
- **The async-CDC write-behind is best-effort, not authoritative — by design.** If the orchestrator (or a reviewer) expects git to be a guaranteed-complete mirror, that expectation is wrong: the DB history is authoritative and git is a regenerable projection. This must be stated explicitly in the morph ADR so no one builds a process that *trusts* git history for correctness.
- **`gix` is not stable yet (0.85.0).** The pure-Rust write-behind is strategically preferable for air-gapped reproducible builds, but feature gaps (push historically incomplete) and instability mean `git2` 0.19.0 (FFI to libgit2) is the safer production default *today*. libgit2's own development "doesn't seem to be progressing much" and it has known quirks — a real tension with no clean winner. Re-evaluate at morph. [web]
- **Embedded-git operational burden is real:** single-writer-per-repo, repo bloat from commits, scheduled GC in maintenance windows. This burden is *acceptable for detection content* (human-paced commits) but would be *unacceptable for config* (machine-paced UI mutations) — which is itself the strongest argument for the domain split.
- **No published benchmarks for TerminusDB or XTDB** in available sources [INCONCLUSIVE on their numbers]; Dolt's benchmarks are vendor-published (DoltHub) — credible (sysbench/TPC-C are standard suites) but vendor-sourced. The ~40% TPC-C throughput figure is the load-bearing concurrency caveat for any git-for-data-as-OLTP claim.
- **Residency-aware push gating is Prism-bespoke** — no surveyed detection-as-code platform documents it natively [INCONCLUSIVE]. The mechanism (filter-before-push by residency tag) is simple and uniform with the existing residency model, but it is net-new build, not lift-from-prior-art.
- **Irmin is OCaml** — not adoptable in a Rust codebase as a maintained artifact [web]; useful only as conceptual prior art for the embedded-branchable-store idea.
- **CrowdStrike-style blast-radius, validate-before-swap, and the per-key authority-domain classification are owned by the C9 depth pass** — this mechanism cut deliberately does not re-derive them.
- **Leans/verdict are discussion input only.** The mechanism split (7c), the write-behind implementation choice (git2 vs gix vs DB-temporal), and the remote-sync directionality are PO+architect+data-engineer adjudication at morph, not decided here.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 1 | `reasoning_effort=high`, `strip_thinking=true`. Six-area depth pass: git-for-data (Dolt/TerminusDB/Irmin/Datomic + git-config-as-primary); DB-primary+git-write-behind pattern + dual-write/CDC hazards; DB-native temporal (SQL:2011/temporal_tables/hyPiRion/XTDB); Rust embedded git (gix/git2); the runtime-config-vs-detection-content domain split + seam; opt-in remote (Panther/Elastic/SecOps/Splunk) + residency. Returned ~88k chars; ~75% read directly (first 66KB via Read; remainder covered by the two targeted searches below which independently sourced the opt-in-remote/residency and temporal sections). |
| Perplexity perplexity_search | 4 | (1) Dolt OLTP-as-primary perf/concurrency limits [9 sources incl. DoltHub 1.0 + 2025-12 + HN + heise]; (2) gitoxide vs git2 maturity [6 sources incl. GitMerge 2024 talk + twdev benchmark]; (3) Panther/Elastic detection-as-code remote sync + air-gap [6 sources incl. panther-analysis repo + private-cloned-repo docs + Elastic dac-beta]; (4) Postgres temporal/SQL:2011 vs git [5 sources incl. hyPiRion + PG FOSDEM PDF + SQL Server Learn]. |
| Perplexity perplexity_ask | 1 | Version-pin verification: gix 0.85.0 / git2 0.19.0 on crates.io; Irmin language fact (OCaml, no production Rust port). ≤2-sentence factual lookup. |
| Perplexity perplexity_reason | 0 | — |
| Context7 | 0 | — (no library-API question this pass; embedded-git versions verified via perplexity_ask against crates.io) |
| Tavily (all variants) | 0 | — |
| WebFetch | 0 | — |
| WebSearch | 0 | — |
| Training data | 3 areas (flagged) | (1) Dolt commit-graph bloat/GC failure modes for the config case — not in DoltHub published docs, flagged [model-knowledge]; (2) TerminusDB/XTDB concurrency/isolation internals — not in available sources, flagged [INCONCLUSIVE]; (3) Irmin internals (concurrency/perf) — OCaml-language fact web-verified, internals [model-knowledge]. |

**Total MCP tool calls:** 6 (1 perplexity_research-high [PRIMARY], 4 perplexity_search, 1 perplexity_ask).
**Training data reliance:** low — every load-bearing claim is [web]/[web-bench] cited; the [model-knowledge]/[INCONCLUSIVE] items are explicitly flagged and routed to morph re-verification (OQ-GW-3 version re-check; TerminusDB/XTDB benchmark gaps noted in Costs & Caveats).

**Resilience note.** The dispatch warned that high-effort `perplexity_research` may fail on overload and to retry-then-drop-to-medium. The high-effort call **succeeded on the first attempt** — it returned an oversized result (88k chars) read via the saved-file path (first 66KB via Read; the unreadable single-line tail's content — opt-in-remote, residency, conclusion — was independently re-sourced by perplexity_search calls 3 and 4 and the temporal/Dolt searches, so no section relies on unread material). No retry/downgrade needed; the file was never at risk of abandonment.

**Deviation note (primary-tool mandate).** The non-trivial multi-area topic was led by the mandated `perplexity_research` at `reasoning_effort=high`, supplemented by 4 targeted `perplexity_search` calls for product/version/benchmark facts (Dolt perf, gix/git2 maturity, detection-as-code remote sync, Postgres temporal) and 1 `perplexity_ask` for version pins + the Irmin language fact — matching the dispatch's explicit "perplexity_search/Context7 for product/version facts" instruction. Context7 was not used because no library-API/usage question arose; the only version facts needed (gix/git2) were resolved against crates.io via perplexity_ask, which is sufficient and faster for two pins.
