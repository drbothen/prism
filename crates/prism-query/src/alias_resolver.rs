//! `AliasResolver` — pre-parse alias expansion, scope resolution, parameter
//! substitution, and cycle detection.
//!
//! Alias resolution is a **pure** transformation: it takes a raw query string
//! and an immutable `AliasStore` reference and returns an expanded query string.
//! No I/O occurs during expansion — the store is already loaded into memory.
//!
//! ## Five-step expansion sequence (BC-2.11.009)
//!
//! 1. **Detection** — scan for `@identifier` tokens using regex
//!    `@([a-zA-Z_][a-zA-Z0-9_]{0,63})`.
//! 2. **Scope resolution** — per-client alias overrides global alias of same name.
//! 3. **Parameter substitution** — replace `{{param}}` placeholders; validate
//!    each substituted value as a PrismQL atomic literal (injection guard).
//! 4. **Recursive expansion** — call `expand()` on the substituted definition
//!    with `depth + 1`; reject when `depth >= 3` with `E-ALIAS-003`.
//! 5. **Security check** — reject expanded query > 64KB with `E-QUERY-003`.
//!
//! Story: S-3.04 — prism-query: Alias System (P1)
//! BCs:   BC-2.11.009

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use regex::Regex;

use prism_core::error::PrismError;

use crate::alias_store::AliasStore;
use crate::alias_types::AliasScope;

/// Maximum alias composition depth (hard ceiling; DI-020 / VP-012).
pub const MAX_ALIAS_DEPTH: u32 = 3;

/// Maximum expanded query size in bytes (security gate; BC-2.11.009 step 5).
pub const MAX_EXPANDED_QUERY_BYTES: usize = 65_536; // 64 KiB

/// Regex pattern for `@alias_name` detection (BC-2.11.009 step 1).
pub const ALIAS_DETECTION_PATTERN: &str = r"@([a-zA-Z_][a-zA-Z0-9_]{0,63})";

/// Regex for compound expression characters that must NOT appear in param values.
///
/// Matches: `|`, `(`, `)`, `=`, `!`, `>`, `<`, `@`, `;`, `\`, SQL-comment sequences
/// `--` and `/* */`, and whitespace-bounded AND/OR/NOT keywords.
/// (CR-001 / SEC-001: extended injection guard)
const INJECTION_CHAR_PATTERN: &str = r"[@;|()=!<>\\]|--|/\*|\*/|(?i)\bAND\b|\bOR\b|\bNOT\b";

/// Duration literal pattern: digits followed by a time unit.
///
/// Only `s` (seconds), `m` (minutes), `h` (hours), and `d` (days) are
/// recognized — matching the four variants of `DurationUnit` in the AST
/// and the BC-2.11.009 specification. Units `w`, `M`, and `y` are NOT
/// accepted here; values like "4w" would pass param validation but fail at
/// the PrismQL parser (CR-P6-001).
const DURATION_PATTERN: &str = r"^\d+[smhd]$";

/// Integer literal pattern (optional leading minus).
const INTEGER_PATTERN: &str = r"^-?\d+$";

/// Float literal pattern.
const FLOAT_PATTERN: &str = r"^-?\d+\.\d+$";

/// Valid identifier pattern for param values (BC-2.11.009: Identifier type).
const IDENTIFIER_PATTERN: &str = r"^[a-zA-Z_][a-zA-Z0-9_]*$";

// ─────────────────────────────────────────────────────────────────────────────
// Cached regex compilation
// ─────────────────────────────────────────────────────────────────────────────

#[allow(clippy::expect_used)]
fn alias_detection_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(ALIAS_DETECTION_PATTERN).expect("ALIAS_DETECTION_PATTERN is a valid regex")
    })
}

#[allow(clippy::expect_used)]
fn injection_char_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(INJECTION_CHAR_PATTERN).expect("INJECTION_CHAR_PATTERN is a valid regex")
    })
}

#[allow(clippy::expect_used)]
fn duration_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(DURATION_PATTERN).expect("DURATION_PATTERN is a valid regex"))
}

#[allow(clippy::expect_used)]
fn integer_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(INTEGER_PATTERN).expect("INTEGER_PATTERN is a valid regex"))
}

#[allow(clippy::expect_used)]
fn float_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(FLOAT_PATTERN).expect("FLOAT_PATTERN is a valid regex"))
}

#[allow(clippy::expect_used)]
fn identifier_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(IDENTIFIER_PATTERN).expect("IDENTIFIER_PATTERN is a valid regex"))
}

// ─────────────────────────────────────────────────────────────────────────────
// AliasResolver
// ─────────────────────────────────────────────────────────────────────────────

/// Pure alias expansion engine.
///
/// All methods are associated functions taking `&AliasStore` — the resolver
/// holds no mutable state. Thread-safety follows from purity.
pub struct AliasResolver;

impl AliasResolver {
    /// Expand all `@alias_name` references in `query`.
    ///
    /// This is the primary entry point. Called on the raw query string BEFORE
    /// it is passed to the Chumsky parser.
    ///
    /// # Parameters
    /// - `query`  — raw query string (may contain `@alias` tokens).
    /// - `store`  — loaded alias registry (immutable).
    /// - `scope`  — client scope for per-client-overrides-global precedence.
    /// - `args`   — caller-supplied parameter values for parameterized aliases.
    /// - `depth`  — current recursion depth; callers pass `0` for top-level.
    ///
    /// # Errors
    /// - `E-ALIAS-001` — references an undefined alias.
    /// - `E-ALIAS-003` — depth limit exceeded (`depth >= MAX_ALIAS_DEPTH`).
    /// - `E-ALIAS-004` — parameter value fails atomic-literal validation.
    /// - `E-QUERY-003` — expanded query exceeds 64KB.
    pub fn expand(
        query: &str,
        store: &AliasStore,
        scope: &AliasScope,
        args: &HashMap<String, String>,
        depth: u32,
    ) -> Result<String, PrismError> {
        // Step 4 pre-check: depth >= MAX_ALIAS_DEPTH means the next level would
        // be depth+1 which exceeds the limit. Fire before alias lookup (VP-012).
        if depth >= MAX_ALIAS_DEPTH {
            return Err(PrismError::AliasDepthExceeded {
                chain: format!("depth={depth}"),
            });
        }

        // Step 5 pre-check: reject oversized input before any expansion.
        if query.len() > MAX_EXPANDED_QUERY_BYTES {
            return Err(PrismError::QueryExecutionFailed {
                detail: format!(
                    "E-QUERY-003: expanded query exceeds 64KB limit ({} bytes)",
                    query.len()
                ),
            });
        }

        // Step 1: detect @alias_name tokens.
        let tokens = Self::detect_alias_tokens(query);

        if tokens.is_empty() {
            // No alias references — return query as-is.
            return Ok(query.to_string());
        }

        // Steps 2–4: for each unique alias token, resolve and substitute.
        let mut result = query.to_string();
        for alias_name in &tokens {
            // Step 2: scope resolution (per-client overrides global).
            let entry = Self::resolve_scope(alias_name, store, scope)?;

            // Step 3: parameter substitution.
            let substituted = Self::substitute_params(&entry.query, entry, args)?;

            // Step 4: recursive expansion with depth + 1.
            let expanded_body = Self::expand(&substituted, store, scope, args, depth + 1)?;

            // Replace all `@alias_name` occurrences in result with expanded body.
            // Use the detection regex with a named-capture closure to avoid replacing
            // prefix-aliases (e.g., expanding @foo must NOT touch @foobar). CR-009.
            let re = alias_detection_regex();
            result = re
                .replace_all(&result, |caps: &regex::Captures| {
                    let captured_name = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                    if captured_name == alias_name.as_str() {
                        expanded_body.clone()
                    } else {
                        caps[0].to_string()
                    }
                })
                .into_owned();
        }

        // Step 5: size check on expanded output.
        if result.len() > MAX_EXPANDED_QUERY_BYTES {
            return Err(PrismError::QueryExecutionFailed {
                detail: format!(
                    "E-QUERY-003: expanded query exceeds 64KB limit ({} bytes)",
                    result.len()
                ),
            });
        }

        Ok(result)
    }

    /// Detect a cycle that would be introduced by adding a new alias named
    /// `name` with the given `definition`.
    ///
    /// Performs a DFS over the alias reference graph starting from `name`. If
    /// `name` is encountered again during traversal, returns
    /// `Err(E-ALIAS-002)` with the exact cycle chain.
    ///
    /// MUST be called at alias creation time so the store never contains a
    /// cycle (BC-2.11.009 invariant DI-020).
    ///
    /// `scope` is used to resolve aliases using the same per-client-overrides-global
    /// precedence as `resolve_scope` (CR-002 fix: per-client cycles are detected).
    pub fn detect_cycle(
        name: &str,
        definition: &str,
        store: &AliasStore,
    ) -> Result<(), PrismError> {
        // Default to global scope for backwards-compat callers that don't supply a scope.
        Self::detect_cycle_scoped(name, definition, store, &AliasScope::Global)
    }

    /// Scope-aware variant of `detect_cycle`.
    ///
    /// Uses `scope` to look up aliases — per-client first, then global — mirroring
    /// `resolve_scope` precedence so that per-client cycles are reliably detected
    /// (CR-002 / SEC-004).
    pub fn detect_cycle_scoped(
        name: &str,
        definition: &str,
        store: &AliasStore,
        scope: &AliasScope,
    ) -> Result<(), PrismError> {
        let tokens = Self::detect_alias_tokens(definition);
        let mut visited: HashSet<String> = HashSet::new();
        visited.insert(name.to_string());

        let mut chain = vec![name.to_string()];
        Self::dfs_cycle(name, &tokens, store, scope, &mut visited, &mut chain)?;
        Ok(())
    }

    /// DFS helper for cycle detection.
    ///
    /// Resolves alias bodies using the same per-client-overrides-global precedence
    /// as `resolve_scope` to ensure per-client cycles are detectable (CR-002).
    fn dfs_cycle(
        origin: &str,
        tokens: &[String],
        store: &AliasStore,
        scope: &AliasScope,
        visited: &mut HashSet<String>,
        chain: &mut Vec<String>,
    ) -> Result<(), PrismError> {
        for token in tokens {
            if token == origin {
                // Self-reference or back-edge to the original alias = cycle.
                chain.push(token.clone());
                let cycle_chain = chain.join(" -> ");
                return Err(PrismError::AliasCycleDetected {
                    name: origin.to_string(),
                    cycle_chain,
                });
            }
            if visited.contains(token) {
                // Already visited this node in a non-origin context — still a cycle if
                // we reach origin through it. Track for accurate reporting.
                chain.push(token.clone());
                let cycle_chain = chain.join(" -> ");
                chain.pop();
                return Err(PrismError::AliasCycleDetected {
                    name: origin.to_string(),
                    cycle_chain,
                });
            }

            // Resolve the token using the same scope-precedence as resolve_scope
            // (per-client overrides global) — CR-002 fix.
            let resolved_entry: Option<String> = {
                let client_entry = if let AliasScope::Client(_) = scope {
                    store
                        .get(token, scope)
                        .ok()
                        .flatten()
                        .map(|e| e.query.clone())
                } else {
                    None
                };
                client_entry.or_else(|| {
                    store
                        .get(token, &AliasScope::Global)
                        .ok()
                        .flatten()
                        .map(|e| e.query.clone())
                })
            };

            if let Some(query) = resolved_entry {
                visited.insert(token.clone());
                chain.push(token.clone());
                let child_tokens = Self::detect_alias_tokens(&query);
                Self::dfs_cycle(origin, &child_tokens, store, scope, visited, chain)?;
                chain.pop();
                visited.remove(token);
            }
            // If alias doesn't exist, that's E-ALIAS-001 at expansion time — not a cycle.
        }
        Ok(())
    }

    /// Scan `query` for `@alias_name` tokens using the detection regex.
    ///
    /// Returns a `Vec` of alias name strings (without the `@` sigil), in the
    /// order they appear in `query`. Dotted field names (e.g., `device.ip`)
    /// are not matched (EC-11-023).
    pub(crate) fn detect_alias_tokens(query: &str) -> Vec<String> {
        let re = alias_detection_regex();
        let mut tokens = Vec::new();
        let mut seen = HashSet::new();

        for cap in re.captures_iter(query) {
            if let Some(m) = cap.get(1) {
                let name = m.as_str().to_string();
                // Only include each alias name once (deduplicate).
                if seen.insert(name.clone()) {
                    tokens.push(name);
                }
            }
        }
        tokens
    }

    /// Resolve scope for a single alias name.
    ///
    /// Precedence (BC-2.11.009 step 2):
    /// 1. Per-client alias if `scope` is `Client(id)` and an alias for that
    ///    client exists.
    /// 2. Global alias as fallback.
    /// 3. `Err(E-ALIAS-001)` if neither exists.
    pub(crate) fn resolve_scope<'a>(
        alias_name: &str,
        store: &'a AliasStore,
        scope: &AliasScope,
    ) -> Result<&'a crate::alias_types::AliasEntry, PrismError> {
        // Try per-client first.
        if let AliasScope::Client(_) = scope {
            if let Ok(Some(entry)) = store.get(alias_name, scope) {
                return Ok(entry);
            }
        }

        // Fall back to global.
        if let Ok(Some(entry)) = store.get(alias_name, &AliasScope::Global) {
            return Ok(entry);
        }

        // Neither found. Include only scope-visible aliases in the error to
        // avoid cross-client alias disclosure (SEC-003).
        // For Client scope: show client aliases + global aliases (both legitimately visible).
        // For Global scope: show only global aliases.
        let visible_entries: Vec<&crate::alias_types::AliasEntry> = match scope {
            AliasScope::Client(_) => {
                let mut v = store.list(Some(scope));
                v.extend(store.list(Some(&AliasScope::Global)));
                v
            }
            AliasScope::Global => store.list(Some(&AliasScope::Global)),
        };
        let available = visible_entries
            .iter()
            .map(|e| e.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");

        Err(PrismError::AliasNotFound {
            name: alias_name.to_string(),
            scope: scope.display_string(),
            available,
        })
    }

    /// Validate that `value` is a PrismQL atomic literal token.
    ///
    /// Accepts: `StringLiteral`, `IntegerLiteral`, `FloatLiteral`,
    /// `BooleanLiteral`, `DurationLiteral`, `Identifier`.
    ///
    /// Rejects values containing compound-expression characters
    /// (`|`, `(`, `)`, `=`, `!=`, `>`, `<`, `>=`, `<=`, `AND`, `OR`, `NOT`)
    /// with `Err(E-ALIAS-004)` (BC-2.11.009 injection guard).
    pub(crate) fn validate_atomic_literal(
        value: &str,
        param: &str,
        alias: &str,
    ) -> Result<(), PrismError> {
        // Reject empty values.
        if value.is_empty() {
            return Err(PrismError::AliasParameterInvalid {
                param: param.to_string(),
                alias: alias.to_string(),
                value: value.to_string(),
                reason: "empty value is not a valid atomic literal".to_string(),
            });
        }

        // Reject compound expression characters first (injection guard).
        let inj_re = injection_char_regex();
        if inj_re.is_match(value) {
            return Err(PrismError::AliasParameterInvalid {
                param: param.to_string(),
                alias: alias.to_string(),
                value: value.to_string(),
                reason: "compound expression rejected; use a single literal token".to_string(),
            });
        }

        // Accept single-quoted or double-quoted string literals.
        // CR-010: also validate interior for disallowed control characters.
        let is_double_quoted = value.starts_with('"') && value.ends_with('"') && value.len() >= 2;
        let is_single_quoted = value.starts_with('\'') && value.ends_with('\'') && value.len() >= 2;
        if is_double_quoted || is_single_quoted {
            let interior = &value[1..value.len() - 1];
            // CR-P2-005: reject tab (\t) in addition to newline, carriage-return, and null.
            if interior.contains('\n')
                || interior.contains('\r')
                || interior.contains('\0')
                || interior.contains('\t')
            {
                return Err(PrismError::AliasParameterInvalid {
                    param: param.to_string(),
                    alias: alias.to_string(),
                    value: value.to_string(),
                    reason: "string literal must not contain newline, tab, or null bytes"
                        .to_string(),
                });
            }
            return Ok(());
        }

        // Accept boolean literals (case-insensitive).
        let upper = value.to_uppercase();
        if upper == "TRUE" || upper == "FALSE" {
            return Ok(());
        }

        // Accept integer literals.
        if integer_regex().is_match(value) {
            return Ok(());
        }

        // Accept float literals.
        if float_regex().is_match(value) {
            return Ok(());
        }

        // Accept duration literals (e.g. 4h, 1d, 30m, 2w).
        if duration_regex().is_match(value) {
            return Ok(());
        }

        // Accept identifiers (single word, no spaces, no operators).
        if identifier_regex().is_match(value) {
            return Ok(());
        }

        // Reject anything else.
        Err(PrismError::AliasParameterInvalid {
            param: param.to_string(),
            alias: alias.to_string(),
            value: value.to_string(),
            reason: "not a recognized PrismQL atomic literal (string, number, bool, duration, or identifier)".to_string(),
        })
    }

    /// Perform `{{param}}` substitution in an alias template.
    ///
    /// For each placeholder `{{param_name}}`:
    /// - Look up `param_name` in `args` first, then in `entry.parameters` defaults.
    /// - If neither provides a value: return `Err(E-ALIAS-004)` (missing required
    ///   parameter — all params must have defaults per BC-2.11.008).
    /// - Validate each substituted value with `validate_atomic_literal`.
    ///
    /// Returns the substituted query string on success.
    pub(crate) fn substitute_params(
        template: &str,
        entry: &crate::alias_types::AliasEntry,
        args: &HashMap<String, String>,
    ) -> Result<String, PrismError> {
        // Find all {{param}} placeholders in the template.
        let placeholder_re = get_placeholder_regex();
        let mut result = template.to_string();

        for cap in placeholder_re.captures_iter(template) {
            // Both capture groups are guaranteed by the regex structure.
            // Group 0 is the full match; group 1 is the param name.
            let (Some(full_match_m), Some(param_name_m)) = (cap.get(0), cap.get(1)) else {
                // This branch is structurally unreachable for a valid regex match.
                continue;
            };
            let full_match = full_match_m.as_str();
            let param_name = param_name_m.as_str();

            // Look up in args first, then defaults.
            let value = if let Some(v) = args.get(param_name) {
                v.clone()
            } else if let Some(params) = &entry.parameters {
                if let Some(default) = params.get(param_name) {
                    default.value.clone()
                } else {
                    // Unknown parameter name — reject E-ALIAS-004.
                    return Err(PrismError::AliasParameterInvalid {
                        param: param_name.to_string(),
                        alias: entry.name.clone(),
                        value: String::new(),
                        reason: format!(
                            "parameter '{}' is not defined in alias '{}'",
                            param_name, entry.name
                        ),
                    });
                }
            } else {
                // No parameters defined on the alias — any {{param}} is unknown.
                return Err(PrismError::AliasParameterInvalid {
                    param: param_name.to_string(),
                    alias: entry.name.clone(),
                    value: String::new(),
                    reason: format!("alias '{}' has no parameters defined", entry.name),
                });
            };

            // Validate the value as an atomic literal (injection guard).
            Self::validate_atomic_literal(&value, param_name, &entry.name)?;

            // Substitute the placeholder.
            result = result.replacen(full_match, &value, 1);
        }

        Ok(result)
    }
}

/// Lazy-compiled placeholder regex for `{{param_name}}` substitution.
#[allow(clippy::expect_used)]
fn get_placeholder_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"\{\{([a-zA-Z_][a-zA-Z0-9_]*)\}\}")
            .expect("placeholder pattern is a valid regex")
    })
}
