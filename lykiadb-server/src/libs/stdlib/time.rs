use lykiadb_lang::ast::Span;

use crate::interpreter::interpreter::{HaltReason, Interpreter};
use crate::value::RV;
use crate::{lykia_module, lykia_native_fn};
use std::time;

pub fn nt_clock<'rv>(
    _interpreter: &mut Interpreter<'rv>,
    called_from: &Span,
    _args: &[RV<'rv>],
) -> Result<RV<'rv>, HaltReason<'rv>> {
    if let Ok(n) = time::SystemTime::now().duration_since(time::UNIX_EPOCH) {
        return Ok(RV::Double(n.as_secs_f64()));
    }
    Ok(RV::Undefined)
}

lykia_module!(time, {
    clock => lykia_native_fn!(nt_clock)
}, {}, []);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::interpreter::Output;
    use crate::interpreter::interpreter::tests::create_test_interpreter;
    use crate::util::alloc_shared;

    #[test]
    fn test_nt_clock() {
        let mut interpreter = create_test_interpreter(Some(alloc_shared(Output::new())));

        // Test clock function
        let result = nt_clock(&mut interpreter, &Span::default(), &[]);

        assert!(matches!(result, Ok(RV::Double(_))));
    }
}
