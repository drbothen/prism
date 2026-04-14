# Pass 3 Deep: Behavioral Contracts -- Round 1

**Project:** Axiathon
**Pass:** 3 (Behavioral Contracts)
**Round:** 1
**Date:** 2026-04-13

---

## Contract Numbering: BC-S.SS.NNN
- S = Subsystem (1=Core, 2=Query, 3=Detection, 4=Storage, 5=Event, 6=Case)
- SS = Sub-area
- NNN = Sequential

---

## 1. Core Types Contracts (BC-1.xx.xxx)

### BC-1.01.001: TenantId.new() validates UUID format (production)

**Preconditions:** Input string provided via `impl Into<String>`
**Postconditions:**
- Empty string -> `Err(AxiathonError::Validation("TenantId cannot be empty"))`
- >128 chars -> `Err(AxiathonError::Validation("TenantId exceeds 128 characters"))`
- Non-UUID format -> `Err(AxiathonError::Validation("TenantId must be a valid UUID, got: {id}"))`
- Valid UUID (any version, any case) -> `Ok(TenantId)`
- Whitespace-padded UUID -> `Err` (UUID parse fails on leading/trailing spaces)
- UUID with extra trailing chars -> `Err`
**Evidence:** `crates/axiathon-core/tests/core_types_integration.rs` lines 56-119 (11 tests)
**Confidence:** HIGH

### BC-1.01.002: TenantId.new() validates alphanumeric format (spike)

**Preconditions:** Input string provided
**Postconditions:**
- Empty string -> `Err(AxiathonError::Tenant("tenant_id cannot be empty"))`
- >128 chars -> `Err(AxiathonError::Tenant("tenant_id exceeds 128 characters"))`
- Contains spaces -> `Err`
- Contains dots -> `Err`
- Alphanumeric + hyphens + underscores -> `Ok(TenantId)`
**Evidence:** `spike/crates/axiathon-core/src/tenant.rs` tests lines 74-99 (3 tests)
**Confidence:** HIGH

### BC-1.01.003: TenantId serde roundtrip preserves identity

**Preconditions:** Valid TenantId
**Postconditions:** `serde_json::to_string() -> from_str()` produces equal TenantId
**Evidence:** `core_types_integration.rs` line 48, `property_types.rs` proptest with UUID regex pattern
**Confidence:** HIGH (property-tested)

### BC-1.02.001: EventId.new() produces unique UUIDv7

**Preconditions:** None
**Postconditions:** Two sequential calls return different values. Display format is 36-char UUID.
**Evidence:** `core_types_integration.rs` lines 124-137, `property_types.rs` proptest (10000 iterations)
**Confidence:** HIGH (property-tested)

### BC-1.02.002: AlertId.new() produces unique UUIDv7

**Preconditions:** None
**Postconditions:** Two sequential calls return different values. Display format is 36-char UUID.
**Evidence:** `core_types_integration.rs` lines 169-180, `property_types.rs` proptest (10000 iterations)
**Confidence:** HIGH (property-tested)

### BC-1.03.001: TenantContext enforces private fields via getters

**Preconditions:** Constructed via `TenantContext::new()`
**Postconditions:** Fields accessible only via `tenant_id()`, `user_id()`, `roles()`, `permissions()`, `trace_id()`. No set methods. All fields preserved exactly as constructed.
**Evidence:** `core_types_integration.rs` lines 234-265
**Confidence:** HIGH

### BC-1.03.002: TenantContext is Send + Sync

**Preconditions:** None (compile-time assertion)
**Postconditions:** `TenantContext` satisfies `Send + Sync` bounds for async contexts
**Evidence:** `core_types_integration.rs` line 269 (`fn assert_send_sync<T: Send + Sync>() {}`)
**Confidence:** HIGH (compile-time)

### BC-1.03.003: TenantScoped trait accepts both context types

**Preconditions:** Function accepts `&impl TenantScoped` or `&dyn TenantScoped`
**Postconditions:** Both TenantContext and SystemContext are accepted. Returns correct tenant_id and trace_id for each.
**Evidence:** `core_types_integration.rs` lines 305-349 (3 tests)
**Confidence:** HIGH

### BC-1.04.001: AxiathonError implements std::error::Error + Send + Sync + 'static

**Preconditions:** None (compile-time assertion)
**Postconditions:** Error is propagatable across thread boundaries and await points
**Evidence:** `core_types_integration.rs` line 426
**Confidence:** HIGH (compile-time)

### BC-1.04.002: AxiathonError::Io transparently wraps std::io::Error

**Preconditions:** `io::Error` converted via `From`
**Postconditions:** `AxiathonError::Io` variant preserves original error message in Display
**Evidence:** `core_types_integration.rs` lines 362-375
**Confidence:** HIGH

### BC-1.04.003: AxiathonError::Json transparently wraps serde_json::Error

**Preconditions:** `serde_json::Error` converted via `From`
**Postconditions:** Display contains line/column info from original JSON error
**Evidence:** `core_types_integration.rs` lines 415-475
**Confidence:** HIGH

### BC-1.05.001: ApiResponse success/error constructors are mutually exclusive

**Preconditions:** None
**Postconditions:**
- `success(data)` -> data=Some, error=None, meta=None
- `error(code, msg, trace_id)` -> data=None, error=Some, meta=None
- `error_with_details(...)` -> data=None, error=Some with details=Some
- JSON serialization omits None fields entirely (not null)
**Evidence:** `core_types_integration.rs` lines 518-738 (15 tests)
**Confidence:** HIGH

### BC-1.05.002: ApiResponse serde roundtrip preserves all fields

**Preconditions:** Valid ApiResponse
**Postconditions:** `to_string -> from_str` preserves data, error code, trace_id, field error details
**Evidence:** `core_types_integration.rs` lines 618-638, `property_types.rs` proptest
**Confidence:** HIGH (property-tested)

---

## 2. FieldRef Contracts (BC-1.06.xxx)

### BC-1.06.001: FieldRef.new() validates and parses field paths

**Preconditions:** Path string provided
**Postconditions:**
- Empty string -> `Err("FieldRef cannot be empty")`
- Path with empty segment (e.g., "a..b") -> `Err("FieldRef contains empty segment")`
- Leading dot (e.g., ".a.b") -> Err (empty first segment)
- Trailing dot (e.g., "a.b.") -> Err (empty last segment)
- Simple path "severity_id" -> Ok, 1 Named segment
- Dotted path "src_endpoint.ip" -> Ok, 2 Named segments
- Array notation "answers[0].value" -> Ok, Index segment + Named segment, has_array=true
- Bracket-quoted "unmapped['vendor.field']" -> Ok, dots inside brackets not treated as separators
**Evidence:** `crates/axiathon-core/tests/property_fieldref.rs` (6 property tests, 4 negative property tests)
**Confidence:** HIGH (property-tested with compositional strategies)

### BC-1.06.002: FieldRef Display roundtrip is idempotent

**Preconditions:** Valid FieldRef
**Postconditions:** `FieldRef::new(path).to_string() == path` for all valid paths. Re-parsing the Display output produces structurally equal FieldRef.
**Evidence:** `property_fieldref.rs` proptest `fieldref_display_roundtrip` and `fieldref_reparse_idempotent`
**Confidence:** HIGH (property-tested)

### BC-1.06.003: FieldRef.has_array_index() iff path contains bracket notation

**Preconditions:** Valid FieldRef
**Postconditions:** `has_array_index()` returns true if and only if the original path contains `[`
**Evidence:** `property_fieldref.rs` proptest `fieldref_array_flag_consistent`
**Confidence:** HIGH (property-tested)

### BC-1.06.004: FieldRef segment count equals dot-separated parts outside brackets

**Preconditions:** Valid FieldRef
**Postconditions:** `segments().len()` equals the count of dots outside bracket notation + 1
**Evidence:** `property_fieldref.rs` proptest `fieldref_segment_count`
**Confidence:** HIGH (property-tested)

---

## 3. AxiQL Parser Contracts (BC-2.xx.xxx)

### BC-2.01.001: parse_axiql() returns partial results on error

**Preconditions:** Input string (any)
**Postconditions:**
- Empty input -> error with "unexpected end of input"
- >64KB -> error "query limit exceeded" (CWE-400)
- >128 nesting depth -> error "depth exceeded" (CWE-674)
- >64 pipe stages -> error "stage limit exceeded" (CWE-400)
- Successful parse -> `(Some(AxiQLStatement), vec![])`
- Parse failure -> `(None, vec![AxiQLError])`
**Evidence:** `crates/axiathon-query/tests/parser_test.rs`, parser.rs constants lines 36-42
**Confidence:** HIGH

### BC-2.01.002: AxiQL keywords are case-insensitive

**Preconditions:** Query containing SQL-family keywords
**Postconditions:** Keywords match regardless of case. `kw()` function uses `text::ident().try_map()` with `eq_ignore_ascii_case()`. Applies to: SELECT, WHERE, AND, OR, NOT, FROM, GROUP, BY, ORDER, ASC, DESC, LIMIT, HEAD, TAIL, STATS, SORT, DEDUP, FIELDS, HAS, MISSING, IN, CONTAINS, MATCHES, CIDR, STARTSWITH, ENDSWITH, etc.
**Evidence:** `crates/axiathon-query/src/parser.rs` lines 56-66 (kw function), grammar spec citation line 67
**Confidence:** HIGH

### BC-2.01.003: AxiQL comment stripping preserves byte offsets

**Preconditions:** AxiQL input with `//` or `#` comments
**Postconditions:** Comment characters replaced with spaces (not removed) to preserve byte offsets for error span reporting. String literal context tracked to avoid false positives. Escape sequences (`\"`) respected.
**Evidence:** `crates/axiathon-query/src/parser.rs` `strip_comments()` function, `crates/axiathon-query/tests/property_comments.rs`
**Confidence:** HIGH

### BC-2.01.004: AxiQL regex patterns validated at parse time

**Preconditions:** Query contains `field =~ "pattern"` or `field MATCHES "pattern"`
**Postconditions:**
- Invalid regex syntax -> parse error immediately
- Pattern >1024 bytes -> rejected (CWE-1333)
- Uses Rust `regex` crate (finite automaton, immune to catastrophic backtracking)
**Evidence:** parser.rs validation logic
**Confidence:** HIGH

### BC-2.01.005: AxiQL CIDR validated at parse time

**Preconditions:** Query contains `field IN CIDR "x.x.x.x/n"`
**Postconditions:** Invalid IP address or out-of-range prefix (>32 IPv4, >128 IPv6) -> parse error
**Evidence:** `validate_cidr()` in parser.rs
**Confidence:** HIGH

### BC-2.01.006: AxiQL supports three query modes with mode detection

**Preconditions:** Valid AxiQL query string
**Postconditions:**
- Filter mode (starts with field predicate): `AxiQLStatement::Filter(FilterExpr)`
- SQL mode (starts with SELECT): `AxiQLStatement::Select { projection, from, filter, group_by, order_by, limit }`
- Pipe mode (filter followed by `|`): `AxiQLStatement::Pipe { filter, stages }`
**Evidence:** parser.rs mode detection logic, parser_test.rs
**Confidence:** HIGH

### BC-2.01.007: AxiQL nesting depth tracked via Rc<Cell<usize>>

**Preconditions:** Recursive expression parsing
**Postconditions:** Nesting beyond MAX_NESTING_DEPTH (128) produces error. Uses `Rc<Cell<usize>>` not `Arc<AtomicUsize>` -- intentional per code comment: parser is synchronous, never held across await points, Rc<Cell> is faster due to compiler coalescing.
**Evidence:** parser.rs lines 16-21 documentation comment, MAX_NESTING_DEPTH constant
**Confidence:** HIGH

---

## 4. Field Alias Resolution Contracts (BC-2.02.xxx)

### BC-2.02.001: Three-tier alias resolution with provenance

**Preconditions:** FieldRef provided to `FieldAliasRegistry.resolve()`
**Postconditions:**
1. Check alias registry (analyst shortcuts + AxiQL canonical) -> `AliasResolved { original, resolved }`
2. Check OCSF direct fields -> `OcsfDirect(field)`
3. Pass through as `Unknown(field)` (supports vendor extensions)
**Evidence:** `crates/axiathon-query/tests/aliases_test.rs` (16 tests)
**Confidence:** HIGH

### BC-2.02.002: Default alias table resolves 7 standard aliases

**Preconditions:** Default `FieldAliasRegistry`
**Postconditions:**
- `src_ip` -> `src_endpoint.ip` (OCSF)
- `src.ip` -> `src_endpoint.ip` (AxiQL canonical also resolves)
- `dst_ip` -> `dst_endpoint.ip`
- `src_port` -> `src_endpoint.port`
- `dst_port` -> `dst_endpoint.port`
- `user` -> `actor.user.name`
- `user.name` -> `actor.user.name` (AxiQL canonical also resolves)
- `hostname` -> `device.hostname`
- `action` -> `activity_name`
**Evidence:** `aliases_test.rs` lines 30-65, 143-165
**Confidence:** HIGH

### BC-2.02.003: OCSF direct fields pass through without aliasing

**Preconditions:** Field registered as OCSF canonical path
**Postconditions:** `resolve()` returns `OcsfDirect(field)` -- field is unchanged
**Evidence:** `aliases_test.rs` lines 82-92
**Confidence:** HIGH

### BC-2.02.004: Unknown fields pass through as Unknown variant

**Preconditions:** Field not in alias registry or OCSF fields
**Postconditions:** `resolve()` returns `Unknown(field)` -- supports vendor extensions
**Evidence:** `aliases_test.rs` lines 96-118
**Confidence:** HIGH

### BC-2.02.005: Custom alias registration overwrites on duplicate

**Preconditions:** Alias registered, then re-registered with same key
**Postconditions:** Latest registration wins
**Evidence:** `aliases_test.rs` lines 198-220
**Confidence:** HIGH

### BC-2.02.006: OcsfVersionAliasMap resolves per-version aliases

**Preconditions:** Aliases registered for specific OCSF versions
**Postconditions:**
- Matching version -> `Some(&FieldRef)`
- Non-matching version -> `None`
- Unknown alias -> `None`
- Same alias, different versions -> different targets
**Evidence:** `aliases_test.rs` lines 244-295 (4 tests)
**Confidence:** HIGH

---

## 5. OCSF Event Contracts (BC-5.xx.xxx)

### BC-5.01.001: AxiathonEvent.get_field() four-tier resolution

**Preconditions:** AxiathonEvent with proto message + optional unmapped JSON
**Postconditions:**
1. `"tenant_id"` -> `Some(FieldValue::String(tenant_id))`
2. `"event_uid"` -> `Some(FieldValue::String(event_uid))`
3. Flat proto field (e.g., "severity_id") -> `Some(FieldValue::Int(...))`
4. Dotted proto path (e.g., "src_endpoint.ip") -> `Some(FieldValue::String(...))`
5. Unmapped JSON key (e.g., "claroty.alert_type") -> `Some(FieldValue::String(...))`
6. Nonexistent field -> `None`
7. Empty proto3 default string ("") -> `None` (treated as absent)
**Evidence:** `spike/crates/axiathon-core/src/event.rs` tests lines 288-422 (8 tests)
**Confidence:** HIGH

### BC-5.01.002: AxiathonEvent.class_uid() extracts OCSF class

**Preconditions:** AxiathonEvent with class_uid set in proto
**Postconditions:** Returns i32 class_uid from EnumNumber or I32 field. Defaults to 0 if not set.
**Evidence:** `event.rs` test `class_uid_and_event_time` line 388
**Confidence:** HIGH

### BC-5.01.003: resolve_unmapped_field() searches JSON blob

**Preconditions:** Event with non-empty `unmapped` string field
**Postconditions:**
1. Parse `unmapped` as JSON
2. Try direct key lookup: `json.get(path)` (matches "claroty.alert_type" as a key)
3. Try dotted path navigation: split on dots, traverse JSON objects
4. Empty JSON strings skipped
**Evidence:** `event.rs` function implementation + detection engine tests
**Confidence:** HIGH

### BC-5.01.004: descriptor_for_class_uid() maps class UIDs to proto descriptors

**Preconditions:** i32 class_uid
**Postconditions:**
- 3002 -> `ocsf.v1_7_0.events.iam.Authentication`
- 2001 -> `ocsf.v1_7_0.events.findings.SecurityFinding`
- 4001 -> `ocsf.v1_7_0.events.network.NetworkActivity`
- Other -> `None`
**Evidence:** `event.rs` lines 276-286, used extensively in tests across all spike crates
**Confidence:** HIGH

### BC-5.01.005: supported_class_uids() returns static mapping

**Preconditions:** None
**Postconditions:** Returns `&[(3002, "authentication"), (2001, "security_finding"), (4001, "network_activity")]`
**Evidence:** `event.rs` lines 257-263
**Confidence:** HIGH

---

## 6. Arrow Schema Contracts (BC-5.02.xxx)

### BC-5.02.001: arrow_schema_for_class() generates two-tier schema

**Preconditions:** Valid class_uid
**Postconditions:**
- 3 Axiathon columns: tenant_id (Utf8, non-null), event_uid (Utf8, non-null), event_time (Timestamp microsecond UTC, non-null)
- Tier 1 hot columns from proto descriptor (top-level scalars + one level of nested for HOT_NESTED_OBJECTS)
- 1 event_data column (Utf8, nullable) for tier 2
- Total columns >20 and <200 per class (Iceberg comfort zone)
- Different classes produce different schemas (different field counts)
**Evidence:** `spike/crates/axiathon-core/src/schema.rs` tests lines 551-697 (8 tests)
**Confidence:** HIGH

### BC-5.02.002: HOT_NESTED_OBJECTS controls tier 1 promotion

**Preconditions:** Proto descriptor with nested message fields
**Postconditions:** Only `src_endpoint`, `dst_endpoint`, `user`, `service`, `finding` are flattened to tier 1. Others (actor, metadata, device, session) remain in event_data JSON only.
**Evidence:** `schema.rs` lines 128-139 constant definition + field_catalog tests
**Confidence:** HIGH

### BC-5.02.003: events_to_record_batch() converts events to columnar format

**Preconditions:** Non-empty slice of AxiathonEvent (same class_uid)
**Postconditions:**
- Schema derived from first event's class_uid
- tenant_id column populated from event.tenant_id
- Hot columns extracted via get_field()
- event_data column contains full DynamicMessage as JSON
- event_time converted from milliseconds to microseconds (ms * 1000)
- Empty batch returns 0 rows with default schema
**Evidence:** `schema.rs` tests lines 619-657
**Confidence:** HIGH

### BC-5.02.004: events_to_record_batch_with_promotions() adds promoted columns

**Preconditions:** Events + promotions list (column_name, field_path)
**Postconditions:** Base batch + additional Utf8 columns for each promotion. Values extracted via get_field() and converted to string via to_string_repr().
**Evidence:** `schema.rs` function implementation, `spike/crates/axiathon-storage/tests/field_promotion.rs`
**Confidence:** HIGH

### BC-5.02.005: field_catalog_for_class() enumerates ALL fields with hot/cold annotation

**Preconditions:** Valid class_uid
**Postconditions:**
- >100 entries for authentication class
- Mix of hot (>20) and cold (>50) fields
- `tenant_id` is hot
- `src_endpoint.ip` is hot
- `user.name` is hot
- Deep nested fields are NOT hot
- Cycle detection via visited set prevents infinite recursion on circular OCSF references
**Evidence:** `schema.rs` tests lines 660-684
**Confidence:** HIGH

---

## 7. Detection Engine Contracts (BC-3.xx.xxx)

### BC-3.01.001: RuleEngine evaluates single-event rules, skips disabled

**Preconditions:** Rules loaded, event provided
**Postconditions:**
- Only enabled rules with SingleEvent match clause are loaded
- Each event evaluated against all loaded rules
- Returns `Vec<RuleMatch>` with matched rules + triggering event
- Disabled rules (`enabled: false`) never evaluated
**Evidence:** `spike/crates/axiathon-detection/src/engine.rs` tests lines 339-503 (7 tests)
**Confidence:** HIGH

### BC-3.01.002: RuleEngine.evaluate_predicate() handles missing fields

**Preconditions:** Event with field referenced by predicate
**Postconditions:** Missing field -> predicate returns false (not an error)
**Evidence:** `engine.rs` line 128 (`None => return false`)
**Confidence:** HIGH

### BC-3.01.003: RuleEngine CIDR matching validates IP and network

**Preconditions:** Predicate with `cidr` operator
**Postconditions:**
- IP in range -> true
- IP out of range -> false
- Invalid IP string -> false (parse error, not crash)
- Invalid CIDR string -> false (parse error, not crash)
**Evidence:** `engine.rs` test `cidr_matching` lines 369-388
**Confidence:** HIGH

### BC-3.01.004: RuleEngine regex matching uses pre-compiled cache

**Preconditions:** Rule with `matches` operator
**Postconditions:**
- Regex patterns pre-compiled during `RuleEngine::new()` and cached by pattern string
- Cache hit -> use cached regex
- Cache miss -> compile on-the-fly (fallback)
- Invalid regex -> warning log, pattern never matches (not a crash)
**Evidence:** `engine.rs` lines 50-92, test `regex_matching` lines 392-411
**Confidence:** HIGH

### BC-3.01.005: RuleEngine `not` operator negates conditions

**Preconditions:** Rule with `not` before a condition
**Postconditions:** Events matching the inner condition do NOT match. Events not matching the inner condition DO match.
**Evidence:** `engine.rs` test `not_operator` lines 413-436
**Confidence:** HIGH

### BC-3.01.006: RuleEngine `or` operator evaluates alternatives

**Preconditions:** Rule with `or` between conditions
**Postconditions:** Event matching either condition matches the rule
**Evidence:** `engine.rs` test `or_condition` lines 438-462
**Confidence:** HIGH

### BC-3.01.007: RuleEngine `contains` operator does substring match

**Preconditions:** Rule with `contains` operator on string field
**Postconditions:** String containment check. Non-string fields -> false.
**Evidence:** `engine.rs` test `contains_operator` lines 464-483
**Confidence:** HIGH

### BC-3.01.008: RuleEngine float comparison uses epsilon tolerance

**Preconditions:** Comparing FieldValue to LiteralValue where one is float
**Postconditions:** Equality uses `(a - b).abs() < f64::EPSILON`. Cross-type comparison (Int vs Float, Float vs Integer) coerces to f64 then compares.
**Evidence:** `engine.rs` lines 144-161
**Confidence:** MEDIUM (from code, no dedicated test for edge cases)

### BC-3.01.009: RuleEngine `in` operator checks list membership

**Preconditions:** Rule with `in` operator and list of values
**Postconditions:** Field value converted to string representation, then checked against list items. Non-list literal -> false.
**Evidence:** `engine.rs` lines 217-225, alert test with OT rule using `in ("Level_0", "Level_1")`
**Confidence:** HIGH

---

## 8. Correlation Engine Contracts (BC-3.02.xxx)

### BC-3.02.001: CorrelationState fires at threshold within window

**Preconditions:** Correlation rule with threshold N, group_by fields, window duration
**Postconditions:**
- Below threshold -> no alert
- At threshold -> fires, returns CorrelationMatch with count, group_value, all event_uids
- After firing -> window cleared (reset), requires N more events to fire again
**Evidence:** `correlation.rs` tests lines 231-301 (5 tests)
**Confidence:** HIGH

### BC-3.02.002: Correlation groups are independent by group_by value

**Preconditions:** Events from different group_by values (e.g., different IPs)
**Postconditions:** Each group_by value has its own sliding window. Events from IP A do not count toward IP B's threshold.
**Evidence:** `correlation.rs` test `different_ips_independent` lines 258-273
**Confidence:** HIGH

### BC-3.02.003: Correlation only counts matching events

**Preconditions:** Events that do not match the correlation's inner condition
**Postconditions:** Non-matching events are not counted, even if they have the same group_by value
**Evidence:** `correlation.rs` test `success_login_not_counted` lines 275-283
**Confidence:** HIGH

### BC-3.02.004: CorrelationState.cleanup() evicts expired windows

**Preconditions:** Windows with entries older than their duration
**Postconditions:** Expired entries removed. Empty windows removed from DashMap.
**Evidence:** `correlation.rs` test `cleanup_removes_empty_windows` lines 303-314
**Confidence:** MEDIUM (limited test -- only verifies recent entries not removed)

---

## 9. Sequence Engine Contracts (BC-3.03.xxx)

### BC-3.03.001: SequenceState fires when all steps complete in order

**Preconditions:** Sequence rule with N steps, key_field, window
**Postconditions:**
- All steps complete in order (for same key_field value) -> fires SequenceMatch
- Returns step_events and step_counts for template interpolation
- After firing -> tracker reset, requires new sequence to fire again
**Evidence:** `sequence.rs` tests lines 241-317 (5 tests)
**Confidence:** HIGH

### BC-3.03.002: Sequence steps are independent by key_field value

**Preconditions:** Events from different key_field values
**Postconditions:** Failures from IP A + success from IP B does NOT complete the sequence
**Evidence:** `sequence.rs` test `different_ip_no_alert` lines 268-280
**Confidence:** HIGH

### BC-3.03.003: Sequence count steps require threshold before advancing

**Preconditions:** Count step with threshold N
**Postconditions:** Fewer than N matching events -> step not advanced. Success event at step 2 ignored while step 1 incomplete.
**Evidence:** `sequence.rs` test `insufficient_failures_no_alert` lines 282-294
**Confidence:** HIGH

### BC-3.03.004: Sequence handles more-than-threshold events gracefully

**Preconditions:** Count step with threshold 3, 5 matching events sent
**Postconditions:** Step advances at 3rd match. Extra events (4th, 5th) do not disrupt the sequence. Next step can still complete.
**Evidence:** `sequence.rs` test `more_than_threshold_failures` lines 319-331
**Confidence:** HIGH

### BC-3.03.005: Sequence trackers expire after window duration

**Preconditions:** Sequence with short window (1s)
**Postconditions:** After window expires, tracker is reset on next event. Cleanup removes expired trackers.
**Evidence:** `sequence.rs` test `cleanup_expired_trackers` lines 333-359
**Confidence:** HIGH

---

## 10. Alert Contracts (BC-3.04.xxx)

### BC-3.04.001: Alert template interpolation resolves field values

**Preconditions:** Alert template with `{field_name}` placeholders
**Postconditions:**
- `{src_endpoint.ip}` -> resolved from event's get_field()
- `{count}` -> correlation count (from extra_vars)
- `{window}` -> correlation window duration string
- `{step_name.field}` -> field from sequence step's event
- `{step_name.count}` -> count from sequence step
- Unresolvable `{field}` -> left as literal `{field}` in output
**Evidence:** `alert.rs` tests lines 339-477 (4 alert generation tests)
**Confidence:** HIGH

### BC-3.04.002: AlertStore queries are tenant-scoped with pagination

**Preconditions:** Alerts stored for multiple tenants
**Postconditions:**
- Query by tenant_id returns only that tenant's alerts
- Pagination via limit+offset works correctly
- Most recent alerts returned first (reverse order)
- Pages do not overlap
**Evidence:** `alert.rs` tests lines 479-559 (3 store tests)
**Confidence:** HIGH

### BC-3.04.003: AlertStore broadcasts to subscribers

**Preconditions:** Subscriber connected via `subscribe()`
**Postconditions:** Added alerts are broadcast. If no subscribers, broadcast is silently dropped (not an error).
**Evidence:** `alert.rs` test `alert_store_broadcast` lines 562-585
**Confidence:** HIGH

---

## 11. Case Management Contracts (BC-6.xx.xxx)

### BC-6.01.001: CaseStatus.can_transition_to() enforces state machine

**Preconditions:** Current CaseStatus and target CaseStatus
**Postconditions:**
- Forward transitions: New->Ack, Ack->Inv, Inv->Res, Res->Closed
- Skip-ahead: New->Inv/Res/Closed, Ack->Res/Closed, Inv->Closed
- Reopen: Res->Inv, Closed->Inv
- Invalid: self-transitions, backwards to New, backwards to Ack from Res/Closed
**Evidence:** `case.rs` tests lines 589-606 (2 tests, 12 assertions)
**Confidence:** HIGH

### BC-6.01.002: CaseStore.update_status() validates transitions

**Preconditions:** Case exists, new status provided
**Postconditions:**
- Valid transition -> status updated, timeline entry added, updated_at set
- Closed/Resolved -> closed_at set
- Reopen -> closed_at cleared
- Invalid transition -> `Err(CaseStoreError::InvalidTransition { from, to })`
- Nonexistent case -> `Err(CaseStoreError::NotFound)`
**Evidence:** `case.rs` tests lines 688-724 (2 tests)
**Confidence:** HIGH

### BC-6.01.003: CaseStore enforces tenant isolation

**Preconditions:** Cases for different tenants
**Postconditions:**
- `get(tenant_a, case_id)` -> Some if case belongs to tenant_a
- `get(tenant_b, case_id)` -> None if case belongs to tenant_a
- `list(tenant_a)` returns only tenant_a's cases
**Evidence:** `case.rs` test `tenant_isolation` lines 637-655
**Confidence:** HIGH

### BC-6.01.004: CaseStore.link_alerts() deduplicates

**Preconditions:** Case with existing alert links
**Postconditions:** Linking already-linked alert_id does NOT create duplicate. New alerts added. Timeline entries created only for newly-linked alerts.
**Evidence:** `case.rs` test `link_alerts_deduplicates` lines 751-769
**Confidence:** HIGH

### BC-6.01.005: Case.mttd_seconds() and mttr_seconds() compute SOC metrics

**Preconditions:** Case with created_at and closed_at
**Postconditions:**
- MTTD = case.created_at - earliest alert creation time (in seconds)
- MTTR = closed_at - created_at (in seconds)
- Unclosed case -> mttr_seconds() returns None
**Evidence:** `case.rs` test `mttd_and_mttr` lines 772-789
**Confidence:** HIGH

### BC-6.01.006: CaseStore.set_disposition() records investigation outcome

**Preconditions:** Case exists
**Postconditions:** Disposition set, timeline entry added with DispositionSet event_type. Four disposition types: TruePositive, FalsePositive, Benign, Inconclusive.
**Evidence:** `case.rs` test `set_disposition_updates_case` lines 831-857
**Confidence:** HIGH

### BC-6.01.007: CaseStore.metrics() computes tenant aggregates

**Preconditions:** Cases in various states for a tenant
**Postconditions:** Returns CaseMetrics with: total, open, closed counts; avg_mttr_seconds (only for closed/resolved); by_status and by_priority breakdowns.
**Evidence:** `case.rs` test `metrics_computation` lines 793-815
**Confidence:** HIGH

---

## 12. Storage Contracts (BC-4.xx.xxx)

### BC-4.01.001: StorageWriter buffers and flushes via Iceberg

**Preconditions:** StorageWriter with Iceberg catalog
**Postconditions:**
- Events buffered by PartitionKey (class_uid + tenant_id + hour)
- Flush writes RecordBatch to Parquet via Iceberg DataFileWriter
- Commits via fast_append transaction per class
- Zstd compression at level 3
- Empty flush is no-op (no snapshot created)
**Evidence:** `spike/crates/axiathon-storage/src/writer.rs` tests lines 457-587 (5 tests)
**Confidence:** HIGH

### BC-4.01.002: StorageWriter auto-flushes when buffer reaches capacity

**Preconditions:** buffer_size threshold configured
**Postconditions:** When total buffered events >= buffer_size, flush_notify signals background task. Background task flushes and commits.
**Evidence:** `writer.rs` test `auto_flush_on_buffer_full` lines 548-574
**Confidence:** HIGH

### BC-4.01.003: StorageWriter writes multi-class events to separate tables

**Preconditions:** Events with different class_uids
**Postconditions:** Auth events (3002) go to authentication table. Finding events (2001) go to security_finding table. Tables with no events get no snapshot.
**Evidence:** `writer.rs` test `write_multi_class_to_separate_tables` lines 482-518
**Confidence:** HIGH

### BC-4.01.004: StorageWriter preserves multi-tenant partitioning

**Preconditions:** Events from different tenants
**Postconditions:** Events partitioned by (tenant_id, hour_epoch) within each class table. Iceberg partition values use identity(tenant_id) + hour(event_time).
**Evidence:** `writer.rs` test `write_multi_tenant_via_iceberg` lines 520-545
**Confidence:** HIGH

### BC-4.01.005: Field promotion lifecycle (schema evolution + dual-write + backfill)

**Preconditions:** Events with vendor fields in unmapped JSON
**Postconditions:**
1. promote_fields() adds new column to Iceberg schema (idempotent)
2. Dual-write: new events populate both typed column and unmapped
3. json_extract_string UDF extracts from old data's unmapped JSON
4. COALESCE wrapper transparently queries old + new data
5. Compaction backfill extracts promoted fields from unmapped into typed column
6. Tenant isolation maintained across all phases
**Evidence:** `spike/crates/axiathon-storage/tests/field_promotion.rs` (1 comprehensive 8-phase test)
**Confidence:** HIGH

### BC-4.01.006: Tenant isolation enforced at query level via TenantFilterRule

**Preconditions:** Multi-tenant data in shared tables
**Postconditions:**
- DataFusion optimizer rule injects tenant_id filter
- Even explicit `WHERE tenant_id = 'wrong_tenant'` is overridden
- Non-existent tenant sees zero events
- No data leakage: tenant A's users never appear in tenant B's results
**Evidence:** `spike/crates/axiathon-storage/tests/tenant_isolation.rs` (7 tests, all `#[ignore]` due to ParquetTableProvider dependency)
**Confidence:** MEDIUM (tests are ignored in CI but well-specified)

---

## 13. Gaps: Behaviors with No Test Coverage

| Area | Gap | Confidence |
|------|-----|------------|
| AxiQL parser | Full SQL mode parsing (SELECT...FROM...WHERE...) | LOW (structure exists but test coverage not verified) |
| AxiQL parser | FilterExpr::Has and FilterExpr::Missing | LOW (variants exist, test coverage unknown) |
| AxiQL parser | FilterExpr::Wildcard | LOW (variant exists, test coverage unknown) |
| AxiQL parser | PipeStage::Tail, Dedup, Fields | LOW (variants exist, test coverage unknown) |
| Detection | ReDoS protection in spike detection DSL | NONE (explicitly marked as gap in source comments) |
| Detection | Detection rule hot-reload via arc-swap | NONE (planned, not implemented) |
| Storage | Iceberg compaction cross-schema handling | LOW (field_promotion test shows it may panic) |
| Query | CrossVersionProjection | NONE (placeholder struct) |
| Query | Type checking integration (Story 5.2) | NONE (TypeError exists but checker not wired in) |
| API | Route handler correctness | LOW (spike only, minimal tests) |
| Error | Error sanitization for API responses | NONE (marked as TODO, routes forward `e.to_string()`) |

---

## Delta Summary
- New contracts added: 52 BC entries across 6 subsystems
- Contracts from broad sweep refined: BC-001 through BC-012 expanded with exact evidence paths, error messages, and edge cases
- Remaining gaps: AxiQL parser pipe stages (Tail/Dedup/Fields), FilterExpr::Has/Missing/Wildcard test evidence, full SQL mode parse tests, detection DSL parser grammar coverage, plugin SDK contracts

## Novelty Assessment
Novelty: SUBSTANTIVE
Round 1 produced 52 behavioral contracts with exact source evidence, far exceeding the broad sweep's 12 high-level descriptions. Key new discoveries include: the float epsilon comparison in detection engine, the alert template interpolation system with step-specific variables, the CaseStatus state machine with reopen transitions and 12 validated transition pairs, the field promotion lifecycle with 8 phases, the tenant filter override behavior (explicit wrong-tenant queries are overridden), and the correlation window reset-after-fire behavior. These contracts define the actual specification surface for downstream work.

## Convergence Declaration
Another round needed -- AxiQL parser pipe stages need test evidence verification, FilterExpr::Has/Missing/Wildcard contracts need source confirmation, and the detection parser grammar needs contract extraction.

## State Checkpoint
```yaml
pass: 3
round: 1
status: complete
files_scanned: 35
timestamp: 2026-04-13T00:00:00Z
novelty: SUBSTANTIVE
```
