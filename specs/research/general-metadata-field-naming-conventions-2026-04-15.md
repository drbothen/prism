---
document_type: research
level: L2
type: general
topic: "Metadata field naming conventions for query languages and security analytics platforms"
version: "1.0"
status: complete
producer: research-agent
timestamp: 2026-04-15T12:00:00
phase: 1a
inputs:
  - product-brief.md
  - domain-spec/L2-INDEX.md
input-hash: "58c33ba"
traces_to: ""
---

# Metadata Field Naming Conventions for Query Languages and Security Analytics Platforms

## Executive Summary

This research surveys how 15+ platforms name system/metadata fields that are injected by the platform rather than originating from the data source. The findings inform PrismQL's naming convention for virtual fields (`sensor`, `client`, `source`) that identify query provenance.

**Key finding:** There is no universal standard. Three dominant patterns emerge:

1. **Single underscore prefix** (`_field`) -- Splunk, BigQuery, Databricks/Spark, ClickHouse
2. **Double underscore prefix** (`__field`) -- Prometheus/Loki (reserved), QRadar (custom fields)
3. **Dollar sign prefix** (`$field`) -- Trino/Presto
4. **Dot-namespaced fields** (`namespace.field`) -- ECS, OCSF, Google Chronicle UDM

For PrismQL, **the underscore prefix (`_sensor`, `_client`, `_source`) is the strongest candidate**, aligning with the dominant convention in analytics platforms SOC analysts already use. A dot-namespaced alternative (`prism.sensor`) is viable but introduces complexity for the AI agent interface.

---

## 1. Splunk: The Underscore Convention Origin

### Convention

Splunk uses **single underscore prefix** for all internal/system fields. This is the most widely recognized convention in security analytics.

**Internal fields (underscore-prefixed, system-managed):**
- `_time` -- event timestamp in Unix time (always displayed)
- `_raw` -- original raw event data (always displayed)
- `_indextime` -- when the event was indexed
- `_sourcetype` -- format of the data input
- `_serial` -- internal event serial number
- `_cd` -- index bucket address
- `_bkt` -- bucket identifier
- `_si` -- splunk server and index name
- `_subsecond` -- subsecond timestamp precision

**Default fields (no underscore, system-assigned but fully queryable):**
- `host` -- the host that generated the event
- `source` -- the file/stream/input the event came from
- `sourcetype` -- the data format (same as `_sourcetype` but user-facing)
- `index` -- which index stores the event
- `linecount` -- number of lines in the event

### Queryability

ALL underscore-prefixed fields are queryable in SPL search/WHERE. However, only `_time` and `_raw` are displayed by default in Splunk Web. Other internal fields require explicit selection.

### Key Insight for Prism

Splunk's `host`, `source`, and `sourcetype` are the closest analogs to Prism's `_sensor`, `_client`, and `_source`. Notably, these are **not** underscore-prefixed in Splunk -- they are "default fields" without underscore. The underscore prefix in Splunk signals "platform internals" (indexing mechanics, raw data), not "source identification metadata."

**Sources:** [Splunk Internal Fields](https://docs.splunk.com/Splexicon:Internalfield), [Splunk Default Fields](https://docs.splunk.com/Documentation/SplunkCloud/latest/Knowledge/Usedefaultfields), [Splunk Field Naming](https://splunk.illinois.edu/splunk-at-illinois/using-splunk/naming-conventions/)

---

## 2. Elasticsearch / Elastic Common Schema (ECS)

### System Metadata Fields (underscore prefix)

Elasticsearch uses underscore-prefixed fields for **document-level system metadata**:
- `_id` -- document unique identifier
- `_index` -- which index contains the document
- `_source` -- the original JSON document body
- `_routing` -- routing value
- `_field_names` -- fields with non-null values
- `_version` -- document version number

These are **NOT queryable as regular document fields**. They exist at the document metadata level, outside `_source`.

### ECS Naming Convention (dot-namespaced)

ECS uses **dot notation namespacing** for all fields, including metadata:

**Observer fields** (the collecting/monitoring entity):
- `observer.hostname` -- device name
- `observer.ip` -- IP address(es)
- `observer.type` -- category (firewall, IDS, proxy)
- `observer.vendor` -- manufacturer
- `observer.product` -- product name
- `observer.name` -- custom identifier
- `observer.version` -- software version

**Event metadata fields:**
- `event.module` -- module/plugin name (e.g., "crowdstrike")
- `event.dataset` -- dataset name (e.g., "crowdstrike.falcon")
- `event.provider` -- source of event

**Custom fields** are permitted but must not conflict with ECS field names. ECS recommends following the same dot-notation grouping pattern.

### Key Insight for Prism

ECS's `observer.*` namespace is the closest conceptual match to Prism's virtual fields. The `event.module` and `event.dataset` fields map to Prism's `_sensor` and `_source` concepts. ECS demonstrates that dot notation works well for namespacing but requires nested object support.

**Sources:** [ES Document Metadata Fields](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/document-metadata-fields), [ECS Observer Fields](https://www.elastic.co/guide/en/ecs/1.12/ecs-observer.html), [ECS Conventions](https://www.elastic.co/guide/en/ecs/current/ecs-conventions.html), [ES|QL Metadata Fields](https://www.elastic.co/guide/en/elasticsearch/reference/current/esql-metadata-fields.html)

---

## 3. Microsoft KQL / Sentinel

### Convention

Microsoft Sentinel uses **PascalCase unprefixed fields** for system metadata columns that are auto-added to every log table:

- `TenantId` -- workspace/tenant identifier
- `TimeGenerated` -- when the event was generated
- `SourceSystem` -- source of the data (e.g., "OpsManager", "Azure")
- `Type` -- the table name the record belongs to
- `_ResourceId` -- Azure resource identifier (underscore prefix, newer addition)

### Cross-Source Identification

KQL's `union` operator has a `withsource` parameter:
```
union withsource=SourceTable SecurityEvent, Syslog
```
This creates an ad-hoc column named by the user (e.g., `SourceTable`) containing the table name. The user chooses the column name at query time.

### Key Insight for Prism

Sentinel's approach is inconsistent -- `TenantId` and `SourceSystem` have no prefix, while `_ResourceId` does. The `withsource` pattern is interesting: let the user name the source column at query time. However, for Prism's always-present virtual fields, a fixed name is better than requiring user specification.

**Sources:** [Azure Sentinel Tables Explained](https://medium.com/wortell/azure-sentinel-tables-explained-d91d8cad6f), [KQL Common Tasks](https://learn.microsoft.com/en-us/kusto/query/tutorials/common-tasks-microsoft-sentinel)

---

## 4. OCSF (Open Cybersecurity Schema Framework)

### Metadata Object

OCSF defines a **required `metadata` object** on every base event with 28 fields, including:

- `metadata.product` -- the reporting product (Required)
- `metadata.tenant_uid` -- tenant identifier (Recommended)
- `metadata.source` -- logical data origin (e.g., "CloudTrail Events")
- `metadata.log_provider` -- logging service
- `metadata.log_source` -- originating system or component
- `metadata.log_name` -- consumer-facing log identifier
- `metadata.version` -- OCSF schema version
- `metadata.original_time` -- source-reported timestamp
- `metadata.uid` -- unique event identifier

### Observer Concept

OCSF does NOT have an explicit `observer` object at the base event level (unlike ECS). The `metadata.product` and `metadata.log_provider` fields serve a similar purpose, identifying the reporting entity and the collection infrastructure.

### Naming Convention

OCSF uses **snake_case with dot-notation for nested objects**:
- Enum fields end with `_id` (e.g., `severity_id`)
- Computed sibling fields use `_name` suffix (e.g., `severity`)
- Objects use dot notation (e.g., `metadata.product`, `device.hostname`)

### Key Insight for Prism

Since Prism normalizes to OCSF, the `metadata.*` namespace is already occupied by OCSF semantics. Prism's virtual fields CANNOT use `metadata.*` without conflicting with OCSF. The `metadata.tenant_uid` field is close to Prism's `_client`, but Prism's virtual fields describe the query engine's context, not the original event's metadata.

**Sources:** [OCSF Metadata Object](https://schema.ocsf.io/objects/metadata), [OCSF Schema Overview](https://ocsflab.com/reference/schema-overview), [OCSF Base Event](https://schema.ocsf.io/1.7.0/classes/base_event)

---

## 5. Google Chronicle / SecOps UDM

### Convention

Google Chronicle's Unified Data Model (UDM) uses **dot-namespaced nouns**:

**Top-level nouns:** `principal`, `src`, `target`, `observer`, `intermediary`, `about`, `metadata`

**Field access:** `metadata.event_type`, `principal.hostname`, `target.ip`, `observer.hostname`

**Naming rules:**
- Lowercase with underscores within field names
- Dot notation for hierarchy traversal
- Labels for custom metadata
- Rules engine prefix: `udm.` (e.g., `udm.metadata.event_type`)
- Parser prefix: `event.idm.read_only_udm.`

### Key Insight for Prism

Chronicle's `udm.` prefix for rules evaluation is analogous to what Prism might do with a `prism.` prefix. The observer noun captures collection infrastructure. However, the deep nesting (`event.idm.read_only_udm.metadata.event_type`) is a usability concern.

**Sources:** [UDM Overview](https://docs.cloud.google.com/chronicle/docs/event-processing/udm-overview), [UDM Field List](https://cloud.google.com/chronicle/docs/reference/udm-field-list), [UDM Usage Guide](https://docs.cloud.google.com/chronicle/docs/unified-data-model/udm-usage)

---

## 6. Grafana Loki / Prometheus

### Convention

- **Double underscore prefix** (`__name__`, `__address__`) is reserved for internal/system use
- Labels beginning and ending with `__` are hidden from UI (label browser, query builder, autocomplete)
- User-defined labels: `[a-zA-Z_:][a-zA-Z0-9_:]*`
- Single underscore as word separator within names is standard (snake_case)

### Key Insight for Prism

The double-underscore convention is the strongest "do not touch" signal in the Prometheus ecosystem. However, it creates ugly field names that are harder to type. For PrismQL where analysts interact through an AI agent, the aesthetic concern is lower but the convention would be unfamiliar to SOC analysts.

**Sources:** [Prometheus Data Model](https://prometheus.io/docs/concepts/data_model/), [Loki Labels](https://grafana.com/docs/loki/latest/get-started/labels/), [Prometheus Naming](https://prometheus.io/docs/practices/naming/)

---

## 7. Apache Arrow / DataFusion

### Convention (Emerging)

DataFusion is actively developing metadata column support (GitHub Issue #20135). The proposed approach follows **Databricks/Spark's convention**:

- `_metadata` as a struct column with sub-fields
- `_metadata.file_path`, `_metadata.file_name`, `_metadata.file_modification_time`
- The underscore prefix signals "not part of the data"

Databricks warns: if source data contains a column named `_metadata`, it must be renamed to `source_metadata` to avoid collision.

An alternative convention uses **`sys_` prefix**: `sys_source_metadata`, `sys_source_file_name`.

### Key Insight for Prism

Since Prism uses DataFusion, aligning with DataFusion's emerging metadata column convention is advantageous. The underscore prefix is the direction DataFusion is heading. However, DataFusion's convention is still evolving (not finalized as of April 2026).

**Sources:** [DataFusion Metadata Columns EPIC](https://github.com/apache/datafusion/issues/20135), [Databricks File Metadata Column](https://docs.databricks.com/aws/en/ingestion/file-metadata-column)

---

## 8. SQL Standards and Major Databases

### PostgreSQL

System columns use **short unprefixed names**: `ctid`, `xmin`, `xmax`, `tableoid`, `cmin`, `cmax`. These are assigned negative attribute numbers internally to distinguish from user columns. They are queryable in SELECT and WHERE.

### BigQuery

Pseudo-columns use **underscore prefix with SCREAMING_CASE**: `_TABLE_SUFFIX`, `_PARTITIONTIME`, `_PARTITIONDATE`. These are reserved names that cannot be used for user-defined columns.

### Trino / Presto

Hidden columns use **dollar sign prefix**: `$path`, `$file_size`, `$file_modified_time`, `$partition`, `$bucket`. These are queryable in SELECT and WHERE but hidden from `SHOW COLUMNS`.

### SQL Standard

The SQL standard does not define a naming convention for virtual/computed columns. Implementations vary widely.

### Key Insight for Prism

BigQuery's `_TABLE_SUFFIX` is the closest SQL analog to Prism's `_source`. The underscore prefix convention is well-established in the SQL analytics world. Trino's dollar-sign prefix works but would be unusual for SOC analysts.

**Sources:** [PostgreSQL System Columns](https://www.postgresql.org/docs/current/ddl-system-columns.html), [BigQuery Wildcard Tables](https://cloud.google.com/bigquery/docs/querying-wildcard-tables), [Trino Hidden Columns](https://github.com/prestosql/presto/issues/2757)

---

## 9. Other Security Analytics Platforms

### IBM QRadar

Custom field mappings automatically receive a **double underscore prefix** (`__`) as the system name. This distinguishes custom fields from built-in fields.

### LogRhythm

Metadata fields are organized into 8 categories (Applications, Classification, Identity, KBytes and Packets, Location, Log, Network, Host). The naming convention is proprietary and does not consistently use prefixes.

### CrowdStrike Falcon LogScale

Uses a pipe-based query language. Field naming follows the original data source conventions without a standardized prefix for system metadata.

**Sources:** [QRadar Log Sources](https://www.ibm.com/docs/SS42VS_7.4/com.ibm.qradar.doc/b_qradar_users_guide.pdf), [LogRhythm Metadata Fields](https://docs.logrhythm.com/lrsiem/docs/metadata-fields)

---

## 10. Comparative Analysis

### Convention Matrix

| Platform | Convention | Example | Queryable | Analyst Familiarity |
|----------|-----------|---------|-----------|-------------------|
| **Splunk** | `_prefix` (internals) / no prefix (source meta) | `_time`, `sourcetype` | Yes | Very High |
| **Elasticsearch** | `_prefix` (system) | `_id`, `_index` | No (system only) | High |
| **ECS** | `namespace.field` | `observer.type`, `event.module` | Yes | High |
| **KQL/Sentinel** | PascalCase, no prefix | `TenantId`, `SourceSystem` | Yes | High |
| **OCSF** | `object.field` | `metadata.tenant_uid` | Yes | Medium |
| **Chronicle UDM** | `noun.field` | `metadata.event_type` | Yes | Medium |
| **Prometheus/Loki** | `__prefix__` | `__name__` | Internal only | Low (for SOC) |
| **BigQuery** | `_PREFIX` | `_TABLE_SUFFIX` | Yes | Medium |
| **Trino/Presto** | `$prefix` | `$path` | Yes | Low |
| **Databricks/Spark** | `_metadata.field` | `_metadata.file_path` | Yes | Medium |
| **QRadar** | `__prefix` | `__custom_field` | Yes | Medium |
| **PostgreSQL** | unprefixed short names | `ctid`, `tableoid` | Yes | Medium |

### Patterns Summary

**Pattern A: Underscore prefix (`_field`)**
- Used by: Splunk, BigQuery, Databricks/Spark, DataFusion (emerging)
- Pros: Widely recognized, unambiguous, easy to type, SQL-safe
- Cons: Splunk reserves `_` for internals (not source metadata); could collide with OCSF fields if OCSF ever adds `_`-prefixed fields (unlikely)

**Pattern B: Dot-namespaced (`namespace.field`)**
- Used by: ECS, OCSF, Chronicle UDM
- Pros: Semantic grouping, self-documenting, extensible
- Cons: Requires nested object support or flattening convention; longer to type; must not collide with OCSF namespaces

**Pattern C: No prefix (plain names)**
- Used by: Splunk (source/host/sourcetype), Sentinel (TenantId/SourceSystem)
- Pros: Simplest, most natural
- Cons: Risk of collision with OCSF fields; ambiguous origin

**Pattern D: Dollar prefix (`$field`)**
- Used by: Trino/Presto
- Pros: Unambiguous
- Cons: Unfamiliar to SOC analysts; some parsers may not handle `$` in identifiers

---

## 11. Analysis for PrismQL

### Requirements Recap

1. **Intuitive for SOC analysts** (primary users interact through AI agent)
2. **Unambiguous** (clearly distinguishable from OCSF event fields)
3. **Queryable** (must work as column names in SQL-like syntax)
4. **Not confusing** when mixed with OCSF fields like `severity_id`, `device.hostname`

### Candidate Evaluation

#### Option 1: `_sensor`, `_client`, `_source`
- **Precedent:** Splunk `_time`/`_raw`, BigQuery `_TABLE_SUFFIX`, Databricks `_metadata`
- **Collision risk:** LOW -- OCSF does not use underscore-prefixed fields. The `metadata.source` field in OCSF is dot-namespaced and distinct from `_source`.
- **Analyst familiarity:** HIGH -- Splunk analysts (the majority of SOC analysts) are accustomed to `_` prefix meaning "system field."
- **AI agent friendliness:** HIGH -- short, predictable, easy to reference in natural language.
- **DataFusion compatibility:** HIGH -- valid SQL identifiers, aligns with DataFusion's emerging `_metadata` convention.
- **Risk:** `_source` may cause confusion with Elasticsearch's `_source` (different meaning). Consider `_source_table` for disambiguation.

#### Option 2: `prism.sensor`, `prism.client`, `prism.source`
- **Precedent:** ECS `observer.*`, OCSF `metadata.*`, Chronicle `udm.*`
- **Collision risk:** ZERO -- unique namespace.
- **Analyst familiarity:** MEDIUM -- familiar to ECS users, unfamiliar to Splunk users.
- **AI agent friendliness:** MEDIUM -- longer, but self-documenting.
- **DataFusion compatibility:** REQUIRES either nested struct columns or field name flattening (dot in column name requires quoting in SQL: `"prism.sensor"`).
- **Risk:** Dots in SQL column names require quoting, which is a friction point. OCSF already uses dots for nested objects; adding a `prism.*` namespace to results mixes two different nesting conventions.

#### Option 3: `__sensor`, `__client`, `__source`
- **Precedent:** Prometheus/Loki, QRadar
- **Collision risk:** ZERO.
- **Analyst familiarity:** LOW -- unfamiliar to SOC analysts.
- **AI agent friendliness:** LOW -- double underscore is visually noisy, easy to miscount.
- **DataFusion compatibility:** HIGH -- valid SQL identifiers.
- **Risk:** Python convention for "private/mangled" may confuse engineering staff.

#### Option 4: No prefix: `sensor`, `client`, `source_table`
- **Precedent:** Splunk `host`/`source`/`sourcetype`, Sentinel `TenantId`
- **Collision risk:** MEDIUM -- `source` could collide with OCSF or user data; `client` is generic.
- **Analyst familiarity:** HIGH -- most natural.
- **AI agent friendliness:** HIGH -- shortest, most natural language-like.
- **DataFusion compatibility:** HIGH.
- **Risk:** Ambiguity with OCSF fields. An analyst seeing `source` in results might not know if it is an OCSF field or a Prism virtual field.

### Recommendation

**Option 1 (`_sensor`, `_client`, `_source`) is the strongest choice**, with one refinement:

- Use `_source_table` instead of `_source` to avoid confusion with Elasticsearch's `_source` concept and to be more descriptive.

**Final recommended fields:**
- `_sensor` -- the sensor identifier (e.g., "crowdstrike", "claroty")
- `_client` -- the client/tenant identifier (e.g., "acme")
- `_source_table` -- the source table (e.g., "crowdstrike_detections")

**Rationale:**
1. SOC analysts already understand `_` prefix from Splunk (the dominant SIEM training ground).
2. OCSF uses `snake_case` with dot notation for objects -- `_sensor` is visually distinct from `severity_id` or `metadata.product`.
3. DataFusion is moving toward `_metadata` convention -- aligned direction.
4. Short names work better for AI agent interfaces (fewer tokens, less ambiguity).
5. No collision with any OCSF field (OCSF has no underscore-prefixed fields).
6. Valid SQL identifiers requiring no quoting in DataFusion.

**If you later need more virtual fields**, the pattern extends naturally: `_query_name`, `_pack_name`, `_analyst_id`, etc. This aligns with the context decorators defined in CAP-026.

---

## 12. OCSF Alignment Note

Prism's virtual fields intentionally sit **outside** the OCSF schema. They describe the query engine's context, not the security event itself. This is the correct design because:

1. OCSF's `metadata.product` describes the sensor that produced the event (e.g., CrowdStrike Falcon).
2. Prism's `_sensor` describes which sensor API Prism queried -- which may differ if an aggregation layer sits between Prism and the sensor.
3. OCSF's `metadata.tenant_uid` describes the tenant in the sensor's context; Prism's `_client` describes the MSSP's client concept, which may not map 1:1 to sensor tenants.

The underscore prefix convention cleanly separates these concerns: OCSF fields use `snake_case` or `object.field` notation; Prism virtual fields use `_snake_case`. There is no ambiguity.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| WebSearch | 12 | Splunk internal fields, ECS conventions, KQL metadata, OCSF schema, Chronicle UDM, BigQuery pseudo-columns, Trino hidden columns, Prometheus labels, DataFusion metadata columns, QRadar/LogRhythm fields, underscore prefix UX, dot-namespace conventions |
| WebFetch | 7 | Splunk default fields docs, OCSF schema browser, ECS conventions page, OCSF metadata object, Chronicle UDM overview, LogRhythm metadata fields, ECS observer fields, DataFusion GitHub issue |
| Context7 | 0 | Not applicable (research is about naming conventions, not library APIs) |
| Training data | 2 areas | General SQL standard knowledge (no standard for virtual column naming); PostgreSQL system column mechanics (verified against docs) |

**Total tool calls:** 19
**Training data reliance:** low -- all platform-specific findings verified via web sources; training data used only for general SQL standard knowledge and cross-referencing
