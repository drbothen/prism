//! BC-2.11.006 v1.16 — Security Perimeter compile-fail test.
//!
//! This file intentionally uses symbols that are `pub(crate)` inside
//! `prism-query` and MUST NOT be accessible to external crates.
//!
//! Expected compile result: ERROR (visibility violation).
//!
//! If `cargo check -p perimeter-violation` ever exits 0, the CI job
//! `perimeter-compile-fail` treats that as a security regression and
//! blocks the merge (a sub-parser became accidentally public).
//!
//! Forbidden symbols (all `pub(crate)` in prism-query, BC-2.11.006 Postconditions: Security Perimeter):
//!
//!   Sub-parser entry points:
//!     - filter_parser::parse_filter              — filter mode sub-parser entry point
//!     - filter_parser::parse_filter_with_limits  — filter mode sub-parser (with limits ctx)
//!     - sql_parser::parse_sql                    — SQL mode sub-parser entry point
//!     - sql_parser::parse_sql_with_limits        — SQL mode sub-parser (with limits ctx)
//!     - pipe_parser::parse_pipe                  — pipe mode sub-parser entry point
//!     - pipe_parser::parse_pipe_with_limits      — pipe mode sub-parser (with limits ctx)
//!
//!   Builder factories (seven):
//!     - filter_parser::build_predicate_parser  — predicate Chumsky builder
//!     - filter_parser::build_source_ref_parser — source-ref Chumsky builder
//!     - filter_parser::build_string_parser     — string literal Chumsky builder
//!     - filter_parser::build_literal_parser    — literal Chumsky builder
//!     - filter_parser::build_expr_parser       — expression Chumsky builder
//!     - filter_parser::build_pipe_mode_parser  — pipe-mode Chumsky builder
//!     - pipe_parser::build_pipe_parser         — pipe Chumsky builder
//!
//!   Thread-local ParseLimits API (F-HIGH-002):
//!     - security::ParseLimits::install_thread_local — sets thread-local limits
//!     - security::ParseLimits::clear_thread_local   — clears thread-local limits
//!     - security::ParseLimits::current_regex_limit  — reads current regex limit
//!     - security::ParseLimits::snapshot             — captures current env limits (F-HIGH-001)
//!       External callers cannot construct a ParseLimits with adversarial field
//!       values (e.g., regex_pattern: usize::MAX) to bypass the regex length guard.
//!
//!   S-3.06 write parser extensions (9 new restricted symbols, BC-2.11.006 v1.16):
//!     - pipe_parser::parse_pipe_with_write    — pipe write-stage entry point
//!     - pipe_parser::build_write_stage_parser — write-stage Chumsky builder
//!     - pipe_parser::build_write_arg_parser   — write-arg Chumsky builder
//!     - pipe_parser::extract_sensor_prefix    — sensor prefix extractor
//!     - sql_parser::parse_sql_dml             — DML statement entry point
//!     - sql_parser::parse_sql_dml_with_limits — DML entry point with caller-provided limits
//!     - sql_parser::is_internal_prism_table   — prism_* table guard
//!     - sql_parser::check_unbounded_write     — unbounded write guard
//!     - filter_parser::reject_write_verbs_in_filter — filter-mode write rejection
//!
//!   S-3.04 alias tools and store (pub(crate) — forbidden from external crates):
//!     - alias_tools::create_alias                          — ungated create_alias (SEC-011)
//!     - alias_tools::create_alias_with_clients             — create_alias with client list (SEC-011)
//!     - alias_tools::delete_alias                          — ungated delete_alias (SEC-011)
//!     - alias_tools::create_alias_with_clients_gated_inner — internal token-store split (F-LOCAL-P2-HIGH-005)
//!     - alias_store::AliasStore::create_or_update          — direct store mutation bypass (CR-018)
//!
//! Reference: adversary pass-5 OBS-003 [process-gap]; adversary pass-6 F-HIGH-001/F-HIGH-002;
//!            adversary pass-7 F-HIGH-001/F-MEDIUM-002; adversary pass-8 F-HIGH-001/OBS-001;
//!            S-3.06 BC-2.11.006 v1.16 INV-SEC-PERIMETER-001;
//!            S-3.04 local adversary pass-1 F-HIGH-003.

// ── Sub-parser entry points ──────────────────────────────────────────────────

// `parse_filter` is `pub(crate)` — forbidden from external crates.
// Expected error: E0603 "function `parse_filter` is private"
use prism_query::filter_parser::parse_filter;

// `parse_filter_with_limits` is `pub(crate)` — forbidden from external crates.
// Expected error: E0603 "function `parse_filter_with_limits` is private"
use prism_query::filter_parser::parse_filter_with_limits;

// `parse_sql` is `pub(crate)` — forbidden from external crates.
// Expected error: E0603 "function `parse_sql` is private"
use prism_query::sql_parser::parse_sql;

// `parse_sql_with_limits` is `pub(crate)` — forbidden from external crates.
// Expected error: E0603 "function `parse_sql_with_limits` is private"
use prism_query::sql_parser::parse_sql_with_limits;

// `parse_pipe` is `pub(crate)` — forbidden from external crates.
// Expected error: E0603 "function `parse_pipe` is private"
use prism_query::pipe_parser::parse_pipe;

// `parse_pipe_with_limits` is `pub(crate)` — forbidden from external crates.
// Expected error: E0603 "function `parse_pipe_with_limits` is private"
use prism_query::pipe_parser::parse_pipe_with_limits;

// ── Builder factories ────────────────────────────────────────────────────────

// `build_pipe_parser` is `pub(crate)` — forbidden from external crates.
// Expected error: E0603 "function `build_pipe_parser` is private"
use prism_query::pipe_parser::build_pipe_parser;

// `build_predicate_parser` is `pub(crate)` — forbidden from external crates.
// Expected error: E0603 "function `build_predicate_parser` is private"
use prism_query::filter_parser::build_predicate_parser;

// `build_source_ref_parser` is `pub(crate)` — forbidden from external crates.
// Expected error: E0603 "function `build_source_ref_parser` is private"
use prism_query::filter_parser::build_source_ref_parser;

// `build_string_parser` is `pub(crate)` — forbidden from external crates.
// Expected error: E0603 "function `build_string_parser` is private"
use prism_query::filter_parser::build_string_parser;

// `build_literal_parser` is `pub(crate)` — forbidden from external crates.
// Expected error: E0603 "function `build_literal_parser` is private"
use prism_query::filter_parser::build_literal_parser;

// `build_expr_parser` is `pub(crate)` — forbidden from external crates.
// Expected error: E0603 "function `build_expr_parser` is private"
use prism_query::filter_parser::build_expr_parser;

// `build_pipe_mode_parser` is `pub(crate)` — forbidden from external crates.
// Expected error: E0603 "function `build_pipe_mode_parser` is private"
use prism_query::filter_parser::build_pipe_mode_parser;

// ── Thread-local ParseLimits API (F-HIGH-002) ────────────────────────────────

// `security::ParseLimits` — key methods are `pub(crate)`.
// External callers CANNOT construct ParseLimits with adversarial values.
// We import `ParseLimits` and attempt to call pub(crate) methods.
use prism_query::security::ParseLimits;

// ── S-3.06 write parser extensions (BC-2.11.006 v1.16, INV-SEC-PERIMETER-001) ─

// `parse_pipe_with_write` is `pub(crate)` — forbidden from external crates.
// Expected error: E0603 "function `parse_pipe_with_write` is private"
use prism_query::pipe_parser::parse_pipe_with_write;

// `build_write_stage_parser` is `pub(crate)` — forbidden from external crates.
// Expected error: E0603 "function `build_write_stage_parser` is private"
use prism_query::pipe_parser::build_write_stage_parser;

// `build_write_arg_parser` is `pub(crate)` — forbidden from external crates.
// Expected error: E0603 "function `build_write_arg_parser` is private"
use prism_query::pipe_parser::build_write_arg_parser;

// `extract_sensor_prefix` is `pub(crate)` — forbidden from external crates.
// Expected error: E0603 "function `extract_sensor_prefix` is private"
use prism_query::pipe_parser::extract_sensor_prefix;

// `parse_sql_dml` is `pub(crate)` — forbidden from external crates.
// Expected error: E0603 "function `parse_sql_dml` is private"
use prism_query::sql_parser::parse_sql_dml;

// `parse_sql_dml_with_limits` is `pub(crate)` — forbidden from external crates.
// Expected error: E0603 "function `parse_sql_dml_with_limits` is private"
// Added in BC-2.11.006 v1.14 (F-PR130-P1-HIGH-002).
use prism_query::sql_parser::parse_sql_dml_with_limits;

// `is_internal_prism_table` is `pub(crate)` — forbidden from external crates.
// Expected error: E0603 "function `is_internal_prism_table` is private"
use prism_query::sql_parser::is_internal_prism_table;

// `check_unbounded_write` is `pub(crate)` — forbidden from external crates.
// Expected error: E0603 "function `check_unbounded_write` is private"
use prism_query::sql_parser::check_unbounded_write;

// `reject_write_verbs_in_filter` is `pub(crate)` — forbidden from external crates.
// Expected error: E0603 "function `reject_write_verbs_in_filter` is private"
use prism_query::filter_parser::reject_write_verbs_in_filter;

// ── S-3.04 alias system security perimeter (F-HIGH-003) ─────────────────────

// `alias_tools::create_alias` is `pub(crate)` — forbidden from external crates.
// MCP layer must use `create_alias_with_clients_gated` to enforce alias.write (SEC-011).
// Expected error: E0603 "function `create_alias` is private"
use prism_query::alias_tools::create_alias;

// `alias_tools::create_alias_with_clients` is `pub(crate)` — forbidden from external crates.
// External callers must use `create_alias_with_clients_gated` (SEC-011).
// Expected error: E0603 "function `create_alias_with_clients` is private"
use prism_query::alias_tools::create_alias_with_clients;

// `alias_tools::delete_alias` is `pub(crate)` — forbidden from external crates.
// External callers must use `delete_alias_gated` (SEC-011).
// Expected error: E0603 "function `delete_alias` is private"
use prism_query::alias_tools::delete_alias;

// `alias_tools::create_alias_with_clients_gated_inner` is `pub(crate)` — forbidden from external crates.
// Internal split to allow test token-store injection; external crates MUST NOT call it directly.
// Expected error: E0603 "function `create_alias_with_clients_gated_inner` is private"
use prism_query::alias_tools::create_alias_with_clients_gated_inner;

// `alias_store::AliasStore::create_or_update` is `pub(crate)` — forbidden from external crates.
// Direct store mutation bypasses keyword/OCSF collision checks (CR-018).
// Expected error: E0624 "method `create_or_update` is private"
use prism_query::alias_store::AliasStore;

fn main() {
    // These calls are unreachable due to compile failure, but are written
    // to prevent the compiler from eliding the use statements via dead-code
    // elimination before the visibility check fires.
    let _ = parse_filter("host = \"example.com\"");
    let _ = parse_filter_with_limits;
    let _ = parse_sql("SELECT * FROM crowdstrike");
    let _ = parse_sql_with_limits;
    let _ = parse_pipe("crowdstrike | filter host = \"x\"");
    let _ = parse_pipe_with_limits;
    let _ = build_pipe_parser();
    let _ = build_predicate_parser();
    let _ = build_source_ref_parser();
    let _ = build_string_parser();
    let _ = build_literal_parser();
    let _ = build_expr_parser();
    let _ = build_pipe_mode_parser();

    // Attempt to call install_thread_local — must fail: pub(crate) method.
    // Expected error: E0624 "method `install_thread_local` is private"
    let limits = ParseLimits::snapshot();
    limits.install_thread_local();

    // Pass-8 F-HIGH-001 closure — function-pointer reference forces visibility check
    // even if the compiler elides the call above.  `snapshot` is pub(crate) so this
    // must fail with E0624 "method `snapshot` is private".
    let _ = ParseLimits::snapshot;

    // Pass-7 F-MEDIUM-002 — exercise each ParseLimits method individually so
    // regressions to pub are caught even when sibling methods remain pub(crate).
    let _ = ParseLimits::clear_thread_local;
    let _ = ParseLimits::current_regex_limit;

    // S-3.06 write parser extensions — force visibility check.
    // These lines are unreachable due to compile failure above, but reference
    // the new symbols to ensure the compiler emits visibility errors for each.
    let _ = parse_pipe_with_write;
    let _ = build_write_stage_parser;
    let _ = build_write_arg_parser;
    let _ = extract_sensor_prefix;
    let _ = parse_sql_dml;
    let _ = parse_sql_dml_with_limits;
    let _ = is_internal_prism_table;
    let _ = check_unbounded_write;
    let _ = reject_write_verbs_in_filter;

    // S-3.04 alias system security perimeter (F-HIGH-003) — force visibility checks.
    // These lines are unreachable due to compile failures above, but reference the
    // alias symbols to ensure visibility errors are emitted for each if they become pub.
    let _ = create_alias;
    let _ = create_alias_with_clients;
    let _ = delete_alias;
    let _ = create_alias_with_clients_gated_inner;
    // create_or_update is a pub(crate) method on AliasStore.
    // Expected error: E0624 "method `create_or_update` is private"
    // The method-reference form forces the visibility check even when unreachable.
    let _ = AliasStore::create_or_update;
}
