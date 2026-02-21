use lykiadb_lang::ast::Span;

use crate::{
    engine::interpreter::{HaltReason, Interpreter},
    lykia_module, lykia_native_fn,
    value::RV,
};

pub fn nt_print<'v>(
    _interpreter: &mut Interpreter<'v>,
    called_from: &Span,
    args: &[RV<'v>],
) -> Result<RV<'v>, HaltReason<'v>> {
    for arg in args {
        print!("{arg:?} ");
    }
    println!();
    Ok(RV::Undefined)
}

lykia_module!(out, {
    print => lykia_native_fn!(nt_print)
}, {}, [print]);
