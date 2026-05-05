//! BC-2.11.006 v1.7 §47 — Security Perimeter compile-fail test.
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
//! Forbidden symbols (all `pub(crate)` in prism-query, BC-2.11.006 v1.7 §47):
//!
//!   Sub-parser entry points:
//!     - filter_parser::parse_filter    — filter mode sub-parser entry point
//!     - sql_parser::parse_sql          — SQL mode sub-parser entry point
//!     - pipe_parser::parse_pipe        — pipe mode sub-parser entry point
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
//!     - security::ParseLimits — struct fields + install_thread_local are pub(crate).
//!       External callers cannot construct a ParseLimits with adversarial field
//!       values (e.g., regex_pattern: usize::MAX) to bypass the regex length guard.
//!
//! Reference: adversary pass-5 OBS-003 [process-gap]; adversary pass-6 F-HIGH-001/F-HIGH-002.

// ── Sub-parser entry points ──────────────────────────────────────────────────

// `parse_filter` is `pub(crate)` — forbidden from external crates.
// Expected error: E0603 "function `parse_filter` is private"
use prism_query::filter_parser::parse_filter;

// `parse_sql` is `pub(crate)` — forbidden from external crates.
// Expected error: E0603 "function `parse_sql` is private"
use prism_query::sql_parser::parse_sql;

// `parse_pipe` is `pub(crate)` — forbidden from external crates.
// Expected error: E0603 "function `parse_pipe` is private"
use prism_query::pipe_parser::parse_pipe;

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

// `security::ParseLimits` — struct and its methods are `pub(crate)`.
// External callers CANNOT construct ParseLimits with adversarial values.
// Expected error: E0603 on install_thread_local (private method)
//
// We import `ParseLimits` (the struct is pub, but fields + key methods are pub(crate))
// and attempt to call install_thread_local — which is pub(crate).
use prism_query::security::ParseLimits;

fn main() {
    // These calls are unreachable due to compile failure, but are written
    // to prevent the compiler from eliding the use statements via dead-code
    // elimination before the visibility check fires.
    let _ = parse_filter("host = \"example.com\"");
    let _ = parse_sql("SELECT * FROM crowdstrike");
    let _ = parse_pipe("crowdstrike | filter host = \"x\"");
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
}
