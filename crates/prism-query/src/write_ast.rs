//! Write mode AST types for PrismQL.
//!
//! Canonical module path: `prism_query::write_ast`
//!
//! Consumed by S-3.07 (write execution) for dispatch without re-parsing.
//! All types in this module are pure data — no I/O, no sensor resolution.
//!
//! # Architecture Compliance (BC-2.11.004, S-3.06)
//! - `WriteNode` and `DmlNode` carry sufficient context for S-3.07 to dispatch
//!   without re-parsing the original query string.
//! - Parse logic producing these nodes is pure (Chumsky combinator extension).
//! - Do NOT add write execution logic here — that is S-3.07's scope.
//!
//! Story: S-3.06 | BC-2.11.004

use serde::{Deserialize, Serialize};

use crate::ast::{Expr, Literal, Predicate, SqlQuery};

// ─────────────────────────────────────────────────────────────────────────────
// Pipe mode write AST
// ─────────────────────────────────────────────────────────────────────────────

/// Terminal write stage in a pipe-mode query (BC-2.11.004).
///
/// Produced when a pipeline ends with a registered write verb, e.g.:
/// `FROM crowdstrike_hosts | where last_seen < 7d | contain`
///
/// `#[non_exhaustive]` enables future fields (e.g. confirmation token,
/// dry-run flag) without breaking S-3.07 match arms.
///
/// # Implements BC-2.11.004 — Write Parser Extension
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WriteNode {
    /// The write verb (e.g. `"contain"`, `"acknowledge"`, `"tag"`).
    /// Must be a verb registered in `WriteVerbRegistry` at parse time.
    pub verb: String,
    /// Key=value arguments from the pipe write stage.
    pub args: Vec<WriteArg>,
    /// Sensor name resolved at parse time from the `source_stage`
    /// (e.g. `"crowdstrike"` from `crowdstrike_hosts`).
    /// `None` if no source prefix was present or could not be resolved.
    pub source_sensor: Option<String>,
}

/// A single key=value argument in a write stage.
///
/// Parsed from `key=literal` tokens following the write verb.
///
/// # Example
/// `tag key="review" value="pending"` produces two `WriteArg` entries:
/// `WriteArg { key: "key", value: Literal::String("review") }` and
/// `WriteArg { key: "value", value: Literal::String("pending") }`.
///
/// # Implements BC-2.11.004 — Write Parser Extension
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WriteArg {
    /// Argument key (bare identifier).
    pub key: String,
    /// Argument value (literal from the query).
    pub value: Literal,
}

// ─────────────────────────────────────────────────────────────────────────────
// SQL mode DML AST
// ─────────────────────────────────────────────────────────────────────────────

/// DML operation discriminant for SQL mode write statements.
///
/// # Implements BC-2.11.004 — Write Parser Extension
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DmlOperation {
    /// `INSERT INTO table_name (col_list) SELECT …`
    InsertInto,
    /// `UPDATE table_name SET col = val [, …] WHERE expr`
    Update,
    /// `DELETE FROM table_name WHERE expr`
    Delete,
}

/// SQL DML statement AST node.
///
/// Produced by the SQL parser for `INSERT INTO`, `UPDATE`, and `DELETE`
/// statements (S-3.06 extension to `SqlStatement`).
///
/// # Security (BC-2.11.004)
/// - `target_table` beginning with `prism_` is rejected at parse time with
///   `E-QUERY-010` ("Internal Prism table is write-protected").
/// - `UPDATE` and `DELETE` without a WHERE clause are rejected with
///   `E-QUERY-022` ("unbounded write").
/// - `INSERT INTO … SELECT` without a LIMIT or WHERE on the source SELECT
///   is rejected with `E-QUERY-022`.
///
/// `#[non_exhaustive]` enables future DML fields (e.g. RETURNING clause)
/// without breaking downstream match arms.
///
/// # Implements BC-2.11.004 — Write Parser Extension
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DmlNode {
    /// Which DML operation this node represents.
    pub operation: DmlOperation,
    /// Target table name (e.g. `"crowdstrike_contained_hosts"`).
    /// Validated at parse time: `prism_*` tables are write-protected.
    pub target_table: String,
    /// Optional column list for `INSERT INTO table (col1, col2) SELECT …`.
    /// `None` when no explicit column list is provided; `Some(vec![])` is not
    /// produced by the parser (at least one column must be named when a list
    /// is present). Empty for `UPDATE` and `DELETE`.
    ///
    /// Preserved so S-3.07 can enforce column-level constraints without
    /// re-parsing the original query string. (F-PR130-CR-003)
    pub columns: Option<Vec<String>>,
    /// SET column=value pairs for `UPDATE`. Empty for `INSERT INTO` and `DELETE`.
    pub assignments: Vec<Assignment>,
    /// WHERE clause predicate for `UPDATE` / `DELETE`.
    /// `None` for `INSERT INTO`; required (enforced at parse time) for
    /// `UPDATE` and `DELETE`.
    ///
    /// Carries the ACTUAL parsed predicate — not a sentinel. S-3.07 can
    /// evaluate this directly to enforce bounded-write semantics at execution
    /// time (F-PR130-SEC-003).
    pub filter: Option<Predicate>,
    /// Source SELECT query for `INSERT INTO … SELECT …`.
    /// `None` for `UPDATE` and `DELETE`.
    pub source_select: Option<SqlQuery>,
}

/// A single SET assignment in an `UPDATE` statement: `column = value`.
///
/// # Implements BC-2.11.004 — Write Parser Extension
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Assignment {
    /// Column name being assigned.
    pub column: String,
    /// New value expression.
    pub value: Expr,
}
