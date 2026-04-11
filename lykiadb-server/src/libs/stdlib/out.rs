use std::sync::Arc;

use lykiadb_lang::ast::Span;

use crate::{
    interpreter::{self, HaltReason, Interpreter},
    lykia_module, lykia_native_fn,
    value::RV,
};

pub fn nt_print<'rv>(
    _interpreter: &mut Interpreter<'rv>,
    called_from: &Span,
    args: &[RV<'rv>],
) -> Result<RV<'rv>, HaltReason<'rv>> {
    for arg in args {
        _interpreter.state.output.clone().unwrap().write().unwrap().push(arg.clone());
    }
    // _interpreter.state.output.clone().unwrap().write().unwrap().push(RV::Str(Arc::new("\n".to_string())));
    Ok(RV::Undefined)
}

lykia_module!(out, {
    print => lykia_native_fn!(nt_print)
}, {}, [print]);
