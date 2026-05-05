//! Visitor pattern for PrismQL AST traversal.
//!
//! The `Visitor` trait provides default `walk_*` implementations for every
//! AST node type. Implementors override only the methods they care about.
//!
//! # Usage
//!
//! ```rust,ignore
//! use prism_query::visit::{Visitor, walk_ast};
//! use prism_query::ast::Ast;
//!
//! struct FieldCollector { fields: Vec<String> }
//!
//! impl Visitor for FieldCollector {
//!     fn visit_field(&mut self, f: &prism_query::ast::FieldPath) {
//!         self.fields.push(f.segments.join("."));
//!     }
//! }
//!
//! let ast = PrismQlParser::parse("src_ip = '10.0.0.1'").unwrap();
//! let mut collector = FieldCollector { fields: vec![] };
//! walk_ast(&mut collector, &ast);
//! assert_eq!(collector.fields, vec!["src_ip"]);
//! ```
//!
//! Story: S-3.01 | P1-001 (type-design audit)
//
// `#[non_exhaustive]` enum catch-alls: the `_ => {}` arms handle future variants
// added by downstream stories (S-3.06+). Clippy warns "unreachable" because all
// current variants are covered — the warning is suppressed at file level.
#![allow(unreachable_patterns)]

use crate::ast::{
    Ast, EnrichStage, Expr, FieldPath, FieldsStage, FilterExpr, FuncCall, Join, JoinStage, Literal,
    OrderExpr, PipeQuery, PipeStage, Predicate, SelectClause, SelectItem, SortExpr, SqlQuery,
    SqlStatement, StatsStage, VirtualField,
};

/// AST visitor trait.
///
/// Every method has a default implementation that calls the corresponding
/// `walk_*` function, which recurses into children. Override methods to
/// hook into specific node types without stopping traversal.
///
/// To stop traversal at a node, override the method and do NOT call the
/// corresponding `walk_*` function.
pub trait Visitor {
    fn visit_ast(&mut self, ast: &Ast) {
        walk_ast(self, ast);
    }
    fn visit_sql_statement(&mut self, s: &SqlStatement) {
        walk_sql_statement(self, s);
    }
    fn visit_sql_query(&mut self, q: &SqlQuery) {
        walk_sql_query(self, q);
    }
    fn visit_filter_expr(&mut self, fe: &FilterExpr) {
        walk_filter_expr(self, fe);
    }
    fn visit_pipe_query(&mut self, q: &PipeQuery) {
        walk_pipe_query(self, q);
    }
    fn visit_pipe_stage(&mut self, s: &PipeStage) {
        walk_pipe_stage(self, s);
    }
    fn visit_predicate(&mut self, p: &Predicate) {
        walk_predicate(self, p);
    }
    fn visit_expr(&mut self, e: &Expr) {
        walk_expr(self, e);
    }
    fn visit_field(&mut self, _f: &FieldPath) {
        // Leaf — no children to walk.
    }
    fn visit_virtual_field(&mut self, _vf: &VirtualField) {
        // Leaf — no children to walk.
    }
    fn visit_literal(&mut self, _l: &Literal) {
        // Leaf — no children to walk.
    }
    fn visit_func_call(&mut self, fc: &FuncCall) {
        walk_func_call(self, fc);
    }
    fn visit_select_clause(&mut self, sc: &SelectClause) {
        walk_select_clause(self, sc);
    }
    fn visit_join(&mut self, j: &Join) {
        walk_join(self, j);
    }
    fn visit_order_expr(&mut self, oe: &OrderExpr) {
        walk_order_expr(self, oe);
    }
    fn visit_stats_stage(&mut self, ss: &StatsStage) {
        walk_stats_stage(self, ss);
    }
    fn visit_join_stage(&mut self, js: &JoinStage) {
        walk_join_stage(self, js);
    }
    fn visit_fields_stage(&mut self, fs: &FieldsStage) {
        walk_fields_stage(self, fs);
    }
    fn visit_sort_expr(&mut self, se: &SortExpr) {
        walk_sort_expr(self, se);
    }
    fn visit_enrich_stage(&mut self, es: &EnrichStage) {
        walk_enrich_stage(self, es);
    }
}

// ── Walk functions (recurse into children) ────────────────────────────────────
//
// The `#[allow]` suppresses `unreachable_patterns` for the `_ => {}` catch-all
// arms that handle future `#[non_exhaustive]` variants added by downstream
// stories (S-3.06+). The catch-all is required for correctness even though all
// current variants are covered.

pub fn walk_ast<V: Visitor + ?Sized>(v: &mut V, ast: &Ast) {
    match ast {
        Ast::Filter(fe) => v.visit_filter_expr(fe),
        Ast::Sql(stmt) => v.visit_sql_statement(stmt),
        Ast::Pipe(pq) => v.visit_pipe_query(pq),
        _ => {} // non_exhaustive catch-all
    }
}

// B-4: walk_sql_statement uses `match` (not `if let`) intentionally.
// `SqlStatement` is `#[non_exhaustive]`, so the `_ => {}` catch-all arm is
// required for forward-correctness when S-3.06 adds Dml/Ddl variants.
// Clippy's `single_match` lint suggests `if let`, but that would silently
// ignore future variants — the match is the safe, correct form here.
#[allow(clippy::single_match)]
pub fn walk_sql_statement<V: Visitor + ?Sized>(v: &mut V, s: &SqlStatement) {
    match s {
        SqlStatement::Select(sq) => v.visit_sql_query(sq),
        // S-3.06 will add Dml and Ddl variants here.
        _ => {}
    }
}

pub fn walk_sql_query<V: Visitor + ?Sized>(v: &mut V, q: &SqlQuery) {
    v.visit_select_clause(&q.select);
    v.visit_field(&q.from.source.as_field_path());
    for join in &q.joins {
        v.visit_join(join);
    }
    if let Some(pred) = &q.where_ {
        v.visit_predicate(pred);
    }
    for expr in &q.group_by {
        v.visit_expr(expr);
    }
    if let Some(pred) = &q.having {
        v.visit_predicate(pred);
    }
    for oe in &q.order_by {
        v.visit_order_expr(oe);
    }
}

pub fn walk_filter_expr<V: Visitor + ?Sized>(v: &mut V, fe: &FilterExpr) {
    v.visit_field(&fe.source.as_field_path());
    v.visit_predicate(&fe.predicate);
}

pub fn walk_pipe_query<V: Visitor + ?Sized>(v: &mut V, q: &PipeQuery) {
    v.visit_field(&q.source.as_field_path());
    for stage in &q.stages {
        v.visit_pipe_stage(stage);
    }
}

pub fn walk_pipe_stage<V: Visitor + ?Sized>(v: &mut V, s: &PipeStage) {
    match s {
        PipeStage::Where(pred) => v.visit_predicate(pred),
        PipeStage::Sort(exprs) => {
            for se in exprs {
                v.visit_sort_expr(se);
            }
        }
        PipeStage::Stats(ss) => v.visit_stats_stage(ss),
        PipeStage::Dedup(fields) => {
            for f in fields {
                v.visit_field(f);
            }
        }
        PipeStage::Fields(fs) => v.visit_fields_stage(fs),
        PipeStage::Join(js) => v.visit_join_stage(js),
        PipeStage::Enrich(es) => v.visit_enrich_stage(es),
        PipeStage::Limit(_) | PipeStage::Tail(_) => {}
        _ => {}
    }
}

pub fn walk_predicate<V: Visitor + ?Sized>(v: &mut V, p: &Predicate) {
    match p {
        Predicate::Compare { lhs, rhs, .. } => {
            v.visit_expr(lhs);
            v.visit_expr(rhs);
        }
        Predicate::StringOp { field, .. }
        | Predicate::Regex { field, .. }
        | Predicate::Has(field)
        | Predicate::Missing(field)
        | Predicate::IsNull { field, .. }
        | Predicate::Wildcard { field, .. }
        | Predicate::Cidr { field, .. } => {
            v.visit_field(field);
        }
        Predicate::In { field, values, .. } => {
            v.visit_field(field);
            for lit in values {
                v.visit_literal(lit);
            }
        }
        Predicate::InSubquery {
            field, subquery, ..
        } => {
            v.visit_field(field);
            v.visit_sql_query(subquery);
        }
        Predicate::Between {
            field, low, high, ..
        } => {
            v.visit_field(field);
            v.visit_literal(low);
            v.visit_literal(high);
        }
        Predicate::Logical { predicates, .. } => {
            for child in predicates {
                v.visit_predicate(child);
            }
        }
        Predicate::Not(inner) => {
            v.visit_predicate(inner);
        }
        _ => {}
    }
}

pub fn walk_expr<V: Visitor + ?Sized>(v: &mut V, e: &Expr) {
    match e {
        Expr::Literal(l) => v.visit_literal(l),
        Expr::Field(f) => v.visit_field(f),
        Expr::VirtualField(vf) => v.visit_virtual_field(vf),
        Expr::Compare { lhs, rhs, .. } => {
            v.visit_expr(lhs);
            v.visit_expr(rhs);
        }
        Expr::Logical { lhs, rhs, .. } => {
            v.visit_expr(lhs);
            v.visit_expr(rhs);
        }
        Expr::Not(inner) => v.visit_expr(inner),
        Expr::In { field, values, .. } => {
            v.visit_field(field);
            for lit in values {
                v.visit_literal(lit);
            }
        }
        Expr::InSubquery {
            field, subquery, ..
        } => {
            v.visit_field(field);
            v.visit_sql_query(subquery);
        }
        Expr::FuncCall(fc) => v.visit_func_call(fc),
        Expr::Star => {}
        _ => {}
    }
}

pub fn walk_func_call<V: Visitor + ?Sized>(v: &mut V, fc: &FuncCall) {
    use crate::ast::FuncCall;
    match fc {
        FuncCall::Aggregate { args, .. } | FuncCall::Scalar { args, .. } => {
            for arg in args {
                v.visit_expr(arg);
            }
        }
        FuncCall::Window { .. } => {}
        _ => {}
    }
}

pub fn walk_select_clause<V: Visitor + ?Sized>(v: &mut V, sc: &SelectClause) {
    for item in &sc.items {
        match item {
            SelectItem::Star | SelectItem::TableStar(_) => {}
            SelectItem::Expr { expr, .. } => v.visit_expr(expr),
            _ => {}
        }
    }
}

pub fn walk_join<V: Visitor + ?Sized>(v: &mut V, j: &Join) {
    v.visit_field(&j.source.as_field_path());
    v.visit_expr(&j.on);
}

pub fn walk_order_expr<V: Visitor + ?Sized>(v: &mut V, oe: &OrderExpr) {
    v.visit_expr(&oe.expr);
}

pub fn walk_stats_stage<V: Visitor + ?Sized>(v: &mut V, ss: &StatsStage) {
    for sf in &ss.aggregates {
        walk_agg_func(v, &sf.func);
    }
    for f in &ss.by_fields {
        v.visit_field(f);
    }
}

pub fn walk_agg_func<V: Visitor + ?Sized>(v: &mut V, func: &crate::ast::AggFunc) {
    use crate::ast::AggFunc;
    match func {
        AggFunc::CountField(f)
        | AggFunc::Sum(f)
        | AggFunc::Avg(f)
        | AggFunc::Min(f)
        | AggFunc::Max(f)
        | AggFunc::DistinctCount(f) => v.visit_field(f),
        AggFunc::Percentile { field, .. } => v.visit_field(field),
        AggFunc::Count => {}
        _ => {}
    }
}

pub fn walk_join_stage<V: Visitor + ?Sized>(v: &mut V, js: &JoinStage) {
    v.visit_field(&js.source.as_field_path());
    use crate::ast::JoinCondition;
    match &js.on {
        JoinCondition::SameField(f) => v.visit_field(f),
        JoinCondition::Pair(l, r) => {
            v.visit_field(l);
            v.visit_field(r);
        }
        _ => {}
    }
}

pub fn walk_fields_stage<V: Visitor + ?Sized>(v: &mut V, fs: &FieldsStage) {
    for f in &fs.fields {
        v.visit_field(f);
    }
}

pub fn walk_sort_expr<V: Visitor + ?Sized>(v: &mut V, se: &SortExpr) {
    v.visit_field(&se.field);
}

pub fn walk_enrich_stage<V: Visitor + ?Sized>(v: &mut V, es: &EnrichStage) {
    v.visit_field(&es.field);
}

// ── Helper: SourceRef as FieldPath for visitor dispatch ───────────────────────

trait AsFieldPath {
    fn as_field_path(&self) -> crate::ast::FieldPath;
}

impl AsFieldPath for crate::ast::SourceRef {
    fn as_field_path(&self) -> crate::ast::FieldPath {
        crate::ast::FieldPath {
            segments: vec![self.raw.clone()],
            span: crate::ast::Span::ZERO,
        }
    }
}
