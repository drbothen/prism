//! BC-2.11.006 v1.7 — Security Perimeter compile-fail test.
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
//! Forbidden symbols (all `pub(crate)` in prism-query):
//!   - filter_parser::parse_filter    — filter mode sub-parser entry point
//!   - sql_parser::parse_sql          — SQL mode sub-parser entry point
//!   - pipe_parser::parse_pipe        — pipe mode sub-parser entry point
//!   - pipe_parser::build_pipe_parser — internal Chumsky parser builder factory
//!
//! Reference: adversary pass-5 OBS-003 [process-gap].

// Category 1: sub-parser entry point (filter mode).
// `parse_filter` is `pub(crate)` — forbidden from external crates.
// Expected error: E0603 "function `parse_filter` is private"
use prism_query::filter_parser::parse_filter;

// Category 2: sub-parser entry point (SQL mode).
// `parse_sql` is `pub(crate)` — forbidden from external crates.
// Expected error: E0603 "function `parse_sql` is private"
use prism_query::sql_parser::parse_sql;

// Category 3: sub-parser entry point (pipe mode).
// `parse_pipe` is `pub(crate)` — forbidden from external crates.
// Expected error: E0603 "function `parse_pipe` is private"
use prism_query::pipe_parser::parse_pipe;

// Category 4: internal builder factory (pipe parser builder).
// `build_pipe_parser` is `pub(crate)` — forbidden from external crates.
// Expected error: E0603 "function `build_pipe_parser` is private"
use prism_query::pipe_parser::build_pipe_parser;

fn main() {
    // These calls are unreachable due to compile failure, but are written
    // to prevent the compiler from eliding the use statements via dead-code
    // elimination before the visibility check fires.
    let _ = parse_filter("host = \"example.com\"");
    let _ = parse_sql("SELECT * FROM crowdstrike");
    let _ = parse_pipe("crowdstrike | filter host = \"x\"");
    let _ = build_pipe_parser();
}
