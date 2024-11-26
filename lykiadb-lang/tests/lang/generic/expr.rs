use std::collections::HashSet;

use lykiadb_lang::{ast::expr::Expr, Literal, Span};

fn create_simple_add_expr(id: usize, left: f64, right: f64) -> Expr {
    Expr::Binary {
        left: Box::new(Expr::Literal {
            value: Literal::Num(left),
            span: Span::default(),
            id: 0,
            raw: left.to_string()
        }),
        operation: lykiadb_lang::ast::expr::Operation::Add,
        right: Box::new(Expr::Literal {
            value: Literal::Num(right),
            span: Span::default(),
            id: 0,
            raw: right.to_string(),
        }),
        span: Span::default(),
        id,
    }
}
#[test]
fn identical_exprs_should_be_equal_when_ids_are_different() {

    let e0 = create_simple_add_expr(0, 1.0, 2.0);

    let e1 = create_simple_add_expr(1, 1.0, 2.0);

    assert_eq!(e0, e1);

    let mut set: HashSet<Expr> = HashSet::new();

    set.insert(e0);

    assert!(set.contains(&e1));

}


#[test]
fn different_exprs_with_same_ids_should_not_be_equal() {

    let e0 = create_simple_add_expr(1, 2.0, 3.0);

    let e1 = create_simple_add_expr(1, 1.0, 2.0);

    assert_ne!(e0, e1);
}


#[test]
fn mirrored_exprs_should_not_be_equal() {

    let e0 = create_simple_add_expr(0, 2.0, 1.0);

    let e1 = create_simple_add_expr(1, 1.0, 2.0);

    assert_ne!(e0, e1);
}