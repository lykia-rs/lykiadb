use crate::{
    engine::{error::ExecutionError, interpreter::HaltReason},
    plan::planner::InClause,
};

use lykiadb_lang::ast::sql::{SqlFrom, SqlJoinType, SqlSource};

use super::{Node, planner::Planner, scope::Scope};

// The source can be of following types:

// - Collection: A regular db collection.
// - Expr: An expression that returns a set of data.
// - Subquery: A subquery that returns a set of data.
// - Join: A join between two or more sources.
// - Group: Cartesian product of two or more sources.
pub fn build_from(
    planner: &mut Planner,
    from: &SqlFrom,
    parent_scope: &mut Scope,
) -> Result<Node, HaltReason> {
    let mut scope = Scope::new();

    let node = match from {
        // SqlSource::* should directly blend in the scope and can be
        // projected. Each source will be accessible by its alias.
        SqlFrom::Source(source) => {
            let wrapped = match source {
                SqlSource::Collection(ident) => Node::Scan {
                    source: ident.clone(),
                    filter: None,
                },
                SqlSource::Expr(expr) => Node::EvalScan {
                    source: expr.clone(),
                    filter: None,
                },
            };

            if let Err(err) = scope.add_source(source.alias(), from.clone()) {
                return Err(HaltReason::Error(ExecutionError::Plan(err)));
            }

            Ok(wrapped)
        }

        // SqlFrom::Select can be projected with the alias. Downstream
        // sources will be merged into single source and will be accessible
        // via the Select's alias.
        SqlFrom::Select { subquery, alias } => {
            let node = Node::Subquery {
                source: Box::new(planner.build_select(subquery)?),
                alias: alias.clone(),
            };

            if let Err(err) = scope.add_source(alias, from.clone()) {
                return Err(HaltReason::Error(ExecutionError::Plan(err)));
            }

            Ok(node)
        }

        // SqlFrom::Group scope gets directly merged into the parent scope.
        // Each source will be accessible by its alias, no merging takes place.
        SqlFrom::Group { values } => {
            let mut froms = values.iter();
            let mut node = build_from(planner, froms.next().unwrap(), &mut scope)?;
            for right in froms {
                node = Node::Join {
                    left: Box::new(node),
                    join_type: SqlJoinType::Cross,
                    right: Box::new(build_from(planner, right, &mut scope)?),
                    constraint: None,
                }
            }
            Ok(node)
        }

        // Similar to SqlFrom::Group, SqlFrom::Join scope gets directly merged into the parent scope.
        // Each source will be accessible by its alias, no merging takes place.
        SqlFrom::Join {
            left,
            join_type,
            right,
            constraint,
        } => {
            let constraint = constraint
                .as_ref()
                .map(|x| planner.build_expr(x, InClause::JoinOn, &mut scope, false, false))
                .transpose()?;

            Ok(Node::Join {
                left: Box::new(build_from(planner, left, &mut scope)?),
                join_type: join_type.clone(),
                right: Box::new(build_from(planner, right, &mut scope)?),
                constraint: constraint.map(|x| x.0),
            })
        }
    };

    if let Err(err) = parent_scope.merge(&scope) {
        return Err(HaltReason::Error(ExecutionError::Plan(err)));
    }

    node
}
