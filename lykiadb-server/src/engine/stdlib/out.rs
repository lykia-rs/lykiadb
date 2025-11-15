use lykiadb_lang::ast::Span;

use crate::{
    engine::interpreter::{HaltReason, Interpreter},
    value::RV,
};

pub fn nt_print(
    _interpreter: &mut Interpreter,
    called_from: &Span,
    args: &[RV],
) -> Result<RV, HaltReason> {
    for arg in args {
        print!("{arg:?} ");
    }
    println!();
    Ok(RV::Undefined)
}
