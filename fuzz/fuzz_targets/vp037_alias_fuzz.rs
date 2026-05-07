//! VP-037 fuzz target: AliasResolver::expand() never panics on arbitrary alias graphs.
//!
//! Property (VP-037): For every byte sequence `data` interpreted as an alias-map
//! + query pair, `AliasResolver::expand()` returns `Ok` or `Err` in bounded time
//! without panicking. Specifically, ALL of the following adversarial inputs must
//! produce `Err(...)`, never a panic, stack overflow, or infinite loop:
//!
//! - **Cycles**: A → A (self-loop), A → B → A (mutual), A → B → C → A (chain)
//! - **Deep nesting**: A → B → C → D → E (depth 5, exceeds limit of 3)
//! - **Self-reference**: `@A` inside A's own definition
//! - **Non-UTF-8 content** (handled by lossy conversion at harness boundary)
//! - **Invalid parameter values** (injection attempts, compound expressions)
//! - **Empty parameter maps** on parameterized aliases
//!
//! Each error case must produce `Err(_)`, not a panic.
//!
//! Source BC: BC-2.11.008 / BC-2.11.009
//! VP: VP-037
//! Method: cargo-fuzz (libFuzzer), coverage-guided
//! Runtime: 30 minutes minimum initial; continuous in CI
//!
//! Story: S-3.04 — prism-query: Alias System (P1)

#![no_main]

use std::collections::HashMap;

use libfuzzer_sys::fuzz_target;
use prism_query::alias_resolver::AliasResolver;
use prism_query::alias_store::AliasStore;
use prism_query::alias_types::AliasScope;

/// Decode a fuzz input byte sequence into an (alias_graph, query) pair.
///
/// Format (best-effort, tolerates malformed input gracefully):
/// - First byte: number of alias entries N (0–9, clamped).
/// - For each entry: next byte = name_len (clamped to 1–16), then name bytes
///   (normalized to `a-zA-Z_`), next byte = def_len (clamped to 0–64), then
///   definition bytes (UTF-8 lossy).
/// - Remaining bytes: interpreted as the query string (UTF-8 lossy).
///
/// This decoder is intentionally lenient — it must never panic on any input.
fn decode_fuzz_input(data: &[u8]) -> (AliasStore, String, AliasScope) {
    // NOTE: This function body is intentionally minimal — the store building
    // and query extraction are pure data operations that must not panic.
    // Full implementation will be provided by the implementer of the alias system.
    // For now, return an empty store and the raw input as the query.
    let query = String::from_utf8_lossy(data).to_string();
    let store = AliasStore::empty("/tmp/fuzz_aliases.toml");
    let scope = AliasScope::Global;
    (store, query, scope)
}

fuzz_target!(|data: &[u8]| {
    // Decode the fuzz input into an alias graph and a query string.
    // decode_fuzz_input must never panic regardless of input content.
    let (store, query, scope) = decode_fuzz_input(data);

    // AliasResolver::expand() MUST NOT panic.
    // It must return Ok(expanded) or Err(structured_error) for ALL inputs.
    //
    // Any panic here is a VP-037 violation and will be flagged by libFuzzer
    // as a crash/failure finding.
    let args: HashMap<String, String> = HashMap::new();
    let _ = AliasResolver::expand(&query, &store, &scope, &args, 0);
});
