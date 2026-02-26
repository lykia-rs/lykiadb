use std::fmt::Display;

use lykiadb_lang::ast::{
    Identifier,
    expr::Expr,
    sql::{
        SqlCollectionIdentifier, SqlCompoundOperator, SqlExpressionSource, SqlJoinType,
        SqlOrdering, SqlProjection,
    },
};
use serde::{Deserialize, Serialize};

use crate::value::{RV, callable::AggregatorFactory};
use derivative::Derivative;

pub mod planner;
pub mod error;
mod aggregation;
mod expr;
mod from;
mod scope;


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum IntermediateExpr<'v> {
    Constant(RV<'v>),
    Expr { expr: Box<Expr> },
}

impl<'v> Display for IntermediateExpr<'v> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntermediateExpr::Constant(rv) => write!(f, "{rv:?}"),
            IntermediateExpr::Expr { expr, .. } => {
                write!(f, "{expr}")
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Plan<'v> {
    Select(Node<'v>),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Node<'v> {
    Compound {
        source: Box<Node<'v>>,
        operator: SqlCompoundOperator,
        right: Box<Node<'v>>,
    },

    Aggregate {
        source: Box<Node<'v>>,
        group_by: Vec<IntermediateExpr<'v>>,
        aggregates: Vec<Aggregation<'v>>,
    },

    Filter {
        source: Box<Node<'v>>,
        predicate: IntermediateExpr<'v>,
        subqueries: Vec<Node<'v>>,
    },

    Projection {
        source: Box<Node<'v>>,
        fields: Vec<SqlProjection>,
    },

    Limit {
        source: Box<Node<'v>>,
        limit: usize,
    },

    Offset {
        source: Box<Node<'v>>,
        offset: usize,
    },

    Order {
        source: Box<Node<'v>>,
        key: Vec<(IntermediateExpr<'v>, SqlOrdering)>,
    },

    Scan {
        source: SqlCollectionIdentifier,
        filter: Option<IntermediateExpr<'v>>,
    },

    EvalScan {
        source: SqlExpressionSource,
        filter: Option<IntermediateExpr<'v>>,
    },

    Join {
        left: Box<Node<'v>>,
        join_type: SqlJoinType,
        right: Box<Node<'v>>,
        constraint: Option<IntermediateExpr<'v>>,
    },

    Subquery {
        source: Box<Node<'v>>,
        alias: Identifier,
    },

    Nothing,
}

impl<'v> Display for Plan<'v> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Plan::Select(node) => write!(f, "{node}"),
        }
    }
}

impl<'v> Node<'v> {
    const TAB: &'static str = "  ";
    const NEWLINE: &'static str = "\n";

    fn _fmt_recursive(&self, f: &mut std::fmt::Formatter<'_>, indent: usize) -> std::fmt::Result {
        let indent_str = Self::TAB.repeat(indent);
        match self {
            Node::Nothing => write!(f, "{}- nothing{}", indent_str, Self::NEWLINE),
            Node::Order { source, key } => {
                let key_description = key
                    .iter()
                    .map(|(expr, ordering)| format!("({expr}, {ordering:?})"))
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(
                    f,
                    "{}- order [{}]{}",
                    indent_str,
                    key_description,
                    Self::NEWLINE
                )?;
                source._fmt_recursive(f, indent + 1)
            }
            Node::Projection { source, fields } => {
                let fields_description = fields
                    .iter()
                    .map(|field| match field {
                        SqlProjection::All { collection } => {
                            if let Some(c) = collection.as_ref() {
                                return format!("* in {}", c.name);
                            }
                            "*".to_string()
                        }
                        SqlProjection::Expr { expr, alias } => {
                            if let Some(alias) = alias {
                                return format!("{} as {}", expr, alias.name);
                            }
                            format!("{expr} as {expr}")
                        }
                    })
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(
                    f,
                    "{}- project [{}]{}",
                    indent_str,
                    fields_description,
                    Self::NEWLINE
                )?;

                source._fmt_recursive(f, indent + 1)
            }
            Node::Filter {
                source,
                predicate,
                subqueries,
            } => {
                write!(f, "{}- filter [{}]{}", indent_str, predicate, Self::NEWLINE)?;
                if !subqueries.is_empty() {
                    write!(f, "{}  > subqueries{}", indent_str, Self::NEWLINE)?;
                    subqueries
                        .iter()
                        .try_for_each(|subquery| subquery._fmt_recursive(f, indent + 2))?;
                }
                source._fmt_recursive(f, indent + 1)
            }
            Node::Subquery { source, alias } => {
                write!(
                    f,
                    "{}- subquery [{}]{}",
                    indent_str,
                    alias.name.clone(),
                    Self::NEWLINE
                )?;
                source._fmt_recursive(f, indent + 1)
            }
            Node::Scan { source, .. } => {
                write!(
                    f,
                    "{}- scan [{} as {}]{}",
                    indent_str,
                    source.name,
                    source.alias.as_ref().unwrap_or(&source.name),
                    Self::NEWLINE
                )
            }
            Node::Compound {
                source,
                operator,
                right,
            } => {
                write!(
                    f,
                    "{}- compound [type={:?}]{}",
                    indent_str,
                    operator,
                    Self::NEWLINE
                )?;
                source._fmt_recursive(f, indent + 1)?;
                right._fmt_recursive(f, indent + 1)
            }
            Node::Limit { source, limit } => {
                write!(
                    f,
                    "{}- limit [count={}]{}",
                    indent_str,
                    limit,
                    Self::NEWLINE
                )?;
                source._fmt_recursive(f, indent + 1)
            }
            Node::Offset { source, offset } => {
                write!(
                    f,
                    "{}- offset [count={}]{}",
                    indent_str,
                    offset,
                    Self::NEWLINE
                )?;
                source._fmt_recursive(f, indent + 1)
            }
            Node::Join {
                left,
                join_type,
                right,
                constraint,
            } => {
                write!(
                    f,
                    "{}- join [type={:?}, {}]{}",
                    indent_str,
                    join_type,
                    constraint
                        .as_ref()
                        .map(|x| x.to_string())
                        .unwrap_or("None".to_string()),
                    Self::NEWLINE
                )?;
                left._fmt_recursive(f, indent + 1)?;
                right._fmt_recursive(f, indent + 1)
            }
            Node::EvalScan { source, .. } => {
                write!(
                    f,
                    "{}- eval_scan [{}]{}",
                    indent_str,
                    source.expr,
                    Self::NEWLINE
                )
            }
            Node::Aggregate {
                source,
                group_by,
                aggregates,
            } => {
                let group_by_description = group_by
                    .iter()
                    .map(|expr| expr.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                let aggregates_description = aggregates
                    .iter()
                    .map(|aggregate| {
                        let args = aggregate
                            .args
                            .iter()
                            .map(|arg| arg.to_string())
                            .collect::<Vec<String>>()
                            .join(", ");
                        format!("{}({})", aggregate.name, args)
                    })
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(
                    f,
                    "{}- aggregate [group_by=[{}], aggregates=[{}]]{}",
                    indent_str,
                    group_by_description,
                    aggregates_description,
                    Self::NEWLINE
                )?;
                source._fmt_recursive(f, indent + 1)
            }
            _ => "<NotImplementedYet>".fmt(f),
        }
    }
}

impl<'v> Display for Node<'v> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self._fmt_recursive(f, 0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Aggregation<'v> {
    pub name: String,
    #[serde(skip)]
    #[derivative(PartialEq = "ignore")]
    #[derivative(Hash = "ignore")]
    pub callable: Option<AggregatorFactory<'v>>,
    pub args: Vec<Expr>,
    pub call_expr: Expr,
    pub call_sign: String,
}

impl<'v> Aggregation<'v> {
    pub fn new(
        agg_name: &str,
        agg_factory: &AggregatorFactory<'v>,
        args: &Vec<Expr>,
        expr: &Expr,
    ) -> Aggregation<'v> {
        Aggregation {
            name: agg_name.to_string(),
            callable: Some(*agg_factory),
            args: args.clone(),
            call_expr: expr.clone(),
            call_sign: expr.sign(),
        }
    }
}

impl<'v> Display for Aggregation<'v> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}({})",
            self.name,
            self.args
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}
