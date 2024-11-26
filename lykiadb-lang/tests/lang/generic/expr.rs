use lykiadb_lang::{ast::expr::Expr, Literal, Span};

#[test]
fn exprs_should_be_equal() {
    let e0 = Expr::Binary {
        left: Box::new(Expr::Literal {
            value: Literal::Num(1.0),
            span: Span::default(),
            id: 0,
            raw: "1.0".to_string(),
        }),
        operation: lykiadb_lang::ast::expr::Operation::Add,
        right: Box::new(Expr::Literal {
            value: Literal::Num(1.0),
            span: Span::default(),
            id: 0,
            raw: "1.0".to_string(),
        }),
        span: Span::default(),
        id: 0,
    };

    let e1 = Expr::Binary {
        left: Box::new(Expr::Literal {
            value: Literal::Num(1.0),
            span: Span::default(),
            id: 0,
            raw: "1.0".to_string(),
        }),
        operation: lykiadb_lang::ast::expr::Operation::Add,
        right: Box::new(Expr::Literal {
            value: Literal::Num(1.0),
            span: Span::default(),
            id: 0,
            raw: "1.0".to_string(),
        }),
        span: Span::default(),
        id: 1,
    };

    assert_eq!(e0, e1);
}
