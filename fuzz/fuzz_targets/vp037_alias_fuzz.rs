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
use prism_query::alias_tools::{create_alias_with_clients_gated, CreateAliasInput};
use prism_query::alias_types::AliasScope;

/// Decode a fuzz input byte sequence into an (alias_graph, query) pair.
///
/// # Wire format
///
/// ```text
/// byte 0         : entry_count — number of alias entries, clamped to 0..=5
/// for each entry:
///   byte N+0     : name_len (1..=16, clamped; 0 → 1)
///   bytes N+1..  : name bytes (name_len bytes); each byte normalized to [a-zA-Z_]
///                  by mapping b % 53 into the printable identifier space
///   byte M+0     : body_len (0..=64, clamped)
///   bytes M+1..  : body bytes (body_len bytes, UTF-8 lossy); may include `@` references
/// remaining bytes: query string (UTF-8 lossy)
/// ```
///
/// All entries are created at `AliasScope::Global`. The decoder is intentionally
/// lenient — truncated or malformed inputs produce partial or empty output without
/// panicking.
fn decode_fuzz_input(data: &[u8]) -> (AliasStore, String, AliasScope) {
    let scope = AliasScope::Global;
    let mut store = AliasStore::empty("/tmp/vp037_fuzz.toml");

    if data.is_empty() {
        return (store, String::new(), scope);
    }

    let mut cursor = 0usize;

    // Byte 0: entry count, clamped to 0..=5.
    let entry_count = (data[cursor] as usize) % 6;
    cursor += 1;

    for _ in 0..entry_count {
        if cursor >= data.len() {
            break;
        }

        // Name length: 1..=16 (clamped; 0 maps to 1).
        let raw_name_len = data[cursor] as usize;
        cursor += 1;
        let name_len = (raw_name_len % 16) + 1;

        // Name bytes: normalize each byte into [a-zA-Z_] space.
        // 26 lowercase + 26 uppercase + 1 underscore = 53 printable identifier chars.
        // First character must be alpha or underscore (guaranteed by ID_CHARS mapping).
        const ID_CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_";
        let available = data.len().saturating_sub(cursor);
        let actual_name_len = name_len.min(available);
        let name_bytes: Vec<u8> = data[cursor..cursor + actual_name_len]
            .iter()
            .map(|&b| ID_CHARS[(b as usize) % ID_CHARS.len()])
            .collect();
        cursor += actual_name_len;
        let name = String::from_utf8(name_bytes).unwrap_or_else(|_| "fuzz_alias".to_string());

        if cursor >= data.len() {
            break;
        }

        // Body length: 0..=64.
        let raw_body_len = data[cursor] as usize;
        cursor += 1;
        let body_len = raw_body_len % 65;

        let available = data.len().saturating_sub(cursor);
        let actual_body_len = body_len.min(available);
        let body = String::from_utf8_lossy(&data[cursor..cursor + actual_body_len]).to_string();
        cursor += actual_body_len;

        // Attempt to add the entry to the store via the public create_alias API.
        // Failures (cycles, depth exceeded, parse failures, I/O) are silently
        // discarded — the fuzz harness cares about `expand()` panics,
        // not create-time validation.
        let input = CreateAliasInput {
            name,
            scope: "global".to_string(),
            query: body,
            parameters: None,
            description: None,
            token_id: None,
        };
        let ocsf = std::collections::HashSet::new();
        let _ = create_alias_with_clients_gated(input, &mut store, &ocsf, &[], None);
    }

    // Remaining bytes form the query string.
    let query = if cursor < data.len() {
        String::from_utf8_lossy(&data[cursor..]).to_string()
    } else {
        String::new()
    };

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
