# Research: SIEM / Security Lake / Data Lake as Federated Source Types — Replace-by-Capability OR Federate-Into

- **Type:** general (technology + positioning)
- **Date:** 2026-06-24
- **Topic slug:** siem-securitylake-datalake-federation
- **Status:** complete
- **Author:** research-agent
- **Consumers:** product-owner (positioning, product brief), architect (lake/SIEM adapter source type, ADR), business-analyst (competitive landscape)

> **Citation discipline note.** Every product/feature claim is tagged to a source URL with a retrieval/publication date. Where a claim is grounded in model reasoning rather than a cited source, it is flagged **[ANALYSIS]**. Where a claim is vendor marketing that I could not independently verify against neutral sources, it is flagged **[VENDOR-MARKETING — UNVERIFIED]**. Where the public record is silent or ambiguous, it is flagged **[INCONCLUSIVE]**.

---

## TL;DR for decision-makers

1. **The dual stance is technically sound and the architectural generalization holds.** "SIEM / Security Lake / Data Lake are all federated source types in Prism's adapter model" is correct at the architecture level: each is, mechanically, a queryable endpoint that Prism can either (a) replace by serving the same capability from live sensor APIs, or (b) federate into. The market itself has converged on the "security is a data-platform problem" framing (Thread 2), which validates treating lakes/SIEMs as *sources* rather than *competitors-only*.

2. **Amazon Security Lake is the highest-value first lake connector and the OCSF fit is real, not marketing.** Security Lake stores OCSF-normalized Apache Parquet on S3, exposed (since Feb 2024) as Apache Iceberg tables in the AWS Glue Data Catalog, governed by Lake Formation. Because Prism already normalizes to OCSF, a Security Lake adapter needs **little-to-no semantic normalization** — the schema is already the target schema. This is a genuine, quantifiable integration advantage no non-OCSF-native engine has.

3. **Recommended connector approach: a "lake/SIEM federated-source" adapter type with two access modes** — (A) **query-access subscriber** (in-place SQL via Iceberg/Glue/Lake Formation, Prism's preferred "Trino for security" mode), and (B) **data-access subscriber** (S3 + SQS event feed for pull-into-cache). Mode A aligns with Prism's ephemeral-federation thesis; Mode B aligns with Prism's demand-driven caching.

4. **Positioning risk is real but manageable.** Pure-federation vendors (Query.AI) are seen as *complements, not replacements*; pure-replace vendors (Panther, Hunters, Databricks Lakewatch) are seen as lake-lock-in. A credible dual stance must be framed as **"capability-first, source-agnostic"** — NOT as "we can be your lake AND query your lake," which buyers read as hedging. The differentiator is the OCSF-native adapter model + demand-driven caching, not the dual stance itself.

5. **"Trino for security" is an OPEN category — no vendor owns that exact label** (verified 2026). This is a positioning *opportunity* (category creation), but also a caution: there is no established analyst segment to slot into, so the messaging must do the work of explaining the analogy.

---

## Thread 1 — Security Lake / Data Lake as Federated Sources (Technical Connectivity)

### 1.1 Amazon Security Lake — storage, schema, and the OCSF advantage

**Storage & schema (verified):**
- Security Lake ingests security logs from AWS-native sources (CloudTrail, VPC Flow Logs, Route 53, S3, Lambda, Security Hub) and converts them to **OCSF**, stored as **Apache Parquet** on customer-owned S3. ([AWS — What is Amazon Security Lake](https://docs.aws.amazon.com/security-lake/latest/userguide/what-is-security-lake.html), accessed 2026-06-24; [AWS — OCSF in Security Lake](https://docs.aws.amazon.com/security-lake/latest/userguide/open-cybersecurity-schema-framework.html), accessed 2026-06-24)
- As of **February 2024**, Security Lake added **Apache Iceberg table** support and **OCSF v1.1.0**, including OCSF Observables (threat-intel/identity matching) and the RFC-3339 datetime profile. ([AWS What's New, 2024-02](https://aws.amazon.com/about-aws/whats-new/2024/02/amazon-security-lake-analytics-ocsf-iceberg/))
- **OCSF version nuance [INCONCLUSIVE]:** AWS docs state OCSF **1.3 and earlier** for *custom* sources ([AWS — Custom sources](https://docs.aws.amazon.com/security-lake/latest/userguide/custom-sources.html), accessed 2026-06-24), while AWS-native "source version 2" maps to **OCSF 1.1.0** ([AWS — Query examples](https://docs.aws.amazon.com/security-lake/latest/userguide/subscriber-query-examples.html)). The exact version-equivalence is not cleanly documented. **Implication for Prism:** the lake adapter must read the OCSF version from table metadata and tolerate ≥1 OCSF minor version; do not hardcode a single version.
- **Iceberg scope [INCONCLUSIVE]:** AWS confirms Iceberg *is supported*, but does not state that *all* tables are Iceberg. The adapter should detect table format (Iceberg vs plain Hive/Parquet) from Glue metadata rather than assume.

**Partitioning (verified):** Custom-source path is `/ext/<source>/region=<region>/accountId=<acct>/eventDay=<YYYYMMDD>/`; records ordered by time within Parquet files; `eventDay` (UTC day) is the primary pruning key. AWS-native sources show the same `eventDay`/`time_dt` filter idiom in query examples. ([AWS — Custom sources](https://docs.aws.amazon.com/security-lake/latest/userguide/custom-sources.html); [AWS — Query examples](https://docs.aws.amazon.com/security-lake/latest/userguide/subscriber-query-examples.html)) **Implication:** Prism's lake adapter MUST push `eventDay`/`time_dt` predicates down or it will scan (and pay for) everything; retention is governed by S3 Lifecycle, so queries beyond the retention window silently return empty.

**The OCSF integration advantage — quantified [ANALYSIS, grounded in cited schema facts]:**
Because Prism's adapter boundary already emits OCSF (per project `project_core_architecture_insight.md` and CLAUDE.md OCSF-normalization convention), and Security Lake *already stores OCSF*, the normalization work for a Security Lake source is **near-zero**: field names, event classes, and the RFC-3339 datetime profile are the same schema on both sides. Contrast with a generic lake (raw vendor JSON in Parquet) where the adapter must map every field. Concretely, the Security Lake adapter's transform stage collapses from "parse + map every field to OCSF" to "pass-through OCSF records + carry the `unmapped` attribute bag." This is the single strongest technical argument for making Security Lake the *first* lake connector. ([AWS — OCSF](https://docs.aws.amazon.com/security-lake/latest/userguide/open-cybersecurity-schema-framework.html); the `unmapped`-attribute lossless-mapping convention is documented in the [AWS Security Lake Transformation Library](https://github.com/aws-samples/amazon-security-lake-transformation-library))

### 1.2 How a federated engine queries Security Lake — the two subscriber models

Security Lake exposes external consumers as **subscribers** in one of two access modes (both verified):

| Mode | Mechanism | Auth/access | Best fit for Prism |
|------|-----------|-------------|--------------------|
| **Query access** | In-place SQL over Lake Formation-governed Glue tables (Iceberg/Parquet on S3). Athena is the reference engine. | Lake Formation `SELECT` grants on specific DBs/tables + cross-account IAM role; API/CLI path requires the `AmazonSecurityLakeMetaStoreManager` IAM role with Glue + Lake Formation permissions. | **Primary** — matches ephemeral federation ("query in place"). |
| **Data access** | Direct read of Parquet objects in S3 + **SQS (or HTTPS) notifications** on new-object arrival. | Cross-account IAM role (account ID + external ID trust) granting S3 read + SQS read. | **Secondary** — matches demand-driven cache hydration / pull-forward. |

Sources: [AWS — Query examples](https://docs.aws.amazon.com/security-lake/latest/userguide/subscriber-query-examples.html); [AWS — Prereqs for query subscriber](https://docs.aws.amazon.com/security-lake/latest/userguide/prereqs-query-subscriber.html); [AWS — Managing data access](https://docs.aws.amazon.com/security-lake/latest/userguide/subscriber-data-access.html); [AWS — Creating a data-access subscriber](https://docs.aws.amazon.com/security-lake/latest/userguide/create-subscriber-data-access.html); [AWS — CreateSubscriber API](https://docs.aws.amazon.com/security-lake/latest/APIReference/API_CreateSubscriber.html).

**Real-world subscriber integration patterns (verified):**
- **Splunk Federated Analytics** uses a **data-access subscriber with SQS**, assuming a cross-account role ARN + subscription endpoint; same-region constraint with the Security Lake roll-up region. ([Splunk docs — Create the Security Lake subscriber for data ingestion](https://help.splunk.com/splunk-cloud-platform/search/federated-search/10.0.2503/ingest-and-search-amazon-security-lake-datasets/create-the-amazon-security-lake-subscriber-for-data-ingestion), accessed 2026-06-24)
- **Amazon OpenSearch** uses a **data-access subscriber with SQS** → ingestion pipeline polls SQS → fetches Parquet → indexes (~15-day retention recommended). ([AWS Security Blog, pub 2024-07-29, upd 2025-04-29](https://aws.amazon.com/blogs/security/how-to-deploy-an-amazon-opensearch-cluster-to-ingest-logs-from-amazon-security-lake/))

**Third-party in-place SQL engines (Trino/Snowflake) — [INCONCLUSIVE for named engines]:** AWS documents the *generic* query-access subscriber + Glue + Iceberg + Lake Formation stack, and AWS Glue supports **Iceberg REST Catalog federation** ([AWS Big Data Blog — catalog federation for Iceberg in Glue](https://aws.amazon.com/blogs/big-data/introducing-catalog-federation-for-apache-iceberg-tables-in-the-aws-glue-data-catalog/); [Apache Iceberg vendors page](https://iceberg.apache.org/vendors/)). It is therefore *architecturally feasible* for any Iceberg-REST-capable engine to query Security Lake in place, but AWS does **not** publish a named Trino/Snowflake-as-subscriber walkthrough. Prism should treat the Iceberg-REST + Glue + Lake Formation path as the supported integration surface and validate it against a live subscriber (do not assume; the project's dtu-validator pattern applies).

### 1.3 Other lakes / lakehouses — how they expose query access

- **Snowflake:** not a SIEM; positioned as the "security data lake" foundation that partner SIEMs (Panther, Securonix BYO-Snowflake) run on. Query access is standard Snowflake SQL / external tables / Snowpark. ([Securonix BYO-Snowflake press release](https://www.securonix.com/press_release/securonix-announces-bring-your-own-snowflake-program-to-power-security-data-lake-for-snowflake-customers/); Panther on Snowflake — see Thread 2)
- **Databricks (Delta Lakehouse):** query via Delta tables, external tables, Delta Sharing; Databricks now ships its own **Lakewatch agentic SIEM** on the lakehouse. ([Databricks — Lakewatch](https://www.databricks.com/blog/databricks-announces-lakewatch-new-agentic-siem); [Databricks — Hunters SOC Platform, 2023-03-29](https://www.databricks.com/blog/2023/03/29/security-operations-data-lakehouse-hunters-soc-platform-now-available.html))
- **Generic Iceberg/Parquet-on-object-store:** the lowest-common-denominator target; query via Iceberg REST Catalog or Glue/Hive metastore + S3/GCS/ABFS reads.
- **Panther:** data-lake-native detection on Snowflake (detection-as-code on the customer's lake). ([Panther webinar — Replacing Legacy SIEM with Panther and Snowflake](https://panther.com/webinar/replacing-legacy-siem-with-panther-and-snowflake))
- **Anvilogic [INCONCLUSIVE in sources]:** known as multi-data-platform detection / BYODL on Snowflake/Databricks; not directly covered in the retrieved sources — treat as model knowledge pending verification if it becomes a target.

### 1.4 Federation/virtualization engine patterns (the "Trino for security" analogy)

Trino/Presto/Starburst connect to Iceberg/Delta/Snowflake/S3 via **catalog connectors** with **predicate + projection push-down**. The well-known gotchas — all directly applicable to a Prism lake adapter **[ANALYSIS, established patterns]**:
- **Large-scan cost:** without partition pruning + predicate push-down, a federated query over a multi-PB lake scans (and bills) enormous data. Mandatory `eventDay`/`time_dt` push-down is not optional. (Grounded in the AWS retention/partition guidance above.)
- **Latency:** object-store + catalog round-trips add seconds; in-place federation is not low-latency-interactive at SIEM-hot-path speeds. (Corroborated by the federation-vs-centralization tradeoff discussion in Thread 2.)
- **Rate limits / API costs:** for SQL-API-fronted lakes (Snowflake), per-query compute billing applies; repeated federated probes multiply cost. ([federation cost profile — Venture in Security](https://ventureinsecurity.net/p/security-is-about-data-how-different))
- **Schema drift / catalog freshness:** Iceberg snapshots evolve; the adapter must read current snapshot metadata, not cache stale schema.

### 1.5 Recommended Prism "lake/SIEM federated-source" adapter type — technical notes

**[ANALYSIS — architecture recommendation; routes to architect for ADR]**

Model the lake/SIEM as a **new source-type in the existing adapter model** with these properties:

1. **Two access modes** mirroring Security Lake's subscriber duality:
   - **`query` mode** (default for ephemeral federation): connect via Iceberg REST Catalog → Glue/metastore for table discovery; assume a cross-account IAM role carrying Lake Formation `SELECT`; push down OCSF time predicates (`time_dt`/`eventDay`) + column projection; read Parquet from object store. Emits OCSF directly (pass-through).
   - **`cache-hydrate` mode** (for demand-driven caching): subscribe to S3+SQS new-object notifications; pull Parquet into Prism's ephemeral cache on demand or on-event.

2. **OCSF fast-path vs map-path:** for **OCSF-native lakes (Security Lake)** the transform stage is pass-through (fast-path). For **non-OCSF lakes** (raw vendor data in Parquet/Delta) the adapter reuses Prism's existing per-sensor OCSF mapping (map-path). The source spec declares `ocsf_native: true|false`.

3. **Auth model in the spec:** cross-account IAM role ARN + external ID + (for query mode) the Lake Formation grant precondition. Credentials remain AI-opaque per AD-017 (reference-based; never in AI context).

4. **Push-down contract:** the adapter MUST translate Prism query predicates on time/partition columns into catalog/Iceberg pruning predicates; a query without a time bound against a lake source should warn or be rejected (cost guardrail), mirroring AWS's "must include a time-based filter" guidance.

5. **Feasibility verdict:** **HIGH for Amazon Security Lake** (OCSF-native, well-documented subscriber + Iceberg/Glue/Lake Formation stack). **MEDIUM for generic Iceberg/Delta/Snowflake** (feasible via standard connectors; more mapping + per-vendor auth work). The named-engine in-place path (Trino/Snowflake as subscriber) is **architecturally feasible but undocumented by AWS** — validate against a live subscriber before claiming support.

---

## Thread 2 — "Replace vs Federate the Lake/SIEM" Positioning

### 2.1 The market debate (2024–2026) — who pushes which model

The market has split into four camps (all vendor stances cited):

**A. Replace-the-SIEM, data-lake-native (lake-lock-in risk):**
- **Panther** — "the first SIEM delivered as a service on top of Snowflake"; campaign literally titled "Replacing Legacy SIEM with Panther and Snowflake." ([Panther webinar](https://panther.com/webinar/replacing-legacy-siem-with-panther-and-snowflake); [thank-you page](https://panther.com/resources/webinar-thank-you/replacing-legacy-siem-with-panther-and-snowflake-wa/))
- **Hunters** — "modern SIEM alternative" on the customer's Databricks Lakehouse. ([Databricks blog, 2023-03-29](https://www.databricks.com/blog/2023/03/29/security-operations-data-lakehouse-hunters-soc-platform-now-available.html); [hunters.security](https://www.hunters.security))
- **Databricks Lakewatch** — "new agentic SIEM" unifying security/IT/business data on the lakehouse, "eliminating vendor lock-in" via open formats. ([Databricks — Lakewatch](https://www.databricks.com/blog/databricks-announces-lakewatch-new-agentic-siem))
- **Securonix** — BYO-Cloud (own AWS account) + BYO-Snowflake; SIEM analytics decoupled from storage. ([Securonix BYO-Cloud](https://www.securonix.com/blog/bring-your-own-cloud-keeping-your-data-where-you-want-it/); [BYO-Snowflake](https://www.securonix.com/press_release/securonix-announces-bring-your-own-snowflake-program-to-power-security-data-lake-for-snowflake-customers/))
- **Stellar Cyber** — BYODL: processes/normalizes/enriches data *before* it lands in the customer's lake. ([Stellar Cyber — BYODL](https://stellarcyber.ai/bring-your-own-data-lake-do-it-the-right-way/))
- **Laminar** — "Security Analytics Without SIEM Costs." ([laminar.stream/security](https://laminar.stream/use-cases/security))

**B. Cloud-native SIEM with lake backend / emerging lake tier:**
- **Google SecOps (Chronicle)** — managed security-data-lake backend, cloud-native SIEM. Google's own content explicitly engages the "SIEM (Decoupled or Not) and Security Data Lakes" debate. ([Google SecOps](https://cloud.google.com/security/products/security-operations); [YouTube — Google SecOps perspective](https://www.youtube.com/watch?v=zn7Ma9MDReA))
- **Microsoft Sentinel data lake** — **GA since September 2025** (verified): analytics tier + data-lake tier, separation of storage/compute, retention up to 12 years; lake-only ingestion GA and XDR Advanced Hunting lake-tier ingestion GA as of Feb 2026. ([Microsoft — What's new in Sentinel](https://learn.microsoft.com/en-us/azure/sentinel/whats-new); [Sentinel data lake overview](https://learn.microsoft.com/en-us/azure/sentinel/datalake/sentinel-lake-overview); confirmed via perplexity_ask 2026-06-24) **This is the most significant competitive development for the federate stance: a hyperscaler is now actively pushing centralize-everything-in-our-lake.**

**C. Federation / query-in-place overlays (complement, not replace):**
- **Query.AI** — pure federated search: "access and get answers from your security data — wherever it is stored... an API bridge to your data wherever it is." Explicitly NOT a SIEM replacement. ([query.ai/federated-search](https://www.query.ai/federated-search/))

**D. Pipelines / data fabric (selective centralization + routing):**
- **Cribl** — telemetry pipeline + Cribl Lake (cheap archive) + Cribl Search (query archive without rehydration). Dual stance: complements SIEMs, partially bypasses them for search. ([cribl.io](https://cribl.io); [Cribl blog — SIEM archive / Cribl Lake](https://cribl.io/blog/drowning-in-your-siems-archive-save-on-costs-and-get-quick-access-to-data-with-cribl-lake/))
- **Zscaler** — "Data Fabric for Security" (vendor-managed fabric over Zscaler telemetry, not BYODL). ([Zscaler whitepaper PDF](https://www.zscaler.com/resources/white-papers/the-role-of-data-fabric-in-security-solutions.pdf))
- Industry framing that "security is about data": ([Venture in Security](https://ventureinsecurity.net/p/security-is-about-data-how-different); [Oort — SIEM vs Security Data Lake](https://docs.oort.io/blogs/siem-vs.-security-data-lake-why-its-time-to-rethink-your-security-program); [CyberSaint — security data lake](https://www.cybersaint.io/blog/is-your-organization-prepared-for-a-security-data-lake); [Software Analyst — convergence of SIEMs and data lakes](https://softwareanalyst.substack.com/p/the-convergence-of-siems-and-data))

### 2.2 How buyers read "you don't need a lake/SIEM — but we'll query yours" — credibility & risk

**The honest finding [ANALYSIS, grounded in cited vendor stances]:**
- **Pure federation (Query.AI) is credible but bounded** — buyers treat it as a *complement* (get more from what you have), not a replacement. Constraint is structural: a federated tool that doesn't own the data plane can't optimize deep historical analytics or long retention, and is bottlenecked by source API rate limits, latency, and per-query cost. ([query.ai](https://www.query.ai/federated-search/); [Venture in Security](https://ventureinsecurity.net/p/security-is-about-data-how-different))
- **Dual stances are read as hedging when incentives are conflicted.** Splunk's "federate OR ingest" is viewed skeptically because its revenue still favors ingestion. Cribl's dual stance is read as *credible* precisely because it has no legacy SIEM revenue to protect. **The lesson for Prism: a dual stance is credible only when the vendor has no incentive to push you toward one mode.** Prism's incentive structure (per-analyst MCP, sensor-API-native) is closer to Cribl's neutral position than to Splunk's conflicted one — this is an asset.
- **Lake-committed buyers see pure federation as transitional.** Organizations already standardized on Snowflake/Databricks prefer SIEM-on-lake (Panther/Hunters/Lakewatch) over generic federation. ([Panther](https://panther.com/webinar/replacing-legacy-siem-with-panther-and-snowflake); [Databricks](https://www.databricks.com/blog/databricks-announces-lakewatch-new-agentic-siem))

### 2.3 Federation + demand-driven caching vs centralize-everything — honest tradeoffs for lakes

| Dimension | Centralize-in-lake (Panther/Hunters/Lakewatch/Sentinel-lake) | Prism federation + demand-driven cache |
|-----------|---------------------------------------------------------------|----------------------------------------|
| **Deep historical analytics** | **Advantage: lake.** Years of retention in cheap object storage; SIEM hot storage "nearly unaffordable after 30–60–90 days." ([Venture in Security](https://ventureinsecurity.net/p/security-is-about-data-how-different); [Oort](https://docs.oort.io/blogs/siem-vs.-security-data-lake-why-its-time-to-rethink-your-security-program)) | **Honest limitation:** federation can't query what a source no longer retains. Prism's cache is ephemeral by design — NOT a retention substitute. Federating *into* a lake recovers this when the customer has one. |
| **Large-scan cost** | Storage cheap; compute for PB-scale scans expensive if unoptimized. ([Databricks](https://www.databricks.com/blog/databricks-announces-lakewatch-new-agentic-siem)) | Pay in API calls / per-query lake compute / egress; repeated probes multiply cost. Mitigated by push-down + caching. ([Venture in Security](https://ventureinsecurity.net/p/security-is-about-data-how-different)) |
| **Latency** | Predictable once data is local/indexed. | Object-store + catalog round-trips add latency; not SIEM-hot-path interactive. |
| **Data gravity / no-duplication** | Requires moving/duplicating data into the lake. | **Advantage: Prism.** No bulk duplication; query where data lives. |
| **Freshness** | As fresh as last ingestion batch. | **Advantage: Prism.** Live sensor APIs are real-time; lake data is batch-delayed. |

**Restated honestly:** Prism's federation + ephemeral cache is *strong on freshness, data-gravity avoidance, and zero-duplication*, and *weak on deep historical/forensic analytics and low-latency repeated heavy analytics* — which is exactly where a lake shines. **This asymmetry is the foundation of the "federate-into, don't replace, the lake" half of the stance:** Prism does not try to out-retain the lake; it queries the lake for history and queries live sensors for freshness.

### 2.4 The "Trino for security" analogy — verified status

**"Trino for security" / "Presto-style federated query engine for security data" is NOT a recognized analyst market label, and no vendor explicitly markets itself with that exact positioning as of 2026** (verified via perplexity_ask 2026-06-24). Trino appears *inside* security platforms as infrastructure, not as an external go-to-market category. **Implication:** the analogy is useful internally and explanatorily, but it is **category-creation, not category-entry** — the messaging must explain the analogy rather than lean on buyer familiarity. Upside: no incumbent owns the label. Risk: no established demand signal to ride.

### 2.5 Recommended positioning statements (replace-or-federate-the-lake)

**[ANALYSIS — drafts for product-owner; cited where grounded]**

1. **Capability-first, source-agnostic (lead statement).**
   *"Prism delivers SIEM, security-lake, and data-lake capabilities directly from your live security-tool APIs — normalized to OCSF, cached on demand. If you already run a SIEM or lake, Prism queries it in place as just another source. You get the capability whether or not you own the storage."*
   — Grounded in: the market's "security is a data-platform problem" convergence ([Venture in Security](https://ventureinsecurity.net/p/security-is-about-data-how-different)) + Prism's OCSF-native federation thesis.

2. **OCSF-native federation (the technical moat).**
   *"Because Prism normalizes everything to OCSF at the adapter boundary, it speaks the same schema as Amazon Security Lake natively — query your lake and your live sensors in one OCSF query, no re-mapping."*
   — Grounded in: [AWS OCSF/Security Lake docs](https://docs.aws.amazon.com/security-lake/latest/userguide/open-cybersecurity-schema-framework.html).

3. **Don't out-retain the lake — complete it (the honest tradeoff, turned into a strength).**
   *"Prism doesn't replace your data lake's deep history — it federates into it for the long view and queries your live sensors for what's happening now. One query, real-time freshness plus historical depth."*
   — Grounded in: federation-vs-centralization tradeoff ([Oort](https://docs.oort.io/blogs/siem-vs.-security-data-lake-why-its-time-to-rethink-your-security-program); [Venture in Security](https://ventureinsecurity.net/p/security-is-about-data-how-different)).

4. **No data gravity, no duplication (vs centralize-everything).**
   *"Stop paying to copy every log into a central lake before you can use it. Prism queries data where it already lives — your sensors, your SIEM, your lake — and caches only what your queries actually touch."*
   — Grounded in: data-gravity/cost critique of centralization ([Venture in Security](https://ventureinsecurity.net/p/security-is-about-data-how-different)).

5. **The neutral-incentive dual stance (credibility statement).**
   *"Prism has no ingestion meter and no storage to sell — so 'replace or federate' is an honest choice we make for your architecture, not a funnel into our billing."*
   — Grounded in: the buyer-credibility finding that dual stances are believed only from neutral-incentive vendors (Cribl credible, Splunk skeptical). ([Cribl](https://cribl.io/blog/drowning-in-your-siems-archive-save-on-costs-and-get-quick-access-to-data-with-cribl-lake/))

### 2.6 Dual-stance positioning RISK assessment

**[ANALYSIS — flag for product-owner]**

| Risk | Severity | Mitigation |
|------|----------|------------|
| **"Replace AND federate" reads as hedging / lack of focus.** Buyers distrust vendors who claim both, especially with conflicted incentives. | **HIGH** | Frame as *capability-first, source-agnostic* (statement 1), NOT as "we can be your lake AND query your lake." Lean on neutral-incentive credibility (statement 5). |
| **Federation seen as transitional by lake-committed buyers.** Snowflake/Databricks shops prefer SIEM-on-lake. | **MEDIUM** | Position Prism as the *cross-lake/cross-sensor* layer (federate-into Snowflake AND Databricks AND live sensors in one OCSF query) — something single-lake SIEMs can't do. |
| **Microsoft Sentinel data lake GA (Sept 2025) raises the centralize bar.** A hyperscaler now pushes centralize-everything with 12-year retention. | **MEDIUM-HIGH** | Don't compete on retention/depth (you lose). Compete on freshness, no-duplication, source-agnosticism, and *federating into Sentinel's lake* as a source. |
| **"Trino for security" is an unowned category — no demand signal to ride.** | **MEDIUM** | Use the analogy to *explain*, not to *position*. Lead with capability outcomes, not the engine analogy. |
| **Pure-federation ceiling (rate limits, latency, can't query expired data).** Honest structural limit. | **MEDIUM** | Demand-driven caching + federate-into-lake-for-history directly address the two biggest federation weaknesses. Be explicit that Prism is federation *plus* ephemeral cache, not pure federation. |

**Overall dual-stance verdict:** **Sound and defensible IF framed as capability-first/source-agnostic and IF the neutral-incentive credibility is foregrounded.** The dual stance fails only if marketed as "we do both modes" (hedging) rather than "the capability is what matters; the source is your choice." Prism's actual architecture (OCSF-native adapters + ephemeral demand-driven cache + no ingestion meter) makes the stance *more* credible than for any incumbent, because Prism has no incentive conflict.

---

## Open questions / inconclusive items (flag for follow-up)

1. **OCSF version matrix** for Security Lake (1.1.0 vs 1.3 for native vs custom) — read from table metadata at runtime; do not hardcode. [INCONCLUSIVE]
2. **Iceberg coverage** — not all Security Lake tables are confirmed Iceberg; detect format from Glue. [INCONCLUSIVE]
3. **Named in-place engine support** (Trino/Snowflake as Security Lake query subscriber) — feasible but AWS-undocumented; validate against a live subscriber (dtu-validator pattern). [INCONCLUSIVE]
4. **Anvilogic** specifics — not in retrieved sources; verify if it becomes a competitive target. [INCONCLUSIVE]

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 2 | Thread 1: Amazon Security Lake storage/schema/query/auth + OCSF fit (high reasoning_effort). Thread 2: decoupled-SIEM / BYODL / security-data-lake-vs-SIEM / data-fabric vendor landscape + federation tradeoffs (high reasoning_effort). |
| Perplexity perplexity_ask | 1 | Two ≤2-sentence factual checks: "Trino for security" category status + Microsoft Sentinel data lake GA status/date. |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_reason | 0 | — |
| Context7 | 0 | — (no library-API question) |
| Tavily (all) | 0 | — |
| WebFetch | 0 | — |
| WebSearch | 0 | — |
| Training data | 2 areas | (a) Trino/Presto connector push-down patterns (established, corroborated by cited cost/latency tradeoffs); (b) Anvilogic/Exabeam/Splunk Federated Search specifics not fully in retrieved sources — flagged [INCONCLUSIVE] where relied upon. |

**Total MCP tool calls:** 3 (2× `perplexity_research` high-effort + 1× `perplexity_ask`)
**Training data reliance:** **low** — every product/feature claim is tagged to a cited URL with date; the two training-data areas (connector push-down patterns, a few non-covered vendors) are either corroborated by cited tradeoff sources or explicitly flagged [INCONCLUSIVE]/[ANALYSIS].

### Source list (all accessed/verified 2026-06-24)

**Thread 1 (Amazon Security Lake / lakes — technical):**
1. https://aws.amazon.com/blogs/security/how-amazon-security-lake-is-helping-customers-simplify-security-data-management-for-proactive-threat-analysis/
2. https://docs.aws.amazon.com/security-lake/latest/userguide/what-is-security-lake.html
3. https://docs.aws.amazon.com/security-lake/latest/userguide/custom-sources.html
4. https://docs.aws.amazon.com/security-lake/latest/userguide/subscriber-query-examples.html
5. https://docs.aws.amazon.com/security-lake/latest/userguide/subscriber-data-access.html
6. https://aws.amazon.com/blogs/security/how-to-deploy-an-amazon-opensearch-cluster-to-ingest-logs-from-amazon-security-lake/ (pub 2024-07-29, upd 2025-04-29)
7. https://iceberg.apache.org/vendors/
8. https://github.com/aws-samples/amazon-security-lake-transformation-library
9. https://aws.amazon.com/about-aws/whats-new/2024/02/amazon-security-lake-analytics-ocsf-iceberg/ (2024-02)
10. https://aws.amazon.com/blogs/big-data/introducing-catalog-federation-for-apache-iceberg-tables-in-the-aws-glue-data-catalog/
11. https://docs.aws.amazon.com/security-lake/latest/userguide/prereqs-query-subscriber.html
12. https://help.splunk.com/splunk-cloud-platform/search/federated-search/10.0.2503/ingest-and-search-amazon-security-lake-datasets/create-the-amazon-security-lake-subscriber-for-data-ingestion
13. https://www.splunk.com/en_us/blog/security/ocsf-goes-into-high-gear-with-amazon-security-lake-launch-and-new-ocsf-release-candidate.html
14. https://docs.aws.amazon.com/security-lake/latest/userguide/create-subscriber-data-access.html
15. https://docs.aws.amazon.com/security-lake/latest/APIReference/API_CreateSubscriber.html
16. https://docs.aws.amazon.com/security-lake/latest/userguide/open-cybersecurity-schema-framework.html
17. https://docs.aws.amazon.com/athena/latest/ug/querying-iceberg.html

**Thread 2 (positioning / vendor landscape):**
18. https://www.youtube.com/watch?v=zn7Ma9MDReA (Google SecOps: SIEM Decoupled or Not, and Security Data Lakes)
19. https://stellarcyber.ai/bring-your-own-data-lake-do-it-the-right-way/
20. https://softwareanalyst.substack.com/p/the-convergence-of-siems-and-data
21. https://www.zscaler.com/resources/white-papers/the-role-of-data-fabric-in-security-solutions.pdf
22. https://www.query.ai/federated-search/
23. https://www.cybersaint.io/blog/is-your-organization-prepared-for-a-security-data-lake
24. https://panther.com/webinar/replacing-legacy-siem-with-panther-and-snowflake
25. https://www.databricks.com/blog/databricks-announces-lakewatch-new-agentic-siem
26. https://www.securonix.com/blog/bring-your-own-cloud-keeping-your-data-where-you-want-it/
27. https://cloud.google.com/security/products/security-operations
28. https://cribl.io
29. https://www.hunters.security
30. https://ventureinsecurity.net/p/security-is-about-data-how-different
31. https://docs.oort.io/blogs/siem-vs.-security-data-lake-why-its-time-to-rethink-your-security-program
32. https://laminar.stream/use-cases/security
33. https://panther.com/resources/webinar-thank-you/replacing-legacy-siem-with-panther-and-snowflake-wa/
34. https://www.securonix.com/press_release/securonix-announces-bring-your-own-snowflake-program-to-power-security-data-lake-for-snowflake-customers/
35. https://cribl.io/blog/drowning-in-your-siems-archive-save-on-costs-and-get-quick-access-to-data-with-cribl-lake/
36. https://www.databricks.com/blog/2023/03/29/security-operations-data-lakehouse-hunters-soc-platform-now-available.html

**Verification (perplexity_ask 2026-06-24):**
37. https://learn.microsoft.com/en-us/azure/sentinel/whats-new (Sentinel data lake GA, Sept 2025)
38. https://learn.microsoft.com/en-us/azure/sentinel/datalake/sentinel-lake-overview
39. https://www.microsoft.com/en-us/security/pricing/microsoft-sentinel
