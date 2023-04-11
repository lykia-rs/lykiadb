use crate::lang::parsing::expr::{Ast, Visitor};
use crate::lang::parsing::parser::Parser;
use crate::lang::parsing::scanner::Scanner;
use crate::lang::parsing::visitors::interpreter::Interpreter;
use crate::lang::parsing::visitors::printer::Printer;

pub fn parse(source: &str) -> Ast {
    let tokens = Scanner::scan(source);
    // println!("{:?}", tokens);
    Parser::parse(&tokens)
}

pub fn print(ast: &Ast) {
    println!("{}", Printer::new().visit_expr(ast));
}

pub fn interpret(ast: &Ast) {
    println!("{:?}", Interpreter::new().visit_expr(&ast));
}